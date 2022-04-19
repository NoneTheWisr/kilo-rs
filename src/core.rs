use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use anyhow::{bail, Result};

#[derive(Clone, Copy, PartialEq)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

pub struct Buffer {
    file_path: Option<String>,
    rows: Vec<String>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            file_path: None,
            rows: vec![String::new()],
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

        let rows = reader.lines().collect::<Result<_, _>>()?;
        let file_path = Some(String::from(file_path));
        let buffer = Self { file_path, rows };

        Ok(buffer)
    }

    pub fn save_as(&mut self, file_path: &str) -> Result<()> {
        fs::write(file_path, self.rows.join("\n"))?;
        self.file_path = Some(String::from(file_path));
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        match self.file_path.clone() {
            None => bail!("No file path associated with the buffer"),
            Some(path) => self.save_as(&path),
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = &String> {
        self.rows.iter()
    }

    pub fn file_path(&self) -> Option<&String> {
        self.file_path.as_ref()
    }

    pub fn join_two_lines(&mut self, first: usize) {
        let second = self.rows.remove(first + 1);
        self.rows[first] += &second;
    }

    pub fn remove_char(&mut self, location: Location) {
        self.rows[location.row].remove(location.col);
    }

    pub fn insert_char(&mut self, location: Location, c: char) {
        self.rows[location.row].insert(location.col, c);
    }

    pub fn insert_row(&mut self, row: usize) {
        self.rows.insert(row, String::new())
    }

    pub fn split_line(&mut self, location: Location) {
        let second_line = self.rows[location.row].split_off(location.col);
        self.rows.insert(location.row + 1, second_line);
    }
}
