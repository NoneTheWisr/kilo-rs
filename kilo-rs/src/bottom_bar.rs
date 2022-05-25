use std::io::Write;
use std::time::Instant;

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};
use kilo_rs_backend::editor::Editor;

use crate::app::{AppMessage, Focus};
use crate::editor_controller::EditorControllerMessage;
use crate::runner::MessageQueue;
use crate::shared::Rectangle;
use crate::term_utils::Cursor;

pub enum BottomBarMessage {
    UpdateStatus(StatusUpdateMessage),
    DisplayPrompt(PromptKind),
    DisplayNotification(NotificationKind),
}

pub struct StatusUpdateMessage {
    pub file_name: Option<String>,
    pub dirty: bool,
    pub cursor_line: usize,
    pub line_count: usize,
}

pub enum PromptKind {
    SaveAs,
    Open,
    Find,
    ConfirmQuit,
}

pub enum NotificationKind {
    SaveSuccess,
    OpenFailure,
}

pub struct BottomBarComponent {
    status_info: StatusInfo,
    prompt_info: Option<PromptInfo>,
    notification_info: Option<NotificationInfo>,
    rect: Rectangle,
}

struct StatusInfo {
    buffer_name: String,
    dirty: bool,
    cursor_line: usize,
    line_count: usize,
}

struct PromptInfo {
    kind: PromptKind,
    message: String,
    input: String,
}

struct NotificationInfo {
    message: String,
    start: Instant,
}

const NOTIFICATION_DURATION: f32 = 1.0;

impl BottomBarComponent {
    pub fn new(rect: Rectangle, editor: &Editor) -> Self {
        let file_name = editor.get_file_name().cloned();

        let buffer_name = buffer_name(file_name);
        let dirty = editor.is_buffer_dirty();
        let cursor_line = editor.get_view_cursor().line + 1;
        let line_count = editor.get_buffer_line_count();

        Self {
            status_info: StatusInfo {
                buffer_name,
                dirty,
                cursor_line,
                line_count,
            },
            prompt_info: None,
            notification_info: None,
            rect,
        }
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<Option<Cursor>> {
        let view_width = self.rect.width() as usize;

        let (bar, cursor) = if let Some(PromptInfo { message, input, .. }) = &self.prompt_info {
            let message = format!("{message} {input}");
            let message = format!("{message:.0$}", view_width.saturating_sub(1));

            let bar = format!("{message:0$}", view_width);
            let cursor = Cursor::new(self.rect.top, message.len() as u16);

            (bar, Some(cursor))
        } else if let Some(NotificationInfo { message, .. }) = &self.notification_info {
            (format!("{message:0$.0$}", view_width), None)
        } else {
            let StatusInfo {
                buffer_name,
                dirty,
                cursor_line,
                line_count,
            } = &self.status_info;

            let dirty = if *dirty { "[+]" } else { "" };
            let left_part = format!("{:.20}{dirty}", buffer_name);
            let right_part = format!("{}/{}", cursor_line, line_count);

            let total_width = left_part.len() + right_part.len();
            let bar = if total_width <= view_width {
                left_part + &" ".repeat(view_width - total_width) + &right_part
            } else {
                format!("{left_part:0$.0$}", view_width)
            };

            (bar, None)
        };

        queue!(writer, MoveTo(self.rect.left, self.rect.top))?;
        queue!(writer, PrintStyledContent(bar.negative()))?;

        Ok(cursor)
    }

    pub fn update(&mut self, message: BottomBarMessage, queue: &mut MessageQueue) -> Result<()> {
        use BottomBarMessage::*;

        if let Some(NotificationInfo { start, .. }) = &self.notification_info {
            if start.elapsed().as_secs_f32() > NOTIFICATION_DURATION {
                self.notification_info = None;
            }
        }

        match message {
            UpdateStatus(status) => {
                self.status_info.cursor_line = status.cursor_line;
                self.status_info.line_count = status.line_count;
                self.status_info.buffer_name = buffer_name(status.file_name);
                self.status_info.dirty = status.dirty;
            }
            DisplayPrompt(prompt_kind) => {
                if let PromptKind::Find = prompt_kind {
                    queue.push(EditorControllerMessage::StartSearch);
                }
                self.prompt_info = Some(PromptInfo::new(prompt_kind));
                queue.push(Focus::BottomBar);
            }
            DisplayNotification(notification_kind) => {
                self.notification_info = Some(NotificationInfo::new(notification_kind));
            }
        }

        Ok(())
    }

    pub fn process_event(&mut self, event: KeyEvent, queue: &mut MessageQueue) -> Result<()> {
        if let Some(PromptInfo { kind, input, .. }) = &mut self.prompt_info {
            use KeyCode::*;
            use KeyModifiers as KM;
            use PromptKind::*;

            let KeyEvent { code, modifiers } = event;
            match kind {
                SaveAs | Open => match (modifiers, code) {
                    (KM::NONE, Char(c)) => {
                        input.push(c);
                    }
                    (KM::SHIFT, Char(c)) => {
                        input.push(c.to_ascii_uppercase());
                    }

                    (KM::NONE, Backspace) => {
                        input.pop();
                    }
                    (KM::NONE, Enter) => {
                        let prompt_info = self.prompt_info.take().unwrap();

                        queue.push(Focus::TextArea);
                        let editor_message = match prompt_info.kind {
                            SaveAs => EditorControllerMessage::SaveAs(prompt_info.input),
                            Open => EditorControllerMessage::Open(prompt_info.input),
                            _ => unreachable!(),
                        };
                        queue.push(editor_message);
                    }

                    _ => {}
                },
                Find => match (modifiers, code) {
                    (KM::NONE, Char(c)) => {
                        input.push(c);
                        queue.push(EditorControllerMessage::SetSearchPattern(input.clone()))
                    }
                    (KM::SHIFT, Char(c)) => {
                        input.push(c.to_ascii_uppercase());
                        queue.push(EditorControllerMessage::SetSearchPattern(input.clone()))
                    }

                    (KM::NONE, Backspace) => {
                        input.pop();
                        queue.push(EditorControllerMessage::SetSearchPattern(input.clone()))
                    }
                    (KM::NONE, Enter) => {
                        self.prompt_info = None;

                        queue.push(EditorControllerMessage::FinishSearch);
                        queue.push(Focus::TextArea);
                    }
                    (KM::NONE, Esc) => {
                        self.prompt_info = None;

                        queue.push(EditorControllerMessage::CancelSearch);
                        queue.push(Focus::TextArea);
                    }
                    (KM::NONE, Right) => {
                        queue.push(EditorControllerMessage::NextSearchResult);
                        queue.push(EditorControllerMessage::SetSearchDirection(true));
                    }
                    (KM::NONE, Left) => {
                        queue.push(EditorControllerMessage::NextSearchResult);
                        queue.push(EditorControllerMessage::SetSearchDirection(false));
                    }

                    _ => {}
                },
                ConfirmQuit => match (modifiers, code) {
                    (_, Char('q')) => {
                        queue.push(AppMessage::Quit);
                    }
                    (_, Esc) => {
                        self.prompt_info = None;
                        queue.push(Focus::TextArea);
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

fn buffer_name(file_name: Option<String>) -> String {
    file_name.unwrap_or("[Scratch]".into())
}

impl PromptInfo {
    fn new(prompt_kind: PromptKind) -> Self {
        Self {
            message: match &prompt_kind {
                PromptKind::SaveAs => "[Save As] Enter file path:".into(),
                PromptKind::ConfirmQuit => concat!(
                    "[Warning] Buffer has unsaved changes. ",
                    "Are you sure you want to quit [q/Esc]?"
                )
                .into(),
                PromptKind::Open => "[Open] Enter file path:".into(),
                PromptKind::Find => "[Find]:".into(),
            },
            kind: prompt_kind,
            input: String::new(),
        }
    }
}

impl NotificationInfo {
    fn new(notification_kind: NotificationKind) -> Self {
        use NotificationKind::*;
        Self {
            message: match notification_kind {
                SaveSuccess => "[Success] The buffer has been saved".into(),
                OpenFailure => "[Fail] Couldn't open the file".into(),
            },
            start: Instant::now(),
        }
    }
}
