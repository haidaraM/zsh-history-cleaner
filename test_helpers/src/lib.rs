use std::io::Write;
use tempfile::NamedTempFile;

pub fn get_tmp_file(commands: &str) -> NamedTempFile {
    let mut tmpfile =
        NamedTempFile::new().expect("Failed to create a temporary file for testing");

    let msg = format!(
        "Failed to write to the temporary file at '{}'",
        tmpfile.path().display()
    );

    let commands = format!("{}\n", commands);

    tmpfile.write_all(commands.as_bytes()).expect(&msg);

    tmpfile
}

/// Create a temporary file with mixed UTF-8 and non-UTF-8 content for testing encoding errors
pub fn get_tmp_file_with_invalid_utf8() -> NamedTempFile {
    let mut tmpfile = NamedTempFile::new().expect("Failed to create temporary file");

    // Write some valid UTF-8 content first
    let valid_line = ": 1732577005:0;echo 'valid command'\n";
    tmpfile
        .write_all(valid_line.as_bytes())
        .expect("Failed to write valid line");

    // Write a line with invalid UTF-8 bytes (control characters)
    let invalid_line = b": 1732577010:0;echo 'invalid \xFF\xFE command'\n";
    tmpfile
        .write_all(invalid_line)
        .expect("Failed to write invalid line");

    tmpfile
}
