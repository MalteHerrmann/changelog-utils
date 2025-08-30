use crate::{changelog, config, errors::CheckDiffError, github};

/// Runs the logic to check for a corresponding diff in the changelog,
/// that details the changes of the given pull request, if one is found.
pub async fn run() -> Result<(), CheckDiffError> {
    let config = config::load()?;
    let git_info = github::get_git_info(&config)?;

    let pr_info = github::get_open_pr(&git_info).await?;
    let target_branch = pr_info.base.ref_field;

    let diff = github::get_diff(&git_info.branch, &target_branch)?;

    let changelog = changelog::load(config)?;

    check_diff(&changelog, &diff, pr_info.number)?;

    println!("changelog contains expected entry");
    Ok(())
}

/// Checks the contents of the given diff for the existence
/// of an entry in the unreleased section of the changelog.
fn check_diff(
    changelog: &changelog::Changelog,
    diff: &str,
    pr_number: u64,
) -> Result<(), CheckDiffError> {
    let unreleased = match changelog.releases.iter().find(|&r| r.is_unreleased()) {
        Some(r) => r,
        None => return Err(CheckDiffError::NoUnreleased),
    };

    if !unreleased
        .change_types
        .iter()
        .flat_map(|ct| ct.entries.clone())
        .any(|e| e.pr_number == pr_number)
    {
        // TODO: add logging here?
        return Err(CheckDiffError::NoEntry);
    };

    // Check if the diff actually contains the entry.
    // If not, it was added before already on a different commit / PR.
    if !get_additions(diff)
        .iter()
        // TODO: avoid hardcoding this here? Maybe use parse for entry here and then check PR
        // number?
        .any(|l| l.contains(format!("[#{}]", pr_number).as_str()))
    {
        return Err(CheckDiffError::NoEntry);
    };

    Ok(())
}

/// Extracts the added lines from the git diff.
///
// TODO: This should probably go into a separate git module
fn get_additions(diff: &str) -> Vec<String> {
    let addition_prefix = "+";

    diff.lines()
        .filter_map(|l| l.strip_prefix(addition_prefix))
        .map(|l| l.to_string())
        .collect()
}
