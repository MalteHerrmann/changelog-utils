use crate::{
    change_type, changelog, config, entry,
    errors::AddError,
    github::{get_open_pr, PRInfo},
    inputs, release,
};
use std::borrow::BorrowMut;

// Runs the logic to add an entry to the unreleased section of the changelog.
pub async fn run(accept: bool) -> Result<(), AddError> {
    let config = config::load()?;

    let mut selectable_change_types: Vec<String> =
        config.change_types.clone().into_keys().collect();
    selectable_change_types.sort();

    let retrieved: bool;
    let pr_info = match get_open_pr(&config).await {
        Ok(i) => {
            retrieved = true;
            i
        }
        Err(_) => {
            retrieved = false;
            PRInfo::default()
        }
    };

    let mut selected_change_type = pr_info.change_type.clone();
    if !accept || !retrieved || !selectable_change_types.contains(&pr_info.change_type) {
        let ct_idx = selectable_change_types
            .iter()
            .position(|ct| ct.eq(&pr_info.change_type))
            .unwrap_or_default();

        selected_change_type = inputs::get_change_type(&config, ct_idx)?;
    }

    let mut pr_number = pr_info.number;
    if !accept || !retrieved {
        pr_number = inputs::get_pr_number(pr_info.number)?;
    }

    let mut cat = pr_info.category.clone();
    if !accept || !retrieved || !config.categories.contains(&cat) {
        let cat_idx = config
            .categories
            .iter()
            .position(|c| c.eq(&pr_info.category))
            .unwrap_or_default();

        cat = inputs::get_category(&config, cat_idx)?;
    }

    let mut desc = pr_info.description.clone();
    if !accept || !retrieved {
        desc = inputs::get_description(pr_info.description.as_str())?;
    }

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
