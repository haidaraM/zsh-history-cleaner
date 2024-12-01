const BINARY_PATH: &str = env!("CARGO_BIN_EXE_zhc");

#[test]
fn test_help() {
    let flags = ["-h", "--help"];

    for flag in flags.iter() {
        let output = std::process::Command::new(BINARY_PATH)
            .arg(flag)
            .output()
            .unwrap_or_else(|_| panic!("the {} flag should succeed", flag));

        assert!(output.status.success());
        let stdout =
            std::str::from_utf8(&output.stdout).expect("the --help output string should be utf8");

        assert!(stdout.contains(
            "Clean your history by removing duplicate commands, commands matching regex etc..."
        ));
    }
}

// TODO: add more tests
