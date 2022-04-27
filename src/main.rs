use std::env;
use std::io::{self, BufWriter, Stdout, Write};

use anyhow::Result;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event;
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{
    self, Clear,
    ClearType::{All, UntilNewLine},
};
use crossterm::QueueableCommand;

use kilo_rs::backend::{editor::Editor, terminal::RawModeOverride};
use kilo_rs::frontend::{Component, Cursor};

struct KiloContext {
    editor: Editor,
}

struct Kilo {
    context: KiloContext,
    is_running: bool,
    stdout: BufWriter<Stdout>,
    components: Vec<Box<dyn Component<Context = KiloContext>>>,
}

impl Kilo {
    fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            context: KiloContext {
                editor: Editor::new(width as usize, height as usize),
            },
            is_running: true,
            stdout: BufWriter::new(io::stdout()),
            components: vec![Box::new(EditorComponent), Box::new(StatusComponent)],
        })
    }

    fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.context.editor.open_file(file_path)
    }

    fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        while self.is_running {
            self.render()?;
            self.process_events()?;
        }

        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        self.is_running = false;
        queue!(self.stdout, Clear(All), MoveTo(0, 0))?;
        Ok(())
    }

    fn render(&mut self) -> anyhow::Result<()> {
        queue!(self.stdout, Hide)?;

        for component in &self.components {
            component.render(&mut self.stdout, &self.context)?;
        }
        for component in &self.components {
            if let Some(Cursor(x, y)) = component.cursor(&self.context) {
                queue!(self.stdout, MoveTo(x, y))?;
                break;
            }
        }

        queue!(self.stdout, Show)?;
        self.stdout.flush()?;

        Ok(())
    }

    fn process_events(&mut self) -> anyhow::Result<()> {
        let event = event::read()?;

        use event::KeyCode::*;
        use event::KeyModifiers as KM;
        if let event::Event::Key(event::KeyEvent { modifiers, code }) = event {
            match (modifiers, code) {
                (KM::CONTROL, Char('q')) => self.terminate()?,
                _ => {}
            }
        }

        for component in &mut self.components {
            component.process_event(&event, &mut self.context)?;
        }

        Ok(())
    }
}

struct EditorComponent;
struct StatusComponent;

impl Component for EditorComponent {
    type Context = KiloContext;

    fn render(&self, writer: &mut dyn std::io::Write, context: &KiloContext) -> anyhow::Result<()> {
        writer.queue(MoveTo(0, 0))?;

        for line in context.editor.get_view_contents() {
            writer.queue(Print(line))?;
            writer.queue(Clear(UntilNewLine))?;
            writer.queue(Print("\r\n"))?;
        }

        Ok(())
    }

    fn cursor(&self, context: &KiloContext) -> Option<kilo_rs::frontend::Cursor> {
        Some(context.editor.get_view_cursor().into())
    }

    #[rustfmt::skip]
    fn process_event(&mut self, event: &crossterm::event::Event, context: &mut KiloContext) -> anyhow::Result<()> {
        use event::KeyCode::*;
        use event::KeyModifiers as KM;

        if let &event::Event::Key(event::KeyEvent{ modifiers, code }) = event {
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
        }

        Ok(())
    }
}

impl Component for StatusComponent {
    type Context = KiloContext;

    fn render(&self, writer: &mut dyn io::Write, context: &KiloContext) -> anyhow::Result<()> {
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
        writer.queue(MoveTo(0, status_line as u16))?;
        writer.queue(PrintStyledContent(status_bar.negative()))?;
        Ok(())
    }

    fn cursor(&self, _context: &KiloContext) -> Option<kilo_rs::frontend::Cursor> {
        None
    }

    fn process_event(
        &mut self,
        _event: &event::Event,
        _context: &mut KiloContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut kilo = Kilo::new()?;

    let args: Vec<_> = env::args().skip(1).collect();
    match args.len() {
        0 => {}
        1 => kilo.open_file(&args[0])?,
        _ => println!("USAGE: kilo [path_to_file]"),
    }

    kilo.run()
}
