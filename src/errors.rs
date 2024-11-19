use thiserror::Error;
#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("Failed to parse '{0}' as a history entry. Make sure this is a valid entry from a Zsh history file.")]
    EntryMatchingError(String),

    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Error when reading the file '{0}': {1}")]
    IoError(String, String),
}
