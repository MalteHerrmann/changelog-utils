use crate::{common, config};
use eyre::{bail, WrapErr};

/// Runs the main logic for the linter, by searching for the changelog file in the
/// current directory and then executing the linting on the found file.
pub fn run(fix: bool) -> eyre::Result<()> {
    let used_config = config::load()
        .wrap_err("Failed to load configuration")?;

    // Load changelog using the common interface (dispatches based on mode)
    let changelog = common::load(&used_config)
        .wrap_err("Failed to load changelog")?;

    if changelog.get_problems().is_empty() {
        println!("changelog has no problems");
        return Ok(());
    }

    if fix {
        // Check if fix is supported for the current mode
        changelog.write(&used_config, changelog.get_path())
            .wrap_err("Failed to write fixed changelog")?;
        println!(
            "automated fixes were applied to {}",
            changelog.path().to_string_lossy()
        );
        return Ok(());
    }

    let problems = changelog.get_problems();

    println!("found problems in changelog:");
    problems.iter().for_each(|p| println!("{}", p));

    println!("\nproblems by type:");
    count_by_type(problems)
        .iter()
        .for_each(|(error_type, count)| println!("  {}: {}", error_type, count));

    bail!("Changelog contains {} problems", problems.len())
}

/// Returns the number of occurrences per type of lint error,
/// sorted by descending amount of occurrences.
fn count_by_type(problems: &[common::Problem]) -> Vec<(common::LintErrorType, usize)> {
    let mut counts: Vec<(common::LintErrorType, usize)> = Vec::new();

    problems.iter().for_each(|p| {
        match counts.iter_mut().find(|(t, _)| *t == p.error_type) {
            Some((_, count)) => *count += 1,
            None => counts.push((p.error_type, 1)),
        };
    });

    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.to_string().cmp(&b.0.to_string())));

    counts
}
