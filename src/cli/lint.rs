use crate::{common, config, errors::LintError};

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> Result<(), LintError> {
    let used_config = config::load()?;

    // Load changelog using the common interface (dispatches based on mode)
    let changelog = common::load(&used_config)?;

    if changelog.get_problems().is_empty() {
        println!("changelog has no problems");
        return Ok(());
    }

    if fix {
        // Check if fix is supported for the current mode
        changelog.write(&used_config, changelog.get_path())?;
        println!(
            "automated fixes were applied to {}",
            changelog.path().to_string_lossy()
        );
        return Ok(());
    }

    println!("found problems in changelog:");
    changelog
        .get_problems()
        .iter()
        .for_each(|p| println!("{}", p));

    Err(LintError::ProblemsInChangelog)
}

