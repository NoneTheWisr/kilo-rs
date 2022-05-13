use kilo_rs_backend::editor::Editor;

pub struct SharedContext {
    pub editor: Editor,
    pub focus: Focus,
}

pub enum Focus {
    TextArea,
    StatusBar,
}
