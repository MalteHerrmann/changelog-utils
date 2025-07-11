use crate::{config, errors::CheckDiffError, github};

/// Runs the logic to check for a corresponding diff in the changelog,
/// that details the changes of the given pull request, if one is found.
pub async fn run() -> Result<(), CheckDiffError> {
    let config = config::load()?;
    let git_info = github::get_git_info(&config)?;

    let pr_info = github::get_open_pr(git_info).await?;
    println!("got pr base: {}", pr_info.base.label.unwrap_or_default());

    Ok(())
}
