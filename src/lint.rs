use crate::{
    changelog::{parse_changelog, Changelog},
    config,
    errors::LintError,
};
use std::{fs, path::Path};

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> Result<(), LintError> {
    let changelog_file = match fs::read_dir(Path::new("./"))?.find(|e| {
        e.as_ref()
            .is_ok_and(|e| e.file_name().to_ascii_lowercase() == "changelog.md")
    }) {
        Some(f) => f.unwrap(),
        None => {
            println!("could not find the changelog in the current directory");
            return Err(LintError::NoChangelogFound);
        }
    };

    let config = config::unpack_config(
        fs::read_to_string(Path::new(".clconfig.json"))?.as_str()
    )?;

    let changelog = lint(config, &changelog_file.path())?;
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
                    fs::write(changelog_file.path(), changelog.fixed.join("\n"))?;
                    println!("automated fixes were applied to {}", changelog_file.path().to_string_lossy());
                    Ok(())
                }
            }
        }
    }
}

/// Executes the linter logic.
pub fn lint(config: config::Config, changelog_path: &Path) -> Result<Changelog, LintError> {
    let contents = fs::read_to_string(changelog_path)?;
    Ok(parse_changelog(config, contents.to_owned().as_str())?)
}
