use chrono::{Local, TimeZone};
use glob::glob;
use pretty_assertions::assert_eq;
use std::fs::{read_to_string, remove_file};
use std::path::PathBuf;
use test_helpers::get_tmp_file;
use zsh_history_cleaner::history::BACKUP_FILE_SUFFIX;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html
const BINARY_PATH: &str = env!("CARGO_BIN_EXE_zhc");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper function to run the binary with some flags and check for expected output
fn run_flags(flags: Vec<&str>, expected: &str) {
    for flag in flags.iter() {
        let output = std::process::Command::new(BINARY_PATH)
            .arg(flag)
            .output()
            .unwrap_or_else(|_| panic!("the {} flag should succeed", flag));

        assert!(
            output.status.success(),
            "the command should exit successfully but got {:?}",
            output
        );
        let stdout =
            String::from_utf8(output.stdout).expect("the --help output string should be utf8");

        assert!(
            stdout.contains(expected),
            "Expected to find '{}' in the output of the command '{}', but got: '{}'",
            expected,
            flag,
            stdout
        );
    }
}

#[test]
fn test_cli_help() {
    run_flags(
        ["-h", "--help"].to_vec(),
        "Clean your commands history by removing duplicate commands, commands between dates, etc...",
    );
}

#[test]
fn test_cli_version() {
    run_flags(
        ["-V", "--version"].to_vec(),
        format!("{} {}", PACKAGE_NAME, PACKAGE_VERSION).as_str(),
    );
}

/// Test that the backup file is created when it's not disabled
#[test]
fn test_backup_file_is_created() {
    let cmds = [
        ": 1732577005:0;echo 'hello world'",
        ": 1732577037:0;tf apply",
        ": 1732577045:0;tf apply",
    ];

    let tmp_file = get_tmp_file(cmds.join("\n").as_str());

    let before_content: String = read_to_string(tmp_file.path()).expect("should read file");

    let output = std::process::Command::new(BINARY_PATH)
        .arg("-H")
        .arg(tmp_file.path())
        .output()
        .expect("executing the command should not fail");

    assert!(
        output.status.success(),
        "the command should exit successfully but got {:?}",
        output
    );

    let entries: Vec<PathBuf> =
        glob(format!("{}{}*", tmp_file.path().display(), BACKUP_FILE_SUFFIX).as_str())
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();

    // There should be exactly one backup file created
    assert_eq!(
        entries.len(),
        1,
        "There should be exactly one backup file created. Found {:?} backup files.",
        entries
    );
    let backup_file_path = entries.first().unwrap();

    let stdout =
        String::from_utf8(output.stdout).expect("the command output string should be utf8");

    assert!(
        stdout.contains(backup_file_path.to_str().unwrap()),
        "the output should mention the backup file path"
    );

    let backup_content: String = read_to_string(backup_file_path).expect("should read file");

    assert_eq!(
        before_content, backup_content,
        "the backup file content should match the original file content"
    );

    // Clean up the backup file
    remove_file(backup_file_path).expect("should remove the backup file");
}

/// Test that the backup file is NOT created when it is disabled (--no-backup flag)
#[test]
fn test_backup_file_is_not_created() {
    let cmds = [
        ": 1732577005:0;echo 'hello world'",
        ": 1732577037:0;tf apply",
        ": 1732577045:0;tf apply",
    ];

    let tmp_file = get_tmp_file(cmds.join("\n").as_str());

    let output = std::process::Command::new(BINARY_PATH)
        .arg("--no-backup")
        .arg("-H")
        .arg(tmp_file.path())
        .output()
        .expect("executing the command should not fail");

    assert!(
        output.status.success(),
        "the command should exit successfully but got {:?}",
        output
    );

    let entries: Vec<PathBuf> = glob(
        format!(
            "{}{}*",
            tmp_file.path().to_str().unwrap(),
            BACKUP_FILE_SUFFIX
        )
        .as_str(),
    )
    .expect("Failed to read glob pattern")
    .filter_map(Result::ok)
    .collect();

    // There should be no backup file created
    assert_eq!(entries.len(), 0, "There should be no backup file created");
}

#[test]
fn test_cli_no_change_in_history() {
    let cmds = [
        ": 1732577005:0;tf fmt -recursive",
        ": 1732577037:0;tf apply",
        ": 1732577040:0;tf out",
        ": 1732577045:0;tf apply",
    ];

    let tmp_file = get_tmp_file(cmds.join("\n").as_str());

    let before_content: String = read_to_string(tmp_file.path()).expect("should read file");

    let output = std::process::Command::new(BINARY_PATH)
        .arg("--keep-duplicates")
        .arg("-H")
        .arg(tmp_file.path())
        .output()
        .expect("executing the command should not fail");

    assert!(
        output.status.success(),
        "the command should exit successfully but got {:?}",
        output
    );

    let after_content: String = read_to_string(tmp_file.path()).expect("should read file");

    assert_eq!(
        before_content, after_content,
        "the history file should not change"
    );
}

#[test]
fn test_cli_remove_between_dates() {
    let date_2020_01_01_10h = Local.with_ymd_and_hms(2020, 1, 1, 10, 0, 0).unwrap();

    let date_2020_01_01_11h = Local.with_ymd_and_hms(2020, 2, 1, 11, 0, 0).unwrap();

    let date_2023_02_26_9h = Local.with_ymd_and_hms(2023, 2, 26, 9, 0, 0).unwrap();

    let date_2024_03_26 = Local.with_ymd_and_hms(2024, 3, 26, 12, 0, 0).unwrap();

    let date_2026_06_26 = Local.with_ymd_and_hms(2026, 6, 26, 12, 0, 0).unwrap();

    let cmds = [
        format!(": {}:0;echo 'delete me'", date_2020_01_01_10h.timestamp()),
        format!(
            ": {}:0;echo 'delete me too'",
            date_2020_01_01_11h.timestamp()
        ),
        format!(
            ": {}:0;echo 'delete me as well'",
            date_2023_02_26_9h.timestamp()
        ),
        format!(": {}:0;echo 'keep me'", date_2024_03_26.timestamp()),
        format!(": {}:0;echo 'keep me too'", date_2026_06_26.timestamp()),
    ];

    let tmp_file = get_tmp_file(cmds.join("\n").as_str());

    let output = std::process::Command::new(BINARY_PATH)
        .arg("--remove-between")
        .arg(format!("{}", date_2020_01_01_10h.format("%Y-%m-%d")))
        .arg(format!("{}", date_2023_02_26_9h.format("%Y-%m-%d")))
        .arg("-H")
        .arg(tmp_file.path())
        .output()
        .expect("executing the command should not fail");

    assert!(
        output.status.success(),
        "the command should exit successfully but got: {:?}",
        output
    );

    let after_content: String = read_to_string(tmp_file.path()).expect("should read file");

    let expected_cmds = [
        format!(": {}:0;echo 'keep me'", date_2024_03_26.timestamp()),
        format!(": {}:0;echo 'keep me too'", date_2026_06_26.timestamp()),
    ]
    .join("\n")
        + "\n";

    assert_eq!(after_content, expected_cmds);

    // Now let's delete everything
    let output = std::process::Command::new(BINARY_PATH)
        .arg("--remove-between")
        .arg(format!("{}", date_2024_03_26.format("%Y-%m-%d")))
        .arg(format!("{}", date_2026_06_26.format("%Y-%m-%d")))
        .arg("-H")
        .arg(tmp_file.path())
        .output()
        .expect("executing the command should not fail");

    assert!(
        output.status.success(),
        "the command should exit successfully but got: {:?}",
        output
    );

    let after_content: String = read_to_string(tmp_file.path()).expect("should read the temp file");

    assert_eq!(after_content, "");
}
