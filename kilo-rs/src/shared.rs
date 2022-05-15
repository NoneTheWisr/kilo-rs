use kilo_rs_backend::editor::Editor;

pub struct SharedContext {
    pub editor: Editor,
    pub focus: Focus,
}

pub enum Focus {
    TextArea,
    BottomBar,
}

pub struct Rectangle {
    pub top: u16,
    pub left: u16,
    pub bottom: u16,
    pub right: u16,
}

impl Rectangle {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            top: y,
            left: x,
            bottom: y + height - 1,
            right: x + width - 1,
        }
    }

    pub fn width(&self) -> u16 {
        self.right - self.left + 1
    }

    pub fn height(&self) -> u16 {
        self.bottom - self.top + 1
    }
}
