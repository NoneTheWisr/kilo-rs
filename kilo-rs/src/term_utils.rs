use anyhow::Result;
use crossterm::terminal;

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

#[derive(Clone, Copy, Default)]
pub struct Cursor {
    pub row: u16,
    pub col: u16,
}

impl Cursor {
    pub fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }
}

pub struct MoveToCursor(pub Cursor);

impl crossterm::Command for MoveToCursor {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        crossterm::cursor::MoveTo(self.0.col, self.0.row).write_ansi(f)
    }
}
