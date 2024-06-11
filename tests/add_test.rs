use clu::{add, changelog, config};
use std::{borrow::BorrowMut, path::Path};

#[cfg(test)]
fn load_example_config() -> config::Config {
    config::unpack_config(include_str!("testdata/evmos_config.json"))
        .expect("failed to load example configuration")
}

#[test]
fn test_pass_add_into_new_change_type() {
    let config = load_example_config();
    let mut changelog = changelog::parse_changelog(
        config.clone(),
        Path::new("tests/testdata/changelog_new_category_after_add.md"),
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
        config.clone(),
        Path::new("tests/testdata/changelog_no_unreleased.md"),
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
