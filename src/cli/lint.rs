use crate::{
    config,
    errors::LintError,
    single_file::{
        self,
        changelog::{self, parse_changelog, SingleFileChangelog},
    },
};
use std::path::Path;

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> Result<(), LintError> {
    let used_config = config::load()?;

    let changelog = match used_config.mode {
        config::Mode::Single => single_file::load(used_config)?,
        config::Mode::Multi => {
            panic!("not implemented")
        }
    };

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
pub fn lint(
    config: config::Config,
    changelog_path: &Path,
) -> Result<SingleFileChangelog, LintError> {
    Ok(parse_changelog(config, changelog_path)?)
}
