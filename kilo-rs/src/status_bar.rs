use std::io;

use crossterm::cursor::MoveTo;
use crossterm::event::KeyEvent;
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};

use kilo_rs_backend::core::Location;

use crate::shared::SharedContext;

pub struct StatusBarComponent;

impl StatusBarComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        writer: &mut impl io::Write,
        context: &SharedContext,
    ) -> anyhow::Result<()> {
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

    pub fn cursor(&self, _context: &SharedContext) -> Option<Location> {
        None
    }

    pub fn process_event(
        &mut self,
        _event: &KeyEvent,
        _context: &mut SharedContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
