use changelog_utils::lint;
use std::path::Path;

#[test]
fn it_should_pass_for_correct_changelogs() {
    println!("Current dir: {:?}", std::env::current_dir().unwrap());
    let correct_changelog = Path::new("tests/testdata/changelog_ok.md");
    let parse_res = lint::lint(correct_changelog);
    assert!(parse_res.is_ok());
    let changelog = parse_res.unwrap();
    assert!(changelog.releases.len() == 3);
}

#[test]
fn it_should_fail_for_incorrect_changelogs() {
    let incorrect_changelog = Path::new("tests/testdata/changelog_fail.md");
    assert!(lint::lint(incorrect_changelog).is_err());
}
