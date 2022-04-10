pub mod rendering;

// TODO! Explore different visibility options. See in a more uniform interface
// is possible.
pub struct ViewGeometry {
    pub row: usize,
    pub col: usize,
    pub width: usize,
    pub height: usize,
}

impl ViewGeometry {
    pub fn new(row: usize, col: usize, width: usize, height: usize) -> Self {
        Self {
            row,
            col,
            width,
            height,
        }
    }

    pub fn last_row(&self) -> usize {
        (self.row + self.height).saturating_sub(1)
    }

    pub fn last_col(&self) -> usize {
        (self.col + self.width).saturating_sub(1)
    }
}
