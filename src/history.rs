use crate::errors;
use regex::Regex;
use std::time::Duration;

#[derive(Debug)]
pub struct HistoryEntry {
    command: String,
    timestamp: u64, // TODO: change this a real timestamp
    duration: Duration,
}

impl TryFrom<String> for HistoryEntry {
    type Error = errors::HistoryError;

    fn try_from(history_line: String) -> Result<Self, Self::Error> {
        // TODO: Do not compile this here. See https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        let re =
            Regex::new(r": (?P<timestamp>\d{10}):(?P<elapsed_seconds>\d);(?P<command>.*)").unwrap();

        match re.captures(&history_line) {
            Some(caps) => {
                let timestamp: u64 = caps["timestamp"].parse()?;
                let elapsed_seconds: u64 = caps["elapsed_seconds"].parse()?;
                let command = caps["command"].to_string();

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
        let entry = HistoryEntry::try_from(": 1731884069:1;sleep 2").unwrap();

        assert_eq!(entry.command, "sleep 2".to_string());
        assert_eq!(entry.timestamp, 1731884069);
        assert_eq!(entry.duration, Duration::from_secs(1));
    }
}
