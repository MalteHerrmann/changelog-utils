use crate::{add, changelog, config, errors::CreateError, github, inputs};

/// Runs the main logic to open a new PR for the current branch.
pub async fn run() -> Result<(), CreateError> {
    let config = config::load()?;
    let git_info = github::get_git_info(&config)?;
    let client = github::get_authenticated_github_client()?;

    if let Ok(pr_info) = github::get_open_pr(git_info.clone()).await {
        return Err(CreateError::ExistingPR(pr_info.number));
    }

    if !github::branch_exists_on_remote(&client, &git_info).await {
        if !inputs::get_permission_to_push(git_info.branch.as_str())? {
            return Err(CreateError::BranchNotOnRemote(git_info.branch.clone()));
        };

        github::push_to_origin(git_info.branch.as_str())?;

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

    let use_ai = inputs::get_use_ai()?;
    if use_ai {
        let diff = github::get_diff(git_info.branch.as_str(), target.as_str())?;
        println!("{}", diff);

        // TODO: implement sending diff to LLM and get suggestions for change type, cat, desc and pr
        // body.
    }

    panic!("check here");

    // TODO: get default chosen values from the recommendations above
    let change_type = inputs::get_change_type(&config, 0)?;
    let cat = inputs::get_category(&config, 0)?;
    let desc = inputs::get_description("")?;
    let pr_body = inputs::get_pr_description()?;

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

    let mut changelog = changelog::load(config.clone())?;
    add::add_entry(
        &config,
        &mut changelog,
        ct,
        &cat,
        &desc,
        created_pr.id.0 as u16,
    );

    changelog.write(&changelog.path)?;

    let cm = inputs::get_commit_message(&config)?;
    if let Err(e) = github::commit_and_push(&config, &cm) {
        // NOTE: we don't want to fail here since the PR was created successfully, just the commit of the changelog failed
        println!("failed to commit and push changes: {}", e);
    }

    Ok(())
}
