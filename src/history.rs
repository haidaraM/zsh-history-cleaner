use crate::errors;
use crate::history_entry::HistoryEntry;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::SystemTime;

pub struct History {
    /// The filename where the history was read
    filename: String,

    /// The history entries
    pub entries: Vec<HistoryEntry>,
}

impl History {
    /// Reads a Zsh history file and populates a `History` struct.
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Result<Self, errors::HistoryError> {
        let path_ref = filepath.as_ref();
        let name = path_ref.to_string_lossy().to_string();

        let file = File::open(path_ref)
            .map_err(|e| errors::HistoryError::IoError(name.clone(), e.to_string()))?;
        let reader = BufReader::new(file);

        let entries = reader
            .lines()
            .filter_map(|line| {
                line.ok()
                    .and_then(|line_str| HistoryEntry::try_from(line_str).ok())
            })
            .collect::<Vec<HistoryEntry>>();

        Ok(History {
            filename: name,
            entries,
        })
    }

    /// Write the history to the filesystem and optional take a backup
    pub fn write(&self, backup: bool) -> Result<(), errors::HistoryError> {
        if backup {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let backup_path = format!("{}.{}", self.filename, now);

            println!("Backing up the history to '{backup_path}'...");
            let backup_file = File::create(&backup_path)
                .map_err(|e| errors::HistoryError::IoError(backup_path, e.to_string()))?;

            let mut writer = BufWriter::new(backup_file);

            for entry in &self.entries {
                let line = format!("{}\n", entry.to_history_line());
                writer.write_all(line.as_ref()).unwrap();
            }

            writer.flush().unwrap();
        }

        Ok(())
    }

    /// Remove the duplicate commands from the history.
    pub fn remove_duplicates(&mut self) {
        let before_count = self.entries.len() as f64;
        let mut seen = HashSet::new();

        self.entries
            .retain(|entry| seen.insert(entry.command().to_string()));

        let percent_of_duplicate =
            (before_count - self.entries.len() as f64) / before_count * 100_f64;

        println!(
            "{} entries after removing duplicates ({percent_of_duplicate:.0}% of duplicates).",
            self.entries.len(),
        );
    }
}
