use crate::errors;
use console::style;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Reads a Zsh history file and processes its contents into a vector of complete commands.
/// This function handles multiline commands (indicated by a trailing backslash `\`) by combining
/// them into a single logical command.
/// It also takes of ZSH
pub(crate) fn read_history_file<P: AsRef<Path>>(
    filepath: &P,
) -> Result<Vec<String>, errors::HistoryError> {
    let mut commands = Vec::new();
    let mut current_command = String::new();
    let mut raw_line = Vec::new();
    let mut counter = 0;

    let name = filepath.as_ref().to_string_lossy().to_string();

    let file = File::open(filepath)
        .map_err(|e| errors::HistoryError::FileIoError(name.clone(), e.to_string()))?;
    let mut reader = BufReader::new(file);

    loop {
        raw_line.clear();
        if reader.read_until(b'\n', &mut raw_line)? == 0 {
            break; // EOF
        }

        let decoded = decode_metafied(&raw_line);
        let line = String::from_utf8(decoded).map_err(|e| {
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

        counter += 1;
    }

    if !current_command.is_empty() {
        commands.push(current_command);
    }

    Ok(commands)
}

/// Decodes Zsh's "metafied" byte encoding into plain bytes.
///
/// Zsh does not store history as raw UTF-8. Instead, bytes in the range
/// `0x80..=0x9F` (i.e. those whose high bit is set but below `0xA0`) are
/// encoded using a two-byte escape sequence:
///
///   `0x83` followed by `(original_byte ^ 0x20)`
///
/// The escape byte itself (`0x83`, called "Meta") is thus never stored
/// literally — it always introduces an escape sequence. The null byte
/// (`0x00`) is also metafied as `0x83 0x20` (since `0x00 ^ 0x20 = 0x20`).
///
/// Decoding reverses this: whenever `0x83` is encountered, the next byte
/// is XOR'd with `0x20` to recover the original. Because XOR is its own
/// inverse (`(b ^ 0x20) ^ 0x20 == b`), the same operation both encodes
/// and decodes.
///
/// A lone `0x83` at the end of the input (no following byte) is passed
/// through unchanged, as valid Zsh history should never produce this.
///
/// Reference: <https://www.zsh.org/mla/users/2011/msg00154.html>
fn decode_metafied(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    const META_BYTE: u8 = 0x83;

    while i < bytes.len() {
        if bytes[i] == META_BYTE && i + 1 < bytes.len() {
            // Escape sequence: discard the 0x83 and XOR the next byte
            out.push(bytes[i + 1] ^ 0x20);
            i += 2;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

/// Helper function to truncate the text used for displaying the command and executables in table cells.
/// If the text exceeds max_len, it will be truncated and "..." will be appended.
pub(crate) fn truncate_count_text(text: &str, max_len: usize, count: usize) -> String {
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

/// Helper function to truncate text from the left side. If the text exceeds max_len, it will be truncated and "..." will be prepended.
pub(crate) fn truncate_text_left(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("...{}", &text[text.len() - max_len..])
    } else {
        text.to_string()
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
    fn test_truncate_count_text() {
        let text = "This is a very long command that exceeds the maximum length.";
        let truncated = truncate_count_text(text, 20, 5);
        // Remove ANSI escape codes for testing
        let truncated = strip_ansi_codes(&truncated);
        assert_eq!(truncated, "This is a very long ... (5 times)");
    }

    #[test]
    fn test_truncate_text_left() {
        let text = "This is a very long command that exceeds the maximum length.";
        let truncated = truncate_text_left(text, 20);
        assert_eq!(truncated, "... the maximum length.");
    }

    #[test]
    fn test_format_rank_icon() {
        assert_eq!(format_rank_icon(1), "🥇");
        assert_eq!(format_rank_icon(2), "🥈");
        assert_eq!(format_rank_icon(3), "🥉");
        assert_eq!(format_rank_icon(4), "4");
    }

    #[test]
    fn test_metafied_plain_ascii() {
        // Plain ASCII bytes should pass through unchanged
        assert_eq!(decode_metafied(b"ls -la"), b"ls -la");
    }

    #[test]
    fn test_metafied_cjk() {
        // "文字" in UTF-8 is \xE6\x96\x87\xE5\xAD\x97
        // Metafied: \x96 → \x83\xB6, \x87 → \x83\xA7, \x97 → \x83\xB7
        let metafied = b"\xE6\x83\xB6\x83\xA7\xE5\xAD\x83\xB7";
        assert_eq!(decode_metafied(metafied), "文字".as_bytes());
    }

    #[test]
    fn test_metafied_empty() {
        assert_eq!(decode_metafied(b""), b"");
    }

    #[test]
    fn test_metafied_lone_meta_byte_at_end() {
        // A lone 0x83 with no following byte should be passed through as-is
        assert_eq!(decode_metafied(b"ls\x83"), b"ls\x83");
    }

    #[test]
    fn test_metafied_emoji() {
        // 🎉 in UTF-8 is \xF0\x9F\x8E\x89
        // \xF0 is above 0x9F so not metafied; \x9F, \x8E, \x89 are in 0x80-0x9F so they are:
        //   \x9F → \x83\xBF  (0x9F ^ 0x20 = 0xBF)
        //   \x8E → \x83\xAE  (0x8E ^ 0x20 = 0xAE)
        //   \x89 → \x83\xA9  (0x89 ^ 0x20 = 0xA9)
        let metafied = b"\xF0\x83\xBF\x83\xAE\x83\xA9";
        assert_eq!(decode_metafied(metafied), "🎉".as_bytes());
    }
}
