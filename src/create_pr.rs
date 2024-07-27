use crate::{config, errors::CreateError, github, inputs};
use octocrab::params::repos::Reference::Branch;

/// Runs the main logic to open a new PR for the current branch.
pub async fn run() -> Result<(), CreateError> {
    let config = config::load()?;
    let git_info = github::get_git_info(&config)?;
    let client = github::get_authenticated_github_client()?;

    if client
        .repos(&git_info.owner, &git_info.repo)
        .get_ref(&Branch(git_info.branch.clone()))
        .await
        .is_err()
    {
        // TODO: add option to push the branch?
        return Err(CreateError::BranchNotOnRemote(git_info.branch.clone()));
    };

    if let Ok(pr_info) = github::get_open_pr(git_info.clone()).await {
        return Err(CreateError::ExistingPR(pr_info.number));
    }

    let change_type = inputs::get_change_type(&config, 0)?;
    let cat = inputs::get_category(&config, 0)?;
    let desc = inputs::get_description("")?;
    let pr_body = inputs::get_pr_description()?;

    let branches = client
        .repos(&git_info.owner, &git_info.repo)
        .list_branches()
        .send()
        .await?;
    let target = inputs::get_target_branch(branches)?;

    let ct = config.change_types.get(&change_type).unwrap();
    let title = format!("{ct}({cat}): {desc}");

    let created_pr = client
        .pulls(&git_info.owner, &git_info.repo)
        .create(title, git_info.branch, target)
        .body(pr_body)
        .send()
        .await?;

    println!(
        "created pull request: {}",
        created_pr
            .html_url
            .expect("received no error creating the PR but html_url was None")
    );

    Ok(())
}
