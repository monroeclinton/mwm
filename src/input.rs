#[derive(PartialEq)]
pub enum Action {
    // Set a specific workspace to be active
    WorkspaceSetActive(usize),
    // Move a window to a workspace
    WindowSetWorkspace(usize),
    // Launch a program
    Spawn(String),
}
