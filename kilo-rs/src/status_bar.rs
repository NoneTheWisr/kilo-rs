use std::io::Write;

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::KeyEvent;
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};

use crate::shared::{Rectangle, SharedContext};
use crate::term_utils::Cursor;

pub struct StatusBarComponent {
    rect: Rectangle,
}

impl StatusBarComponent {
    pub fn new(rect: Rectangle) -> Self {
        Self { rect }
    }

    pub fn render(&self, writer: &mut impl Write, context: &SharedContext) -> Result<()> {
        let file_name = match context.editor.get_file_name() {
            Some(name) => name,
            None => "[Scratch]",
        };

        let left_part = format!("{:.20}", file_name);
        let right_part = format!(
            "{}/{}",
            context.editor.get_buffer_cursor().line + 1,
            context.editor.get_buffer_line_count()
        );
        let total_len = left_part.len() + right_part.len();

        let view_width = self.rect.width() as usize;
        let status_bar = if total_len <= view_width {
            left_part + &" ".repeat(view_width - total_len) + &right_part
        } else {
            format!("{left_part:0$.0$}", view_width)
        };

        queue!(writer, MoveTo(self.rect.left, self.rect.top))?;
        queue!(writer, PrintStyledContent(status_bar.negative()))?;
        Ok(())
    }

    pub fn cursor(&self, _context: &SharedContext) -> Option<Cursor> {
        None
    }

    pub fn process_event(&mut self, _event: &KeyEvent, _context: &mut SharedContext) -> Result<()> {
        Ok(())
    }
}
