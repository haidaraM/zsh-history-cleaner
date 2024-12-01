use crate::errors;
use crate::history_entry::HistoryEntry;
use chrono::Local;
use std::collections::HashSet;
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

    let path_ref = filepath.as_ref();
    let name = path_ref.to_string_lossy().to_string();

    let file = File::open(filepath)
        .map_err(|e| errors::HistoryError::IoError(name.clone(), e.to_string()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|e| errors::HistoryError::IoError(name.clone(), e.to_string()))?;
        let trimmed = line.trim_end(); // Trim trailing whitespace
        if trimmed.ends_with('\\') {
            // Remove the backslash and keep appending
            current_command.push_str(trimmed);
        } else {
            if !current_command.is_empty() {
                // Still appending a multi line command
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
        // TODO: expand user in the filepath.

        let commands = preprocess_history(&filepath)?;

        let entries = commands
            .into_iter()
            .filter_map(|line| HistoryEntry::try_from(line).ok())
            .collect::<Vec<HistoryEntry>>();

        Ok(History {
            filename: filepath.as_ref().to_string_lossy().to_string(),
            entries,
        })
    }

    /// Write the history to the filesystem and optionally take a backup
    pub fn write(&self, backup: bool) -> Result<(), errors::HistoryError> {
        if backup {
            let now = Local::now().format("%Y-%m-%d-%H:%M:%S%.3f").to_string();

            let backup_path = format!("{}.{}", self.filename, now);

            println!("Backing up the history to '{backup_path}'...");
            fs::copy(&self.filename, backup_path.clone())
                .map_err(|e| errors::HistoryError::BackUpError(backup_path, e.to_string()))?;
        }

        let output_file = File::create(&self.filename)
            .map_err(|e| errors::HistoryError::IoError(self.filename.clone(), e.to_string()))?;

        let mut writer = BufWriter::new(output_file);

        for entry in &self.entries {
            let line = format!("{}\n", entry.to_history_line());
            writer.write_all(line.as_ref()).unwrap();
        }

        writer.flush().unwrap();
        Ok(())
    }

    /// Remove the duplicate commands from the history.
    pub fn remove_duplicates(&mut self) {
        let before_count = self.entries.len() as f64;
        let mut seen = HashSet::new();

        self.entries
            .retain(|entry| seen.insert(entry.command().to_string()));

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::time::Duration;
    use test_helpers::get_tmp_file;

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
    fn test_remove_duplicates() {}
}
