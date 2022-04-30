use std::env;
use std::io::{self, BufWriter, Stdout, Write};

use anyhow::{Result, Context};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, KeyEvent};
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{
    self, Clear,
    ClearType::{All, UntilNewLine},
};

use kilo_rs::{core::Location, editor::Editor, terminal::RawModeOverride};

trait Component {
    fn new() -> Self;
    fn render(&self, writer: &mut impl io::Write, context: &SharedContext) -> Result<()>;
    fn cursor(&self, context: &SharedContext) -> Option<Location>;
    fn process_event(&mut self, event: &KeyEvent, context: &mut SharedContext) -> Result<()>;
}

struct SharedContext {
    editor: Editor,
    execution_state: ExecutionState,
    logical_state: LogicalState,
}

enum ExecutionState {
    Initialization,
    Running,
    Closing,
}

enum LogicalState {
    EditorFocus,
    PromptFocus,
}

struct App {
    root: RootComponent,
    context: SharedContext,
    stdout: BufWriter<Stdout>,
}

impl App {
    fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            root: RootComponent::new(),
            context: SharedContext {
                editor: Editor::new(width as usize, height as usize),
                execution_state: ExecutionState::Initialization,
                logical_state: LogicalState::EditorFocus,
            },
            stdout: BufWriter::new(io::stdout()),
        })
    }

    fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.context.editor.open_file(file_path)
    }

    fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        self.context.execution_state = ExecutionState::Running;
        while let ExecutionState::Running = self.context.execution_state {
            self.render()?;
            self.process_events()?;
        }

        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        self.context.execution_state = ExecutionState::Closing;
        queue!(self.stdout, Clear(All), MoveTo(0, 0))?;
        Ok(())
    }

    fn render(&mut self) -> anyhow::Result<()> {
        queue!(self.stdout, Hide)?;

        self.root.render(&mut self.stdout, &self.context)?;

        let Location { line, col } = self.root.cursor(&self.context)
            .context("failed to get cursor location")?;
        queue!(self.stdout, MoveTo(col as u16, line as u16))?;

        queue!(self.stdout, Show)?;
        self.stdout.flush()?;

        Ok(())
    }

    fn process_events(&mut self) -> anyhow::Result<()> {
        use event::KeyCode::*;
        use event::KeyModifiers as KM;

        if let event::Event::Key(event @ KeyEvent { modifiers, code }) = event::read()? {
            match (modifiers, code) {
                (KM::CONTROL, Char('q')) => self.terminate()?,
                _ => self.root.process_event(&event, &mut self.context)?,
            };
        }

        Ok(())
    }
}

struct EditorComponent;
struct StatusComponent;
struct RootComponent {
    editor: EditorComponent,
    status: StatusComponent,
}

impl Component for RootComponent {
    fn new() -> Self {
        Self {
            editor: EditorComponent::new(),
            status: StatusComponent::new(),
        }
    }

    fn render(&self, writer: &mut impl io::Write, context: &SharedContext) -> Result<()> {
        self.editor.render(writer, context)?;
        self.status.render(writer, context)?;

        Ok(())
    }

    fn cursor(&self, context: &SharedContext) -> Option<Location> {
        match context.logical_state {
            LogicalState::EditorFocus => self.editor.cursor(context),
            LogicalState::PromptFocus => self.status.cursor(context),
        }
    }

    fn process_event(&mut self, event: &KeyEvent, context: &mut SharedContext) -> Result<()> {
        match context.logical_state {
            LogicalState::EditorFocus => self.editor.process_event(event, context)?,
            LogicalState::PromptFocus => self.status.process_event(event, context)?,
        }

        Ok(())
    }
}

impl Component for EditorComponent {
    fn new() -> Self {
        Self
    }

    fn render(
        &self,
        writer: &mut impl std::io::Write,
        context: &SharedContext,
    ) -> anyhow::Result<()> {
        queue!(writer, MoveTo(0, 0))?;

        for line in context.editor.get_view_contents() {
            queue!(writer, Print(line))?;
            queue!(writer, Clear(UntilNewLine))?;
            queue!(writer, Print("\r\n"))?;
        }

        Ok(())
    }

    fn cursor(&self, context: &SharedContext) -> Option<Location> {
        Some(context.editor.get_view_cursor())
    }

    #[rustfmt::skip]
    fn process_event(&mut self, event: &KeyEvent, context: &mut SharedContext) -> anyhow::Result<()> {
        use event::KeyCode::*;
        use event::KeyModifiers as KM;

        let &KeyEvent{ modifiers, code } = event;
        match (modifiers, code) {
            (KM::NONE, Up) => context.editor.move_cursor_up(),
            (KM::NONE, Down) => context.editor.move_cursor_down(),
            (KM::NONE, Left) => context.editor.move_cursor_left(),
            (KM::NONE, Right) => context.editor.move_cursor_right(),

            (KM::NONE, Home) => context.editor.move_cursor_to_line_start(),
            (KM::NONE, End) => context.editor.move_cursor_to_line_end(),

            (KM::NONE, PageUp) => context.editor.move_one_view_up(),
            (KM::NONE, PageDown) => context.editor.move_one_view_down(),

            (KM::CONTROL, PageUp) => context.editor.move_cursor_to_buffer_top(),
            (KM::CONTROL, PageDown) => context.editor.move_cursor_to_buffer_bottom(),

            (KM::NONE, Backspace) => context.editor.remove_char_behind(),
            (KM::NONE, Delete) => context.editor.remove_char_in_front(),

            (KM::NONE, Char(c)) => context.editor.insert_char(c),
            (KM::NONE, Enter) => context.editor.insert_line(),

            _ => {}
        }
        
        Ok(())
    }
}

impl Component for StatusComponent {
    fn new() -> Self {
        Self
    }

    fn render(&self, writer: &mut impl io::Write, context: &SharedContext) -> anyhow::Result<()> {
        let file_name = match context.editor.get_file_name() {
            Some(name) => name,
            None => "[Scratch]",
        };

        let left_part = format!("{:.20}", file_name);
        let right_part = format!(
            "{}/{}",
            context.editor.get_buffer_cursor().line + 1,
            context.editor.get_buffer_line_count()
        );
        let total_len = left_part.len() + right_part.len();

        let view_width = context.editor.get_view_width();
        let status_bar = if total_len <= view_width {
            left_part + &" ".repeat(view_width - total_len) + &right_part
        } else {
            format!("{left_part:0$.0$}", view_width)
        };

        let status_line = context.editor.get_view_height();
        queue!(writer, MoveTo(0, status_line as u16))?;
        queue!(writer, PrintStyledContent(status_bar.negative()))?;
        Ok(())
    }

    fn cursor(&self, _context: &SharedContext) -> Option<Location> {
        None
    }

    fn process_event(
        &mut self,
        _event: &KeyEvent,
        _context: &mut SharedContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut kilo = App::new()?;

    let args: Vec<_> = env::args().skip(1).collect();
    match args.len() {
        0 => {}
        1 => kilo.open_file(&args[0])?,
        _ => println!("USAGE: kilo [path_to_file]"),
    }

    kilo.run()
}
