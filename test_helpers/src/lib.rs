use std::io::Write;
use tempfile::NamedTempFile;

pub fn get_tmp_file(commands: &str) -> NamedTempFile {
    let mut tmpfile =
        NamedTempFile::new().expect("The tests should be able to create a temporary file!");

    let path = tmpfile.path().to_string_lossy().to_string();
    let msg = format!("We should able to write to the temporary file '{path}'").clone();
    
    let mut commands = commands.to_string();
    commands.push('\n');

    tmpfile.write_all(commands.as_bytes()).expect(&msg);

    tmpfile
}
