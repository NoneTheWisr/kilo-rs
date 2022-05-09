use anyhow::Result;
use rustea::crossterm::{terminal, Command};

pub struct RawModeOverride;

impl RawModeOverride {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeOverride {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
    }
}

pub struct Cursor {
    pub row: u16,
    pub col: u16,
}

impl Cursor {
    pub fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }
}

pub struct MoveTo(pub Cursor);

impl Command for MoveTo {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        rustea::crossterm::cursor::MoveTo(self.0.col, self.0.row).write_ansi(f)
    }
}
