use thiserror::Error;
#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("'{0}' doesn't seem to match an history entry. Make sure this is a valid entry from a Zsh history file.")]
    MatchingError(String),

    #[error("Failed to parse integer: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}
