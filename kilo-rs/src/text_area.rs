use std::io::Write;

use anyhow::Result;

use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType::UntilNewLine};

use kilo_rs_backend::core::Location;
use kilo_rs_backend::editor::Editor;

use crate::editor_controller::EditorControllerMessage;
use crate::runner::MessageQueue;
use crate::shared::SharedContext;
use crate::term_utils::Cursor;

pub enum TextAreaMessage {
    Update(UpdateMessage),
}

pub struct UpdateMessage {
    pub lines: Box<dyn Iterator<Item = String> + Send>,
    pub cursor: kilo_rs_backend::core::Location,
}

pub struct TextAreaComponent {
    lines: Vec<String>,
    cursor: Cursor,
}

impl TextAreaComponent {
    pub fn new(context: &SharedContext) -> Self {
        Self {
            lines: get_editor_lines(&context.editor),
            cursor: get_editor_cursor(&context.editor),
        }
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        queue!(writer, MoveTo(0, 0))?;

        for line in &self.lines {
            queue!(writer, Print(line))?;
            queue!(writer, Clear(UntilNewLine))?;
            queue!(writer, Print("\r\n"))?;
        }

        Ok(())
    }

    pub fn cursor(&self) -> Option<Cursor> {
        Some(self.cursor)
    }

    pub fn update(&mut self, message: TextAreaMessage) -> Result<()> {
        let TextAreaMessage::Update(message) = message;

        self.lines = message.lines.collect();
        let Location { line, col } = message.cursor;
        self.cursor = Cursor::new(line as u16, col as u16);

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
            (KM::NONE, Enter) => InsertLine,

            _ => return Ok(()),
        };

        queue.push_front(message);
        Ok(())
    }
}

fn get_editor_lines(editor: &Editor) -> Vec<String> {
    editor.get_view_contents().collect()
}

fn get_editor_cursor(editor: &Editor) -> Cursor {
    let Location { line, col } = editor.get_view_cursor();
    Cursor::new(line as u16, col as u16)
}
