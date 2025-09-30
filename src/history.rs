use crate::entry::HistoryEntry;
use crate::errors;
use chrono::Local;
use expand_tilde::expand_tilde;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub struct History {
    /// The filename where the history was read
    filename: String,

    /// The history entries
    entries: Vec<HistoryEntry>,
}

/// Reads a Zsh history file and processes its contents into a vector of complete commands.
/// This function handles multiline commands (indicated by a trailing backslash `\`) by combining them into a single logical command.
fn preprocess_history<P: AsRef<Path>>(filepath: &P) -> Result<Vec<String>, errors::HistoryError> {
    let mut commands = Vec::new();
    let mut current_command = String::new();

    let name = filepath.as_ref().to_string_lossy().to_string();

    let file = File::open(filepath)
        .map_err(|e| errors::HistoryError::IoError(name.clone(), e.to_string()))?;
    let reader = BufReader::new(file);

    for (counter, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            errors::HistoryError::LineEncodingError((counter + 1).to_string(), e.to_string())
        })?;
        let trimmed = line.trim_end(); // Trim trailing whitespace
        if trimmed.ends_with('\\') {
            // Remove the backslash and keep appending
            current_command.push_str(trimmed);
        } else {
            if !current_command.is_empty() {
                // Still appending a multi-line command
                current_command.push('\n');
            }
            current_command.push_str(trimmed);

            commands.push(current_command.clone());
            current_command.clear();
        }
    }

    if !current_command.is_empty() {
        commands.push(current_command);
    }

    Ok(commands)
}

impl History {
    /// Reads a Zsh history file and populates a `History` struct
    pub fn from_file<P: AsRef<Path>>(filepath: &P) -> Result<Self, errors::HistoryError> {
        let expanded_path =
            expand_tilde(filepath).expect("Failed to expand tilde in the file path");

        let commands = preprocess_history(&expanded_path)?;

        let entries = commands
            .into_iter()
            .filter_map(|line| HistoryEntry::try_from(line).ok())
            .collect::<Vec<HistoryEntry>>();

        Ok(History {
            filename: expanded_path.to_str().unwrap().to_string(),
            entries,
        })
    }

    /// Write the history to the filesystem and optionally take a backup.
    /// Returns the path to the backup file if a backup was taken.
    /// Otherwise, returns `None`.
    pub fn write(&self, backup: bool) -> Result<Option<String>, errors::HistoryError> {
        let backup_path = if backup {
            let now = Local::now().format("%Y-%m-%d-%Hh%Mm%Ss%3fms").to_string();
            let backup_path = format!("{}.{}", self.filename, now);

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

    /// Remove the duplicate commands from the history.
    /// This function retains the last occurrence of a command when duplicates are found.
    pub fn remove_duplicates(&mut self) {
        let before_count = self.entries.len() as f64;
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

        self.entries = new_entries;

        let percent_of_duplicate =
            (before_count - self.entries.len() as f64) / before_count * 100.0;
        println!(
            "{} entries after removing duplicates ({percent_of_duplicate:.0}% of duplicates).",
            self.entries.len(),
        );
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        history.remove_duplicates();
        assert_eq!(history.entries.len(), 3, "Wrong number of history entries!");
        assert_eq!(history.entries[0].command(), "tf fmt -recursive");

        assert_eq!(history.entries[1].command(), "tf apply");
        assert_eq!(*history.entries[1].timestamp(), 1732577157);

        assert_eq!(history.entries[2].command(), "echo 'hello world'");
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
        // The precision of SystemTime can depend on the underlying OS-specific time format. So we add a few milliseconds of sleep to ensure the modified time is different.
        sleep(Duration::from_millis(400));
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

    #[test]
    fn test_write_with_no_change_at_all() {
        // TODO: implement this test
    }
}
