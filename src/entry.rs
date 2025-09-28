use crate::errors;
use chrono::Local;
use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::time::Duration;

// Compile regex once and reuse. See https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
static HISTORY_LINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^: (?P<timestamp>\d{10}):(?P<elapsed_seconds>\d+);(?P<command>.*(\n.*)*)")
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

    /// Converts the UNIX timestamp to a `DateTime<Local>`, returning None for invalid timestamps.
    pub fn timestamp_as_local_date_time(&self) -> Option<DateTime<Local>> {
        DateTime::from_timestamp(self.timestamp as i64, 0).map(|dt| dt.with_timezone(&Local))
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
            "Command executed at '{}' for '{}s': {}",
            self.timestamp_as_local_date_time()
                .map_or_else(|| self.timestamp.to_string(), |dt| dt.to_string()),
            self.duration.as_secs(),
            self.command,
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
                let command: String = caps["command"].to_string();

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
    use pretty_assertions::{assert_eq, assert_ne};

    // Test with simple commands without special characters or multiple lines
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

    // Test with a command that includes a backslash for line continuation
    #[test]
    fn test_two_lines_command() {
        let cmd = r#": 1731622185:9;brew update\
brew install opentofu"#;
        let entry = HistoryEntry::try_from(cmd).unwrap();
        let expected_cmd = r#"brew update\
brew install opentofu"#;

        assert_eq!(entry.timestamp, 1731622185);
        assert_eq!(entry.duration, Duration::from_secs(9));
        assert_eq!(entry.command, expected_cmd);
    }

    // Test with a command that spans multiple lines using backslashes
    #[test]
    fn test_multiple_lines_command() {
        let cmd = r#": 1733005037:0;docker run -d --name mysql \\
-v mysql:/var/lib/mysql \\
-e MYSQL_ROOT_PASSWORD=vaalala -e MYSQL_DATABASE=vaalala -e MYSQL_USER=vaalala \\
-e MYSQL_PASSWORD=vaalala \\
-p 3306:3306 mysql:8
"#;
        let entry = HistoryEntry::try_from(cmd).unwrap();

        let expected_cmd = r#"docker run -d --name mysql \\
-v mysql:/var/lib/mysql \\
-e MYSQL_ROOT_PASSWORD=vaalala -e MYSQL_DATABASE=vaalala -e MYSQL_USER=vaalala \\
-e MYSQL_PASSWORD=vaalala \\
-p 3306:3306 mysql:8
"#;
        println!("{}", entry.command);
        assert_eq!(entry.command, expected_cmd);
    }

    // Test with a command that includes a backslash at the end of the line
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

    // Test the conversion of HistoryEntry back to the original history line format
    #[test]
    fn test_to_history_line() {
        let cmds = [
            ": 1731317544:12;for d in VWT.*; do l $d; done",
            r#": 1733005037:0;docker run -d --name mysql \\
-v mysql:/var/lib/mysql \\
-e MYSQL_ROOT_PASSWORD=vaalala -e MYSQL_DATABASE=vaalala -e MYSQL_USER=vaalala \\
-e MYSQL_PASSWORD=vaalala \\
-p 3306:3306 mysql:8
"#,
        ];

        for cmd in cmds {
            let entry = HistoryEntry::try_from(cmd).unwrap();
            assert_eq!(entry.to_history_line(), cmd);
        }
    }

    // Specifically test the conversion for a multi-line command
    #[test]
    fn test_to_history_line_for_multiple_lines() {
        let cmd = r#": 1733005037:0;docker run -d --name mysql \\
-v mysql:/var/lib/mysql \\
-e MYSQL_ROOT_PASSWORD=vaalala -e MYSQL_DATABASE=vaalala -e MYSQL_USER=vaalala \\
-e MYSQL_PASSWORD=vaalala \\
-p 3306:3306 mysql:8
"#;
        let for_loop = HistoryEntry::try_from(cmd).unwrap();
        assert_eq!(for_loop.to_history_line(), cmd.to_string());
    }

    // Test with a more complex command that includes special characters and multiple statements
    #[test]
    fn test_parsing_complex_history_entry() {
        let complex =
            HistoryEntry::try_from(": 1731317544:12;for d in VWT.*; do l $d; done").unwrap();

        assert_eq!(complex.command, "for d in VWT.*; do l $d; done".to_string());
        assert_eq!(complex.timestamp, 1731317544);
        assert_eq!(complex.duration, Duration::from_secs(12));
    }

    // Test with an invalid history entry that does not match the expected format
    #[test]
    fn test_parsing_history_entry_no_matching() {
        let entry = HistoryEntry::try_from(": 1731884069;");
        assert!(matches!(
            entry.unwrap_err(),
            errors::HistoryError::EntryMatchingError(_)
        ));
    }

    // Test with an invalid history entry that has a negative duration
    #[test]
    fn test_parsing_history_entry_from_invalid_duration() {
        let entry = HistoryEntry::try_from(": 1731884069:-10;sleep 2");
        assert!(entry.is_err());
    }

    // Test the equality and inequality of HistoryEntry instances based on the command field
    #[test]
    fn test_entry_equality() {
        let entry_1 = HistoryEntry::try_from(": 1731884069:0;ls".to_string()).unwrap();
        let entry_2 = HistoryEntry::try_from(": 1731084669:10;ls".to_string()).unwrap();
        assert_eq!(entry_1, entry_2);

        let entry_3 = HistoryEntry::try_from(": 1731084669:1;terraform apply".to_string()).unwrap();
        assert_ne!(entry_1, entry_3);
    }

    #[test]
    fn test_timestamp_as_local_date_time() {
        let entry = HistoryEntry::try_from(": 1759099958:0;ls").unwrap();
        assert_eq!(
            entry.timestamp_as_local_date_time().unwrap().timestamp() as u64,
            1759099958
        );

        assert_eq!(
            entry.timestamp,
            entry.timestamp_as_local_date_time().unwrap().timestamp() as u64
        );
    }

    #[test]
    fn test_timestamp_as_local_date_time_edge_cases() {
        // Test timestamp 0 (Unix epoch)
        let entry_zero = HistoryEntry::try_from(": 0000000000:0;ls").unwrap();
        assert!(entry_zero.timestamp_as_local_date_time().is_some());
    }
}
