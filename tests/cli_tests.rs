use pretty_assertions::assert_eq;
use std::fs::File;
use std::io::Read;
use test_helpers::get_tmp_file;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html
const BINARY_PATH: &str = env!("CARGO_BIN_EXE_zhc");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn run_flags(flags: Vec<&str>, expected: &str) {
    for flag in flags.iter() {
        let output = std::process::Command::new(BINARY_PATH)
            .arg(flag)
            .output()
            .unwrap_or_else(|_| panic!("the {} flag should succeed", flag));

        assert!(
            output.status.success(),
            "the command should not exit successfully"
        );
        let stdout =
            String::from_utf8(output.stdout).expect("the --help output string should be utf8");

        assert!(stdout.contains(expected), "Expected to find '{}' in the output of the command '{}', but got: '{}'", expected, flag, stdout);
    }
}

#[test]
fn test_help() {
    run_flags(
        ["-h", "--help"].to_vec(),
        "Clean your commands history by removing duplicate commands, commands between dates, etc...",
    );
}

#[test]
fn test_version() {
    run_flags(
        ["-V", "--version"].to_vec(),
        format!("{} {}", PACKAGE_NAME, PACKAGE_VERSION).as_str(),
    );
}

#[test]
fn test_no_change_in_history() {
    let cmds = [
        ": 1732577005:0;tf fmt -recursive",
        ": 1732577037:0;tf apply",
        ": 1732577040:0;tf out",
        ": 1732577045:0;tf apply",
    ];

    let tmp_file = get_tmp_file(cmds.join("\n").as_str());

    let mut before_content: String = String::new();
    File::open(&tmp_file)
        .expect("should open the temp file")
        .read_to_string(&mut before_content)
        .expect("should read the temp file");

    let output = std::process::Command::new(BINARY_PATH)
        .arg("--keep-duplicates")
        .arg("-H")
        .arg(tmp_file.path().to_string_lossy().to_string())
        .output()
        .expect("executing the command should not fail");

    assert!(
        output.status.success(),
        "the command should not exit successfully"
    );

    let mut after_content: String = String::new();
    File::open(&tmp_file)
        .expect("should open the temp file")
        .read_to_string(&mut after_content)
        .expect("should read the temp file");

    assert_eq!(before_content, after_content);
}
