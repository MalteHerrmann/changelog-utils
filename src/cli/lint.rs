use crate::{
    config,
    errors::LintError,
    single_file::changelog::{self, parse_changelog, Changelog},
};
use std::path::Path;

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> Result<(), LintError> {
    let changelog = changelog::load(config::load()?)?;
    if changelog.problems.is_empty() {
        println!("changelog has no problems");
        return Ok(());
    }

    if fix {
        changelog.write(changelog.path.as_path())?;
        println!(
            "automated fixes were applied to {}",
            changelog.path.to_string_lossy()
        );

        return Ok(());
    }

    println!("found problems in changelog:");
    changelog.problems.iter().for_each(|p| println!("{}", p));

    Err(LintError::ProblemsInChangelog)
}

/// Executes the linter logic.
pub fn lint(config: config::Config, changelog_path: &Path) -> Result<Changelog, LintError> {
    Ok(parse_changelog(config, changelog_path)?)
}
