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

    pub fn find(&self, pattern: &str, forward: bool, start: Location) -> Option<Location> {
        if forward {
            let ahead = self.lines[start.line].get(start.col + 1..);
            if let Some(col) = ahead.map_or(None, |ahead| ahead.find(pattern)) {
                return Some(Location::new(start.line, start.col + 1 + col));
            }
        } else {
            let ahead = self.lines[start.line].get(..start.col);
            if let Some(col) = ahead.map_or(None, |ahead| ahead.rfind(pattern)) {
                return Some(Location::new(start.line, col));
            }
        }

        let result = if forward {
            let mut lines = self
                .lines
                .iter()
                .enumerate()
                .cycle()
                .skip(start.line + 1)
                .take(self.lines.len().saturating_sub(1));
            lines.find_map(|(num, line)| line.find(pattern).map(|col| Location::new(num, col)))
        } else {
            let mut lines = self
                .lines
                .iter()
                .enumerate()
                .rev()
                .cycle()
                .skip(self.lines.len() - start.line)
                .take(self.lines.len().saturating_sub(1));
            lines.find_map(|(num, line)| line.rfind(pattern).map(|col| Location::new(num, col)))
        };
        if result.is_some() {
            return result;
        }

        if forward {
            let behind = self.lines[start.line].get(..start.col);
            if let Some(col) = behind.map_or(None, |ahead| ahead.find(pattern)) {
                return Some(Location::new(start.line, col));
            }
        } else {
            let behind = self.lines[start.line].get(start.col + 1..);
            if let Some(col) = behind.map_or(None, |ahead| ahead.rfind(pattern)) {
                return Some(Location::new(start.line, start.col + 1 + col));
            }
        }

        None
    }
}
