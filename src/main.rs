use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{event, terminal};
use std::time::Duration;

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    CrosstermError(crossterm::ErrorKind),
    KiloError(String),
}

impl From<crossterm::ErrorKind> for Error {
    fn from(error_kind: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(error_kind)
    }
}

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
