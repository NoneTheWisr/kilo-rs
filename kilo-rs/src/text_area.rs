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
    app::message_command, editor_controller::EditorControllerMessage, shared::SharedContext,
    term_utils::Cursor,
};

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

    pub fn update(&mut self, message: TextAreaMessage) -> Option<rustea::Command> {
        let TextAreaMessage::Update(message) = message;

        self.lines = message.lines.collect();
        let Location { line, col } = message.cursor;
        self.cursor = Cursor::new(line as u16, col as u16);

        None
    }

    pub fn process_events(&mut self, event: KeyEvent) -> Option<rustea::Command> {
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

            _ => return None,
        };

        Some(message_command(message))
    }
}

fn get_editor_lines(editor: &Editor) -> Vec<String> {
    editor.get_view_contents().collect()
}

fn get_editor_cursor(editor: &Editor) -> Cursor {
    let Location { line, col } = editor.get_view_cursor();
    Cursor::new(line as u16, col as u16)
}
