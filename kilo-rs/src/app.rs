use std::io::{self, BufWriter, Stdout, Write};

use anyhow::{Context, Result};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, KeyEvent};
use crossterm::queue;
use crossterm::terminal::{self, Clear, ClearType::All};

use crate::root::RootComponent;
use crate::shared::{ExecutionState, Focus, SharedContext};
use kilo_rs_backend::{core::Location, editor::Editor, terminal::RawModeOverride};

pub struct App {
    root: RootComponent,
    context: SharedContext,
    stdout: BufWriter<Stdout>,
}

impl App {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            root: RootComponent::new(),
            context: SharedContext {
                editor: Editor::new(width as usize, height as usize),
                execution_state: ExecutionState::Initialization,
                logical_state: Focus::TextArea,
            },
            stdout: BufWriter::new(io::stdout()),
        })
    }

    pub fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.context.editor.open_file(file_path)
    }

    pub fn run(&mut self) -> Result<()> {
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

        let Location { line, col } = self
            .root
            .cursor(&self.context)
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
