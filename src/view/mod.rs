pub mod rendering;

// I think keeping this as close to POD as possible is fine. If I want to create
// a better interface, I'd have to introduce a lot of methods like
// * move -> left / right, up / down
// * move_to -> place
// * set -> specific coord / dimension
// I'm choosing to keep it transparent, just grouping some variables together.
// Kind of like an implementation detail and not a part of some API.
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
