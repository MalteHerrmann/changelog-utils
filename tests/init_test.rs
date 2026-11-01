use assert_fs::{prelude::*, TempDir};
use clu::{
    cli::init,
    config::{self, ChangeTypeConfig},
    single_file::changelog::{get_settings_from_existing_changelog, parse_changelog},
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
fn test_init_discovers_optional_categories() {
    for (categories, expected_use_categories) in [
        (vec!["", ""], false),
        (vec!["", "Historical-API"], false),
        (vec!["Historical-API", ""], false),
        (vec!["Historical-API", "cli"], true),
        (vec![], true),
    ] {
        let temp_dir = TempDir::new().unwrap();
        let mut contents = "# Changelog\n\n## Unreleased\n\n### Features\n".to_string();
        for (index, category) in categories.iter().enumerate() {
            let category = if category.is_empty() {
                String::new()
            } else {
                format!("({category}) ")
            };
            contents.push_str(&format!(
                "- {category}[#{}](https://github.com/example/repo/pull/{}) Add command.\n",
                index + 1,
                index + 1
            ));
        }
        temp_dir.child("CHANGELOG.md").write_str(&contents).unwrap();
        init::init_in_folder(temp_dir.path().to_path_buf()).unwrap();
        let saved = fs::read_to_string(temp_dir.child(".clconfig.json")).unwrap();
        let mut config = config::unpack_config(&saved).unwrap();
        assert_eq!(config.use_categories, expected_use_categories, "{contents}");
        assert!(!config.categories.iter().any(String::is_empty));
        assert_eq!(config.change_types[0].long, "Features");
        if !expected_use_categories {
            // Disabled discovery retains the defaults instead of deriving a list
            // from historical categories (or adding an empty category).
            assert_eq!(config.categories, config::Config::default().categories);
            config.target_repo = "https://github.com/example/repo".into();
            let parsed = parse_changelog(&config, temp_dir.child("CHANGELOG.md").path()).unwrap();
            assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
            let entries = &parsed.releases[0].change_types[0].entries;
            assert_eq!(entries.len(), categories.len());
            for (entry, category) in entries.iter().zip(&categories) {
                assert_eq!(entry.category, *category);
                assert!(contents.lines().any(|line| line == entry.fixed));
            }
        } else if !categories.is_empty() {
            assert_eq!(config.categories, vec!["cli", "historical-api"]);
        }
        temp_dir.child("CHANGELOG.md").assert(contents);
    }
}

#[test]
fn test_settings_discovery_preserves_disabled_category_list() {
    for category in ["", "Historical-API"] {
        let mut config = config::Config {
            use_categories: false,
            categories: vec!["keep-this".into(), "also-keep-this".into()],
            target_repo: "https://github.com/example/repo".into(),
            ..config::Config::default()
        };
        let contents = format!(
            "# Changelog\n\n## Unreleased\n\n### Features\n- {}[#1](https://github.com/example/repo/pull/1) Add command.\n",
            if category.is_empty() { String::new() } else { format!("({category}) ") }
        );
        get_settings_from_existing_changelog(&mut config, &contents);
        assert!(!config.use_categories);
        assert_eq!(config.categories, vec!["keep-this", "also-keep-this"]);
        assert_eq!(config.change_types[0].long, "Features");
    }
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
