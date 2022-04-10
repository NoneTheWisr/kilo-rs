use std::{iter, env};
use std::io::{BufRead, self, BufReader, BufWriter, Stdout, Write};
use std::fs::File;

use anyhow::Result;

use crossterm::style::Print;
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
        // self.render_status_bar()?;

        let Location { row, col } = self.editor.get_view_relative_cursor_position();
        queue!(self.stdout, Show)?;
        queue!(self.stdout, MoveTo(col as u16, row as u16))?;

        self.stdout.flush()?;
        Ok(())
    }
    
    fn render_rows(&mut self) -> Result<()>  {
        for line in self.editor.get_view_contents() {
            queue!(self.stdout, Print(line))?;
            queue!(self.stdout, Clear(UntilNewLine))?;
            queue!(self.stdout, Print("\r\n"))?;
        }
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