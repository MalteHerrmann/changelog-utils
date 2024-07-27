use crate::{config, errors::CreateError, github, inputs};
use octocrab::params::repos::Reference::Branch;

/// Runs the main logic to open a new PR for the current branch.
pub async fn run() -> Result<(), CreateError> {
    let config = config::load()?;

    if let Ok(pr_info) = github::get_open_pr(&config).await {
        return Err(CreateError::ExistingPR(pr_info.number))
    }

    let git_info = github::get_git_info(&config)?;

    let change_type = inputs::get_change_type(&config, 0)?;
    let cat = inputs::get_category(&config, 0)?;
    let desc = inputs::get_description("")?;
    let pr_body = inputs::get_pr_description()?;

    let ct = config.change_types.get(&change_type).unwrap();
    let title = format!("{ct}({cat}): {desc}");

    let client = github::get_authenticated_github_client()?;
    if let Err(_) = client.repos(&git_info.owner, &git_info.repo)
        .get_ref(&Branch(git_info.branch.clone()))
        .await {
            // TODO: add option to push the branch?
            return Err(CreateError::BranchNotOnRemote(git_info.branch))
        };

    let created_pr = client.pulls(&git_info.owner, &git_info.repo)
        // TODO: enable targeting another branch other than main
        .create(title, git_info.branch, "main")
        .body(pr_body)
        .send()
        .await?;

    println!("Created PR #{}", created_pr.number);
    println!("Created PR {}", created_pr.url);

    Ok(())
}