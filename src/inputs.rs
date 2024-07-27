use crate::{config::Config, errors::InputError};
use inquire::{Editor, Select, Text};
use octocrab::{models::repos::Branch, Page};

pub fn get_change_type(config: &Config, start: usize) -> Result<String, InputError> {
    let mut selectable_change_types: Vec<String> =
        config.change_types.clone().into_keys().collect();
    selectable_change_types.sort();

    Ok(
        Select::new("Select change type to add into:", selectable_change_types)
            .with_starting_cursor(start)
            .prompt()?,
    )
}

pub fn get_pr_number(default_value: u16) -> Result<u16, InputError> {
    Ok(Text::new("Please provide the PR number:")
        .with_initial_value(format!("{}", &default_value).as_str())
        .prompt()?
        .parse::<u16>()?)
}

pub fn get_category(config: &Config, default_idx: usize) -> Result<String, InputError> {
    Ok(Select::new(
        "Select the category of the made changes:",
        config.categories.clone(),
    )
    .with_starting_cursor(default_idx)
    .prompt()?)
}

pub fn get_description(default_value: &str) -> Result<String, InputError> {
    Ok(
        Text::new("Please provide a one-line description of the made changes:\n")
            .with_initial_value(default_value)
            .prompt()?,
    )
}

pub fn get_pr_description() -> Result<String, InputError> {
    Ok(
        Editor::new("Please provide the Pull Request body with a description of the made changes.\n")
            .prompt()?,
    )
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
