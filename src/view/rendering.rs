use std::iter;

use crate::core::Buffer;

const TAB_STOP: usize = 8;

pub struct RenderedBuffer {
    lines: Vec<String>,
}

impl RenderedBuffer {
    pub fn empty() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn render(buffer: &Buffer) -> Self {
        let lines = buffer.lines().map(|line| render_string(line)).collect();
        Self { lines }
    }

    pub fn rect(&self, row: usize, col: usize, width: usize, height: usize) -> Vec<String> {
        self.lines
            .iter()
            .skip(row)
            .take(height)
            .map(|line| line.chars().skip(col).take(width).collect())
            .collect()
    }

    pub fn eol_col(&self, line_number: usize) -> usize {
        self.lines[line_number].len()
    }

    pub fn last_col(&self, line_number: usize) -> usize {
        self.eol_col(line_number).saturating_sub(1)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn last_line(&self) -> usize {
        self.line_count().saturating_sub(1)
    }
}

fn render_string(raw: &str) -> String {
    let mut rendered = String::new();
    for c in raw.chars() {
        if c == '\t' {
            let count = TAB_STOP - (rendered.len() % TAB_STOP);
            rendered.extend(iter::repeat(' ').take(count));
        } else {
            rendered.push(c);
        }
    }
    rendered
}
