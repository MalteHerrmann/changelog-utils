use assert_fs::{prelude::*, TempDir};
use clu::init;
use predicates::prelude::*;
use clu::errors::InitError;

#[test]
fn test_init_empty_folder() {
    let temp_dir = TempDir::new()
        .expect("failed to create temporary directory");

    assert!(
        init::init_in_folder(temp_dir.path().to_path_buf()).is_ok(),
        "failed to initialize in empty folder."
    );

    temp_dir
        .child("CHANGELOG.md")
        .assert(predicate::path::exists());

    temp_dir
        .child(".clconfig.json")
        .assert(predicate::path::exists());
}

#[test]
fn test_init_changelog_exists() {
    let temp_dir = TempDir::new()
        .expect("failed to create temporary directory");

    temp_dir
        .child("CHANGELOG.md")
        .touch()
        .expect("failed to create dummy changelog");

    assert!(
        init::init_in_folder(temp_dir.path().to_path_buf()).is_ok(),
        "failed to initialize with existing changelog"
    );

    temp_dir
        .child("CHANGELOG.md")
        .assert(predicate::path::exists());

    temp_dir
        .child(".clconfig.json")
        .assert(predicate::path::exists());

}
#[test]
fn test_init_changelog_and_config_exists() {
    let temp_dir = TempDir::new()
        .expect("failed to create temporary directory");

    temp_dir
        .child("CHANGELOG.md")
        .touch()
        .expect("failed to create dummy changelog");

    temp_dir
        .child(".clconfig.json")
        .touch()
        .expect("failed to create dummy config");

    let res = init::init_in_folder(temp_dir.path().to_path_buf());
    assert!(res.is_err(), "expected failure trying to initialize with config already existing");
    assert_eq!(res.unwrap_err().to_string(), InitError::ConfigAlreadyFound.to_string())
}
