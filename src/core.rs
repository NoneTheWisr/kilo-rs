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
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            file_path: None,
            lines: vec![String::new()],
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
        let buffer = Self { file_path, lines };

        Ok(buffer)
    }

    pub fn save(&mut self) -> Result<()> {
        match self.file_path.clone() {
            None => bail!("No file path associated with the buffer"),
            Some(path) => self.save_as(&path),
        }
    }

    pub fn save_as(&mut self, file_path: &str) -> Result<()> {
        fs::write(file_path, self.lines.join("\n"))?;
        self.file_path = Some(String::from(file_path));
        Ok(())
    }

    pub fn file_path(&self) -> Option<&String> {
        self.file_path.as_ref()
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    pub fn insert_char(&mut self, location: Location, c: char) {
        self.lines[location.line].insert(location.col, c);
    }

    pub fn remove_char(&mut self, location: Location) {
        self.lines[location.line].remove(location.col);
    }

    pub fn insert_line(&mut self, line_number: usize) {
        self.lines.insert(line_number, String::new())
    }

    pub fn join_two_lines(&mut self, first_line: usize) {
        let second_line = self.lines.remove(first_line + 1);
        self.lines[first_line] += &second_line;
    }

    pub fn split_line(&mut self, location: Location) {
        let second_line = self.lines[location.line].split_off(location.col);
        self.lines.insert(location.line + 1, second_line);
    }
}
