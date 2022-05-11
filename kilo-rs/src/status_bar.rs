use std::io::Write;

use anyhow::Result;

use rustea::crossterm::{
    cursor::MoveTo,
    queue,
    style::{PrintStyledContent, Stylize},
};

use crate::{shared::SharedContext, term_utils::Cursor};

pub enum StatusBarMessage {
    Update(UpdateMessage),
}

pub struct UpdateMessage {
    pub file_name: Option<String>,
    pub cursor_line: usize,
    pub line_count: usize,
}

pub struct StatusBarComponent {
    buffer_name: String,
    cursor_line: usize,
    line_count: usize,
}

impl StatusBarComponent {
    pub fn new() -> Self {
        Self {
            buffer_name: String::from("[Scratch]"),
            cursor_line: 1,
            line_count: 1,
        }
    }

    pub fn render(&self, writer: &mut impl Write, context: &SharedContext) -> Result<()> {
        let left_part = format!("{:.20}", self.buffer_name);
        let right_part = format!("{}/{}", self.cursor_line, self.line_count,);
        let total_len = left_part.len() + right_part.len();

        let view_width = context.editor.get_view_width();
        let status_bar = if total_len <= view_width {
            left_part + &" ".repeat(view_width - total_len) + &right_part
        } else {
            format!("{left_part:0$.0$}", view_width)
        };

        let status_line = context.editor.get_view_height();
        queue!(writer, MoveTo(0, status_line as u16))?;
        queue!(writer, PrintStyledContent(status_bar.negative()))?;

        Ok(())
    }

    pub fn cursor(&self, _context: &SharedContext) -> Option<Cursor> {
        None
    }

    pub fn update(&mut self, msg: rustea::Message) -> Option<rustea::Command> {
        if let Ok(message) = msg.downcast::<StatusBarMessage>() {
            let StatusBarMessage::Update(message) = *message;

            self.cursor_line = message.cursor_line;
            self.line_count = message.line_count;
            self.buffer_name = message.file_name.unwrap_or("[Scratch]".into());
        }

        None
    }
}
