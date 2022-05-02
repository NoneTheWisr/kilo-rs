use std::io;

use anyhow::Result;

use crossterm::event::{self, KeyEvent};

use kilo_rs_backend::core::Location;

use crate::{
    shared::{Focus, SharedContext},
    status_bar::StatusBarComponent,
    text_area::TextAreaComponent,
};

pub struct RootComponent {
    text_area: TextAreaComponent,
    status_bar: StatusBarComponent,
}

impl RootComponent {
    pub fn new() -> Self {
        Self {
            text_area: TextAreaComponent::new(),
            status_bar: StatusBarComponent::new(),
        }
    }

    pub fn render(&self, writer: &mut impl io::Write, context: &SharedContext) -> Result<()> {
        self.text_area.render(writer, context)?;
        self.status_bar.render(writer, context)?;

        Ok(())
    }

    pub fn cursor(&self, context: &SharedContext) -> Option<Location> {
        match context.logical_state {
            Focus::TextArea => self.text_area.cursor(context),
            Focus::StatusBar => self.status_bar.cursor(context),
        }
    }

    #[allow(unused_imports)]
    pub fn process_event(&mut self, event: &KeyEvent, context: &mut SharedContext) -> Result<()> {
        use event::KeyCode::*;
        use event::KeyModifiers as KM;

        let &KeyEvent { modifiers, code } = event;
        match (modifiers, code) {
            _ => match context.logical_state {
                Focus::TextArea => self.text_area.process_event(event, context)?,
                Focus::StatusBar => self.status_bar.process_event(event, context)?,
            },
        }

        Ok(())
    }
}
