use std::io::Write;

use anyhow::Result;

use kilo_rs_backend::{core::Location, editor::Editor};
use rustea::crossterm::{
    cursor::MoveTo,
    event::{KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::Print,
    terminal::{Clear, ClearType::UntilNewLine},
};

use crate::{
    editor_controller::EditorControllerMessage, shared::SharedContext, term_utils::Cursor,
};

pub enum TextAreaMessage {
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,

    MoveCursorToLineStart,
    MoveCursorToLineEnd,

    MoveOneViewUp,
    MoveOneViewDown,

    MoveCursorToBufferTop,
    MoveCursorToBufferBottom,

    RemoveCharBehind,
    RemoveCharInFront,

    InsertChar(char),
    InsertLine,
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

    pub fn update(&mut self, msg: rustea::Message) -> Option<rustea::Command> {
        if let Some(&KeyEvent { modifiers, code }) = msg.downcast_ref::<KeyEvent>() {
            use KeyCode::*;
            use KeyModifiers as KM;
            use TextAreaMessage::*;

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

                _ => return None,
            };

            Some(Box::new(|| Some(Box::new(message))))
        } else if let Ok(message) = msg.downcast::<EditorControllerMessage>() {
            use EditorControllerMessage::*;

            match *message {
                UpdateView { lines, cursor } => {
                    self.lines = lines.collect();

                    let Location { line, col } = cursor;
                    self.cursor = Cursor::new(line as u16, col as u16);
                }
            }

            None
        } else {
            None
        }
    }
}

fn get_editor_lines(editor: &Editor) -> Vec<String> {
    editor.get_view_contents().collect()
}

fn get_editor_cursor(editor: &Editor) -> Cursor {
    let Location { line, col } = editor.get_view_cursor();
    Cursor::new(line as u16, col as u16)
}
