use crate::errors;
use regex::Regex;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// Represents a single history entry from a Zsh history file.
///
/// # Fields
/// - `command`: The command executed by the user.
/// - `timestamp`: The UNIX timestamp when the command was executed.
/// - `duration`: The time it took to execute the command.
#[derive(Debug)]
pub struct HistoryEntry {
    /// The command executed by the user.
    command: String,

    /// The UNIX timestamp when the command was executed.
    timestamp: u64, // TODO: change this a real timestamp

    /// The time it took to execute the command.
    duration: Duration,
}

impl HistoryEntry {
    /// Provides a human-readable description of the history entry.
    ///
    /// # Returns
    /// A string summarizing the command, timestamp, and duration.
    pub fn describe(&self) -> String {
        format!(
            "Command: '{}', Executed at: {}, Duration: {}s",
            self.command,
            self.timestamp,
            self.duration.as_secs()
        )
    }
}

impl Hash for HistoryEntry {
    /// Custom hash implementation, only considers the `command` field.
    ///
    /// This ensures that entries with the same command are treated as identical,
    /// regardless of their timestamp or duration.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.command.hash(state);
    }
}

/// Marker trait to complete equivalence relation
impl Eq for HistoryEntry {}

impl PartialEq for HistoryEntry {
    /// Equality is determined solely based on the `command` field.
    ///
    /// Entries with the same `command` are considered equal,
    /// even if their `timestamp` or `duration` fields differ.
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
    }
}

impl TryFrom<String> for HistoryEntry {
    type Error = errors::HistoryError;

    fn try_from(history_line: String) -> Result<Self, Self::Error> {
        // TODO: Do not compile this here. See https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        let re =
            Regex::new(r": (?P<timestamp>\d{10}):(?P<elapsed_seconds>\d+);(?P<command>.*)").unwrap();

        match re.captures(&history_line) {
            Some(caps) => {
                let timestamp: u64 = caps["timestamp"].parse()?;
                let elapsed_seconds: u64 = caps["elapsed_seconds"].parse()?;
                let command = caps["command"].trim().to_string();

                Ok(HistoryEntry {
                    command,
                    timestamp,
                    duration: Duration::from_secs(elapsed_seconds),
                })
            }
            None => Err(errors::HistoryError::MatchingError(history_line)),
        }
    }
}

impl TryFrom<&str> for HistoryEntry {
    type Error = errors::HistoryError;

    fn try_from(history_line: &str) -> Result<Self, Self::Error> {
        history_line.to_string().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_history_entry_from_string() {
        let entry = HistoryEntry::try_from(": 1731884069:0;sleep 2".to_string()).unwrap();

        assert_eq!(entry.command, "sleep 2".to_string());
        assert_eq!(entry.timestamp, 1731884069);
        assert_eq!(entry.duration, Duration::from_secs(0));
    }

    #[test]
    fn test_parsing_history_entry_from_str() {
        let entry = HistoryEntry::try_from(": 1731884069:10;sleep 2").unwrap();

        assert_eq!(entry.command, "sleep 2".to_string());
        assert_eq!(entry.timestamp, 1731884069);
        assert_eq!(entry.duration, Duration::from_secs(10));
    }

    #[test]
    fn test_parsing_history_entry_from_invalid() {
        let entry = HistoryEntry::try_from(": 1731884069;");
        assert!(entry.is_err());
    }
    
    #[test]
    fn test_entry_equality() {
        let entry_1 = HistoryEntry::try_from(": 1731884069:0;ls".to_string()).unwrap();
        let entry_2 = HistoryEntry::try_from(": 1731084669:10;ls".to_string()).unwrap();
        assert_eq!(entry_1, entry_2);
    }
}
