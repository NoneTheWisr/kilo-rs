use std::io::Write;
use std::time::Instant;

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};

use crate::app::Focus;
use crate::editor_controller::EditorControllerMessage;
use crate::runner::MessageQueue;
use crate::shared::{Rectangle, SharedContext};
use crate::term_utils::Cursor;

pub enum BottomBarMessage {
    UpdateStatus(StatusUpdate),
    DisplayPrompt(PromptKind),
}

pub struct StatusUpdate {
    pub file_name: Option<String>,
    pub cursor_line: usize,
    pub line_count: usize,
}

pub enum PromptKind {
    SaveAs,
}

pub struct BottomBarComponent {
    status_info: StatusInfo,
    prompt_info: Option<PromptInfo>,
    notification_info: Option<NotificationInfo>,
    rect: Rectangle,
}

struct StatusInfo {
    buffer_name: String,
    cursor_line: usize,
    line_count: usize,
}

struct PromptInfo {
    message: String,
    input: String,
}

struct NotificationInfo {
    message: String,
    start: Instant,
}

impl BottomBarComponent {
    pub fn new(rect: Rectangle, context: &SharedContext) -> Self {
        Self {
            status_info: StatusInfo {
                buffer_name: context
                    .editor
                    .get_file_name()
                    .cloned()
                    .unwrap_or("[Scratch]".into()),
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
        } else {
            let left_part = format!("{:.20}", self.status_info.buffer_name);
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
        match message {
            BottomBarMessage::UpdateStatus(status) => {
                self.status_info.cursor_line = status.cursor_line;
                self.status_info.line_count = status.line_count;
                self.status_info.buffer_name = status.file_name.unwrap_or("[Scratch]".into());
            }
            BottomBarMessage::DisplayPrompt(prompt_kind) => {
                self.prompt_info = Some(PromptInfo::new(prompt_kind));

                queue.push_front(Focus::BottomBar);
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
        if let Some(PromptInfo { input, .. }) = &mut self.prompt_info {
            use KeyCode::*;
            use KeyModifiers as KM;

            let KeyEvent { code, modifiers } = event;
            match (modifiers, code) {
                (KM::NONE, Char(c)) => {
                    input.push(c);
                }

                (KM::NONE, Backspace) => {
                    input.pop();
                }
                (KM::NONE, Enter) => {
                    let prompt_info = self.prompt_info.take().unwrap();

                    queue.push_front(Focus::TextArea);
                    queue.push_front(EditorControllerMessage::SaveAs(prompt_info.input));
                }

                _ => {}
            };
        }

        Ok(())
    }
}

impl PromptInfo {
    fn new(prompt_kind: PromptKind) -> Self {
        match prompt_kind {
            PromptKind::SaveAs => Self {
                message: String::from("[Save As] Enter file path:"),
                input: String::new(),
            },
        }
    }
}
