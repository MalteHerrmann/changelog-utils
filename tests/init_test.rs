use assert_fs::{prelude::*, TempDir};
use clu::{
    cli::init,
    config::{self, ChangeTypeConfig},
};
use predicates::prelude::*;
use std::fs;
use std::process::Command;

#[test]
fn test_init_normalizes_origin() {
    for (origin, expected) in [
        (
            "https://github.com/example/my.repo.git",
            "https://github.com/example/my.repo",
        ),
        (
            "git@github.com:Example/my.repo.git",
            "https://github.com/Example/my.repo",
        ),
        (
            "ssh://git@github.com/AnotherOwner/other-repo.git",
            "https://github.com/AnotherOwner/other-repo",
        ),
    ] {
        let temp_dir = TempDir::new().unwrap();
        for args in [
            vec!["init", "--quiet"],
            vec!["remote", "add", "origin", origin],
        ] {
            let output = Command::new("git")
                .current_dir(temp_dir.path())
                .args(args)
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
        }

        // Use a child process to avoid changing other tests' working directory.
        let output = Command::new(env!("CARGO_BIN_EXE_clu"))
            .current_dir(temp_dir.path())
            .arg("init")
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output);

        // Inspect the saved JSON directly: loading Config also normalizes URLs.
        let saved: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(temp_dir.child(".clconfig.json")).unwrap())
                .unwrap();
        assert_eq!(saved["target_repo"], expected, "origin: {origin}");
    }
}

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
        .copy_from("tests/testdata/single_file", &["changelog_fail.md"])
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

    let expected_change_types = vec![
        ChangeTypeConfig {
            long: "State Machine Breaking".into(),
            short: "stat".into(),
        },
        ChangeTypeConfig {
            long: "API Breaking".into(),
            short: "api".into(),
        },
        ChangeTypeConfig {
            long: "Improvements".into(),
            short: "impr".into(),
        },
        ChangeTypeConfig {
            long: "Bug Fixes".into(),
            short: "bug".into(),
        },
        ChangeTypeConfig {
            long: "Invalid Category".into(),
            short: "inva".into(),
        },
    ];

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
    let err = res.unwrap_err();
    assert!(err
        .to_string()
        .contains("Configuration file already exists"));
}
