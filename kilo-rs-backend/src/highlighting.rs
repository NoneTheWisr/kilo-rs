use std::{ops::Range, rc::Rc};

use syntect::{
    highlighting::{HighlightState, Highlighter, RangedHighlightIterator, Theme, ThemeSet},
    parsing::{ParseState, ScopeStack, SyntaxSet},
};

pub use syntect::highlighting::Color;
pub use syntect::highlighting::ThemeSettings;

// I'm unhappy with Highlighter's lifetime bounds, so I'm using unsfe to "erase" them.
// I believe this should be fine so long as the theme Rc is untouched and highlighter
// doesn't get leaked outside the struct.
pub(crate) struct SyntaxHighlighter {
    theme: Rc<Theme>,
    syntax_set: Rc<SyntaxSet>,
    highlighter: Highlighter<'static>,
    parse_state: ParseState,
    highlight_state: HighlightState,
}

impl SyntaxHighlighter {
    pub(crate) fn new(
        theme: Rc<Theme>,
        syntax_set: Rc<SyntaxSet>,
        extension: Option<&str>,
    ) -> Self {
        let highlighter = Highlighter::new(&theme);
        let highlighter: Highlighter<'static> = unsafe { std::mem::transmute(highlighter) };

        let syntax_ref = match extension {
            Some(extension) => syntax_set.find_syntax_by_extension(extension),
            None => None,
        };
        let syntax_ref = syntax_ref.unwrap_or(syntax_set.find_syntax_plain_text());

        let parse_state = ParseState::new(syntax_ref);
        let highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

        SyntaxHighlighter {
            theme,
            syntax_set,
            highlighter,
            parse_state,
            highlight_state,
        }
    }

    pub(crate) fn simple(extension: Option<&str>) -> Self {
        let syntax_set = Rc::new(SyntaxSet::load_defaults_nonewlines());
        let mut theme_set = ThemeSet::new();
        theme_set
            .add_from_folder("themes")
            .expect("couldn't load themes");
        let theme = Rc::new(theme_set.themes["gruvbox"].clone());

        Self::new(theme, syntax_set, extension)
    }

    pub(crate) fn theme(&self) -> &ThemeSettings {
        &self.theme.settings
    }

    pub(crate) fn highlight_line(&mut self, line: &str) -> anyhow::Result<LineHighlighting> {
        let ops = self.parse_state.parse_line(line, &self.syntax_set)?;
        let iter = RangedHighlightIterator::new(
            &mut self.highlight_state,
            &ops[..],
            line,
            &self.highlighter,
        );
        Ok(iter
            .map(|(style, _, range)| Highlight::new(style.into(), range))
            .collect())
    }

    pub(crate) fn highlight<Iter, Item>(
        &mut self,
        lines: Iter,
    ) -> anyhow::Result<Vec<LineHighlighting>>
    where
        Iter: Iterator<Item = Item>,
        Item: AsRef<str>,
    {
        lines
            .map(|line| self.highlight_line(line.as_ref()))
            .collect()
    }
}

pub type LineHighlighting = Vec<Highlight>;

#[derive(Clone)]
pub struct Highlight {
    pub style: HighlightStyle,
    pub range: Range<usize>,
}

impl Highlight {
    fn new(style: HighlightStyle, range: Range<usize>) -> Self {
        Self { style, range }
    }
}

#[derive(Clone)]
pub struct HighlightStyle {
    pub foreground: Color,
    pub background: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl From<syntect::highlighting::Style> for HighlightStyle {
    fn from(style: syntect::highlighting::Style) -> Self {
        use syntect::highlighting::FontStyle;
        Self {
            foreground: style.foreground,
            background: style.background,
            bold: style.font_style.contains(FontStyle::BOLD),
            italic: style.font_style.contains(FontStyle::ITALIC),
            underline: style.font_style.contains(FontStyle::UNDERLINE),
        }
    }
}
