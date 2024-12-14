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
