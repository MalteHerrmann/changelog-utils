use crate::{changelog::{parse_changelog, Changelog}, changelog, config, errors::LintError};
use std::{path::Path};

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> Result<(), LintError> {
    let changelog = changelog::load(config::load()?)?;
    match changelog.problems.is_empty() {
        true => {
            println ! ("changelog has no problems");
            Ok(())
        },
        false => {
            match fix {
                false => {
                    println!("found problems in changelog:");
                    for problem in changelog.problems {
                        println!("{}", problem);
                    }
                    Err(LintError::ProblemsInChangelog)
                },
                true => {
                    changelog.write(changelog.path.as_path())?;
                    println!("automated fixes were applied to {}", changelog.path.to_string_lossy());
                    Ok(())
                }
            }
        }
    }
}

/// Executes the linter logic.
pub fn lint(config: config::Config, changelog_path: &Path) -> Result<Changelog, LintError> {
    Ok(parse_changelog(config, changelog_path)?)
}
