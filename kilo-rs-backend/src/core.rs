use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use anyhow::{bail, Result};

#[derive(Clone, Copy, PartialEq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

impl Location {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Clone)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

pub struct Buffer {
    file_path: Option<String>,
    lines: Vec<String>,
    dirty: bool,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            file_path: None,
            lines: vec![String::new()],
            dirty: false,
        }
    }
}

impl Buffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_file(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let lines = reader.lines().collect::<Result<_, _>>()?;
        let file_path = Some(String::from(file_path));
        let buffer = Self {
            file_path,
            lines,
            dirty: false,
        };

        Ok(buffer)
    }

    pub fn save(&mut self) -> Result<()> {
        match self.file_path.clone() {
            None => bail!("No file path associated with the buffer"),
            Some(path) => self.save_as(&path),
        }?;

        self.dirty = false;
        Ok(())
    }

    pub fn save_as(&mut self, file_path: &str) -> Result<()> {
        fs::write(file_path, self.lines.join("\n"))?;
        self.file_path = Some(String::from(file_path));

        self.dirty = false;
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn file_path(&self) -> Option<&String> {
        self.file_path.as_ref()
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    pub fn get_line(&self, line_number: usize) -> &String {
        &self.lines[line_number]
    }

    pub fn insert_char(&mut self, location: Location, c: char) {
        self.lines[location.line].insert(location.col, c);
        self.dirty = true;
    }

    pub fn remove_char(&mut self, location: Location) {
        self.lines[location.line].remove(location.col);
        self.dirty = true;
    }

    pub fn insert_line(&mut self, line_number: usize) {
        self.lines.insert(line_number, String::new());
        self.dirty = true;
    }

    pub fn join_two_lines(&mut self, first_line: usize) {
        let second_line = self.lines.remove(first_line + 1);
        self.lines[first_line] += &second_line;
        self.dirty = true;
    }

    pub fn split_line(&mut self, location: Location) {
        let second_line = self.lines[location.line].split_off(location.col);
        self.lines.insert(location.line + 1, second_line);
        self.dirty = true;
    }

    pub fn find(&self, pattern: &str, forward: bool, start: Location) -> Option<Span> {
        let line = &self.lines[start.line];
        let line_split = ((0, start.col), (start.col + 1, line.len()));
        let find = if forward { str::find } else { str::rfind };

        // Part of the first line after the cursor
        let (beg, end) = if forward { line_split.1 } else { line_split.0 };
        let ahead = &self.lines[start.line][beg..end];
        if let Some(col) = find(&ahead, pattern) {
            return Some(Span {
                start: Location::new(start.line, beg + col),
                end: Location::new(start.line, beg + col + pattern.len()),
            });
        }

        // All the other lines wrapped around
        let line_matches = self.lines.iter().enumerate().map(|(num, line)| {
            find(line, pattern).map(|col| Span {
                start: Location::new(num, col),
                end: Location::new(num, col + pattern.len()),
            })
        });

        let before = line_matches.clone().take(start.line);
        let after = line_matches.skip(start.line + 1);

        if let Some(span) = match forward {
            true => after.chain(before).find(Option::is_some),
            false => before.rev().chain(after.rev()).find(Option::is_some),
        } {
            return span;
        }

        // Wrap-around part of the first line
        let (beg, end) = if forward { line_split.0 } else { line_split.1 };
        let behind = &self.lines[start.line][beg..end];
        if let Some(col) = find(&behind, pattern) {
            return Some(Span {
                start: Location::new(start.line, beg + col),
                end: Location::new(start.line, beg + col + pattern.len()),
            });
        }

        None
    }
}
