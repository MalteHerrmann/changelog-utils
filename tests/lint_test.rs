use changelog_utils::{config::Config, lint};
use std::path::Path;

#[cfg(test)]
fn load_test_config() -> Config {
    Config::load(include_str!("testdata/evmos_config.json")).expect("failed to load example config")
}

#[test]
fn it_should_pass_for_correct_changelogs() {
    let correct_changelog = Path::new("tests/testdata/changelog_ok.md");
    let parse_res = lint::lint(load_test_config(), correct_changelog);
    assert!(parse_res.is_ok());
    let changelog = parse_res.unwrap();
    assert_eq!(changelog.releases.len(), 3);
}

#[test]
fn it_should_fail_for_incorrect_changelogs() {
    let incorrect_changelog = Path::new("tests/testdata/changelog_fail.md");
    assert!(lint::lint(load_test_config(), incorrect_changelog).is_err());
}
