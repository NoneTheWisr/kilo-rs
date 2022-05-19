use std::iter;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use crate::core::Buffer;

const TAB_STOP: usize = 8;

pub struct RenderedBuffer {
    lines: Vec<String>,
    extension: Option<String>,
    highlighting: Option<Vec<Vec<(Style, String)>>>,
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
            let ts = ThemeSet::load_defaults();
            let syntax = ps.find_syntax_by_extension("rs").unwrap();
            let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

            lines
                .iter()
                .map(|line| {
                    h.highlight_line(line, &ps)
                        .unwrap()
                        .into_iter()
                        .map(|tuple| (tuple.0, tuple.1.to_owned()))
                        .collect()
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

    pub fn get_view(&self, line: usize, col: usize, width: usize, height: usize) -> Vec<String> {
        if let Some(highlighting) = &self.highlighting {
            highlighting
                .iter()
                .skip(line)
                .take(height)
                .map(|ranges| {
                    as_24_bit_terminal_escaped(
                        ranges
                            .into_iter()
                            .map(|thing| (thing.0, thing.1.as_str()))
                            .collect::<Vec<(_, &str)>>()
                            .as_slice(),
                        true,
                    )
                })
                .map(|line| line.chars().skip(col).take(width).collect())
                .collect()
        } else {
            self.lines
                .iter()
                .skip(line)
                .take(height)
                .map(|line| line.chars().skip(col).take(width).collect())
                .collect()
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
