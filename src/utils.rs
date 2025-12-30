use crate::errors;
use console::style;
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

/// Helper function to truncate the text used for displaying the command and executables in table cells.
/// If the text exceeds max_len, it will be truncated and "..." will be appended.
pub(crate) fn truncate_count_text_for_table_cell(
    text: &str,
    max_len: usize,
    count: usize,
) -> String {
    if text.len() > max_len {
        format!(
            "{}... {}",
            &text[..max_len],
            style(format!("({} times)", count)).dim().italic()
        )
    } else {
        format!(
            "{} {}",
            text,
            style(format!("({} times)", count)).dim().italic()
        )
    }
}

/// Helper function to format ranking with medal icons for top 3.
pub(crate) fn format_rank_icon(rank: usize) -> String {
    match rank {
        1 => "🥇".to_string(),
        2 => "🥈".to_string(),
        3 => "🥉".to_string(),
        _ => format!("{}", rank),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::strip_ansi_codes;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_truncate_text_for_table_cell() {
        let text = "This is a very long command that exceeds the maximum length.";
        let truncated = truncate_count_text_for_table_cell(text, 20, 5);
        // Remove ANSI escape codes for testing
        let truncated = strip_ansi_codes(&truncated);
        assert_eq!(truncated, "This is a very long ... (5 times)");
    }

    #[test]
    fn test_format_rank_icon() {
        assert_eq!(format_rank_icon(1), "🥇");
        assert_eq!(format_rank_icon(2), "🥈");
        assert_eq!(format_rank_icon(3), "🥉");
        assert_eq!(format_rank_icon(4), "4");
    }
}
