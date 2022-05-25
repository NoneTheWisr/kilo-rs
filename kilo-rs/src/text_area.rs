use std::io::Write;
use std::iter::zip;

use anyhow::Result;

use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{
    Attribute, Attributes, Color, ContentStyle, Print, ResetColor, SetAttributes,
    SetBackgroundColor, StyledContent, Stylize,
};
use crossterm::terminal::{Clear, ClearType::UntilNewLine};

use kilo_rs_backend::core::Location;
use kilo_rs_backend::editor::Editor;
use kilo_rs_backend::highlighting::{self, Highlight, LineHighlighting, ThemeSettings};

use crate::editor_controller::EditorControllerMessage;
use crate::runner::MessageQueue;
use crate::term_utils::{Cursor, MoveToCursor};

pub enum TextAreaMessage {
    Update(UpdateMessage),
}

pub struct UpdateMessage {
    pub lines: Vec<String>,
    pub highlighting: Vec<LineHighlighting>,
    pub cursor: kilo_rs_backend::core::Location,
    pub search_mode: bool,
    pub theme_settings: ThemeSettings,
}

pub struct TextAreaComponent {
    lines: Vec<String>,
    highlighting: Vec<LineHighlighting>,
    cursor: Cursor,
    search_mode: bool,
    theme_settings: ThemeSettings,
}

impl TextAreaComponent {
    pub fn new(editor: &Editor) -> Self {
        let Location { line, col } = editor.get_view_cursor();

        let (lines, highlighting) = editor.get_view_contents();
        let cursor = Cursor::new(line as u16, col as u16);
        let search_mode = editor.is_search_mode_active();
        let theme_settings = editor.theme().clone();

        Self {
            lines,
            highlighting,
            cursor,
            search_mode,
            theme_settings,
        }
    }

    pub fn render(&self, writer: &mut impl Write) -> Result<Option<Cursor>> {
        queue!(writer, MoveTo(0, 0))?;

        for (line, highlighting) in zip(&self.lines, &self.highlighting) {
            render_styled_line(writer, line, highlighting)?;
            fill_background(
                writer,
                self.theme_settings
                    .background
                    .unwrap_or(highlighting::Color::BLACK),
            )?;
        }

        if self.search_mode {
            queue!(writer, MoveToCursor(self.cursor))?;
            queue!(writer, Print(self.get_char_at_cursor().negative()))?;
        }

        Ok(Some(self.cursor))
    }

    pub fn update(&mut self, message: TextAreaMessage) -> Result<()> {
        let TextAreaMessage::Update(message) = message;

        self.lines = message.lines;
        self.highlighting = message.highlighting;
        let Location { line, col } = message.cursor;
        self.cursor = Cursor::new(line as u16, col as u16);
        self.search_mode = message.search_mode;
        self.theme_settings = message.theme_settings;

        Ok(())
    }

    pub fn process_event(&mut self, event: KeyEvent, queue: &mut MessageQueue) -> Result<()> {
        use EditorControllerMessage::*;
        use KeyCode::*;
        use KeyModifiers as KM;

        let KeyEvent { code, modifiers } = event;
        let message = match (modifiers, code) {
            (KM::NONE, Up) => MoveCursorUp,
            (KM::NONE, Down) => MoveCursorDown,
            (KM::NONE, Left) => MoveCursorLeft,
            (KM::NONE, Right) => MoveCursorRight,

            (KM::NONE, Home) => MoveCursorToLineStart,
            (KM::NONE, End) => MoveCursorToLineEnd,

            (KM::NONE, PageUp) => MoveOneViewUp,
            (KM::NONE, PageDown) => MoveOneViewDown,

            (KM::CONTROL, PageUp) => MoveCursorToBufferTop,
            (KM::CONTROL, PageDown) => MoveCursorToBufferBottom,

            (KM::NONE, Backspace) => RemoveCharBehind,
            (KM::NONE, Delete) => RemoveCharInFront,

            (KM::NONE, Char(c)) => InsertChar(c),
            (KM::SHIFT, Char(c)) => InsertChar(c.to_ascii_uppercase()),

            (KM::NONE, Enter) => InsertLine,

            _ => return Ok(()),
        };

        queue.push(message);
        Ok(())
    }

    fn get_char_at_cursor(&self) -> char {
        self.lines[self.cursor.row as usize]
            .chars()
            .nth(self.cursor.col as usize)
            .unwrap()
    }
}

fn render_styled_line(
    writer: &mut impl Write,
    line: &str,
    highlighting: &LineHighlighting,
) -> Result<()> {
    // Taken from syntect
    fn blend_fg_color(fg: highlighting::Color, bg: highlighting::Color) -> highlighting::Color {
        if fg.a == 0xff {
            return fg;
        }
        let ratio = fg.a as u32;
        let r = (fg.r as u32 * ratio + bg.r as u32 * (255 - ratio)) / 255;
        let g = (fg.g as u32 * ratio + bg.g as u32 * (255 - ratio)) / 255;
        let b = (fg.b as u32 * ratio + bg.b as u32 * (255 - ratio)) / 255;
        highlighting::Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: 255,
        }
    }

    for Highlight { style, range } in highlighting {
        let part = &line[range.clone()];
        let fg = blend_fg_color(style.foreground, style.background);
        let bg = style.background;

        let mut builder = ContentStyle::new()
            .with(convert_color(fg))
            .on(convert_color(bg));
        if style.bold {
            builder = builder.attribute(Attribute::Bold);
        }
        if style.italic {
            builder = builder.attribute(Attribute::Italic);
        }
        if style.underline {
            builder = builder.attribute(Attribute::Underlined);
        }

        queue!(writer, Print(StyledContent::new(builder, part)))?;
    }

    Ok(())
}

fn fill_background(writer: &mut impl Write, bg: highlighting::Color) -> Result<()> {
    queue!(writer, SetBackgroundColor(convert_color(bg)))?;
    queue!(writer, Clear(UntilNewLine))?;
    queue!(writer, Print("\r\n"))?;
    queue!(writer, ResetColor)?;
    Ok(())
}

fn convert_color(color: highlighting::Color) -> Color {
    Color::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}
