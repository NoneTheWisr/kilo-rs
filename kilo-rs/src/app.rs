use std::io::Write;

use anyhow::Result;

use rustea::command;
use rustea::crossterm::cursor::{Hide, Show};
use rustea::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustea::crossterm::{queue, terminal};

use crate::shared::{ExecutionState, Focus, SharedContext};
use crate::status_bar::StatusBarComponent;
use crate::term_utils::{Cursor, MoveTo};
use crate::text_area::TextAreaComponent;

use kilo_rs_backend::editor::Editor;

pub struct App {
    status_bar: StatusBarComponent,
    text_area: TextAreaComponent,
    context: SharedContext,
}

impl rustea::App for App {
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

    fn view(&self, stdout: &mut impl Write) {
        queue!(stdout, Hide).unwrap();

        self.status_bar.render(stdout, &self.context).unwrap();
        self.text_area.render(stdout, &self.context).unwrap();

        queue!(stdout, MoveTo(self.cursor()), Show).unwrap();

        stdout.flush().unwrap();
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

    fn cursor(&self) -> Cursor {
        match self.context.focus {
            Focus::TextArea => self.text_area.cursor(&self.context).unwrap(),
            Focus::StatusBar => self.status_bar.cursor(&self.context).unwrap(),
        }
    }
}
