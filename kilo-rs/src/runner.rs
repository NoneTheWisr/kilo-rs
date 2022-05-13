use std::collections::VecDeque;
use std::io::{self, BufWriter, Stdout, Write};

use anyhow::{Context, Result};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event};
use crossterm::queue;
use crossterm::terminal::{self, Clear, ClearType::All};

use crate::app::{App, AppMessage};
use crate::shared::{Focus, SharedContext};
use crate::term_utils::RawModeOverride;
use kilo_rs_backend::{core::Location, editor::Editor};

pub struct AppRunner {
    app: App,
    context: SharedContext,
    stdout: BufWriter<Stdout>,
    queue: MessageQueue,
}

pub enum ShouldQuit {
    Yes,
    No,
}

pub type MessageQueue = VecDeque<AppMessage>;

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
            queue: MessageQueue::new(),
        })
    }

    pub fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.context.editor.open_file(file_path)
    }

    pub fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        self.render()?;

        loop {
            if let ShouldQuit::Yes = self.process_events()? {
                break;
            }
            self.update()?;
            self.render()?;
        }

        self.terminate()
    }

    fn terminate(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(All), MoveTo(0, 0))?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        while let Some(message) = self.queue.pop_front() {
            self.app.update(message, &mut self.queue)?;
        }
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
            self.app
                .process_event(&event, &mut self.queue, &mut self.context)
        } else {
            Ok(ShouldQuit::No)
        }
    }
}
