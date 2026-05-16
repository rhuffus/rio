use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn shows_help() {
    Command::cargo_bin("rio")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("dotfile manager"));
}

#[test]
fn shows_version() {
    Command::cargo_bin("rio")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("0.1.0"));
}

#[test]
fn init_creates_block_and_sidecar() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args([
            "init",
            target.to_str().unwrap(),
            "--content",
            "export FOO=bar",
        ])
        .assert()
        .success()
        .stdout(contains("initialized"));

    let body = fs::read_to_string(&target).unwrap();
    assert!(body.contains("# >>> RhuffusIO Managed Block >>>"));
    assert!(body.contains("export FOO=bar"));

    let sidecar = PathBuf::from(format!("{}.rio", target.display()));
    assert!(sidecar.exists(), "sidecar should exist at {}", sidecar.display());
}

#[test]
fn init_refuses_when_sidecar_already_exists() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "first"])
        .assert()
        .success();

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "second"])
        .assert()
        .failure()
        .stderr(contains("sidecar already exists"));
}

#[test]
fn apply_is_idempotent_and_status_reports_clean() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args(["apply", target.to_str().unwrap(), "--content", "alias gs='git status'"])
        .assert()
        .success();

    // Second apply with same content: same hash, still clean.
    Command::cargo_bin("rio")
        .unwrap()
        .args(["apply", target.to_str().unwrap(), "--content", "alias gs='git status'"])
        .assert()
        .success();

    Command::cargo_bin("rio")
        .unwrap()
        .args(["status", target.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("clean"));
}

#[test]
fn status_reports_drift_after_manual_edit() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "FOO=bar"])
        .assert()
        .success();

    // Manually edit the file, replacing the managed block content.
    let body = fs::read_to_string(&target).unwrap();
    let tampered = body.replace("FOO=bar", "FOO=tampered");
    fs::write(&target, tampered).unwrap();

    Command::cargo_bin("rio")
        .unwrap()
        .args(["status", target.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("DRIFT"));
}

#[test]
fn diff_reports_no_changes_when_content_matches() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "X=1"])
        .assert()
        .success();

    Command::cargo_bin("rio")
        .unwrap()
        .args(["diff", target.to_str().unwrap(), "--content", "X=1"])
        .assert()
        .success()
        .stdout(contains("no changes"));
}
