use anyhow::Result;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyEvent};
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{self, Clear, ClearType::*};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Stdout, Write};
use std::{env, iter};

const TAB_STOP: usize = 8;

struct Row {
    raw: String,
    rendered: String,
}

impl From<String> for Row {
    fn from(string: String) -> Self {
        let raw = string;
        let mut rendered = String::new();
        for c in raw.chars() {
            if c == '\t' {
                let count = TAB_STOP - (rendered.len() % TAB_STOP);
                rendered.extend(iter::repeat(' ').take(count));
            } else {
                rendered.push(c);
            }
        }
        Self { raw, rendered }
    }
}

pub struct Kilo {
    cursor: Cursor,
    view_dimentions: Dimentions,
    is_running: bool,
    stdout: BufWriter<Stdout>,
    view_offset: Position,
    rows: Vec<Row>,
    file_name: Option<String>,
}

impl Kilo {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;

        Ok(Self {
            cursor: Cursor::new(0, 0, 0),
            view_dimentions: Dimentions::new(width, height - 1),
            is_running: true,
            stdout: BufWriter::new(io::stdout()),
            view_offset: Position::new(0, 0),
            rows: Vec::new(),
            file_name: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let _override = RawModeOverride::new()?;

        while self.is_running {
            self.scroll();
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

    fn scroll(&mut self) {
        self.cursor.rx = 0;
        if (self.cursor.fy as usize) < self.rows.len() {
            let row = &self.rows[self.cursor.fy as usize].raw;
            self.cursor.rx = row.chars().take(self.cursor.fx as usize).fold(0, |i, c| {
                i + if c == '\t' {
                    TAB_STOP - (i % TAB_STOP)
                } else {
                    1
                }
            }) as u16;
        }

        if self.cursor.fy < self.view_offset.y {
            self.view_offset.y = self.cursor.fy;
        } else if self.cursor.fy >= self.view_offset.y + self.view_dimentions.height {
            self.view_offset.y = self.cursor.fy + 1 - self.view_dimentions.height;
        }

        if self.cursor.rx < self.view_offset.x {
            self.view_offset.x = self.cursor.rx;
        } else if self.cursor.rx >= self.view_offset.x + self.view_dimentions.width {
            self.view_offset.x = self.cursor.rx + 1 - self.view_dimentions.width;
        }
    }

    fn render(&mut self) -> Result<()> {
        queue!(self.stdout, Hide)?;
        queue!(self.stdout, MoveTo(0, 0))?;

        self.render_rows()?;
        self.render_status_bar()?;

        queue!(self.stdout, Show)?;
        queue!(
            self.stdout,
            MoveTo(
                self.cursor.rx - self.view_offset.x,
                self.cursor.fy - self.view_offset.y
            )
        )?;

        self.stdout.flush()?;
        Ok(())
    }

    fn render_rows(&mut self) -> Result<()> {
        for row in 0..self.view_dimentions.height {
            let file_row = (row + self.view_offset.y) as usize;

            if file_row >= self.rows.len() {
                if row == self.view_dimentions.height / 3 {
                    let message_row = self.make_message_row();
                    queue!(self.stdout, Print(message_row))?;
                } else {
                    queue!(self.stdout, Print("~"))?;
                }
            } else {
                let row = &self.rows[file_row as usize].rendered;
                let row = if self.view_offset.x as usize >= row.len() {
                    String::new()
                } else {
                    String::from(&row[self.view_offset.x as usize..])
                };
                let trimmed_row = self.trim_to_width(&row);
                queue!(self.stdout, Print(trimmed_row))?;
            }

            queue!(self.stdout, Clear(UntilNewLine))?;
            queue!(self.stdout, Print("\r\n"))?;
        }

        Ok(())
    }

    fn render_status_bar(&mut self) -> Result<()> {
        let file_name = match self.file_name {
            Some(ref name) => name,
            None => "[Scratch]",
        };

        let left_part = format!("{:.20}", file_name);
        let right_part = format!("{}/{}", self.cursor.fy + 1, self.rows.len());
        let total_len = left_part.len() + right_part.len();

        let status_bar = if total_len <= self.view_dimentions.width as usize {
            left_part + &" ".repeat(self.view_dimentions.width as usize - total_len) + &right_part
        } else {
            format!("{left_part:0$.0$}", self.view_dimentions.width as usize)
        };

        queue!(self.stdout, PrintStyledContent(status_bar.negative()))?;
        Ok(())
    }

    fn make_message_row(&self) -> String {
        let message = self.trim_to_width("kilo-rs (week 2)");

        let padding_len = (self.view_dimentions.width as usize - message.len()) / 2;
        if padding_len > 0 {
            format!("~{}{message}", " ".repeat(padding_len - 1))
        } else {
            String::from(message)
        }
    }

    fn trim_to_width<'a>(&self, string: &'a str) -> &'a str {
        let width_bound = std::cmp::min(string.len(), self.view_dimentions.width as usize);
        &string[..width_bound]
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

    pub fn move_cursor_up(&mut self) {
        if self.cursor.fy > 0 {
            self.cursor.fy -= 1;
        }
        self.fix_cursor_past_eol();
    }

    pub fn move_cursor_down(&mut self) {
        if (self.cursor.fy as usize) + 1 < self.rows.len() {
            self.cursor.fy += 1;
        }
        self.fix_cursor_past_eol();
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor.fx > 0 {
            self.cursor.fx -= 1;
        } else if self.cursor.fy > 0 {
            self.cursor.fy -= 1;
            self.cursor.fx = self.rows[self.cursor.fy as usize].raw.len() as u16;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if (self.cursor.fy as usize) < self.rows.len() {
            let row_size = self.rows[self.cursor.fy as usize].raw.len();
            if (self.cursor.fx as usize) < row_size {
                self.cursor.fx += 1;
            } else if (self.cursor.fx as usize) == row_size
                && (self.cursor.fy as usize) + 1 < self.rows.len()
            {
                self.cursor.fy += 1;
                self.cursor.fx = 0;
            }
        }
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor.fx = 0;
    }

    pub fn move_cursor_to_line_end(&mut self) {
        if (self.cursor.fy as usize) < self.rows.len() {
            self.cursor.fx = self.rows[self.cursor.fy as usize].raw.len() as u16;
        }
    }

    pub fn move_cursor_to_screen_top(&mut self) {
        self.cursor.fy = self.view_offset.y;
        for _ in 0..self.view_dimentions.height {
            self.move_cursor_up();
        }
    }

    pub fn move_cursor_to_screen_bottom(&mut self) {
        self.cursor.fy = std::cmp::min(
            (self.view_offset.y + self.view_dimentions.height).saturating_sub(1),
            self.rows.len() as u16,
        );
        for _ in 0..self.view_dimentions.height {
            self.move_cursor_down();
        }
    }

    fn fix_cursor_past_eol(&mut self) {
        if (self.cursor.fy as usize) < self.rows.len() {
            let row_len = self.rows[self.cursor.fy as usize].raw.len();
            if (self.cursor.fx as usize) > row_len {
                self.cursor.fx = row_len as u16;
            }
        }
    }

    pub fn open_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        self.rows = reader
            .lines()
            .map(|line| line.map(Row::from))
            .collect::<Result<_, _>>()?;

        self.file_name = Some(String::from(path));
        Ok(())
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
    fx: u16,
    fy: u16,
    rx: u16,
}

impl Cursor {
    fn new(fx: u16, fy: u16, rx: u16) -> Self {
        Self { fx, fy, rx }
    }
}

struct Position {
    x: u16,
    y: u16,
}

impl Position {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

struct Dimentions {
    width: u16,
    height: u16,
}

impl Dimentions {
    fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

fn main() -> Result<()> {
    let mut kilo = Kilo::new()?;

    let args: Vec<_> = env::args().skip(1).collect();
    match args.len() {
        0 => {}
        1 => kilo.open_file(&args[0])?,
        _ => println!("USAGE: kilo [path_to_file]"),
    }

    kilo.run()
}

#[test]
fn test_row_from_no_tabs() {
    let string = String::from("test  ");
    let row = Row::from(string.clone());
    assert_eq!(row.rendered, row.raw);
    assert_eq!(row.raw, string);
}

#[test]
fn test_row_from_simple_tabs() {
    let row = Row::from(String::from("\t"));
    assert_eq!(row.rendered, " ".repeat(TAB_STOP));
}

#[test]
fn test_row_from_complicated_tabs() {
    let row = Row::from(String::from("    \tthing\tsomething\t."));
    assert_eq!(
        row.rendered,
        String::from("        thing   something       .")
    );
}
