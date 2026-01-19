use super::{add, inputs};
use crate::{
    ai::diff_prompt,
    common::changelog::Changelog,
    config,
    errors::CreateError,
    single_file::changelog,
    utils::{git, github},
};

/// Runs the main logic to open a new PR for the current branch.
pub async fn run() -> Result<(), CreateError> {
    let config = config::load()?;
    let git_info = git::get_git_info(&config)?;
    let client = github::get_authenticated_github_client()?;

    if let Ok(pr_info) = github::get_open_pr(&git_info).await {
        return Err(CreateError::ExistingPR(pr_info.number));
    }

    if !github::branch_exists_on_remote(&client, &git_info).await {
        if !inputs::get_permission_to_push(git_info.branch.as_str())? {
            return Err(CreateError::BranchNotOnRemote(git_info.branch.clone()));
        };

        git::push_to_origin(git_info.branch.as_str())?;

        if !github::branch_exists_on_remote(&client, &git_info).await {
            return Err(CreateError::BranchNotOnRemote(git_info.branch.clone()));
        }
    };

    let branches = client
        .repos(&git_info.owner, &git_info.repo)
        .list_branches()
        .send()
        .await?;

    let target = inputs::get_target_branch(branches)?;

    let diff = match git::get_diff(&git_info.branch, &target) {
        Ok(diff) => diff,
        Err(e) => return Err(e.into()),
    };

    let use_ai = inputs::get_use_ai()?;
    let mut suggestions = diff_prompt::Suggestions::default();
    if use_ai {
        suggestions = diff_prompt::get_suggestions(&config, &diff).await?;
    }

    let change_type = inputs::get_change_type(&config, &suggestions.change_type)?;
    let cat = inputs::get_category(&config, &suggestions.category)?;
    let desc = inputs::get_description(&suggestions.title)?;
    let pr_body = inputs::get_pr_description(&suggestions.pr_description)?;

    let ct = config.get_long_change_type(&change_type).unwrap().short;
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

    let mut changelog = changelog::load(&config)?;
    add::add_entry(
        &config,
        &mut changelog,
        &change_type,
        &cat,
        &desc,
        created_pr.number,
    );

    changelog.write(&config, &changelog.path)?;

    let cm = inputs::get_commit_message(&config)?;
    if let Err(e) = git::commit_and_push(&config, &cm) {
        // NOTE: we don't want to fail here since the PR was created successfully,
        // just the commit of the changelog failed
        println!("failed to commit and push changes: {}", e);
    }

    Ok(())
}
