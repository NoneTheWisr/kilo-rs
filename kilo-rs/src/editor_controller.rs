use anyhow::Result;
use kilo_rs_backend::editor::Editor;

use crate::{
    bottom_bar::{self, BottomBarMessage, NotificationKind, PromptKind},
    runner::MessageQueue,
    shared::SharedContext,
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

    Save,
    SaveAs(String),
}

impl EditorControllerComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        message: EditorControllerMessage,
        queue: &mut MessageQueue,
        context: &mut SharedContext,
    ) -> Result<()> {
        use EditorControllerMessage::*;

        if let Save = message {
            if context.editor.get_file_name().is_some() {
                context.editor.save_file()?;

                queue.push_front(SAVE_NOTIFICATION_MESSAGE);
            } else {
                queue.push_front(BottomBarMessage::DisplayPrompt(PromptKind::SaveAs));
            }
        } else if let SaveAs(path) = message {
            context.editor.save_file_as(&path).unwrap();

            queue.push_front(SAVE_NOTIFICATION_MESSAGE);
            queue.push_front(make_update_bottom_bar_message(&context.editor));
        } else {
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
                SaveAs(_) => unreachable!(),
                Save => unreachable!(),
            };

            queue.push_front(make_update_text_area_message(&context.editor));
            queue.push_front(make_update_bottom_bar_message(&context.editor));
        }

        Ok(())
    }
}

fn make_update_bottom_bar_message(editor: &Editor) -> BottomBarMessage {
    BottomBarMessage::UpdateStatus(bottom_bar::StatusUpdate {
        file_name: editor.get_file_name().cloned(),
        dirty: editor.is_buffer_dirty(),
        cursor_line: editor.get_view_cursor().line.saturating_add(1),
        line_count: editor.get_buffer_line_count(),
    })
}

fn make_update_text_area_message(editor: &Editor) -> TextAreaMessage {
    TextAreaMessage::Update(text_area::UpdateMessage {
        lines: Box::new(editor.get_view_contents()),
        cursor: editor.get_view_cursor(),
    })
}

const SAVE_NOTIFICATION_MESSAGE: BottomBarMessage =
    BottomBarMessage::DisplayNotification(NotificationKind::SaveSuccess);
