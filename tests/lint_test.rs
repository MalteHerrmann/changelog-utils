use clu::{
    common::{Changelog, Problem},
    config,
    single_file::changelog,
};
use std::{fs, path::Path};

#[cfg(test)]
fn format_problems(problems: &[Problem]) -> Vec<String> {
    problems.iter().map(|p| p.to_string()).collect()
}

#[test]
fn lint_reports_explicit_types_and_summary_despite_keywords_in_entry_text() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(".clconfig.json"),
        include_str!("testdata/single_file/evmos_config.json"),
    )
    .unwrap();
    fs::write(
        dir.path().join("CHANGELOG.md"),
        concat!(
            "# Changelog\n\n## Unreleased\n\n### BugFixes\n\n",
            "- (app) [#1](https://github.com/evmos/evmos/pull/1) Move directory handling\n",
            "- (app) [#2](https://github.com/evmos/evmos/pull/3) Add category filters\n",
            "- malformed filename entry\n",
        ),
    )
    .unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_clu"))
        .current_dir(dir.path())
        .arg("lint")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "CHANGELOG.md:5 [change type]: 'Bug Fixes' should be used instead of 'BugFixes'",
        "CHANGELOG.md:7 [description]: PR description should end with a dot: 'Move directory handling'",
        "CHANGELOG.md:8 [description]: PR description should end with a dot: 'Add category filters'",
        "CHANGELOG.md:8 [pr]: PR link is not matching PR number 2:",
        "CHANGELOG.md:9 [description]: malformed entry: '- malformed filename entry'",
    ] {
        assert!(stdout.contains(expected), "missing {expected:?} in {stdout}");
    }
    assert!(
        stdout.ends_with("\nproblems by type:\n  description: 3\n  change type: 1\n  pr: 1\n"),
        "unexpected summary: {stdout}"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Changelog contains 5 problems"), "{stderr}");
}

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::unpack_config(include_str!("testdata/single_file/evmos_config.json"))
        .expect("failed to load example config")
}

#[test]
fn it_should_pass_for_correct_changelogs() {
    let correct_changelog = Path::new("tests/testdata/single_file/changelog_ok.md");
    let changelog = changelog::parse_changelog(&load_test_config(), correct_changelog)
        .expect("failed to parse correct changelog");
    assert_eq!(changelog.releases.len(), 3);
    assert!(changelog.problems.is_empty());

    let first_release = changelog.releases.get(0).unwrap();
    assert_eq!(first_release.change_types.len(), 4);
    assert_eq!(first_release.change_types.first().unwrap().entries.len(), 4);
}

#[test]
fn it_should_pass_for_incorrect_changelogs_that_has_no_critical_flaws() {
    let incorrect_changelog = Path::new("tests/testdata/single_file/changelog_fail.md");
    let changelog = changelog::parse_changelog(&load_test_config(), incorrect_changelog)
        .expect("failed to parse incorrect changelog");
    assert_eq!(changelog.releases.len(), 3);
    assert_eq!(
        format_problems(&changelog.problems),
        vec![
            "tests/testdata/single_file/changelog_fail.md:11 [pr]: PR link is not matching PR number 1948: 'https://github.com/evmos/evmos/pull/1949'",
            "tests/testdata/single_file/changelog_fail.md:21 [description]: 'ABI' should be used instead of 'ABi'",
            "tests/testdata/single_file/changelog_fail.md:25 [description]: PR description should end with a dot: 'Fixed the problem `gas_used` is 0'",
            "tests/testdata/single_file/changelog_fail.md:27 [change type]: 'Invalid Category' is not a valid change type",
            "tests/testdata/single_file/changelog_fail.md:31 [duplicate]: duplicate change type in release Unreleased: Bug Fixes",
            "tests/testdata/single_file/changelog_fail.md:43 [duplicate]: duplicate release: v15.0.0",
            "tests/testdata/single_file/changelog_fail.md:47 [duplicate]: duplicate PR: #1862",
            "tests/testdata/single_file/changelog_fail.md:50 [description]: malformed entry: '- another malformed entry in changelog'",
        ]
    );
}

#[test]
fn it_should_fix_the_changelog_as_expected() {
    let incorrect_changelog = Path::new("tests/testdata/single_file/changelog_to_be_fixed.md");

    let config = load_test_config();

    let changelog = changelog::parse_changelog(&config, incorrect_changelog)
        .expect("failed to parse changelog");

    let expected = fs::read_to_string(Path::new("tests/testdata/single_file/changelog_fixed.md"))
        .expect("failed to load correct changelog");

    assert_eq!(
        expected.trim(),
        changelog
            .get_fixed_contents(&config)
            .expect("failed to get fixed contents")
            .trim(),
        "expected different fixed changelog"
    );
}

#[test]
fn category_free_lint_fix_preserves_history_and_other_rules() {
    use std::process::Command;

    let dir = tempfile::tempdir().unwrap();
    let mut config = load_test_config();
    config.use_categories = false;
    config.categories.clear();
    config.legacy_version = None;
    config.export(&dir.path().join(".clconfig.json")).unwrap();
    let path = dir.path().join("CHANGELOG.md");
    fs::write(&path, concat!(
        "# Changelog\n\n## Unreleased\n\n### Bug Fixes\n\n",
        "- [#42](https://github.com/evmos/evmos/pull/42) Fixed an issue.\n",
        "- (Old_unknown Scope!) [#43](https://github.com/evmos/evmos/pull/43) Keep (parenthesized) description.\n",
        "  -  [#44] (https://github.com/wrong/repo/pull/45) fix ABi\n",
    )).unwrap();
    let run = |command| {
        Command::new(env!("CARGO_BIN_EXE_clu"))
            .current_dir(dir.path())
            .arg(command)
            .output()
            .unwrap()
    };
    let lint = run("lint");
    assert!(!lint.status.success());
    let problems = String::from_utf8(lint.stdout).unwrap();
    for kind in ["[whitespace]", "[pr]", "[description]"] {
        assert!(problems.contains(kind), "{problems}");
    }
    assert!(!problems.contains("[category]"), "{problems}");
    assert!(!problems.contains("malformed entry"), "{problems}");

    assert!(run("fix").status.success());
    let fixed = fs::read_to_string(&path).unwrap();
    for entry in [
        "- [#42](https://github.com/evmos/evmos/pull/42) Fixed an issue.",
        "- (Old_unknown Scope!) [#43](https://github.com/evmos/evmos/pull/43) Keep (parenthesized) description.",
        "- [#44](https://github.com/evmos/evmos/pull/44) Fix ABI.",
    ] {
        assert!(fixed.lines().any(|line| line == entry), "{fixed}");
    }
    let lint = run("lint");
    assert!(lint.status.success(), "{:?}", lint);
    assert!(run("fix").status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), fixed);

    // Toggling the setting must preserve the list, not silently clear it.
    config.categories = vec!["app".into(), "evm".into()];
    config.export(&dir.path().join(".clconfig.json")).unwrap();
    for value in ["true", "false"] {
        let output = Command::new(env!("CARGO_BIN_EXE_clu"))
            .current_dir(dir.path())
            .args(["config", "use-categories", "set", value])
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output);
        let saved =
            config::unpack_config(&fs::read_to_string(dir.path().join(".clconfig.json")).unwrap())
                .unwrap();
        assert_eq!(saved.categories, ["app", "evm"]);
        assert_eq!(saved.use_categories, value == "true");
        if value == "true" {
            let lint = run("lint");
            assert!(!lint.status.success());
            assert!(String::from_utf8_lossy(&lint.stdout).contains("malformed entry"));
        }
    }
}
