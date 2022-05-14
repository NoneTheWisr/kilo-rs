use std::io::Write;

use anyhow::Result;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use kilo_rs_backend::editor::Editor;

use crate::{
    editor_controller::{EditorControllerComponent, EditorControllerMessage},
    runner::{MessageQueue, ShouldQuit},
    shared::{Focus, Rectangle, SharedContext},
    status_bar::{StatusBarComponent, StatusBarMessage},
    term_utils::Cursor,
    text_area::{TextAreaComponent, TextAreaMessage},
};

pub enum AppMessage {
    EditorControllerMessage(EditorControllerMessage),
    TextAreaMessage(TextAreaMessage),
    StatusBarMessage(StatusBarMessage),
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

impl From<StatusBarMessage> for AppMessage {
    fn from(message: StatusBarMessage) -> Self {
        Self::StatusBarMessage(message)
    }
}

pub struct App {
    context: SharedContext,
    editor_controller: EditorControllerComponent,
    text_area: TextAreaComponent,
    status_bar: StatusBarComponent,
}

impl App {
    pub fn new(args: Vec<String>) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let rect = Rectangle::new(0, 0, width, height);

        let mut context = SharedContext {
            editor: Editor::new(width as usize, height.saturating_sub(1) as usize),
            focus: Focus::TextArea,
        };

        if args.len() == 2 {
            context.editor.open_file(&args[1])?;
        }

        let editor_controller = EditorControllerComponent::new();
        let text_area = TextAreaComponent::new(&context);
        let status_bar = StatusBarComponent::new(Rectangle {
            top: rect.bottom,
            left: rect.left,
            bottom: rect.bottom,
            right: rect.right,
        });

        Ok(Self {
            context,
            editor_controller,
            text_area,
            status_bar,
        })
    }

    pub fn update(&mut self, message: AppMessage, queue: &mut MessageQueue) -> Result<()> {
        use AppMessage::*;

        match message {
            EditorControllerMessage(message) => {
                self.editor_controller
                    .update(message, queue, &mut self.context)?
            }
            TextAreaMessage(message) => self.text_area.update(message)?,
            StatusBarMessage(_) => (),
        }

        Ok(())
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        self.text_area.render(writer)?;
        self.status_bar.render(writer, &self.context)?;

        Ok(())
    }

    pub fn cursor(&self) -> Option<Cursor> {
        match self.context.focus {
            Focus::TextArea => self.text_area.cursor(),
            Focus::StatusBar => self.status_bar.cursor(&self.context),
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
            (KM::CONTROL, Char('q')) => return Ok(ShouldQuit::Yes),
            _ => match self.context.focus {
                Focus::TextArea => self.text_area.process_event(event, queue)?,
                Focus::StatusBar => self.status_bar.process_event(&event, &mut self.context)?,
            },
        }

        Ok(ShouldQuit::No)
    }
}
