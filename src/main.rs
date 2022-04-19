use std::env;
use std::io::{self, BufWriter, Stdout, Write};

use anyhow::Result;

use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::event::{KeyEvent, Event, self};
use crossterm::terminal::{self, Clear, ClearType::{All, UntilNewLine}};
use crossterm::queue;
use crossterm::cursor::{Hide, MoveTo, Show};

use kilo_rs::{terminal::RawModeOverride, editor::Editor, core::Location};

struct Kilo {
    editor: Editor,
    is_running: bool,
    stdout: BufWriter<Stdout>,
}

impl Kilo {
    fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            editor: Editor::new(width as usize, height as usize),
            is_running: true,
            stdout: BufWriter::new(io::stdout()),
        })
    }
    
    fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.editor.open_file(file_path)
    }

    fn run(&mut self) -> Result<()> {
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
        self.render_status_bar()?;

        let Location { row, col } = self.editor.get_view_cursor();
        queue!(self.stdout, Show)?;
        queue!(self.stdout, MoveTo(col as u16, row as u16))?;

        self.stdout.flush()?;
        Ok(())
    }
    
    fn render_rows(&mut self) -> Result<()>  {
        for row in self.editor.get_view_contents() {
            queue!(self.stdout, Print(row))?;
            queue!(self.stdout, Clear(UntilNewLine))?;
            queue!(self.stdout, Print("\r\n"))?;
        }
        Ok(())
    }
    
    fn render_status_bar(&mut self) -> Result<()> {
        let file_name = match self.editor.get_file_name() {
            Some(name) => name,
            None => "[Scratch]",
        };

        let left_part = format!("{:.20}", file_name);
        let right_part = format!("{}/{}", self.editor.get_buffer_cursor().row + 1, self.editor.get_buffer_line_count());
        let total_len = left_part.len() + right_part.len();

        let view_width = self.editor.get_view_width();
        let status_bar = if total_len <= view_width {
            left_part + &" ".repeat(view_width - total_len) + &right_part
        } else {
            format!("{left_part:0$.0$}", view_width)
        };

        queue!(self.stdout, PrintStyledContent(status_bar.negative()))?;
        Ok(())
    }

    #[rustfmt::skip]
    fn process_input(&mut self) -> Result<()> {
        if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
            use event::KeyCode::*;
            use event::KeyModifiers as KM;

            match (modifiers, code) {
                (KM::NONE, Up)       => self.editor.move_cursor_up(),
                (KM::NONE, Down)     => self.editor.move_cursor_down(),
                (KM::NONE, Left)     => self.editor.move_cursor_left(),
                (KM::NONE, Right)    => self.editor.move_cursor_right(),

                (KM::NONE, Home)     => self.editor.move_cursor_to_line_start(),
                (KM::NONE, End)      => self.editor.move_cursor_to_line_end(),

                (KM::NONE, PageUp)   => self.editor.move_one_view_up(),
                (KM::NONE, PageDown) => self.editor.move_one_view_down(),

                (KM::CONTROL, PageUp)   => self.editor.move_cursor_to_buffer_top(),
                (KM::CONTROL, PageDown) => self.editor.move_cursor_to_buffer_bottom(),

                (KM::CONTROL, Char('q')) => self.terminate()?,
                
                (KM::NONE, Backspace) => self.editor.remove_char_behind(),
                (KM::NONE, Delete) => self.editor.remove_char_in_front(),
                
                (KM::NONE, Char(c)) => self.editor.insert_char(c),
                (KM::NONE, Enter) => self.editor.insert_row(),

                _ => {},
            }
        }        

        Ok(())
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