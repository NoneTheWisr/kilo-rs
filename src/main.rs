use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{event, terminal};
use std::time::Duration;
use anyhow::Result;

struct Kilo;

impl Kilo {
    fn new() -> Self {
        terminal::enable_raw_mode().unwrap();
        Self
    }

    fn run(&mut self) -> Result<()> {
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

impl Drop for Kilo {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
    }
}

fn main() -> Result<()> {
    Kilo::new().run()
}
