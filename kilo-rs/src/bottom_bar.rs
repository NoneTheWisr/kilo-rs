use std::io::Write;

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::KeyEvent;
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};

use crate::shared::{Rectangle, SharedContext};
use crate::term_utils::Cursor;

pub enum BottomBarMessage {
    Update(UpdateMessage),
}

pub struct UpdateMessage {
    pub file_name: Option<String>,
    pub cursor_line: usize,
    pub line_count: usize,
}

pub struct BottomBarComponent {
    buffer_name: String,
    cursor_line: usize,
    line_count: usize,
    rect: Rectangle,
}

impl BottomBarComponent {
    pub fn new(rect: Rectangle, context: &SharedContext) -> Self {
        Self {
            buffer_name: context
                .editor
                .get_file_name()
                .cloned()
                .unwrap_or("[Scratch]".into()),
            cursor_line: context.editor.get_view_cursor().line + 1,
            line_count: context.editor.get_buffer_line_count(),
            rect,
        }
    }

    pub fn render(&self, writer: &mut impl Write, context: &SharedContext) -> Result<()> {
        let left_part = format!("{:.20}", self.buffer_name);
        let right_part = format!("{}/{}", self.cursor_line, self.line_count,);
        let total_len = left_part.len() + right_part.len();

        let view_width = context.editor.get_view_width();
        let bottom_bar = if total_len <= view_width {
            left_part + &" ".repeat(view_width - total_len) + &right_part
        } else {
            format!("{left_part:0$.0$}", view_width)
        };

        queue!(writer, MoveTo(self.rect.left, self.rect.top))?;
        queue!(writer, PrintStyledContent(bottom_bar.negative()))?;
        Ok(())
    }

    pub fn update(&mut self, message: BottomBarMessage) -> Result<()> {
        let BottomBarMessage::Update(message) = message;

        self.cursor_line = message.cursor_line;
        self.line_count = message.line_count;
        self.buffer_name = message.file_name.unwrap_or("[Scratch]".into());

        Ok(())
    }

    pub fn cursor(&self, _context: &SharedContext) -> Option<Cursor> {
        None
    }

    pub fn process_event(&mut self, _event: &KeyEvent, _context: &mut SharedContext) -> Result<()> {
        Ok(())
    }
}
