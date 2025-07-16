use crate::{
    change_type, changelog, config, entry,
    errors::AddError,
    git::{commit, get_git_info},
    github::{get_pr_info, PRInfo},
    inputs, release,
};
use std::collections::HashMap;

/// Determines if user input is required based on the accept flag and whether PR info was retrieved.
fn should_get_user_input(accept: bool, retrieved: bool) -> bool {
    !accept || !retrieved
}

/// Handles all user input for the changelog entry, either using existing PR info or prompting for input.
fn get_entry_inputs(
    config: &config::Config,
    pr_info: &mut PRInfo,
    accept: bool,
    retrieved: bool,
) -> Result<(String, u16, String, String), AddError> {
    let selectable_change_types: Vec<String> = config
        .change_types
        .iter()
        .map(|ct| ct.long.to_owned())
        .collect();

    // populate the map with false if user input is not required, otherwise true
    let mut get_inputs: HashMap<&str, bool> =
        ["change_type", "pr_number", "category", "description"]
            .into_iter()
            .map(|key| (key, should_get_user_input(accept, retrieved)))
            .collect();

    let mut selected_change_type = pr_info.change_type.clone();
    if !selectable_change_types.contains(&pr_info.change_type) {
        get_inputs.insert("change_type", true);
    }

    if get_inputs["change_type"] {
        selected_change_type = inputs::get_change_type(config, &pr_info.change_type)?;
    }

    let mut pr_number = pr_info.number;
    if get_inputs["pr_number"] {
        pr_number = inputs::get_pr_number(pr_info.number)?;
    }

    let mut cat = pr_info.category.clone();
    if !config.categories.contains(&cat) {
        get_inputs.insert("category", true);
    }

    if get_inputs["category"] {
        cat = inputs::get_category(config, &pr_info.category)?;
    }

    let mut desc = pr_info.description.clone();
    if get_inputs["description"] {
        desc = inputs::get_description(pr_info.description.as_str())?;
    }

    Ok((selected_change_type, pr_number, cat, desc))
}

// Runs the logic to add an entry to the unreleased section of the changelog.
//
// After adding the new entry, the user is queried for a commit message to use
// to commit the changes.
//
// NOTE: the changes are NOT pushed to the origin when running the `add` command.
pub async fn run(pr_number: Option<u16>, accept: bool) -> Result<(), AddError> {
    let config = config::load()?;
    let git_info = get_git_info(&config)?;

    let mut pr_info = get_pr_info(&config, &git_info, pr_number).await?;
    let retrieved = pr_info.number != 0;

    let (selected_change_type, pr_number, cat, desc) =
        get_entry_inputs(&config, &mut pr_info, accept, retrieved)?;

    let mut changelog = changelog::load(config.clone())?;
    add_entry(
        &config,
        &mut changelog,
        &selected_change_type,
        &cat,
        &desc,
        pr_number,
    );

    changelog.write(&changelog.path)?;

    let cm = inputs::get_commit_message(&config)?;
    Ok(commit(&config, &cm)?)
}

/// Adds the given contents into a new entry in the unreleased section
/// of the changelog.
pub fn add_entry(
    config: &config::Config,
    changelog: &mut changelog::Changelog,
    change_type: &str,
    cat: &str,
    desc: &str,
    pr: u16,
) {
    let unreleased = match changelog.releases.iter_mut().find(|r| r.is_unreleased()) {
        Some(r) => r,
        None => {
            let mut new_releases = vec![release::new_unreleased()];
            new_releases.append(changelog.releases.as_mut());

            changelog.releases = new_releases;
            changelog.releases.get_mut(0).unwrap()
        }
    };

    // TODO: improve this to avoid using the lookup via the for loop, but rather use map
    let mut idx = 0;
    let mut change_type_is_found = false;
    for (i, ct) in unreleased.clone().change_types.into_iter().enumerate() {
        if ct.name.eq(&change_type) {
            idx = i;
            change_type_is_found = true;
        }
    }

    let new_entry = entry::Entry::new(config, cat, desc, pr);
    // NOTE: we're re-parsing the entry from the fixed version to incorporate all possible fixes
    let new_fixed_entry = entry::parse(config, new_entry.fixed.as_str()).unwrap();

    // Get the mutable change type to add the entry into.
    // NOTE: If it's not found yet, we add a new section to the changelog.
    if change_type_is_found {
        let mut_ct = unreleased
            .change_types
            .get_mut(idx)
            .expect("failed to get change type");

        mut_ct.entries.insert(0, new_fixed_entry);
    } else {
        let new_ct = change_type::new(change_type.to_owned(), Some(vec![new_fixed_entry]));
        unreleased.change_types.push(new_ct);
    }
}
