use std::io;

use anyhow::Result;

use rustea::crossterm::cursor::{MoveTo, Show};
use rustea::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustea::crossterm::{queue, terminal};
use rustea::{command, Message};

use crate::shared::{ExecutionState, Focus, SharedContext};
use crate::status_bar::StatusBarComponent;
use crate::text_area::TextAreaComponent;

use kilo_rs_backend::{core::Location, editor::Editor};

pub struct App {
    status_bar: StatusBarComponent,
    text_area: TextAreaComponent,
    context: SharedContext,
}

impl rustea::App for App {
    fn init(&self) -> Option<rustea::Command> {
        Some(Box::new(show_cursor))
    }

    fn update(&mut self, msg: rustea::Message) -> Option<rustea::Command> {
        if let Some(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
        }) = msg.downcast_ref::<KeyEvent>()
        {
            return Some(Box::new(command::quit));
        }

        match self.context.focus {
            Focus::TextArea => self.text_area.update(msg, &mut self.context),
            Focus::StatusBar => self.status_bar.update(msg, &mut self.context),
        }
    }

    fn view(&self) -> String {
        let mut view = Vec::new();

        self.status_bar.render(&mut view, &self.context).unwrap();
        self.text_area.render(&mut view, &self.context).unwrap();

        let Location { line, col } = self.cursor();
        queue!(view, MoveTo(col as u16, line as u16)).unwrap();

        String::from_utf8_lossy(&view).into_owned()
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(1);

        Ok(Self {
            status_bar: StatusBarComponent::new(),
            text_area: TextAreaComponent::new(),
            context: SharedContext {
                editor: Editor::new(width as usize, height as usize),
                state: ExecutionState::Initialization,
                focus: Focus::TextArea,
            },
        })
    }

    pub fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.context.editor.open_file(file_path)
    }

    fn cursor(&self) -> Location {
        match self.context.focus {
            Focus::TextArea => self.text_area.cursor(&self.context).unwrap(),
            Focus::StatusBar => self.status_bar.cursor(&self.context).unwrap(),
        }
    }
}

fn show_cursor() -> Option<Message> {
    queue!(io::stdout(), Show).unwrap();
    None
}
