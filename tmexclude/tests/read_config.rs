use std::fs;
use tempfile::TempDir;

#[test]
fn must_create_default_when_not_initialized() {
    let mock_home = TempDir::new().unwrap();
    let config_path = mock_home.path().join(".config/tmexclude.yaml");

    assert_cmd::Command::cargo_bin("tmexclude")
        .unwrap()
        .env("HOME", mock_home.path())
        .arg("read-config")
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(config_path).unwrap(),
        include_str!("../../config.example.yaml")
    );
}

#[test]
fn must_not_overwrite_default_if_exists() {
    let mock_home = TempDir::new().unwrap();
    let config_path = mock_home.path().join(".config/tmexclude.yaml");

    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    let modified_config = include_str!("../../config.example.yaml").replace("Library", "Library2");
    fs::write(&config_path, &modified_config).unwrap();

    assert_cmd::Command::cargo_bin("tmexclude")
        .unwrap()
        .env("HOME", mock_home.path())
        .arg("read-config")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(config_path).unwrap(), modified_config);
}

#[test]
fn must_not_create_if_given_exists() {
    let mock_home = TempDir::new().unwrap();
    let default_config_path = mock_home.path().join(".config/tmexclude.yaml");
    let config_path = mock_home.path().join("config.yaml");

    fs::write(&config_path, include_str!("../../config.example.yaml")).unwrap();

    assert_cmd::Command::cargo_bin("tmexclude")
        .unwrap()
        .env("HOME", mock_home.path())
        .arg("-c")
        .arg(config_path)
        .arg("read-config")
        .assert()
        .success();

    assert!(!default_config_path.exists());
}

#[test]
fn must_not_create_if_given_not_exists() {
    let mock_home = TempDir::new().unwrap();
    let default_config_path = mock_home.path().join(".config/tmexclude.yaml");
    let config_path = mock_home.path().join("config.yaml");

    assert_cmd::Command::cargo_bin("tmexclude")
        .unwrap()
        .env("HOME", mock_home.path())
        .arg("-c")
        .arg(config_path)
        .arg("read-config")
        .assert()
        .failure();

    assert!(!default_config_path.exists());
}
