use rustea::command;

use crate::{shared::SharedContext, text_area::TextAreaMessage};

pub struct EditorControllerComponent;

pub struct UpdateViewMessage {
    pub lines: Box<dyn Iterator<Item = String> + Send>,
    pub cursor: kilo_rs_backend::core::Location,
}

pub struct UpdateStatusMessage {
    pub file_name: Option<String>,
    pub cursor_line: usize,
    pub line_count: usize,
}

impl EditorControllerComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        msg: rustea::Message,
        context: &mut SharedContext,
    ) -> Option<rustea::Command> {
        use crate::text_area::TextAreaMessage::*;

        if let Some(message) = msg.downcast_ref::<TextAreaMessage>() {
            match message {
                MoveCursorUp => context.editor.move_cursor_up(),
                MoveCursorDown => context.editor.move_cursor_down(),
                MoveCursorLeft => context.editor.move_cursor_left(),
                MoveCursorRight => context.editor.move_cursor_right(),
                MoveCursorToLineStart => context.editor.move_cursor_to_line_start(),
                MoveCursorToLineEnd => context.editor.move_cursor_to_line_end(),
                MoveOneViewUp => context.editor.move_one_view_up(),
                MoveOneViewDown => context.editor.move_one_view_down(),
                MoveCursorToBufferTop => context.editor.move_cursor_to_buffer_top(),
                MoveCursorToBufferBottom => context.editor.move_cursor_to_buffer_bottom(),
                RemoveCharBehind => context.editor.remove_char_behind(),
                RemoveCharInFront => context.editor.remove_char_in_front(),
                InsertChar(c) => context.editor.insert_char(c.clone()),
                InsertLine => context.editor.insert_line(),
            };

            let update_view_message = UpdateViewMessage {
                lines: Box::new(context.editor.get_view_contents()),
                cursor: context.editor.get_view_cursor(),
            };

            let update_status_message = UpdateStatusMessage {
                file_name: context.editor.get_file_name().cloned(),
                cursor_line: context.editor.get_view_cursor().line.saturating_add(1),
                line_count: context.editor.get_buffer_line_count(),
            };

            Some(command::batch(vec![
                Box::new(|| Some(Box::new(update_view_message))),
                Box::new(|| Some(Box::new(update_status_message))),
            ]))
        } else {
            None
        }
    }
}
