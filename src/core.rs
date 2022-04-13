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

    pub fn rows(&self) -> impl Iterator<Item = &String> {
        self.rows.iter()
    }

    pub fn file_path(&self) -> Option<&String> {
        self.file_path.as_ref()
    }
}
