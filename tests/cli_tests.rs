const BINARY_PATH: &str = env!("CARGO_BIN_EXE_zhc");

#[test]
fn test_help() {
    let output = std::process::Command::new(BINARY_PATH)
        .arg("--help")
        .output()
        .expect("the --help flag should succeed");

    assert!(output.status.success());

}

// TODO: add more tests

