use std::io::Write;
use std::iter::zip;
use std::ops::Range;

use anyhow::Result;

use syntect::highlighting::Style;
use syntect::util::as_24_bit_terminal_escaped;

use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{Clear, ClearType::UntilNewLine};

use kilo_rs_backend::core::Location;
use kilo_rs_backend::editor::Editor;

use crate::editor_controller::EditorControllerMessage;
use crate::runner::MessageQueue;
use crate::term_utils::{Cursor, MoveToCursor};

pub enum TextAreaMessage {
    Update(UpdateMessage),
}

pub struct UpdateMessage {
    pub lines: Box<dyn Iterator<Item = String> + Send>,
    pub cursor: kilo_rs_backend::core::Location,
    pub search_mode: bool,
    pub highlighting: Option<Box<dyn Iterator<Item = Vec<(Style, Range<usize>)>> + Send>>,
}

pub struct TextAreaComponent {
    lines: Vec<String>,
    highlighting: Option<Vec<Vec<(Style, Range<usize>)>>>,
    cursor: Cursor,
    search_mode: bool,
}

impl TextAreaComponent {
    pub fn new(editor: &Editor) -> Self {
        let Location { line, col } = editor.get_view_cursor();

        let (lines, highlighting) = editor.get_view_contents();
        let lines = lines.collect();
        let highlighting = highlighting.map(Iterator::collect);

        let cursor = Cursor::new(line as u16, col as u16);
        let search_mode = editor.is_search_mode_active();

        Self {
            lines,
            cursor,
            search_mode,
            highlighting,
        }
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<Option<Cursor>> {
        queue!(writer, MoveTo(0, 0))?;

        if let Some(highlighting) = &self.highlighting {
            for (line, highlighting) in zip(&self.lines, highlighting) {
                let whatever: Vec<_> = highlighting
                    .into_iter()
                    .map(|&(style, Range { start, end })| (style, &line.as_str()[start..end]))
                    .collect();
                queue!(
                    writer,
                    Print(as_24_bit_terminal_escaped(&whatever[..], true))
                )?;
                queue!(writer, Clear(UntilNewLine))?;
                queue!(writer, Print("\r\n"))?;
            }
        } else {
            for line in &self.lines {
                queue!(writer, Print(line))?;
                queue!(writer, Clear(UntilNewLine))?;
                queue!(writer, Print("\r\n"))?;
            }
        }

        if self.search_mode {
            queue!(writer, MoveToCursor(self.cursor))?;
            queue!(writer, Print(self.get_char_at_cursor().negative()))?;
        }

        Ok(Some(self.cursor))
    }

    pub fn update(&mut self, message: TextAreaMessage) -> Result<()> {
        let TextAreaMessage::Update(message) = message;

        self.lines = message.lines.collect();
        let Location { line, col } = message.cursor;
        self.cursor = Cursor::new(line as u16, col as u16);
        self.search_mode = message.search_mode;
        self.highlighting = message.highlighting.map(Iterator::collect);

        Ok(())
    }

    pub fn process_event(&mut self, event: KeyEvent, queue: &mut MessageQueue) -> Result<()> {
        use EditorControllerMessage::*;
        use KeyCode::*;
        use KeyModifiers as KM;

        let KeyEvent { code, modifiers } = event;
        let message = match (modifiers, code) {
            (KM::NONE, Up) => MoveCursorUp,
            (KM::NONE, Down) => MoveCursorDown,
            (KM::NONE, Left) => MoveCursorLeft,
            (KM::NONE, Right) => MoveCursorRight,

            (KM::NONE, Home) => MoveCursorToLineStart,
            (KM::NONE, End) => MoveCursorToLineEnd,

            (KM::NONE, PageUp) => MoveOneViewUp,
            (KM::NONE, PageDown) => MoveOneViewDown,

            (KM::CONTROL, PageUp) => MoveCursorToBufferTop,
            (KM::CONTROL, PageDown) => MoveCursorToBufferBottom,

            (KM::NONE, Backspace) => RemoveCharBehind,
            (KM::NONE, Delete) => RemoveCharInFront,

            (KM::NONE, Char(c)) => InsertChar(c),
            (KM::SHIFT, Char(c)) => InsertChar(c.to_ascii_uppercase()),

            (KM::NONE, Enter) => InsertLine,

            _ => return Ok(()),
        };

        queue.push(message);
        Ok(())
    }

    fn get_char_at_cursor(&self) -> char {
        self.lines[self.cursor.row as usize]
            .chars()
            .nth(self.cursor.col as usize)
            .unwrap()
    }
}
