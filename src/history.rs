use crate::entry::HistoryEntry;
use crate::errors;
use crate::util::{TERMINAL_MAX_WIDTH, format_rank_icon, format_truncated, read_history_file};
use chrono::{Duration, Local, NaiveDate};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use console::style;
use expand_tilde::expand_tilde;
use humanize_duration::Truncate;
use humanize_duration::prelude::DurationExt;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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

        self.entries = new_entries;

        before_count - self.entries.len()
    }

    /// Return the top n most frequent commands.
    /// If n is 0, returns an empty vector.
    pub fn top_n_commands(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 || self.entries.is_empty() {
            return Vec::new();
        }

        // Count occurrences of each command. The key is the command string slice.
        let mut commands_count: HashMap<&str, usize> = HashMap::new();

        for entry in &self.entries {
            if let Some(command) = entry.valid_command() {
                *commands_count.entry(command).or_insert(0) += 1;
            }
        }

        let mut count_vec: Vec<(&str, usize)> = commands_count.into_iter().collect();
        // sort by count descending (then command name for ties), and take top n
        count_vec.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        count_vec.truncate(n);

        count_vec
            .into_iter()
            .map(|(cmd, count)| (cmd.to_string(), count))
            .collect()
    }

    /// Return the top n most frequent binaries (first word of the command).
    /// If n is 0, returns an empty vector.
    pub fn top_n_binaries(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 || self.entries.is_empty() {
            return Vec::new();
        }

        // Count occurrences of each binary (first word of the command)
        let mut binaries_count: HashMap<&str, usize> = HashMap::new();

        for entry in &self.entries {
            if let Some(command) = entry.valid_command()
                && let Some(binary) = command.split_whitespace().next()
            {
                *binaries_count.entry(binary).or_insert(0) += 1;
            }
        }

        let mut count_vec: Vec<(&str, usize)> = binaries_count.into_iter().collect();
        // sort by count descending (then binary name for ties), and take top n
        count_vec.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        count_vec.truncate(n);

        count_vec
            .into_iter()
            .map(|(bin, count)| (bin.to_string(), count))
            .collect()
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

    /// Analyze the History and return a TimeAnalysis struct
    pub fn analyze_by_time(&self) -> TimeAnalysis {
        let date_range = self.date_range().unwrap_or_else(|| {
            let now = Local::now().date_naive();
            (now, now)
        });
        TimeAnalysis {
            filename: self.filename.clone(),
            size: self.entries.len(),
            date_range,
            top_n_commands: self.top_n_commands(10),
            top_n_binaries: self.top_n_binaries(10),
        }
    }

    /// Returns the range of dates covered by the commands (min_date, max_date)
    pub fn date_range(&self) -> Option<(NaiveDate, NaiveDate)> {
        self.entries
            .iter()
            .filter_map(|entry| entry.timestamp_as_date_time())
            .map(|dt| dt.date_naive())
            .fold(None, |acc: Option<(NaiveDate, NaiveDate)>, current_date| {
                Some(match acc {
                    None => (current_date, current_date), // Initialize with the first date
                    Some((min, max)) => (min.min(current_date), max.max(current_date)),
                })
            })
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
}

/// Represents the analysis of history commands by time
/// # Fields
/// - `filename`: The filename where the history was read
/// - `size`: The number of commands in the history
/// - `date_range`: The range of dates covered by the commands (min_date, max_date)
#[derive(Debug)]
pub struct TimeAnalysis {
    /// The filename where the history was read
    pub filename: String,
    /// The number of commands in the history
    pub size: usize,
    /// The range of dates covered by the commands (min_date, max_date)
    pub date_range: (NaiveDate, NaiveDate),
    /// The top N most frequent commands
    pub top_n_commands: Vec<(String, usize)>,
    /// The top N most frequent binaries
    pub top_n_binaries: Vec<(String, usize)>,
    // The number of duplicate commands found
    // pub duplicates_count: usize,
    //pub commands_per_day: HashMap<NaiveDate, usize>,
    //pub commands_per_week: HashMap<u32, usize>, // Week number
    //pub commands_per_month: HashMap<(i32, u32), usize>, // (Year, Month)
    //pub commands_per_year: HashMap<i32, usize>, // Year
}

/// Display implementation for TimeAnalysis.
/// This formats the analysis in a human-readable way.
impl Display for TimeAnalysis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let duration: Duration = self.date_range.1.signed_duration_since(self.date_range.0);
        let human_duration = duration.human(Truncate::Day);

        // Create a visually appealing stats box
        let box_width = TERMINAL_MAX_WIDTH as usize;
        let top_border = format!("╭{}╮", "─".repeat(box_width - 2));
        let bottom_border = format!("╰{}╯", "─".repeat(box_width - 2));

        // Format the title
        let title = format!(
            "📊 History Analysis for {}",
            style(&self.filename).cyan().bold()
        );

        // Format date range with colored dates
        let date_range_text = format!(
            "🗓️ {} → {} {}",
            style(&self.date_range.0).green().bold(),
            style(&self.date_range.1).green().bold(),
            style(format!("({})", human_duration)).dim()
        );

        // Format total commands with highlighted number
        let total_commands = format!("📝 Total Commands: {}", style(&self.size).yellow().bold());

        // Print the stats box
        writeln!(f, "{}", style(top_border).blue())?;
        writeln!(f, "{} {} {}", style("│").blue(), title, style("│").blue())?;
        writeln!(
            f,
            "{} {} {}",
            style("│").blue(),
            date_range_text,
            style("│").blue()
        )?;
        writeln!(
            f,
            "{} {} {}",
            style("│").blue(),
            total_commands,
            style("│").blue()
        )?;
        writeln!(f, "{}", style(bottom_border).blue())?;
        writeln!(f)?;

        // Section header for top items
        writeln!(
            f,
            "{} {}",
            style("🔥").bold(),
            style(format!(
                "Top {} Most Used:",
                self.top_n_commands.len().max(self.top_n_binaries.len())
            ))
            .magenta()
            .bold()
        )?;

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("").add_attribute(Attribute::Bold),
                Cell::new(style("Commands").cyan().bold().to_string())
                    .add_attribute(Attribute::Bold),
                Cell::new(style("Binaries").cyan().bold().to_string())
                    .add_attribute(Attribute::Bold),
            ])
            .set_width(TERMINAL_MAX_WIDTH.into());

        // The top N commands and binaries may have different lengths
        for i in 0..self.top_n_commands.len().max(self.top_n_binaries.len()) {
            let rank_cell = Cell::new(format_rank_icon(i + 1));

            let command_cell = self
                .top_n_commands
                .get(i)
                .map(|(cmd, count)| Cell::new(format_truncated(cmd, 39, *count)))
                .unwrap_or_else(|| Cell::new(""));

            let binary_cell = self
                .top_n_binaries
                .get(i)
                .map(|(bin, count)| Cell::new(format_truncated(bin, 39, *count)))
                .unwrap_or_else(|| Cell::new(""));

            table.add_row(vec![rank_cell, command_cell, binary_cell]);
        }

        writeln!(f, "{table}")?;

        write!(f, "")
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
        // Common case with multiple entries
        let cmds = [
            ": 1707258478:0;echo 'first command'",
            ": 1766959482:0;echo 'second command'",
        ];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();
        let date_range = history.date_range().unwrap();
        assert_eq!(date_range.0, NaiveDate::from_ymd_opt(2024, 2, 6).unwrap());
        assert_eq!(date_range.1, NaiveDate::from_ymd_opt(2025, 12, 28).unwrap());

        // Empty history
        let cmds: [&str; 0] = [];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let empty_history = History::from_file(&tmp_hist_file).unwrap();
        assert!(empty_history.date_range().is_none());

        // Reverse order entries
        let cmds = [
            ": 1766959482:0;echo 'second command'",
            ": 1707258478:0;echo 'first command'",
        ];
        let tmp_hist_file = get_tmp_file(cmds.join("\n").as_str());
        let history = History::from_file(&tmp_hist_file).unwrap();
        let date_range = history.date_range().unwrap();
        assert_eq!(date_range.0, NaiveDate::from_ymd_opt(2024, 2, 6).unwrap());
        assert_eq!(date_range.1, NaiveDate::from_ymd_opt(2025, 12, 28).unwrap());
    }
}
