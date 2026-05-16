use assert_cmd::Command;
use predicates::str::contains;

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
fn init_subcommand_is_wired() {
    Command::cargo_bin("rio")
        .unwrap()
        .args(["init", "/tmp/does-not-matter"])
        .assert()
        .failure()
        .stderr(contains("not yet implemented"));
}
