use std::io::{self, BufWriter, Stdout, Write};

use anyhow::{Context, Result};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event};
use crossterm::queue;
use crossterm::terminal::{self, Clear, ClearType::All};

use crate::app::App;
use crate::shared::{Focus, SharedContext};
use crate::term_utils::RawModeOverride;
use kilo_rs_backend::{core::Location, editor::Editor};

pub struct AppRunner {
    app: App,
    context: SharedContext,
    stdout: BufWriter<Stdout>,
}

pub enum ShouldQuit {
    Yes,
    No,
}

impl AppRunner {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            app: App::new(),
            context: SharedContext {
                editor: Editor::new(width as usize, height as usize),
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

        loop {
            self.render()?;

            if let ShouldQuit::Yes = self.process_events()? {
                break;
            }
        }

        self.terminate()
    }

    fn terminate(&mut self) -> Result<()> {
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

    fn process_events(&mut self) -> Result<ShouldQuit> {
        if let Event::Key(event) = event::read()? {
            self.app.process_event(&event, &mut self.context)
        } else {
            Ok(ShouldQuit::No)
        }
    }
}
