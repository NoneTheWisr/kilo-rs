use std::io::Write;
use std::time::Instant;

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};

use crate::app::{AppMessage, Focus};
use crate::editor_controller::EditorControllerMessage;
use crate::runner::MessageQueue;
use crate::shared::{Rectangle, SharedContext};
use crate::term_utils::Cursor;

pub enum BottomBarMessage {
    UpdateStatus(StatusUpdate),
    DisplayPrompt(PromptKind),
    DisplayNotification(NotificationKind),
}

pub struct StatusUpdate {
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
    pub fn new(rect: Rectangle, context: &SharedContext) -> Self {
        Self {
            status_info: StatusInfo {
                buffer_name: context
                    .editor
                    .get_file_name()
                    .cloned()
                    .unwrap_or("[Scratch]".into()),
                dirty: context.editor.is_buffer_dirty(),
                cursor_line: context.editor.get_view_cursor().line + 1,
                line_count: context.editor.get_buffer_line_count(),
            },
            prompt_info: None,
            notification_info: None,
            rect,
        }
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        if let Some(PromptInfo { message, input, .. }) = &self.prompt_info {
            let message = format!("{message} {input}");

            let width = self.rect.width() as usize;
            let status_bar = format!("{message:0$.1$}", width, width.saturating_sub(1));

            queue!(writer, MoveTo(self.rect.left, self.rect.top))?;
            queue!(writer, PrintStyledContent(status_bar.negative()))?;
        } else if let Some(NotificationInfo { message, .. }) = &self.notification_info {
            let width = self.rect.width() as usize;
            let status_bar = format!("{message:0$.0$}", width);

            queue!(writer, MoveTo(self.rect.left, self.rect.top))?;
            queue!(writer, PrintStyledContent(status_bar.negative()))?;
        } else {
            let dirty = if self.status_info.dirty { "[+]" } else { "" };
            let left_part = format!("{:.20}{dirty}", self.status_info.buffer_name,);
            let right_part = format!(
                "{}/{}",
                self.status_info.cursor_line, self.status_info.line_count,
            );
            let total_len = left_part.len() + right_part.len();

            let width = self.rect.width() as usize;
            let bottom_bar = if total_len <= width {
                left_part + &" ".repeat(width - total_len) + &right_part
            } else {
                format!("{left_part:0$.0$}", width)
            };

            queue!(writer, MoveTo(self.rect.left, self.rect.top))?;
            queue!(writer, PrintStyledContent(bottom_bar.negative()))?;
        }

        Ok(())
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
                self.status_info.buffer_name = status.file_name.unwrap_or("[Scratch]".into());
                self.status_info.dirty = status.dirty;
            }
            DisplayPrompt(prompt_kind) => {
                if let PromptKind::Find = prompt_kind {
                    queue.push_front(EditorControllerMessage::StartSearch);
                }
                self.prompt_info = Some(PromptInfo::new(prompt_kind));
                queue.push_front(Focus::BottomBar);
            }
            DisplayNotification(notification_kind) => {
                self.notification_info = Some(NotificationInfo::new(notification_kind));
            }
        }

        Ok(())
    }

    pub fn cursor(&self) -> Option<Cursor> {
        if let Some(PromptInfo { message, input, .. }) = &self.prompt_info {
            let message = format!("{message} {input}");

            let width = self.rect.width() as usize;
            let status_bar = format!("{message:.0$}", width.saturating_sub(1));

            Some(Cursor::new(self.rect.top, status_bar.len() as u16))
        } else {
            None
        }
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

                    (KM::NONE, Backspace) => {
                        input.pop();
                    }
                    (KM::NONE, Enter) => {
                        let prompt_info = self.prompt_info.take().unwrap();

                        queue.push_front(Focus::TextArea);
                        let editor_message = match prompt_info.kind {
                            SaveAs => EditorControllerMessage::SaveAs(prompt_info.input),
                            Open => EditorControllerMessage::Open(prompt_info.input),
                            _ => unreachable!(),
                        };
                        queue.push_front(editor_message);
                    }

                    _ => {}
                },
                Find => match (modifiers, code) {
                    (KM::NONE, Char(c)) => {
                        input.push(c);
                        queue.push_front(EditorControllerMessage::SetSearchPattern(input.clone()))
                    }

                    (KM::NONE, Backspace) => {
                        input.pop();
                        queue.push_front(EditorControllerMessage::SetSearchPattern(input.clone()))
                    }
                    (KM::NONE, Enter) => {
                        self.prompt_info = None;

                        queue.push_front(EditorControllerMessage::FinishSearch);
                        queue.push_front(Focus::TextArea);
                    }
                    (KM::NONE, Esc) => {
                        self.prompt_info = None;

                        queue.push_front(EditorControllerMessage::CancelSearch);
                        queue.push_front(Focus::TextArea);
                    }
                    (KM::NONE, Right) => {
                        queue.push_front(EditorControllerMessage::NextSearchResult);
                        queue.push_front(EditorControllerMessage::SetSearchDirection(true));
                    }
                    (KM::NONE, Left) => {
                        queue.push_front(EditorControllerMessage::NextSearchResult);
                        queue.push_front(EditorControllerMessage::SetSearchDirection(false));
                    }

                    _ => {}
                },
                ConfirmQuit => match (modifiers, code) {
                    (_, Char('y') | Char('q')) => {
                        queue.push_front(AppMessage::Quit);
                    }
                    (_, Char('n') | Esc) => {
                        self.prompt_info = None;
                        queue.push_front(Focus::TextArea);
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

impl PromptInfo {
    fn new(prompt_kind: PromptKind) -> Self {
        Self {
            message: match &prompt_kind {
                PromptKind::SaveAs => SAVE_AS_MESSAGE.into(),
                PromptKind::ConfirmQuit => CONFIRM_QUIT_MESSAGE.into(),
                PromptKind::Open => OPEN_MESSAGE.into(),
                PromptKind::Find => FIND_MESSAGE.into(),
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
                SaveSuccess => SAVE_SUCCESS_MESSAGE.into(),
                OpenFailure => OPEN_FAILURE_MESSAGE.into(),
            },
            start: Instant::now(),
        }
    }
}

const SAVE_AS_MESSAGE: &str = "[Save As] Enter file path:";
const OPEN_MESSAGE: &str = "[Open] Enter file path:";
const FIND_MESSAGE: &str = "[Find]:";
const CONFIRM_QUIT_MESSAGE: &str =
    "[Warning] The buffer has unsaved changes. Are you sure you want to quit [y(q)\\n]?";

const SAVE_SUCCESS_MESSAGE: &str = "[Success] The buffer has been saved";
const OPEN_FAILURE_MESSAGE: &str = "[Fail] Couldn't open the file";
