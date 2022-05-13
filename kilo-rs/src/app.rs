use std::io::Write;

use anyhow::Result;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use kilo_rs_backend::core::Location;

use crate::{
    runner::{MessageQueue, ShouldQuit},
    shared::{Focus, SharedContext},
    status_bar::StatusBarComponent,
    text_area::TextAreaComponent,
};

pub enum AppMessage {}

pub struct App {
    text_area: TextAreaComponent,
    status_bar: StatusBarComponent,
}

impl App {
    pub fn new() -> Self {
        Self {
            text_area: TextAreaComponent::new(),
            status_bar: StatusBarComponent::new(),
        }
    }

    pub fn update(&mut self, _message: AppMessage, _queue: &mut MessageQueue) -> Result<()> {
        Ok(())
    }

    pub fn render(&self, writer: &mut impl Write, context: &SharedContext) -> Result<()> {
        self.text_area.render(writer, context)?;
        self.status_bar.render(writer, context)?;

        Ok(())
    }

    pub fn cursor(&self, context: &SharedContext) -> Option<Location> {
        match context.focus {
            Focus::TextArea => self.text_area.cursor(context),
            Focus::StatusBar => self.status_bar.cursor(context),
        }
    }

    #[allow(unused_imports)]
    pub fn process_event(
        &mut self,
        event: &KeyEvent,
        _queue: &mut MessageQueue,
        context: &mut SharedContext,
    ) -> Result<ShouldQuit> {
        use KeyCode::*;
        use KeyModifiers as KM;

        let &KeyEvent { modifiers, code } = event;
        match (modifiers, code) {
            (KM::CONTROL, Char('q')) => return Ok(ShouldQuit::Yes),
            _ => match context.focus {
                Focus::TextArea => self.text_area.process_event(event, context)?,
                Focus::StatusBar => self.status_bar.process_event(event, context)?,
            },
        }

        Ok(ShouldQuit::No)
    }
}
