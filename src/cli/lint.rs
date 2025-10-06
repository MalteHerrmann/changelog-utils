use crate::{
    config,
    errors::LintError,
    multi_file,
    single_file::{
        self,
        changelog::{parse_changelog, SingleFileChangelog},
    },
};
use std::path::Path;

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> Result<(), LintError> {
    let used_config = config::load()?;

    // // TODO: this should be unified once both types fulfill the same trait / are under one enum
    // let changelog = match used_config.mode {
    //     config::Mode::Single => single_file::load(used_config)?,
    //     config::Mode::Multi => multi_file::load(&used_config)?,
    // };

    match used_config.mode {
        config::Mode::Single => {
            let changelog = single_file::load(&used_config)?;

            if changelog.problems.is_empty() {
                println!("changelog has no problems");
                return Ok(());
            }

            if fix {
                changelog.write(&used_config, changelog.path.as_path())?;
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
        config::Mode::Multi => {
            let changelog = multi_file::load(&used_config)?;

            if changelog.problems.is_empty() {
                println!("changelog has no problems");
                return Ok(());
            }

            println!("found problems in changelog:");
            changelog.problems.iter().for_each(|p| println!("{}", p));

            Err(LintError::ProblemsInChangelog)
        }
    }
}

/// Executes the linter logic.
pub fn lint(
    config: config::Config,
    changelog_path: &Path,
) -> Result<SingleFileChangelog, LintError> {
    Ok(parse_changelog(&config, changelog_path)?)
}
