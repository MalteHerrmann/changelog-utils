use crate::{config::Config, utils::release_type::ReleaseType};
use eyre::WrapErr;
use inquire::{Confirm, Editor, MultiSelect, Select, Text};
use octocrab::{models::repos::Branch, Page};

pub fn get_change_type(config: &Config, suggestion: &str) -> eyre::Result<String> {
    let selectable_change_types: Vec<String> = config
        .change_types
        .iter()
        .map(|ct| ct.long.clone())
        .collect();

    let ct_idx = selectable_change_types
        .iter()
        .position(|ct| ct.eq(suggestion))
        .unwrap_or_default();

    Ok(
        Select::new("Select change type to add into:", selectable_change_types)
            .with_starting_cursor(ct_idx)
            .prompt()
            .wrap_err("Failed to get change type selection")?,
    )
}

pub fn get_pr_number(default_value: u64) -> eyre::Result<u64> {
    let input = Text::new("Please provide the PR number:")
        .with_initial_value(format!("{}", &default_value).as_str())
        .prompt()
        .wrap_err("Failed to get PR number input")?;

    input.parse::<u64>()
        .wrap_err("Failed to parse PR number as integer")
}

pub fn get_category(config: &Config, suggestion: &str) -> eyre::Result<String> {
    let idx = config
        .categories
        .iter()
        .position(|cat| cat.eq(suggestion))
        .unwrap_or_default();

    Ok(Select::new(
        "Select the category of the made changes:",
        config.categories.clone(),
    )
    .with_starting_cursor(idx)
    .prompt()
    .wrap_err("Failed to get category selection")?)
}

pub fn get_commit_message(config: &Config) -> eyre::Result<String> {
    Ok(Text::new("Please provide the commit message:\n")
        .with_initial_value(&config.commit_message)
        .prompt()
        .wrap_err("Failed to get commit message input")?)
}

pub fn get_description(default_value: &str) -> eyre::Result<String> {
    Ok(
        Text::new("Please provide a one-line description of the made changes:\n")
            .with_initial_value(default_value)
            .prompt()
            .wrap_err("Failed to get description input")?,
    )
}

pub fn get_permission_to_push(branch: &str) -> eyre::Result<bool> {
    match Select::new(
        format!(
            "Branch {} not found on remote 'origin'. Push the branch?",
            branch
        )
        .as_str(),
        vec!["yes", "no"],
    )
    .prompt()
    .wrap_err("Failed to get permission to push branch")?
    {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => eyre::bail!("Invalid selection for push permission"),
    }
}

pub fn get_pr_description(suggestion: &str) -> eyre::Result<String> {
    Ok(Editor::new(
        "Please provide the Pull Request body with a description of the made changes.\n",
    )
    .with_predefined_text(suggestion)
    .prompt()
    .wrap_err("Failed to get PR description from editor")?)
}

pub fn get_release_type() -> eyre::Result<ReleaseType> {
    let available_types: Vec<&str> = ReleaseType::all().iter().map(|t| t.as_str()).collect();

    let selected_type = Select::new("Select the release type:", available_types)
        .prompt()
        .wrap_err("Failed to get release type selection")?;

    // Convert the selected string back to the ReleaseType enum
    if let Some(r) = ReleaseType::all()
        .iter()
        .find(|&r| r.as_str() == selected_type)
    {
        return Ok(r.clone());
    }

    eyre::bail!("Invalid release type selection: {}", selected_type)
}

pub fn get_target_branch(branches_page: Page<Branch>) -> eyre::Result<String> {
    let mut branches = Vec::new();
    let mut start_idx: usize = 0;

    branches_page.into_iter().enumerate().for_each(|(idx, b)| {
        branches.push(b.name.clone());
        if b.name.eq("main") {
            start_idx = idx;
        }
    });

    Ok(Select::new(
        "Select the target branch to merge the changes into:",
        branches,
    )
    .with_starting_cursor(start_idx)
    .prompt()
    .wrap_err("Failed to get target branch selection")?)
}

pub fn get_use_ai() -> eyre::Result<bool> {
    Ok(
        Confirm::new(
            "Do you want to use AI to suggest changelog contents? Requires API keys to be set in environment. (y/n)\n")
        .prompt()
        .wrap_err("Failed to get AI usage confirmation")?
    )
}

pub fn select_prs_to_add(pr_list: Vec<(u64, String)>) -> eyre::Result<Vec<u64>> {
    let options: Vec<String> = pr_list
        .iter()
        .map(|(num, title)| format!("#{}: {}", num, title))
        .collect();

    let selected = MultiSelect::new(
        "Select PRs to add (use Space to toggle, Enter to confirm):",
        options,
    )
    .with_all_selected_by_default()
    .prompt()
    .wrap_err("Failed to get PR selection")?;

    // Extract PR numbers from selected items
    let selected_prs: Vec<u64> = selected
        .iter()
        .filter_map(|s| {
            // Parse "#123: title" to extract 123
            s.split(':')
                .next()?
                .trim_start_matches('#')
                .trim()
                .parse::<u64>()
                .ok()
        })
        .collect();

    Ok(selected_prs)
}
