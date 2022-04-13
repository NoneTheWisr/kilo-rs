use std::iter;

use crate::core::Buffer;

const TAB_STOP: usize = 8;

pub struct RenderedBuffer {
    rows: Vec<String>,
}

impl From<&Buffer> for RenderedBuffer {
    fn from(buffer: &Buffer) -> Self {
        let rows = buffer.rows().map(|row| render_row(row)).collect();
        Self { rows }
    }
}

impl RenderedBuffer {
    pub fn get_view(&self, row: usize, col: usize, width: usize, height: usize) -> Vec<String> {
        self.rows
            .iter()
            .skip(row)
            .take(height)
            .map(|row| row.chars().skip(col).take(width).collect())
            .collect()
    }

    pub fn eol_col(&self, row: usize) -> usize {
        self.rows[row].len()
    }

    pub fn last_col(&self, row: usize) -> usize {
        self.eol_col(row).saturating_sub(1)
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn last_row(&self) -> usize {
        self.row_count().saturating_sub(1)
    }
}

fn render_row(row: &str) -> String {
    let mut rendered = String::new();
    for c in row.chars() {
        if c == '\t' {
            let count = TAB_STOP - (rendered.len() % TAB_STOP);
            rendered.extend(iter::repeat(' ').take(count));
        } else {
            rendered.push(c);
        }
    }
    rendered
}
