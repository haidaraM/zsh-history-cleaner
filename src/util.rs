use crate::errors;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Reads a Zsh history file and processes its contents into a vector of complete commands.
/// This function handles multiline commands (indicated by a trailing backslash `\`) by combining them into a single logical command.
pub(crate) fn read_history_file<P: AsRef<Path>>(
    filepath: &P,
) -> Result<Vec<String>, errors::HistoryError> {
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

/// Helper function to format text with truncation and ellipsis indicator.
/// If the text exceeds max_len, it will be truncated and "..." will be appended.
pub(crate) fn format_truncated(text: &str, max_len: usize, count: usize) -> String {
    if text.len() > max_len {
        format!("{}... ({} times)", &text[..max_len], count)
    } else {
        format!("{} ({} times)", text, count)
    }
}
