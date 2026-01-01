use crate::entry::HistoryEntry;
use crate::errors;
use crate::utils::read_history_file;
use chrono::{Local, NaiveDate};
use expand_tilde::expand_tilde;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Suffix to append to the backup files before the local timestamp
pub const BACKUP_FILE_SUFFIX: &str = ".zhc_backup_";
/// Timestamp format for the backup files
const BACKUP_FILE_TIMESTAMP_FORMAT: &str = "%Y-%m-%d-%Hh%Mm%Ss%3fms";

pub struct History {
    /// The filename where the history was read
    filename: String,

    /// The history entries
    entries: Vec<HistoryEntry>,
}

impl History {
    /// Reads a Zsh history file and populates a `History` struct
    pub fn from_file<P: AsRef<Path>>(filepath: &P) -> Result<Self, errors::HistoryError> {
        let expanded_path =
            expand_tilde(filepath).expect("Failed to expand tilde in the file path");

        let commands = read_history_file(&expanded_path)?;

        let entries = commands
            .into_iter()
            .filter_map(|line| HistoryEntry::try_from(line).ok())
            .collect::<Vec<HistoryEntry>>();

        Ok(History {
            filename: expanded_path.to_string_lossy().to_string(),
            entries,
        })
    }

    /// Write the history to the filesystem and optionally take a backup.
    /// Returns the path to the backup file if a backup was taken.
    /// Otherwise, returns `None`.
    pub fn write(&self, backup: bool) -> Result<Option<String>, errors::HistoryError> {
        let backup_path = if backup {
            let now = Local::now()
                .format(BACKUP_FILE_TIMESTAMP_FORMAT)
                .to_string();
            let backup_path = format!("{}{}{}", self.filename, BACKUP_FILE_SUFFIX, now);

            println!("Backing up the history to '{backup_path}'");
            fs::copy(&self.filename, backup_path.clone()).map_err(|e| {
                errors::HistoryError::BackUpError(backup_path.clone(), e.to_string())
            })?;

            Some(backup_path)
        } else {
            None
        };

        let output_file = File::create(&self.filename)
            .map_err(|e| errors::HistoryError::IoError(self.filename.clone(), e.to_string()))?;

        let mut writer = BufWriter::new(output_file);

        for entry in &self.entries {
            let line = format!("{}\n", entry.to_history_line());
            writer.write_all(line.as_ref()).unwrap();
        }
        writer.flush().unwrap();

        Ok(backup_path)
    }

    /// Count the number of duplicate commands in the history.
    pub fn duplicate_commands_count(&self) -> usize {
        let mut command_counts: HashMap<&str, usize> = HashMap::new();

        for entry in &self.entries {
            *command_counts.entry(entry.command()).or_insert(0) += 1;
        }

        command_counts.values().filter(|&count| *count > 1).count()
    }

    /// Remove the duplicate commands from the history.
    /// This function retains the last occurrence of a command when duplicates are found.
    /// Returns the number of removed duplicate commands.
    pub fn remove_duplicates(&mut self) -> usize {
        let before_count = self.entries.len();
        let mut command_to_last_index: HashMap<&str, usize> = HashMap::new();

        // Single pass to find last occurrence of each command
        for (index, entry) in self.entries.iter().enumerate() {
            command_to_last_index.insert(entry.command(), index);
        }

        // Create new vector with only the entries at their last occurrence
        let mut new_entries = Vec::with_capacity(command_to_last_index.len());
        for (index, entry) in self.entries.iter().enumerate() {
            if command_to_last_index[entry.command()] == index {
                new_entries.push(entry.clone());
            }
        }

        // Replace the entries with the deduplicated vector
        self.entries = new_entries;

        before_count - self.entries.len()
    }

    /// Remove commands between two dates (inclusive).
    pub fn remove_between_dates(&mut self, start: &NaiveDate, end: &NaiveDate) -> usize {
        let before_count = self.entries.len();

        self.entries.retain(|entry| {
            entry
                .timestamp_as_date_time()
                .map(|dt| {
                    let date = dt.date_naive();
                    !(date >= *start && date <= *end)
                })
                .unwrap_or(true) // Keep entries with invalid timestamps
        });

        let removed_count = before_count - self.entries.len();

        println!(
            "{} commands removed between {} and {}.",
            removed_count,
            start.format("%Y-%m-%d"),
            end.format("%Y-%m-%d"),
        );

        removed_count
    }

    /// Returns the number of entries in the history
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the filename where the history was read
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns a read-only slice of the history entries
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use std::thread::sleep;
    use std::time::Duration;
    use test_helpers::{get_tmp_file, get_tmp_file_with_invalid_utf8};

    #[test]
    fn test_empty_history() {
        assert_eq!(History::from_file(&get_tmp_file("")).unwrap().size(), 0);
    }

    #[test]
    fn test_from_file() {
        let cmds = [
            ": 1732577005:0;tf fmt -recursive",
            ": 1732577037:0;tf apply",
        ];

        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();

        assert_eq!(history.entries.len(), 2, "Wrong number of history entries!");

        assert_eq!(history.entries[0].command(), "tf fmt -recursive");
        assert_eq!(*history.entries[0].duration(), Duration::from_secs(0));
        assert_eq!(*history.entries[0].timestamp(), 1732577005);

        assert_eq!(history.entries[1].command(), "tf apply");
        assert_eq!(*history.entries[1].duration(), Duration::from_secs(0));
        assert_eq!(*history.entries[1].timestamp(), 1732577037);
    }

    #[test]
    fn test_from_file_multiline_commands() {
        let cmds = [
            r#": 1733015058:0;echo 'hello hacha \
world'"#,
            r#": 1732659779:0;echo 'multiple \
line'"#,
            r#": 1732659789:0;reload"#,
        ];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();

        assert_eq!(history.entries.len(), 3, "Wrong number of history entries!");

        // First command
        assert_eq!(
            history.entries[0].command(),
            r#"echo 'hello hacha \
world'"#
        );

        // Second command
        assert_eq!(
            history.entries[1].command(),
            r#"echo 'multiple \
line'"#
        );

        // Third command
        assert_eq!(history.entries[2].command(), "reload");
    }

    #[test]
    fn test_non_utf_8_chars_in_the_fle() {
        // Create a temporary file with non-UTF-8 content using test_helpers
        let tmpfile = get_tmp_file_with_invalid_utf8();

        // Try to read the history file - this should fail with LineEncodingError
        let result = History::from_file(&tmpfile);

        assert!(
            result.is_err(),
            "Expected an error when reading non-UTF-8 content"
        );

        if let Err(error) = result {
            match error {
                errors::HistoryError::LineEncodingError(line_number, error_msg) => {
                    assert_eq!(line_number, "2", "Error should occur on line 2");
                    assert!(
                        error_msg.contains("stream did not contain valid UTF-8"),
                        "Error message should mention UTF-8: {}",
                        error_msg
                    );
                }
                _other_error => {
                    panic!("Expected LineEncodingError, but got a different error type")
                }
            }
        }
    }

    // Remove duplicate commands from the history
    #[test]
    fn test_remove_duplicates() {
        let cmds = [
            ": 1732577005:0;tf fmt -recursive",
            ": 1732577037:0;tf apply",
            ": 1732577157:0;tf apply",
            ": 1732577197:0;echo 'hello world'",
        ];

        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let mut history = History::from_file(&tmp_hist_file).unwrap();

        assert_eq!(history.entries.len(), 4);
        let count = history.remove_duplicates();
        assert_eq!(count, 1, "One duplicate command should have been removed");
        assert_eq!(history.entries.len(), 3, "Wrong number of history entries!");
        assert_eq!(history.entries[0].command(), "tf fmt -recursive");

        assert_eq!(history.entries[1].command(), "tf apply");
        assert_eq!(*history.entries[1].timestamp(), 1732577157);

        assert_eq!(history.entries[2].command(), "echo 'hello world'");
    }

    #[test]
    fn test_duplicate_commands_count() {
        let cmds = [
            ": 1732577005:0;tf fmt -recursive",
            ": 1732577037:0;tf apply",
            ": 1732577157:0;tf apply",
            ": 1732577197:0;echo 'hello world'",
            ": 1732577200:0;echo 'hello world'",
        ];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();
        let duplicate_count = history.duplicate_commands_count();
        assert_eq!(duplicate_count, 2, "There should be 2 duplicate commands");
    }

    // Write the history to a file with a backup
    #[test]
    fn test_write_with_a_backup() {
        // Create a history with some entries
        let cmds = [
            ": 1732577005:0;tf fmt -recursive",
            ": 1732577037:0;tf apply",
            ": 1732577157:0;echo 'hello world'",
        ];

        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let hist_file_modified_before = fs::metadata(&tmp_hist_file).unwrap().modified().unwrap();
        let history = History::from_file(&tmp_hist_file).unwrap();

        // The precision of SystemTime can depend on the underlying OS-specific time format.
        // So we add a few milliseconds of sleep to ensure the modified time is different.
        sleep(Duration::from_millis(500));

        // Write with backup enabled
        let backup_path = history
            .write(true)
            .expect("Could not write history to temporary backup");

        let backup_path = backup_path.expect("Expected Some backup path, got None");

        // Backup file should exist
        assert!(Path::new(&backup_path).exists(), "Backup file should exist");

        // Backup file should contain the original content
        let backup_content = fs::read_to_string(&backup_path).expect("Failed to read backup file");
        let expected_content = format!("{}\n", cmds.join("\n"));
        assert_eq!(
            backup_content, expected_content,
            "Backup file content mismatch"
        );

        // Original file should still exist and contain the history entries
        let original_content =
            fs::read_to_string(tmp_hist_file.path()).expect("Failed to read original file");
        assert_eq!(
            original_content, expected_content,
            "Original file content mismatch"
        );

        // Check if the has been modified
        let hist_file_modified_after = fs::metadata(&tmp_hist_file).unwrap().modified().unwrap();
        assert!(
            hist_file_modified_after > hist_file_modified_before,
            "History file should have been modified. Before: {:?}, After: {:?}",
            hist_file_modified_before,
            hist_file_modified_after
        );

        // Clean up the backup file
        fs::remove_file(&backup_path).unwrap();
    }

    // Write the history to a file without a backup
    #[test]
    fn test_write_with_no_backup() {
        // Create a history with some entries
        let cmds = [
            ": 1732577005:0;tf fmt -recursive",
            ": 1732577037:0;tf apply",
            ": 1732577157:0;echo 'hello world'",
        ];

        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let hist_file_modified_before = fs::metadata(&tmp_hist_file).unwrap().modified().unwrap();
        let history = History::from_file(&tmp_hist_file).unwrap();

        // The precision of SystemTime can depend on the underlying OS-specific time format.
        // So we add a few milliseconds of sleep to ensure the modified time is different.
        sleep(Duration::from_millis(500));

        // Write with backup disabled
        let backup_path = history
            .write(false)
            .expect("Could not write history to temporary backup");
        assert_eq!(backup_path, None, "Expected no backup path");

        // Original file should still exist and contain the history entries
        let expected_content = format!("{}\n", cmds.join("\n"));
        let original_content =
            fs::read_to_string(tmp_hist_file.path()).expect("Failed to read original file");
        assert_eq!(
            original_content, expected_content,
            "Original file content mismatch"
        );

        // Check if the has been modified
        let hist_file_modified_after = fs::metadata(&tmp_hist_file).unwrap().modified().unwrap();
        assert!(
            hist_file_modified_after > hist_file_modified_before,
            "History file should have been modified. Before: {:?}, After: {:?}",
            hist_file_modified_before,
            hist_file_modified_after
        );

        // No backup file should have been created - we can't easily test this without knowing
        // the exact timestamp format, but we've verified that backup_path is None
    }

    // Remove commands between two dates
    #[test]
    fn test_remove_between_dates() {
        let cmds = [
            format!(
                ": {}:0;echo 'first command'",
                Local
                    .with_ymd_and_hms(2020, 1, 1, 12, 0, 0)
                    .unwrap()
                    .timestamp()
            ),
            format!(
                ": {}:0;echo 'second command'",
                Local
                    .with_ymd_and_hms(2024, 2, 1, 12, 0, 0)
                    .unwrap()
                    .timestamp()
            ),
            format!(
                ": {}:0;echo 'third command'",
                Local
                    .with_ymd_and_hms(2025, 3, 3, 12, 0, 0)
                    .unwrap()
                    .timestamp()
            ),
            format!(
                ": {}:0;echo 'fourth command'",
                Local
                    .with_ymd_and_hms(2026, 4, 3, 12, 0, 0)
                    .unwrap()
                    .timestamp()
            ),
            format!(
                ": {}:0;echo 'fifth command'",
                Local
                    .with_ymd_and_hms(2027, 4, 3, 12, 0, 0)
                    .unwrap()
                    .timestamp()
            ),
        ];

        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let mut history = History::from_file(&tmp_hist_file).unwrap();

        let remove_count = history.remove_between_dates(
            &NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            &NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        );

        assert_eq!(remove_count, 2, "We should have removed 2 entries");

        assert_eq!(
            history.entries.len(),
            3,
            "We should have 2 entries left after removal"
        );
        assert_eq!(history.entries[0].command(), "echo 'first command'");
        assert_eq!(history.entries[1].command(), "echo 'fourth command'");

        // Remove with a date range that matches no entries
        let removed_count = history.remove_between_dates(
            &NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
            &NaiveDate::from_ymd_opt(2030, 12, 31).unwrap(),
        );
        assert_eq!(removed_count, 0, "No entries should have been removed");
        assert_eq!(history.entries.len(), 3, "We should still have 3 entries");
    }

    /// Test the date_range function makes sure it correctly identifies the min and max dates
    #[test]
    fn test_date_range() {
        use crate::analyze::HistoryAnalyzer;

        // Common case with multiple entries
        let cmds = [
            ": 1707258478:0;echo 'first command'",
            ": 1766959482:0;echo 'second command'",
        ];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();
        let analyzer = HistoryAnalyzer::new(&history);
        let date_range = analyzer.date_range().unwrap();
        assert_eq!(date_range.0, NaiveDate::from_ymd_opt(2024, 2, 6).unwrap());
        assert_eq!(date_range.1, NaiveDate::from_ymd_opt(2025, 12, 28).unwrap());

        // Empty history
        let cmds: [&str; 0] = [];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let empty_history = History::from_file(&tmp_hist_file).unwrap();
        let empty_analyzer = HistoryAnalyzer::new(&empty_history);
        assert!(empty_analyzer.date_range().is_none());

        // Reverse order entries
        let cmds = [
            ": 1766959482:0;echo 'second command'",
            ": 1707258478:0;echo 'first command'",
        ];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();
        let analyzer = HistoryAnalyzer::new(&history);
        let date_range = analyzer.date_range().unwrap();
        assert_eq!(date_range.0, NaiveDate::from_ymd_opt(2024, 2, 6).unwrap());
        assert_eq!(date_range.1, NaiveDate::from_ymd_opt(2025, 12, 28).unwrap());
    }
}
