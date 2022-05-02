use std::iter;

use crate::core::Buffer;

const TAB_STOP: usize = 8;

pub struct RenderedBuffer {
    lines: Vec<String>,
}

impl From<&Buffer> for RenderedBuffer {
    fn from(buffer: &Buffer) -> Self {
        Self {
            lines: buffer.lines().map(|line| render_line(line)).collect(),
        }
    }
}

impl RenderedBuffer {
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
        self.lines
            .iter()
            .skip(line)
            .take(height)
            .map(|line| line.chars().skip(col).take(width).collect())
            .collect()
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
