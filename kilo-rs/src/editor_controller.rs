use anyhow::Result;
use kilo_rs_backend::editor::Editor;

use crate::{
    bottom_bar::{
        BottomBarMessage::{
            self, DisplayNotification, DisplayPrompt, UpdateStatus as BottomBarUpdateMessage,
        },
        NotificationKind, PromptKind, StatusUpdate as BottomBarUpdate,
    },
    runner::MessageQueue,
    shared::SharedContext,
    text_area::{
        TextAreaMessage::{self, Update as TextAreaUpdateMessage},
        UpdateMessage as TextAreaUpdate,
    },
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

    Open(String),

    StartSearch,
    FinishSearch,
    CancelSearch,
    SetSearchPattern(String),
    SetSearchDirection(bool),
    NextSearchResult,
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

        match message {
            MoveCursorUp => {
                context.editor.move_cursor_up();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorDown => {
                context.editor.move_cursor_down();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorLeft => {
                context.editor.move_cursor_left();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorRight => {
                context.editor.move_cursor_right();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorToLineStart => {
                context.editor.move_cursor_to_line_start();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorToLineEnd => {
                context.editor.move_cursor_to_line_end();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveOneViewUp => {
                context.editor.move_one_view_up();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveOneViewDown => {
                context.editor.move_one_view_down();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorToBufferTop => {
                context.editor.move_cursor_to_buffer_top();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            MoveCursorToBufferBottom => {
                context.editor.move_cursor_to_buffer_bottom();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            RemoveCharBehind => {
                context.editor.remove_char_behind();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            RemoveCharInFront => {
                context.editor.remove_char_in_front();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            InsertChar(c) => {
                context.editor.insert_char(c.clone());
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            InsertLine => {
                context.editor.insert_line();
                update_text_area_and_bottom_bar(queue, &context.editor)
            }
            Save => {
                if context.editor.get_file_name().is_some() {
                    context.editor.save_file()?;
                    queue.push(DisplayNotification(NotificationKind::SaveSuccess));
                } else {
                    queue.push(DisplayPrompt(PromptKind::SaveAs));
                }
            }
            SaveAs(path) => {
                context.editor.save_file_as(&path)?;
                queue.push(DisplayNotification(NotificationKind::SaveSuccess));
                queue.push(make_update_bottom_bar_message(&context.editor));
            }
            Open(path) => {
                if let Err(_) = context.editor.open_file(&path) {
                    queue.push(DisplayNotification(NotificationKind::OpenFailure));
                } else {
                    update_text_area_and_bottom_bar(queue, &context.editor);
                }
            }
            StartSearch => context.editor.start_search(),
            FinishSearch => {
                context.editor.finish_search();
                update_text_area_and_bottom_bar(queue, &context.editor);
            }
            CancelSearch => {
                context.editor.cancel_search();
                update_text_area_and_bottom_bar(queue, &context.editor);
            }
            SetSearchPattern(pattern) => {
                context.editor.set_search_pattern(&pattern);
                update_text_area_and_bottom_bar(queue, &context.editor);
            }
            SetSearchDirection(forward) => {
                context.editor.set_search_direction(forward);
            }
            NextSearchResult => {
                context.editor.next_search_result();
                update_text_area_and_bottom_bar(queue, &context.editor);
            }
        }

        Ok(())
    }
}

fn update_text_area_and_bottom_bar(queue: &mut MessageQueue, editor: &Editor) {
    queue.push(make_update_text_area_message(editor));
    queue.push(make_update_bottom_bar_message(editor));
}

fn make_update_bottom_bar_message(editor: &Editor) -> BottomBarMessage {
    BottomBarUpdateMessage(BottomBarUpdate {
        file_name: editor.get_file_name().cloned(),
        dirty: editor.is_buffer_dirty(),
        cursor_line: editor.get_view_cursor().line.saturating_add(1),
        line_count: editor.get_buffer_line_count(),
    })
}

fn make_update_text_area_message(editor: &Editor) -> TextAreaMessage {
    TextAreaUpdateMessage(TextAreaUpdate {
        lines: Box::new(editor.get_view_contents()),
        cursor: editor.get_view_cursor(),
        search_mode: editor.is_search_mode_active(),
    })
}
