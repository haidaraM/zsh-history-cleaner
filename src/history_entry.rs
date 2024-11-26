use crate::errors;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::time::Duration;

// Compile regex once and reuse. See https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
static HISTORY_LINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^: (?P<timestamp>\d{10}):(?P<elapsed_seconds>\d+);(?P<command>.*(\n.*)?)")
        .expect("The regex to parse the history should compile")
});

/// Represents a single history entry from a Zsh history file.
///
/// # Fields
/// - `command`: The command executed by the user.
/// - `timestamp`: The UNIX timestamp when the command was executed.
/// - `duration`: The time it took to execute the command.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The command executed by the user.
    command: String,

    /// The UNIX timestamp when the command was executed.
    timestamp: u64,

    /// The time it took to execute the command.
    duration: Duration,
}

impl HistoryEntry {
    /// Converts the `HistoryEntry` into the Zsh history file format.
    pub fn to_history_line(&self) -> String {
        format!(
            ": {}:{};{}",
            self.timestamp,
            self.duration.as_secs(),
            self.command
        )
    }

    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn duration(&self) -> &Duration {
        &self.duration
    }
}

/// Provides a human-readable description of the history entry.
impl Display for HistoryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command: '{}', Executed at: {}, Duration: {}s",
            self.command,
            self.timestamp,
            self.duration.as_secs()
        )
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

    fn try_from(history_command: String) -> Result<Self, Self::Error> {
        HISTORY_LINE_REGEX
            .captures(&history_command)
            .ok_or_else(|| errors::HistoryError::EntryMatchingError(history_command.clone()))
            .and_then(|caps| {
                let timestamp: u64 = caps["timestamp"].parse()?;
                let elapsed_seconds: u64 = caps["elapsed_seconds"].parse()?;
                let command: String = caps["command"].trim().to_string();

                Ok(HistoryEntry {
                    command,
                    timestamp,
                    duration: Duration::from_secs(elapsed_seconds),
                })
            })
    }
}

impl TryFrom<&str> for HistoryEntry {
    type Error = errors::HistoryError;

    fn try_from(history_command: &str) -> Result<Self, Self::Error> {
        history_command.to_string().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_simple_history_entry() {
        let sleep = HistoryEntry::try_from(": 1731884069:0;sleep 2".to_string()).unwrap();

        assert_eq!(sleep.command, "sleep 2".to_string());
        assert_eq!(sleep.timestamp, 1731884069);
        assert_eq!(sleep.duration, Duration::from_secs(0));

        let cargo_build = HistoryEntry::try_from(": 1731884069:10;cargo build").unwrap();

        assert_eq!(cargo_build.command, "cargo build".to_string());
        assert_eq!(cargo_build.timestamp, 1731884069);
        assert_eq!(cargo_build.duration, Duration::from_secs(10));
    }

    #[test]
    fn test_multiline_command() {
        let cmd = r#": 1731622185:9;brew update\
    brew install opentofu"#;
        let entry = HistoryEntry::try_from(cmd).unwrap();
        let expected_cmd = r#"brew update\
    brew install opentofu"#;

        assert_eq!(entry.timestamp, 1731622185);
        assert_eq!(entry.duration, Duration::from_secs(9));
        assert_eq!(entry.command, expected_cmd);
    }

    #[test]
    fn test_multiline_command_back_slash_at_the_end() {
        let cmd = r#": 1732663091:0;echo 'hello hacha\
world'\"#;
        let entry = HistoryEntry::try_from(cmd).unwrap();
        let expected_cmd = r#"echo 'hello hacha\
world'\"#;

        assert_eq!(entry.timestamp, 1732663091);
        assert_eq!(entry.duration, Duration::from_secs(0));
        assert_eq!(entry.command, expected_cmd);
    }

    #[test]
    fn test_to_history_line() {
        let cmd = ": 1731317544:12;for d in VWT.*; do l $d; done";
        let for_loop = HistoryEntry::try_from(cmd).unwrap();
        assert_eq!(for_loop.to_history_line(), cmd.to_string());
    }

    #[test]
    fn test_parsing_complex_history_entry() {
        let complex =
            HistoryEntry::try_from(": 1731317544:12;for d in VWT.*; do l $d; done").unwrap();

        assert_eq!(complex.command, "for d in VWT.*; do l $d; done".to_string());
        assert_eq!(complex.timestamp, 1731317544);
        assert_eq!(complex.duration, Duration::from_secs(12));
    }

    #[test]
    fn test_parsing_history_entry_no_matching() {
        let entry = HistoryEntry::try_from(": 1731884069;");
        assert!(matches!(
            entry.unwrap_err(),
            errors::HistoryError::EntryMatchingError(_)
        ));
    }

    #[test]
    fn test_parsing_history_entry_from_invalid_duration() {
        let entry = HistoryEntry::try_from(": 1731884069:-10;sleep 2");
        assert!(entry.is_err());
    }

    #[test]
    fn test_entry_equality() {
        let entry_1 = HistoryEntry::try_from(": 1731884069:0;ls".to_string()).unwrap();
        let entry_2 = HistoryEntry::try_from(": 1731084669:10;ls".to_string()).unwrap();
        assert_eq!(entry_1, entry_2);

        let entry_3 = HistoryEntry::try_from(": 1731084669:1;terraform apply".to_string()).unwrap();
        assert_ne!(entry_1, entry_3);
    }

    #[test]
    fn test_display_history() {
        let history = HistoryEntry::try_from(": 1731884069:10;cd ~").unwrap();
        assert_eq!(
            "Command: 'cd ~', Executed at: 1731884069, Duration: 10s",
            format!("{}", history)
        );
    }
}
