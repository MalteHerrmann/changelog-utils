use clu::{config, multi_file, single_file::changelog};
use std::{fs, path::Path};

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::unpack_config(include_str!("testdata/single_file/evmos_config.json"))
        .expect("failed to load example config")
}

#[test]
fn it_should_pass_for_correct_changelogs() {
    let correct_changelog = Path::new("tests/testdata/single_file/changelog_ok.md");
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
    let incorrect_changelog = Path::new("tests/testdata/single_file/changelog_fail.md");
    let changelog = changelog::parse_changelog(load_test_config(), incorrect_changelog)
        .expect("failed to parse incorrect changelog");
    assert_eq!(changelog.releases.len(), 3);
    assert_eq!(
        changelog.problems,
        vec![
            "tests/testdata/single_file/changelog_fail.md:11: PR link is not matching PR number 1948: 'https://github.com/evmos/evmos/pull/1949'",
            "tests/testdata/single_file/changelog_fail.md:21: 'ABI' should be used instead of 'ABi'",
            "tests/testdata/single_file/changelog_fail.md:25: PR description should end with a dot: 'Fixed the problem `gas_used` is 0'",
            "tests/testdata/single_file/changelog_fail.md:27: 'Invalid Category' is not a valid change type",
            "tests/testdata/single_file/changelog_fail.md:31: duplicate change type in release Unreleased: Bug Fixes",
            "tests/testdata/single_file/changelog_fail.md:43: duplicate release: v15.0.0",
            "tests/testdata/single_file/changelog_fail.md:47: duplicate PR: #1862",
            "tests/testdata/single_file/changelog_fail.md:50: invalid entry: - another malformed entry in changelog",
        ]
    );
}

#[test]
fn it_should_fix_the_changelog_as_expected() {
    let incorrect_changelog = Path::new("tests/testdata/single_file/changelog_to_be_fixed.md");
    let changelog = changelog::parse_changelog(load_test_config(), incorrect_changelog)
        .expect("failed to parse changelog");

    let expected = fs::read_to_string(Path::new("tests/testdata/single_file/changelog_fixed.md"))
        .expect("failed to load correct changelog");

    assert_eq!(
        expected.trim(),
        changelog.get_fixed_contents().trim(),
        "expected different fixed changelog"
    );
}

#[cfg(test)]
fn load_multi_test_config() -> config::Config {
    config::unpack_config(include_str!("testdata/multi_file/noble_config.json"))
        .expect("failed to load example config")
}

#[test]
fn it_should_pass_for_correct_multi_file_changelogs() {
    let correct_changelog = Path::new("tests/testdata/multi_file/ok/.changelog");
    let changelog = multi_file::parse_changelog(&load_multi_test_config(), correct_changelog)
        .expect("failed to parse correct changelog");
    assert_eq!(changelog.releases.len(), 2);
    assert!(changelog.problems.is_empty());
}

#[test]
fn it_should_pass_for_incorrect_multi_file_changelogs_that_has_no_critical_flaws() {
    let incorrect_changelog = Path::new("tests/testdata/multi_file/fail/.changelog");
    let changelog = multi_file::parse_changelog(&load_multi_test_config(), incorrect_changelog)
        .expect("failed to parse incorrect changelog");

    assert_eq!(changelog.releases.len(), 2);
    assert_eq!(changelog.problems, vec![
        "tests/testdata/multi_file/fail/.changelog/v8.0.5/dependencies/466-bump-comet.md:1: PR link is not matching PR number 466: 'https://github.com/noble-assets/noble/pull/467'",
        "tests/testdata/multi_file/fail/.changelog/v9.0.0/features/448-integrate-dollar.md:1: '$USDN' should be used instead of '$UsDN'",
        "tests/testdata/multi_file/fail/.changelog/v9.0.0/dependencies/495-bump-sdk.md:1: PR description should end with a dot: 'Bump Cosmos SDK to [`v0.50.12`](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.50.12)'",
    ]);
}
