use std::collections::VecDeque;
use std::io::{self, BufWriter, Stdout, Write};

use anyhow::{Context, Result};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event};
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType::All};

use crate::app::{App, AppMessage};
use crate::term_utils::{MoveToCursor, RawModeOverride};

pub struct AppRunner {
    app: App,
    stdout: BufWriter<Stdout>,
    queue: MessageQueue,
}

pub enum ShouldQuit {
    Yes,
    No,
}

pub type MessageQueue = VecDeque<AppMessage>;

impl AppRunner {
    pub fn new(app: App) -> Self {
        Self {
            app,
            stdout: BufWriter::new(io::stdout()),
            queue: MessageQueue::new(),
        }
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

        self.app.render(&mut self.stdout)?;

        let cursor = self.app.cursor().context("failed to get cursor location")?;
        queue!(self.stdout, MoveToCursor(cursor))?;

        queue!(self.stdout, Show)?;
        self.stdout.flush()?;

        Ok(())
    }

    fn process_events(&mut self) -> Result<ShouldQuit> {
        if let Event::Key(event) = event::read()? {
            self.app.process_event(&event, &mut self.queue)
        } else {
            Ok(ShouldQuit::No)
        }
    }
}
