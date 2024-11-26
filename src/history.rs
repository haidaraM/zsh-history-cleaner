use crate::errors;
use crate::history_entry::HistoryEntry;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

pub struct History {
    /// The filename where the history was read
    filename: String,

    /// The history entries
    pub entries: Vec<HistoryEntry>,
}

fn preprocess_history<P: AsRef<Path>>(filepath: &P) -> Result<Vec<String>, errors::HistoryError> {
    let mut commands = Vec::new();
    let mut current_command = String::new();

    let path_ref = filepath.as_ref();
    let name = path_ref.to_string_lossy().to_string();

    let file = File::open(path_ref)
        .map_err(|e| errors::HistoryError::IoError(name.clone(), e.to_string()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|e| errors::HistoryError::IoError(name.clone(), e.to_string()))?;
        let trimmed = line.trim_end(); // Trim trailing whitespace
        if trimmed.ends_with('\\') {
            // Remove the backslash and keep appending
            current_command.push_str(trimmed.strip_suffix('\\').unwrap());
        } else {
            // Final line of a command
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
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Result<Self, errors::HistoryError> {
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
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("The SystemTime is before UNIX EPOCH! which should not happen!")
                .as_secs();
            let backup_path = format!("{}.{}", self.filename, now);

            println!("Backing up the history to '{backup_path}'...");
            fs::copy(&self.filename, backup_path.clone())
                .map_err(|e| errors::HistoryError::BackUpError(backup_path, e.to_string()))?;
        }
        // TODO: handle multi lines before writing
        // TODO: write the entries to the file
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

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_file() {}
}
