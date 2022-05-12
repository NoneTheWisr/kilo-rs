use rustea::command;

use crate::{
    app::message_command,
    shared::SharedContext,
    status_bar::{self, StatusBarMessage},
    text_area::{self, TextAreaMessage},
};

pub struct EditorControllerComponent;

pub enum EditorControllerMessage {
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

impl EditorControllerComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        message: EditorControllerMessage,
        context: &mut SharedContext,
    ) -> Option<rustea::Command> {
        use EditorControllerMessage::*;

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

        let update_text_area_message = TextAreaMessage::Update(text_area::UpdateMessage {
            lines: Box::new(context.editor.get_view_contents()),
            cursor: context.editor.get_view_cursor(),
        });

        let update_status_bar_message = StatusBarMessage::Update(status_bar::UpdateMessage {
            file_name: context.editor.get_file_name().cloned(),
            cursor_line: context.editor.get_view_cursor().line.saturating_add(1),
            line_count: context.editor.get_buffer_line_count(),
        });

        Some(command::batch(vec![
            message_command(update_text_area_message),
            message_command(update_status_bar_message),
        ]))
    }
}
