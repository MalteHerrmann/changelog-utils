use crate::errors::AddError;
use crate::{change_type, changelog, config, entry, github::get_open_pr, release};
use inquire::{Select, Text};
use std::borrow::BorrowMut;

// Runs the logic to add an entry to the unreleased section of the changelog.
pub async fn run() -> Result<(), AddError> {
    let config = config::load()?;

    let mut selectable_change_types: Vec<String> =
        config.change_types.clone().into_keys().collect();
    selectable_change_types.sort();

    let pr_info = get_open_pr(&config).await?;

    let ct_idx = selectable_change_types
        .iter()
        .position(|ct| ct.eq(&pr_info.change_type))
        .unwrap_or_default();

    let selected_change_type =
        Select::new("Select change type to add into:", selectable_change_types)
            .with_starting_cursor(ct_idx)
            .prompt()?;

    let pr_number = match Text::new("Please provide the PR number:")
        .with_initial_value(&pr_info.number)
        .prompt()?
        .parse::<u16>()
    {
        Ok(pr) => pr,
        Err(e) => return Err(AddError::Input(e.into())),
    };

    let cat_idx = config
        .categories
        .iter()
        .position(|c| c.eq(&pr_info.category))
        .unwrap_or_default();

    let cat = Select::new(
        "Select the category of the made changes:",
        config.categories.clone(),
    )
    .with_starting_cursor(cat_idx)
    .prompt()?;

    let desc = Text::new("Please provide a short description of the made changes:\n")
        .with_initial_value(&pr_info.description)
        .prompt()?;

    let mut changelog = changelog::load(config.clone())?;
    add_entry(
        &config,
        changelog.borrow_mut(),
        selected_change_type.as_str(),
        cat.as_str(),
        desc.as_str(),
        pr_number,
    );

    Ok(changelog.write(&changelog.path)?)
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

    let mut idx = 0;
    let mut found = false;
    for (i, ct) in unreleased.clone().change_types.into_iter().enumerate() {
        if ct.name.eq(&change_type) {
            idx = i;
            found = true;
        }
    }

    let new_entry = entry::Entry::new(config, cat, desc, pr);

    // Get the mutable change type to add the entry into.
    // NOTE: If it's not found yet, we add a new section to the changelog.
    match found {
        false => {
            let new_ct = change_type::new(change_type.to_owned(), Some(vec![new_entry]));
            unreleased.change_types.push(new_ct);
        }
        true => {
            let mut_ct = unreleased
                .change_types
                .get_mut(idx)
                .expect("failed to get change type");

            mut_ct.entries.insert(0, new_entry);
        }
    }
}
