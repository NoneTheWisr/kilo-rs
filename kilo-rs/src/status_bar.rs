use std::io::Write;

use anyhow::Result;

use rustea::crossterm::{
    cursor::MoveTo,
    queue,
    style::{PrintStyledContent, Stylize},
};

use crate::{shared::SharedContext, term_utils::Cursor};

pub struct StatusBarComponent;

impl StatusBarComponent {
    pub fn new() -> Self {
        Self
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

    pub fn update(
        &mut self,
        _msg: rustea::Message,
        _context: &mut SharedContext,
    ) -> Option<rustea::Command> {
        None
    }
}
