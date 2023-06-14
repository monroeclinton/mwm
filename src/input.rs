#[derive(PartialEq)]
pub enum Action {
    // Set a specific workspace to be active
    WorkspaceSetActive(usize),
    // Launch a program
    Spawn(String),
}
