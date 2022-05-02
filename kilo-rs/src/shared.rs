use kilo_rs_backend::editor::Editor;

pub struct SharedContext {
    pub editor: Editor,
    pub state: ExecutionState,
    pub focus: Focus,
}

pub enum ExecutionState {
    Initialization,
    Running,
    Closing,
}

pub enum Focus {
    TextArea,
    StatusBar,
}
