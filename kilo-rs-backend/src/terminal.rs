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
