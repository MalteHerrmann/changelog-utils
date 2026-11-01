use assert_fs::NamedTempFile;
use clu::{cli::add, common::Changelog, config, single_file::changelog};
use std::{borrow::BorrowMut, path::Path};

#[test]
fn disabled_categories_skip_interactive_picker_and_generated_scope() {
    let mut config = load_example_config();
    config.use_categories = false;
    config.categories.clear();
    // The shared add/create-PR picker must return without accessing a terminal,
    // even when GitHub or AI supplied a historical category suggestion.
    assert_eq!(
        clu::cli::inputs::get_category(&config, "OldScope").unwrap(),
        ""
    );
    let entry = clu::single_file::entry::Entry::new(&config, "OldScope", "Fix a bug.", 7);
    assert_eq!(entry.category, "");
    assert_eq!(
        entry.fixed,
        "- [#7](https://github.com/evmos/evmos/pull/7) Fix a bug."
    );
}

#[test]
fn non_interactive_add_honors_category_setting() {
    use std::{fs, process::Command};

    for use_categories in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        for args in [
            vec!["init", "--quiet"],
            vec!["config", "user.name", "Test User"],
            vec!["config", "user.email", "test@example.com"],
        ] {
            assert!(Command::new("git")
                .current_dir(dir.path())
                .args(args)
                .status()
                .unwrap()
                .success());
        }
        let mut config = load_example_config();
        config.use_categories = use_categories;
        config.legacy_version = None;
        config.categories = if use_categories {
            vec!["app".into()]
        } else {
            vec![]
        };
        config.export(&dir.path().join(".clconfig.json")).unwrap();
        fs::write(
            dir.path().join("CHANGELOG.md"),
            "# Changelog\n\n## Unreleased\n",
        )
        .unwrap();

        let run_add = |extra: &[&str]| {
            Command::new(env!("CARGO_BIN_EXE_clu"))
                .current_dir(dir.path())
                .args([
                    "add",
                    "42",
                    "--change-type",
                    "Bug Fixes",
                    "--description",
                    "fix a thing",
                ])
                .args(extra)
                .output()
                .unwrap()
        };
        let rejected = run_add(if use_categories {
            &[]
        } else {
            &["--category", "app"]
        });
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains(if use_categories {
                "also --category"
            } else {
                "--category cannot be used"
            }),
            "{error}"
        );
        assert!(!fs::read_to_string(dir.path().join("CHANGELOG.md"))
            .unwrap()
            .contains("#42"));

        let output = run_add(if use_categories {
            &["--category", "app"]
        } else {
            &[]
        });
        assert!(output.status.success(), "{:?}", output);
        let expected = if use_categories {
            "- (app) [#42](https://github.com/evmos/evmos/pull/42) Fix a thing."
        } else {
            "- [#42](https://github.com/evmos/evmos/pull/42) Fix a thing."
        };
        let saved = fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert!(saved.lines().any(|line| line == expected), "{saved}");
        let committed = Command::new("git")
            .current_dir(dir.path())
            .args(["show", "HEAD:CHANGELOG.md"])
            .output()
            .unwrap();
        assert!(committed.status.success());
        assert_eq!(String::from_utf8(committed.stdout).unwrap(), saved);
        let lint = Command::new(env!("CARGO_BIN_EXE_clu"))
            .current_dir(dir.path())
            .arg("lint")
            .output()
            .unwrap();
        assert!(lint.status.success(), "{:?}", lint);
    }
}

#[cfg(test)]
fn load_example_config() -> config::Config {
    config::unpack_config(include_str!("testdata/single_file/evmos_config.json"))
        .expect("failed to load example configuration")
}

#[test]
fn test_pass_add_into_new_change_type() {
    let config = load_example_config();
    let mut changelog = changelog::parse_changelog(
        &config,
        Path::new("tests/testdata/single_file/changelog_new_category_after_add.md"),
    )
    .expect("failed to parse example changelog");
    assert_eq!(changelog.releases.len(), 2);

    add::add_entry(
        &config,
        changelog.borrow_mut(),
        "Bug Fixes",
        "test",
        "Test object.",
        15,
    );

    let first_release = changelog.releases.get(0).unwrap();
    assert_eq!(first_release.change_types.len(), 3);
    let new_change_type = first_release.change_types.get(2).unwrap();
    assert_eq!(new_change_type.name, "Bug Fixes");
    assert_eq!(new_change_type.entries.len(), 1);

    let added_entry = new_change_type.entries.get(0).unwrap();
    assert_eq!(added_entry.pr_number, 15);
    assert_eq!(
        added_entry.fixed,
        "- (test) [#15](https://github.com/evmos/evmos/pull/15) Test object."
    );
}

#[test]
fn test_pass_add_with_no_unreleased_section() {
    let config = load_example_config();
    let mut changelog = changelog::parse_changelog(
        &config,
        Path::new("tests/testdata/single_file/changelog_no_unreleased.md"),
    )
    .expect("failed to parse example changelog");
    assert_eq!(changelog.releases.len(), 2);

    add::add_entry(
        &config,
        changelog.borrow_mut(),
        "Bug Fixes",
        "test",
        "Test object.",
        15,
    );

    assert_eq!(changelog.releases.len(), 3);
    let first_release = changelog.releases.get(0).unwrap();
    assert_eq!(first_release.change_types.len(), 1);
    let new_change_type = first_release.change_types.get(0).unwrap();
    assert_eq!(new_change_type.name, "Bug Fixes");
    assert_eq!(new_change_type.entries.len(), 1);

    let added_entry = new_change_type.entries.get(0).unwrap();
    assert_eq!(added_entry.pr_number, 15);
    assert_eq!(
        added_entry.fixed,
        "- (test) [#15](https://github.com/evmos/evmos/pull/15) Test object."
    );
}

#[test]
fn test_pass_add_new_with_auto_fix() {
    let config = load_example_config();
    let mut changelog = changelog::parse_changelog(
        &config,
        Path::new("tests/testdata/single_file/changelog_new_category_after_add.md"),
    )
    .expect("failed to parse example changelog");
    assert_eq!(changelog.releases.len(), 2);

    add::add_entry(
        &config,
        &mut changelog,
        "Bug Fixes",
        "all",
        "adding an entry that's auto-fixable",
        15,
    );

    // export to temporary file
    let tmp_path = NamedTempFile::new("tmp_changelog.md").expect("failed to save tmp changelog");
    changelog
        .write(&config, tmp_path.path())
        .expect("failed to write tmp changelog");

    let updated_changelog = changelog::parse_changelog(&config, tmp_path.path()).unwrap();
    let added_entry = updated_changelog
        .releases
        .get(0)
        .unwrap()
        .change_types
        .get(2)
        .unwrap()
        .entries
        .get(0)
        .unwrap();

    // NOTE: we're expecting to have the first letter capitalized and the dot at the end added
    assert!(
        added_entry.problems.is_empty(),
        "expected line to have been corrected before writing to changelog."
    );
}
