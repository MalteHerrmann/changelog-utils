use crate::{
    config,
    errors::CheckDiffError,
    single_file::changelog,
    utils::{git, github},
};

/// Runs the logic to check for a corresponding diff in the changelog,
/// that details the changes of the given pull request, if one is found.
pub async fn run() -> Result<(), CheckDiffError> {
    let config = config::load()?;
    let git_info = git::get_git_info(&config)?;

    let pr_info = github::get_open_pr(&git_info).await?;
    let target_branch = pr_info.base.ref_field;

    let diff = git::get_diff(&git_info.branch, &target_branch)?;

    let changelog = changelog::load(config)?;

    check_diff(&changelog, &diff, pr_info.number)?;

    println!("changelog contains expected entry");
    Ok(())
}

/// Checks the contents of the given diff for the existence
/// of an entry in the unreleased section of the changelog.
fn check_diff(
    // TODO: this should use a common util instead of the single file thing
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
        println!("no changelog entry found for PR {}", pr_number);
        return Err(CheckDiffError::NoEntry);
    };

    // Check if the diff actually contains the entry.
    // If not, it was added before already on a different commit / PR.
    if !git::get_additions(diff)
        .iter()
        // TODO: avoid hardcoding this here? Maybe use parse for entry here and then check PR
        // number?
        .any(|l| l.contains(format!("[#{}]", pr_number).as_str()))
    {
        println!("changelog entry for PR {} was already present", pr_number);
        return Err(CheckDiffError::NoEntry);
    };

    Ok(())
}
