use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Result;

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
    lines: Vec<String>,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            file_path: None,
            lines: Vec::new(),
        }
    }

    pub fn from_file(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let lines = reader.lines().collect::<Result<_, _>>()?;
        let file_path = Some(String::from(file_path));
        let buffer = Self { file_path, lines };

        Ok(buffer)
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    pub fn file_path(&self) -> &Option<String> {
        &self.file_path
    }
}
