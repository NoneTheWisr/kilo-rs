use anyhow::Result;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType::*};
use crossterm::queue;
use std::io::{self, BufWriter, Stdout, Write};

pub struct Kilo {
    cursor: Cursor,
    dimentions: Dimentions,
    is_running: bool,
    stdout: BufWriter<Stdout>,
}

impl Kilo {
    pub fn new() -> Result<Self> {
        Ok(Self {
            cursor: Cursor::new(0, 0),
            dimentions: Dimentions::from(terminal::size()?),
            is_running: true,
            stdout: BufWriter::new(io::stdout()),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        while self.is_running {
            self.render()?;
            self.process_input()?;
        }

        Ok(())
    }

    fn terminate(&mut self) -> Result<()> {
        self.is_running = false;
        queue!(self.stdout, Clear(All), MoveTo(0, 0))?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        queue!(self.stdout, Hide)?;
        queue!(self.stdout, MoveTo(0, 0))?;

        self.render_rows()?;

        queue!(self.stdout, Show)?;
        queue!(self.stdout, MoveTo(self.cursor.x, self.cursor.y))?;

        self.stdout.flush()?;
        Ok(())
    }

    fn render_rows(&mut self) -> Result<()> {
        for row in 0..self.dimentions.height {
            if row == self.dimentions.height / 3 {
                let message_row = self.make_message_row();
                queue!(self.stdout, Print(message_row))?;
            } else {
                queue!(self.stdout, Print("~"))?;
            }

            queue!(self.stdout, Clear(UntilNewLine))?;
            if row < self.dimentions.height - 1 {
                queue!(self.stdout, Print("\r\n"))?;
            }
        }

        Ok(())
    }

    fn make_message_row(&self) -> String {
        let message = "kilo-rs (week 2)";
        let width_bound = std::cmp::min(message.len(), self.dimentions.width as usize);
        let message = &message[..width_bound];

        let padding_len = (self.dimentions.width as usize - message.len()) / 2;
        if padding_len > 0 {
            format!("~{}{message}", " ".repeat(padding_len - 1))
        } else {
            String::from(message)
        }
    }

    #[rustfmt::skip]
    fn process_input(&mut self) -> Result<()> {
        if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
            use event::KeyCode::*;
            use event::KeyModifiers as KM;

            match (modifiers, code) {
                (KM::NONE, Up)       => self.move_cursor_up(),
                (KM::NONE, Down)     => self.move_cursor_down(),
                (KM::NONE, Left)     => self.move_cursor_left(),
                (KM::NONE, Right)    => self.move_cursor_right(),

                (KM::NONE, Home)     => self.move_cursor_to_line_start(),
                (KM::NONE, End)      => self.move_cursor_to_line_end(),

                (KM::NONE, PageUp)   => self.move_cursor_to_screen_top(),
                (KM::NONE, PageDown) => self.move_cursor_to_screen_bottom(),

                (KM::CONTROL, Char('q')) => self.terminate()?,

                _ => {},
            }
        }

        Ok(())
    }

    fn move_cursor_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor.y + 1 < self.dimentions.height {
            self.cursor.y += 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor.x + 1 < self.dimentions.width {
            self.cursor.x += 1;
        }
    }

    fn move_cursor_to_line_start(&mut self) {
        self.cursor.x = 0;
    }

    fn move_cursor_to_line_end(&mut self) {
        self.cursor.x = self.dimentions.width.saturating_sub(1);
    }

    fn move_cursor_to_screen_top(&mut self) {
        for _ in 0..self.dimentions.height {
            self.move_cursor_up();
        }
    }

    fn move_cursor_to_screen_bottom(&mut self) {
        for _ in 0..self.dimentions.height {
            self.move_cursor_down();
        }
    }
}

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

struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

struct Dimentions {
    width: u16,
    height: u16,
}

impl From<(u16, u16)> for Dimentions {
    fn from((columns, rows): (u16, u16)) -> Self {
        Self {
            width: columns,
            height: rows,
        }
    }
}

fn main() -> Result<()> {
    Kilo::new()?.run()
}
