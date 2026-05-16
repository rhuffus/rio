use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
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
    assert!(
        sidecar.exists(),
        "sidecar should exist at {}",
        sidecar.display()
    );
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
        .args([
            "apply",
            target.to_str().unwrap(),
            "--content",
            "alias gs='git status'",
        ])
        .assert()
        .success();

    // Second apply with same content: same hash, still clean.
    Command::cargo_bin("rio")
        .unwrap()
        .args([
            "apply",
            target.to_str().unwrap(),
            "--content",
            "alias gs='git status'",
        ])
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

#[test]
fn init_with_from_flag_reads_content_file() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.sh");
    let target = dir.path().join("target.sh");
    fs::write(&source, "alias k=kubectl\nalias d=docker\n").unwrap();

    Command::cargo_bin("rio")
        .unwrap()
        .args([
            "init",
            target.to_str().unwrap(),
            "--from",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let body = fs::read_to_string(&target).unwrap();
    assert!(body.contains("alias k=kubectl"));
    assert!(body.contains("alias d=docker"));
}

#[test]
fn apply_after_drift_restores_clean_status() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "FOO=bar"])
        .assert()
        .success();

    // Tamper.
    let body = fs::read_to_string(&target).unwrap();
    fs::write(&target, body.replace("FOO=bar", "FOO=tampered")).unwrap();

    // Apply with the original content restores the block and rebases the hash.
    Command::cargo_bin("rio")
        .unwrap()
        .args(["apply", target.to_str().unwrap(), "--content", "FOO=bar"])
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
fn status_reports_unmanaged_file() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("plain.sh");
    fs::write(&target, "echo hello\n").unwrap();

    Command::cargo_bin("rio")
        .unwrap()
        .args(["status", target.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("not managed"));
}

#[test]
fn diff_shows_both_sides_when_content_differs() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("test.sh");

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "OLD=1"])
        .assert()
        .success();

    Command::cargo_bin("rio")
        .unwrap()
        .args(["diff", target.to_str().unwrap(), "--content", "NEW=2"])
        .assert()
        .success()
        .stdout(contains("OLD=1").and(contains("NEW=2")));
}

#[test]
fn init_creates_file_when_missing() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("brand-new.sh");
    assert!(!target.exists());

    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", target.to_str().unwrap(), "--content", "echo hi"])
        .assert()
        .success();

    assert!(target.exists());
    let body = fs::read_to_string(&target).unwrap();
    assert!(body.contains("# >>> RhuffusIO Managed Block >>>"));
    assert!(body.contains("echo hi"));
}
