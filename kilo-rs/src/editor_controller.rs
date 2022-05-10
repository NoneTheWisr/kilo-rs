use crate::{shared::SharedContext, text_area::TextAreaMessage};

pub struct EditorControllerComponent;

pub enum EditorControllerMessage {
    UpdateView {
        lines: Box<dyn Iterator<Item = String> + Send>,
        cursor: kilo_rs_backend::core::Location,
    },
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

            let responce = EditorControllerMessage::UpdateView {
                lines: Box::new(context.editor.get_view_contents()),
                cursor: context.editor.get_view_cursor(),
            };

            Some(Box::new(|| Some(Box::new(responce))))
        } else {
            None
        }
    }
}
