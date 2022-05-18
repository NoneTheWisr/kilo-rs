use std::io::Write;

use anyhow::Result;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use kilo_rs_backend::editor::Editor;

use crate::{
    bottom_bar::{BottomBarComponent, BottomBarMessage, PromptKind},
    editor_controller::{EditorControllerComponent, EditorControllerMessage},
    runner::{MessageQueue, ShouldQuit},
    shared::{Rectangle, SharedContext},
    term_utils::Cursor,
    text_area::{TextAreaComponent, TextAreaMessage},
};

pub enum AppMessage {
    EditorControllerMessage(EditorControllerMessage),
    TextAreaMessage(TextAreaMessage),
    BottomBarMessage(BottomBarMessage),
    SwitchFocus(Focus),
    Quit,
}

impl From<EditorControllerMessage> for AppMessage {
    fn from(message: EditorControllerMessage) -> Self {
        Self::EditorControllerMessage(message)
    }
}

impl From<TextAreaMessage> for AppMessage {
    fn from(message: TextAreaMessage) -> Self {
        Self::TextAreaMessage(message)
    }
}

impl From<BottomBarMessage> for AppMessage {
    fn from(message: BottomBarMessage) -> Self {
        Self::BottomBarMessage(message)
    }
}

impl From<Focus> for AppMessage {
    fn from(focus: Focus) -> Self {
        Self::SwitchFocus(focus)
    }
}

pub struct App {
    context: SharedContext,
    editor_controller: EditorControllerComponent,
    text_area: TextAreaComponent,
    bottom_bar: BottomBarComponent,
    focus: Focus,
}

pub enum Focus {
    TextArea,
    BottomBar,
}

#[derive(Default)]
pub struct StartupArgs {
    pub file: Option<String>,
}

impl App {
    pub fn new(args: StartupArgs) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let rect = Rectangle::new(0, 0, width, height);

        let mut context = SharedContext {
            editor: Editor::new(width as usize, height.saturating_sub(1) as usize),
        };

        if let Some(file_path) = args.file {
            context.editor.open_file(&file_path)?;
        }

        let editor_controller = EditorControllerComponent::new();
        let text_area = TextAreaComponent::new(&context);
        let bottom_bar = BottomBarComponent::new(
            Rectangle {
                top: rect.bottom,
                left: rect.left,
                bottom: rect.bottom,
                right: rect.right,
            },
            &context,
        );

        Ok(Self {
            context,
            editor_controller,
            text_area,
            bottom_bar,
            focus: Focus::TextArea,
        })
    }

    pub fn update(&mut self, message: AppMessage, queue: &mut MessageQueue) -> Result<ShouldQuit> {
        use AppMessage::*;

        match message {
            EditorControllerMessage(message) => {
                self.editor_controller
                    .update(message, queue, &mut self.context)?
            }
            TextAreaMessage(message) => self.text_area.update(message)?,
            BottomBarMessage(message) => self.bottom_bar.update(message, queue)?,
            SwitchFocus(focus) => self.focus = focus,
            Quit => return Ok(ShouldQuit::Yes),
        }

        Ok(ShouldQuit::No)
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        self.text_area.render(writer)?;
        self.bottom_bar.render(writer)?;

        Ok(())
    }

    pub fn cursor(&self) -> Option<Cursor> {
        match self.focus {
            Focus::TextArea => self.text_area.cursor(),
            Focus::BottomBar => self.bottom_bar.cursor(),
        }
    }

    #[allow(unused_imports)]
    pub fn process_event(
        &mut self,
        event: KeyEvent,
        queue: &mut MessageQueue,
    ) -> Result<ShouldQuit> {
        use KeyCode::*;
        use KeyModifiers as KM;

        let KeyEvent { modifiers, code } = event;
        match (modifiers, code) {
            (KM::CONTROL, Char('q')) => {
                if self.context.editor.is_buffer_dirty() {
                    queue.push(BottomBarMessage::DisplayPrompt(PromptKind::ConfirmQuit))
                } else {
                    return Ok(ShouldQuit::Yes);
                }
            }
            (mods, Char('q')) if mods == KM::CONTROL | KM::ALT => {
                return Ok(ShouldQuit::Yes);
            }

            (KM::CONTROL, Char('o')) => {
                queue.push(BottomBarMessage::DisplayPrompt(PromptKind::Open))
            }
            (KM::CONTROL, Char('f')) => {
                queue.push(BottomBarMessage::DisplayPrompt(PromptKind::Find))
            }
            (KM::CONTROL, Char('s')) => queue.push(EditorControllerMessage::Save),
            (mods, Char('s')) if mods == KM::CONTROL | KM::ALT => {
                queue.push(BottomBarMessage::DisplayPrompt(PromptKind::SaveAs));
            }

            _ => match self.focus {
                Focus::TextArea => self.text_area.process_event(event, queue)?,
                Focus::BottomBar => self.bottom_bar.process_event(event, queue)?,
            },
        }

        Ok(ShouldQuit::No)
    }
}
