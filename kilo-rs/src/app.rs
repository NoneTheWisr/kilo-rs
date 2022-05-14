use std::io::Write;

use anyhow::Result;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use kilo_rs_backend::editor::Editor;

use crate::{
    runner::{MessageQueue, ShouldQuit},
    shared::{Focus, SharedContext},
    status_bar::StatusBarComponent,
    term_utils::Cursor,
    text_area::TextAreaComponent,
};

pub enum AppMessage {}

pub struct App {
    context: SharedContext,
    text_area: TextAreaComponent,
    status_bar: StatusBarComponent,
}

impl App {
    pub fn new(args: Vec<String>) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        let mut context = SharedContext {
            editor: Editor::new(width as usize, height as usize),
            focus: Focus::TextArea,
        };

        if args.len() == 2 {
            context.editor.open_file(&args[1])?;
        }

        Ok(Self {
            context,
            status_bar: StatusBarComponent::new(),
            text_area: TextAreaComponent::new(),
        })
    }

    pub fn update(&mut self, _message: AppMessage, _queue: &mut MessageQueue) -> Result<()> {
        Ok(())
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        self.text_area.render(writer, &self.context)?;
        self.status_bar.render(writer, &self.context)?;

        Ok(())
    }

    pub fn cursor(&self) -> Option<Cursor> {
        match self.context.focus {
            Focus::TextArea => self.text_area.cursor(&self.context),
            Focus::StatusBar => self.status_bar.cursor(&self.context),
        }
    }

    #[allow(unused_imports)]
    pub fn process_event(
        &mut self,
        event: &KeyEvent,
        _queue: &mut MessageQueue,
    ) -> Result<ShouldQuit> {
        use KeyCode::*;
        use KeyModifiers as KM;

        let &KeyEvent { modifiers, code } = event;
        match (modifiers, code) {
            (KM::CONTROL, Char('q')) => return Ok(ShouldQuit::Yes),
            _ => match self.context.focus {
                Focus::TextArea => self.text_area.process_event(event, &mut self.context)?,
                Focus::StatusBar => self.status_bar.process_event(event, &mut self.context)?,
            },
        }

        Ok(ShouldQuit::No)
    }
}
