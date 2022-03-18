use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{event, terminal};
use std::time::Duration;
use anyhow::Result;

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

pub struct Kilo;

impl Kilo {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        loop {
            let mut key = None;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(event) = event::read()? {
                    key = Some(event);
                }
            }

            let key = key;
            match key {
                Some(key) => print!("{:?}\r\n", key),
                None => print!("None\r\n"),
            }

            if matches!(
                key,
                Some(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                })
            ) {
                break;
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    Kilo::new().run()
}
