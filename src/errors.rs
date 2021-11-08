use thiserror::Error;

pub type Result<T> = std::result::Result<T, MWMError>;

#[derive(Debug, Error)]
pub enum MWMError {
}
