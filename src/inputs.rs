use crate::{config::Config, errors::InputError, release_type::ReleaseType};
use inquire::{Confirm, Editor, Select, Text};
use octocrab::{models::repos::Branch, Page};

pub fn get_change_type(config: &Config, suggestion: &str) -> Result<String, InputError> {
    let mut selectable_change_types: Vec<String> =
        config.change_types.clone().into_keys().collect();
    selectable_change_types.sort();

    let ct_idx = selectable_change_types
        .iter()
        .position(|ct| ct.eq(suggestion))
        .unwrap_or_default();

    Ok(
        Select::new("Select change type to add into:", selectable_change_types)
            .with_starting_cursor(ct_idx)
            .prompt()?,
    )
}

pub fn get_pr_number(default_value: u16) -> Result<u16, InputError> {
    Ok(Text::new("Please provide the PR number:")
        .with_initial_value(format!("{}", &default_value).as_str())
        .prompt()?
        .parse::<u16>()?)
}

pub fn get_category(config: &Config, suggestion: &str) -> Result<String, InputError> {
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
    .prompt()?)
}

pub fn get_commit_message(config: &Config) -> Result<String, InputError> {
    Ok(Text::new("Please provide the commit message:\n")
        .with_initial_value(&config.commit_message)
        .prompt()?)
}

pub fn get_description(default_value: &str) -> Result<String, InputError> {
    Ok(
        Text::new("Please provide a one-line description of the made changes:\n")
            .with_initial_value(default_value)
            .prompt()?,
    )
}

pub fn get_permission_to_push(branch: &str) -> Result<bool, InputError> {
    match Select::new(
        format!(
            "Branch {} not found on remote 'origin'. Push the branch?",
            branch
        )
        .as_str(),
        vec!["yes", "no"],
    )
    .prompt()?
    {
        "yes" => Ok(true),
        "no" => Ok(false),
        &_ => Err(InputError::InvalidSelection),
    }
}

pub fn get_pr_description(suggestion: &str) -> Result<String, InputError> {
    Ok(Editor::new(
        "Please provide the Pull Request body with a description of the made changes.\n",
    )
    .with_predefined_text(suggestion)
    .prompt()?)
}

pub fn get_release_type() -> Result<ReleaseType, InputError> {
    let available_types: Vec<&str> = ReleaseType::all().iter().map(|t| t.as_str()).collect();

    let selected_type = Select::new("Select the release type:", available_types).prompt()?;

    // Convert the selected string back to the ReleaseType enum
    if let Some(r) = ReleaseType::all()
        .iter()
        .find(|&r| r.as_str() == selected_type)
    {
        return Ok(r.clone());
    }

    Err(InputError::InvalidSelection)
}

pub fn get_target_branch(branches_page: Page<Branch>) -> Result<String, InputError> {
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
    .prompt()?)
}

pub fn get_use_ai() -> Result<bool, InputError> {
    Ok(
        Confirm::new(
            "Do you want to use AI to suggest changelog contents? Requires API keys to be set in environment. (y/n)\n")
        .prompt()?
    )
}
