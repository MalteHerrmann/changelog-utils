use assert_fs::{prelude::*, TempDir};
use clu::{config, errors::InitError, init};
use predicates::prelude::*;
use std::{collections::BTreeMap, fs};

#[test]
fn test_init_empty_folder() {
    let temp_dir = TempDir::new().expect("failed to create temporary directory");

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
    let temp_dir = TempDir::new().expect("failed to create temporary directory");

    temp_dir
        .copy_from("tests/testdata", &["changelog_fail.md"])
        .expect("failed to create dummy changelog");

    assert!(fs::rename(
        temp_dir.child("changelog_fail.md"),
        temp_dir.child("CHANGELOG.md"),
    )
    .is_ok());

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

    let config = config::unpack_config(
        fs::read_to_string(temp_dir.child(".clconfig.json"))
            .expect("failed to read config")
            .as_str(),
    )
    .expect("failed to unpack config");

    assert_eq!(
        config.categories,
        vec![
            "ante".to_string(),
            "distribution-precompile".to_string(),
            "evm".to_string(),
            "inflation".to_string(),
            "p256-precompile".to_string(),
            "stride-outpost".to_string(),
            "testnet".to_string(),
            "vesting".to_string(),
        ]
    );

    let expected_change_types: BTreeMap<String, String> = BTreeMap::from([
        ("Improvements".into(), "improvements".into()),
        ("Bug Fixes".into(), "bug\\s*fixes".into()),
        ("API Breaking".into(), "api\\s*breaking".into()),
        (
            "State Machine Breaking".into(),
            "state\\s*machine\\s*breaking".into(),
        ),
        ("Invalid Category".into(), "invalid\\s*category".into()),
    ]);

    assert_eq!(config.change_types, expected_change_types);
}

#[test]
fn test_init_changelog_and_config_exists() {
    let temp_dir = TempDir::new().expect("failed to create temporary directory");

    temp_dir
        .child("CHANGELOG.md")
        .touch()
        .expect("failed to create dummy changelog");

    temp_dir
        .child(".clconfig.json")
        .touch()
        .expect("failed to create dummy config");

    let res = init::init_in_folder(temp_dir.path().to_path_buf());
    assert!(
        res.is_err(),
        "expected failure trying to initialize with config already existing"
    );
    assert_eq!(
        res.unwrap_err().to_string(),
        InitError::ConfigAlreadyFound.to_string()
    )
}
