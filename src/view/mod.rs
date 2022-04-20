pub mod rendering;

// TODO! Explore different visibility options. See in a more uniform interface
// is possible.
pub struct ViewGeometry {
    pub line: usize,
    pub col: usize,
    pub width: usize,
    pub height: usize,
}

impl ViewGeometry {
    pub fn new(line: usize, col: usize, width: usize, height: usize) -> Self {
        Self {
            line,
            col,
            width,
            height,
        }
    }

    pub fn last_line(&self) -> usize {
        (self.line + self.height).saturating_sub(1)
    }

    pub fn last_col(&self) -> usize {
        (self.col + self.width).saturating_sub(1)
    }
}
