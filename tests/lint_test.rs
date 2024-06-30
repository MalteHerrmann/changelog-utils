use clu::{changelog, config};
use std::{fs, path::Path};

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::unpack_config(include_str!("testdata/evmos_config.json"))
        .expect("failed to load example config")
}

#[test]
fn it_should_pass_for_correct_changelogs() {
    let correct_changelog = Path::new("tests/testdata/changelog_ok.md");
    let changelog = changelog::parse_changelog(load_test_config(), correct_changelog)
        .expect("failed to parse correct changelog");
    assert_eq!(changelog.releases.len(), 3);
    assert!(changelog.problems.is_empty());

    let first_release = changelog.releases.get(0).unwrap();
    assert_eq!(first_release.change_types.len(), 4);
    assert_eq!(first_release.change_types.first().unwrap().entries.len(), 4);
}

#[test]
fn it_should_pass_for_incorrect_changelogs_that_has_no_critical_flaws() {
    let incorrect_changelog = Path::new("tests/testdata/changelog_fail.md");
    let changelog = changelog::parse_changelog(load_test_config(), incorrect_changelog)
        .expect("failed to parse incorrect changelog");
    assert_eq!(changelog.releases.len(), 3);
    assert_eq!(
        changelog.problems,
        vec![
            "tests/testdata/changelog_fail.md:10: PR link is not matching PR number 1948: 'https://github.com/evmos/evmos/pull/1949'",
            "tests/testdata/changelog_fail.md:19: There should be no backslash in front of the # in the PR link",
            "tests/testdata/changelog_fail.md:20: 'ABI' should be used instead of 'ABi'",
            "tests/testdata/changelog_fail.md:24: PR description should end with a dot: 'Fixed the problem `gas_used` is 0'",
            "tests/testdata/changelog_fail.md:26: 'Invalid Category' is not a valid change type",
            "tests/testdata/changelog_fail.md:30: duplicate change type in release Unreleased: Bug Fixes",
            "tests/testdata/changelog_fail.md:39: duplicate PR: #1801",
            "tests/testdata/changelog_fail.md:41: duplicate release: v15.0.0",
            "tests/testdata/changelog_fail.md:45: duplicate PR: #1862",
            "tests/testdata/changelog_fail.md:46: invalid entry: - malformed entry in changelog",
        ]
    );
}

#[test]
fn it_should_fix_the_changelog_as_expected() {
    let incorrect_changelog = Path::new("tests/testdata/changelog_to_be_fixed.md");
    let changelog = changelog::parse_changelog(load_test_config(), incorrect_changelog)
        .expect("failed to parse changelog");

    let expected = fs::read_to_string(Path::new("tests/testdata/changelog_fixed.md"))
        .expect("failed to load correct changelog");

    assert_eq!(
        expected.trim(),
        changelog.get_fixed().trim(),
        "expected different fixed changelog"
    );
}
