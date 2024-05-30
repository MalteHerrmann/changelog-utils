use clu::{config, lint};
use std::path::Path;

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::load(include_str!("testdata/evmos_config.json")).expect("failed to load example config")
}

#[test]
fn it_should_pass_for_correct_changelogs() {
    let correct_changelog = Path::new("tests/testdata/changelog_ok.md");
    let changelog = lint::lint(load_test_config(), correct_changelog)
        .expect("failed to parse correct changelog");
    assert_eq!(changelog.releases.len(), 3);
    assert!(changelog.problems.is_empty());
}

#[test]
fn it_should_pass_for_incorrect_changelogs_that_has_no_critical_flaws() {
    let incorrect_changelog = Path::new("tests/testdata/changelog_fail.md");
    let changelog = lint::lint(load_test_config(), incorrect_changelog)
        .expect("failed to parse incorrect changelog");
    assert_eq!(changelog.releases.len(), 2);
    assert_eq!(changelog.problems, vec![
        "PR link is not matching PR number 1948: 'https://github.com/evmos/evmos/pull/1949'",
        "There should be no backslash in front of the # in the PR link",
        "'ABI' should be used instead of 'ABi'",
        "PR description should end with a dot: 'Fixed the problem `gas_used` is 0'",
        "'Invalid Category' is not a valid change type",
        "duplicate change type in release Unreleased: Bug Fixes",
        "duplicate PR in v15.0.0->API Breaking: 1801",
        "duplicate release: v15.0.0",
        "duplicate PR in v15.0.0->API Breaking: 1862",
        "invalid entry: - malformed entry in changelog",
    ]);
}
