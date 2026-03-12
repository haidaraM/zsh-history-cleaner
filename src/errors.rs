use thiserror::Error;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error(
        "Failed to parse '{0}' as a history entry. Make sure this is a valid entry from a Zsh history file."
    )]
    EntryMatchingError(String),

    #[error("Failed to parse integer: {0}.")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("IO error when reading the file: {0}.")]
    IOError(#[from] std::io::Error),

    #[error("Error when handling the file '{0}': {1}.")]
    FileIoError(String, String),

    #[error("Error when reading line {0}: {1}.")]
    LineEncodingError(String, String),

    #[error("Error when backing up the history to '{0}': {1}.")]
    BackUpError(String, String),
}
