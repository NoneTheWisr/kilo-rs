use std::iter::{self, zip};
use std::ops::Range;

use syntect::highlighting::ThemeSet;
use syntect::highlighting::{HighlightState, Highlighter, RangedHighlightIterator, Style};
use syntect::parsing::SyntaxSet;
use syntect::parsing::{ParseState, ScopeStack};

use crate::core::Buffer;

const TAB_STOP: usize = 8;

pub struct RenderedBuffer {
    lines: Vec<String>,
    extension: Option<String>,
    highlighting: Option<Vec<Vec<(Style, Range<usize>)>>>,
}

impl From<&Buffer> for RenderedBuffer {
    fn from(buffer: &Buffer) -> Self {
        let lines: Vec<String> = buffer.lines().map(|line| render_line(line)).collect();
        let extension: Option<String> = match buffer.file_path().cloned() {
            Some(path) => path.rsplit_once('.').map(|split| split.1.into()),
            None => None,
        };

        let highlighting = extension.clone().map(|extension| {
            let ps = SyntaxSet::load_defaults_nonewlines();
            let ts = ThemeSet::load_from_folder("themes").unwrap();
            let syntax = ps.find_syntax_by_extension(&extension).unwrap();

            let theme = &ts.themes["gruvbox"];

            let highlighter = Highlighter::new(theme);
            let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
            let mut parse_state = ParseState::new(syntax);

            lines
                .iter()
                .map(|line| {
                    let ops = parse_state.parse_line(&line, &ps).unwrap();
                    let iter = RangedHighlightIterator::new(
                        &mut highlight_state,
                        &ops[..],
                        &line,
                        &highlighter,
                    );
                    iter.map(|(style, _, range)| (style, range)).collect()
                })
                .collect()
        });

        Self {
            lines,
            extension,
            highlighting,
        }
    }
}

impl RenderedBuffer {
    pub fn highlight(&mut self) {}

    pub fn update_line(&mut self, line_number: usize, buffer: &Buffer) {
        self.lines[line_number] = render_line(buffer.get_line(line_number));
    }

    pub fn insert_line(&mut self, line_number: usize, buffer: &Buffer) {
        self.lines
            .insert(line_number, render_line(buffer.get_line(line_number)));
    }

    pub fn remove_line(&mut self, line_number: usize) {
        self.lines.remove(line_number);
    }

    pub fn get_view(
        &self,
        line: usize,
        col: usize,
        width: usize,
        height: usize,
    ) -> (Vec<String>, Option<Vec<Vec<(Style, Range<usize>)>>>) {
        if let Some(highlighting) = &self.highlighting {
            let (lines, styles) = zip(self.lines.iter(), highlighting.into_iter())
                .skip(line)
                .take(height)
                .map(|(line, styles)| {
                    let line: String = line.chars().skip(col).take(width).collect();
                    let mut styles: Vec<_> = styles
                        .into_iter()
                        .filter(|(_, range)| range.start < line.len())
                        .cloned()
                        .collect();
                    if let Some(range) = styles.last_mut() {
                        range.1.end = line.len();
                    }
                    (line, styles)
                })
                .unzip();
            (lines, Some(styles))
        } else {
            (
                self.lines
                    .iter()
                    .skip(line)
                    .take(height)
                    .map(|line| line.chars().skip(col).take(width).collect())
                    .collect(),
                None,
            )
        }
        // self.lines
        //     .iter()
        //     .skip(line)
        //     .take(height)
        //     .map(|line| line.chars().skip(col).take(width).collect())
        //     .collect()
    }

    pub fn eol_col(&self, line: usize) -> usize {
        self.lines[line].len()
    }

    pub fn last_col(&self, line: usize) -> usize {
        self.eol_col(line).saturating_sub(1)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn last_line(&self) -> usize {
        self.line_count().saturating_sub(1)
    }
}

fn render_line(line: &str) -> String {
    let mut rendered = String::new();
    for c in line.chars() {
        if c == '\t' {
            let count = TAB_STOP - (rendered.len() % TAB_STOP);
            rendered.extend(iter::repeat(' ').take(count));
        } else {
            rendered.push(c);
        }
    }
    rendered
}
