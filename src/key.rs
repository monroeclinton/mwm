#[derive(Eq, PartialEq, Hash)]
pub struct KeyPair {
    pub modifiers: u16,
    pub keysym: u32,
}
