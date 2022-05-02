use kilo_rs_backend::editor::Editor;

pub struct SharedContext {
    pub editor: Editor,
    pub execution_state: ExecutionState,
    pub logical_state: Focus,
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
