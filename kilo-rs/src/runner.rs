use std::io::{self, BufWriter, Stdout, Write};

use anyhow::{Context, Result};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::terminal::{self, Clear, ClearType::All};

use crate::app::App;
use crate::shared::{ExecutionState, Focus, SharedContext};
use crate::term_utils::RawModeOverride;
use kilo_rs_backend::{core::Location, editor::Editor};

pub struct AppRunner {
    app: App,
    context: SharedContext,
    stdout: BufWriter<Stdout>,
}

impl AppRunner {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            app: App::new(),
            context: SharedContext {
                editor: Editor::new(width as usize, height as usize),
                state: ExecutionState::Initialization,
                focus: Focus::TextArea,
            },
            stdout: BufWriter::new(io::stdout()),
        })
    }

    pub fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.context.editor.open_file(file_path)
    }

    pub fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        self.context.state = ExecutionState::Running;
        while let ExecutionState::Running = self.context.state {
            self.render()?;
            self.process_events()?;
        }

        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        self.context.state = ExecutionState::Closing;
        queue!(self.stdout, Clear(All), MoveTo(0, 0))?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        queue!(self.stdout, Hide)?;

        self.app.render(&mut self.stdout, &self.context)?;

        let Location { line, col } = self
            .app
            .cursor(&self.context)
            .context("failed to get cursor location")?;
        queue!(self.stdout, MoveTo(col as u16, line as u16))?;

        queue!(self.stdout, Show)?;
        self.stdout.flush()?;

        Ok(())
    }

    fn process_events(&mut self) -> Result<()> {
        use KeyCode::*;
        use KeyModifiers as KM;

        if let Event::Key(event @ KeyEvent { modifiers, code }) = event::read()? {
            match (modifiers, code) {
                (KM::CONTROL, Char('q')) => self.terminate()?,
                _ => self.app.process_event(&event, &mut self.context)?,
            };
        }

        Ok(())
    }
}