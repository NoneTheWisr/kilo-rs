use anyhow::Result;
use kilo_rs_backend::editor::Editor;

use crate::{
    app::AppMessage,
    bottom_bar::{
        BottomBarMessage::{
            self, DisplayNotification, DisplayPrompt, UpdateStatus as BottomBarUpdateMessage,
        },
        NotificationKind, PromptKind, StatusUpdateMessage as BottomBarUpdate,
    },
    runner::MessageQueue,
    text_area::{
        TextAreaMessage::{self, Update as TextAreaUpdateMessage},
        UpdateMessage as TextAreaUpdate,
    },
};

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

    RequestQuit,
}

pub struct EditorControllerComponent {
    editor: Editor,
}

impl EditorControllerComponent {
    pub fn new(editor: Editor) -> Self {
        Self { editor }
    }

    pub fn update(
        &mut self,
        message: EditorControllerMessage,
        queue: &mut MessageQueue,
    ) -> Result<()> {
        use EditorControllerMessage::*;

        match message {
            MoveCursorUp => {
                self.editor.move_cursor_up();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorDown => {
                self.editor.move_cursor_down();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorLeft => {
                self.editor.move_cursor_left();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorRight => {
                self.editor.move_cursor_right();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorToLineStart => {
                self.editor.move_cursor_to_line_start();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorToLineEnd => {
                self.editor.move_cursor_to_line_end();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveOneViewUp => {
                self.editor.move_one_view_up();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveOneViewDown => {
                self.editor.move_one_view_down();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorToBufferTop => {
                self.editor.move_cursor_to_buffer_top();
                self.update_text_area_and_bottom_bar(queue);
            }
            MoveCursorToBufferBottom => {
                self.editor.move_cursor_to_buffer_bottom();
                self.update_text_area_and_bottom_bar(queue);
            }
            RemoveCharBehind => {
                self.editor.remove_char_behind();
                self.update_text_area_and_bottom_bar(queue);
            }
            RemoveCharInFront => {
                self.editor.remove_char_in_front();
                self.update_text_area_and_bottom_bar(queue);
            }
            InsertChar(c) => {
                self.editor.insert_char(c.clone());
                self.update_text_area_and_bottom_bar(queue);
            }
            InsertLine => {
                self.editor.insert_line();
                self.update_text_area_and_bottom_bar(queue);
            }
            Save => {
                if self.editor.get_file_name().is_some() {
                    self.editor.save_file()?;
                    queue.push(DisplayNotification(NotificationKind::SaveSuccess));
                } else {
                    queue.push(DisplayPrompt(PromptKind::SaveAs));
                }
            }
            SaveAs(path) => {
                self.editor.save_file_as(&path)?;
                queue.push(DisplayNotification(NotificationKind::SaveSuccess));
                queue.push(self.make_update_bottom_bar_message());
            }
            Open(path) => {
                if let Err(_) = self.editor.open_file(&path) {
                    queue.push(DisplayNotification(NotificationKind::OpenFailure));
                } else {
                    self.update_text_area_and_bottom_bar(queue);
                }
            }
            StartSearch => self.editor.start_search(),
            FinishSearch => {
                self.editor.finish_search();
                self.update_text_area_and_bottom_bar(queue);
            }
            CancelSearch => {
                self.editor.cancel_search();
                self.update_text_area_and_bottom_bar(queue);
            }
            SetSearchPattern(pattern) => {
                self.editor.set_search_pattern(&pattern);
                self.update_text_area_and_bottom_bar(queue);
            }
            SetSearchDirection(forward) => {
                self.editor.set_search_direction(forward);
            }
            NextSearchResult => {
                self.editor.next_search_result();
                self.update_text_area_and_bottom_bar(queue);
            }
            RequestQuit => {
                if self.editor.is_buffer_dirty() {
                    queue.push(DisplayPrompt(PromptKind::ConfirmQuit));
                } else {
                    queue.push(AppMessage::Quit)
                }
            }
        }

        Ok(())
    }

    fn update_text_area_and_bottom_bar(&self, queue: &mut MessageQueue) {
        queue.push(self.make_update_text_area_message());
        queue.push(self.make_update_bottom_bar_message());
    }

    fn make_update_bottom_bar_message(&self) -> BottomBarMessage {
        BottomBarUpdateMessage(BottomBarUpdate {
            file_name: self.editor.get_file_name().cloned(),
            dirty: self.editor.is_buffer_dirty(),
            cursor_line: self.editor.get_view_cursor().line.saturating_add(1),
            line_count: self.editor.get_buffer_line_count(),
        })
    }

    fn make_update_text_area_message(&self) -> TextAreaMessage {
        TextAreaUpdateMessage(TextAreaUpdate {
            lines: Box::new(self.editor.get_view_contents()),
            cursor: self.editor.get_view_cursor(),
            search_mode: self.editor.is_search_mode_active(),
        })
    }
}
