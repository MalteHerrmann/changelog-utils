use crate::{config, errors::CreateError, github, inputs};

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

    let ct = config.change_types.get(&change_type).unwrap();
    let title = format!("{ct}({cat}): {desc}");

    let client = github::get_authenticated_github_client()?;
    let created_pr = client.pulls(git_info.owner, git_info.repo)
        .create(title, git_info.branch, "main")
        .body("")
        .send()
        .await?;

    println!("Created PR #{}", created_pr.number);

    Ok(())
}