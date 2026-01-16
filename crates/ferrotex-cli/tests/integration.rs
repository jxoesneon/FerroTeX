use assert_cmd::Command;
use std::fs::File;
use std::io::Write;

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ferrotex-cli"));
    cmd.arg("--help").assert().success();
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ferrotex-cli"));
    cmd.arg("--version").assert().success();
}

#[test]
fn test_cli_parse() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ferrotex-cli"));

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("test.log");
    let mut file = File::create(&log_path).unwrap();
    file.write_all(b"LaTeX Warning: Something wrong on input line 5.")
        .unwrap();

    cmd.arg("parse")
        .arg(log_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("Warning"));
}

#[test]
fn test_cli_verify_missing_lock() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ferrotex-cli"));
    let temp = tempfile::tempdir().unwrap();
    let lock_path = temp.path().join("nonexistent.lock");

    cmd.arg("verify").arg(lock_path).assert().failure();
}
