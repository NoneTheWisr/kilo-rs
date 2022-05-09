use std::io::Write;

use anyhow::Result;

use kilo_rs_backend::core::Location;
use rustea::crossterm::{
    cursor::MoveTo,
    event::{KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::Print,
    terminal::{Clear, ClearType::UntilNewLine},
};

use crate::{shared::SharedContext, term_utils::Cursor};

pub struct TextAreaComponent;

impl TextAreaComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, writer: &mut impl Write, context: &SharedContext) -> Result<()> {
        queue!(writer, MoveTo(0, 0))?;

        for line in context.editor.get_view_contents() {
            queue!(writer, Print(line))?;
            queue!(writer, Clear(UntilNewLine))?;
            queue!(writer, Print("\r\n"))?;
        }

        Ok(())
    }

    pub fn cursor(&self, context: &SharedContext) -> Option<Cursor> {
        let Location { line, col } = context.editor.get_view_cursor();
        Some(Cursor::new(line as u16, col as u16))
    }

    pub fn update(
        &mut self,
        msg: rustea::Message,
        context: &mut SharedContext,
    ) -> Option<rustea::Command> {
        if let Some(&KeyEvent { modifiers, code }) = msg.downcast_ref::<KeyEvent>() {
            use KeyCode::*;
            use KeyModifiers as KM;

            match (modifiers, code) {
                (KM::NONE, Up) => context.editor.move_cursor_up(),
                (KM::NONE, Down) => context.editor.move_cursor_down(),
                (KM::NONE, Left) => context.editor.move_cursor_left(),
                (KM::NONE, Right) => context.editor.move_cursor_right(),

                (KM::NONE, Home) => context.editor.move_cursor_to_line_start(),
                (KM::NONE, End) => context.editor.move_cursor_to_line_end(),

                (KM::NONE, PageUp) => context.editor.move_one_view_up(),
                (KM::NONE, PageDown) => context.editor.move_one_view_down(),

                (KM::CONTROL, PageUp) => context.editor.move_cursor_to_buffer_top(),
                (KM::CONTROL, PageDown) => context.editor.move_cursor_to_buffer_bottom(),

                (KM::NONE, Backspace) => context.editor.remove_char_behind(),
                (KM::NONE, Delete) => context.editor.remove_char_in_front(),

                (KM::NONE, Char(c)) => context.editor.insert_char(c),
                (KM::NONE, Enter) => context.editor.insert_line(),

                _ => {}
            }
        }

        None
    }
}
