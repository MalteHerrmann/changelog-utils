use crate::{changelog::{Changelog, parse_changelog}, errors::LintError};
use std::{
    fs,
    path::Path,
};

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run() -> Result<Changelog, LintError> {
    let changelog_file = match fs::read_dir(Path::new("./"))?.find(|e| {
        e.as_ref()
            .is_ok_and(|e| e.file_name().to_ascii_lowercase() == "changelog.md")
    }) {
        Some(f) => f.unwrap(),
        None => return Err(LintError::NoChangelogFound),
    };

    lint(changelog_file.path().as_path())
}

/// Executes the linter logic.
///
/// TODO: Check if this is actually necessary or parse_changelog can be used directly?
pub fn lint(changelog_path: &Path) -> Result<Changelog, LintError> {
    let contents = fs::read_to_string(changelog_path)?;
    Ok(parse_changelog(contents.as_str())?)
}
