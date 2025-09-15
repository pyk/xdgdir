use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_defaults() {
    let mut cmd = Command::cargo_bin("printdirs").unwrap();
    cmd.env_clear().env("HOME", "/home/fake_user").arg("my-app");
    cmd.assert()
        .success()
        .stdout(contains("home=/home/fake_user"))
        .stdout(contains("config=/home/fake_user/.config/my-app"))
        .stdout(contains("data=/home/fake_user/.local/share/my-app"));
}

#[test]
fn test_overrides() {
    let mut cmd = Command::cargo_bin("printdirs").unwrap();

    cmd.env_clear()
        .env("HOME", "/home/fake_user")
        .env("XDG_CONFIG_HOME", "/custom/conf")
        .env("XDG_DATA_HOME", "/custom/data")
        .arg("my-app");

    cmd.assert()
        .success()
        .stdout(contains("config=/custom/conf/my-app"))
        .stdout(contains("data=/custom/data/my-app"))
        .stdout(contains("cache=/home/fake_user/.cache/my-app"));
}

#[test]
fn test_failures() {
    let mut cmd = Command::cargo_bin("printdirs").unwrap();
    cmd.env_clear().arg("my-app");
    cmd.assert()
        .failure()
        .stderr(contains("$HOME is not set or empty"));
}
