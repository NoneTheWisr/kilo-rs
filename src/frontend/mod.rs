use std::io;

use anyhow::Result;
use crossterm::event;

use crate::backend::core::Location;

pub struct Cursor(pub u16, pub u16);

impl From<Location> for Cursor {
    fn from(location: Location) -> Self {
        Self(location.col as u16, location.line as u16)
    }
}

pub trait Component {
    type Context;

    fn render(&self, writer: &mut dyn io::Write, context: &Self::Context) -> Result<()>;
    fn cursor(&self, context: &Self::Context) -> Option<Cursor>;
    fn process_event(&mut self, event: &event::Event, context: &mut Self::Context) -> Result<()>;
}
