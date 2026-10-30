use super::{add, inputs};
use crate::{
    ai::diff_prompt,
    common::changelog::Changelog,
    config,
    single_file::changelog,
    utils::{git, github},
};
use eyre::{bail, WrapErr};

/// Runs the main logic to open a new PR for the current branch.
pub async fn run() -> eyre::Result<()> {
    let config = config::load()
        .wrap_err("Failed to load configuration")?;
    let git_info = git::get_git_info(&config)
        .wrap_err("Failed to get git information")?;
    let client = github::get_authenticated_github_client()
        .wrap_err("Failed to get authenticated GitHub client")?;

    if let Ok(pr_info) = github::get_open_pr(&git_info).await {
        bail!("An open PR already exists for this branch: #{}", pr_info.number);
    }

    if !github::branch_exists_on_remote(&client, &git_info).await {
        if !inputs::get_permission_to_push(git_info.branch.as_str())? {
            bail!("Branch '{}' not found on remote and permission to push was denied", git_info.branch);
        };

        git::push_to_origin(git_info.branch.as_str())
            .wrap_err_with(|| format!("Failed to push branch '{}' to origin", git_info.branch))?;

        if !github::branch_exists_on_remote(&client, &git_info).await {
            bail!("Branch '{}' still not found on remote after pushing", git_info.branch);
        }
    };

    let branches = client
        .repos(&git_info.owner, &git_info.repo)
        .list_branches()
        .send()
        .await
        .wrap_err("Failed to list branches from GitHub")?;

    let target = inputs::get_target_branch(branches)
        .wrap_err("Failed to get target branch selection")?;

    let diff = git::get_diff(&git_info.branch, &target)
        .wrap_err_with(|| format!("Failed to get diff between {} and {}", git_info.branch, target))?;

    let use_ai = inputs::get_use_ai()
        .wrap_err("Failed to get AI usage preference")?;
    let mut suggestions = diff_prompt::Suggestions::default();
    if use_ai {
        suggestions = diff_prompt::get_suggestions(&config, &diff)
            .await
            .wrap_err("Failed to get AI suggestions for PR")?;
    }

    let change_type = inputs::get_change_type(&config, &suggestions.change_type)
        .wrap_err("Failed to get change type for PR")?;
    let cat = inputs::get_category(&config, &suggestions.category)
        .wrap_err("Failed to get category for PR")?;
    let desc = inputs::get_description(&suggestions.title)
        .wrap_err("Failed to get description for PR")?;
    let pr_body = inputs::get_pr_description(&suggestions.pr_description)
        .wrap_err("Failed to get PR description")?;

    let ct = config.get_long_change_type(&change_type).unwrap().short;
    let title = format!("{ct}({cat}): {desc}");

    let created_pr = client
        .pulls(&git_info.owner, &git_info.repo)
        .create(title, git_info.branch, target)
        .body(pr_body)
        .send()
        .await
        .wrap_err("Failed to create pull request on GitHub")?;

    let pr_url = created_pr
        .html_url
        .ok_or_else(|| eyre::eyre!("PR was created but no URL was returned"))?;
    println!("created pull request: {}", pr_url);

    let mut changelog = changelog::load(&config)
        .wrap_err("Failed to load changelog")?;
    add::add_entry(
        &config,
        &mut changelog,
        &change_type,
        &cat,
        &desc,
        created_pr.number,
    );

    changelog.write(&config, &changelog.path)
        .wrap_err("Failed to write changelog with PR entry")?;

    let cm = inputs::get_commit_message(&config)
        .wrap_err("Failed to get commit message")?;
    if let Err(e) = git::commit_and_push(&config, &cm) {
        // NOTE: we don't want to fail here since the PR was created successfully,
        // just the commit of the changelog failed
        println!("failed to commit and push changes: {}", e);
    }

    Ok(())
}
