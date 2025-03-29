use crate::{
    change_type, changelog, config, entry,
    errors::{AddError, GitHubError},
    github::{commit, extract_pr_info, get_git_info, get_open_pr, get_pr_by_number, PRInfo, GitInfo},
    inputs, release,
};
use std::borrow::BorrowMut;

/// Retrieves PR information either from a specific PR number or from an open PR.
/// If a PR number is provided but no PR is found, returns an error.
async fn get_pr_info(config: &config::Config, git_info: &GitInfo, pr_number: Option<u16>) -> Result<(PRInfo, bool), AddError> {
    let pr_info: PRInfo;

    if let Some(pr_number) = pr_number {
        // Try to fetch PR information using the provided PR number
        match get_pr_by_number(git_info, pr_number).await {
            Ok(pr) => {
                pr_info = extract_pr_info(config, &pr)?;
                return Ok((pr_info, true));
            }
            Err(_) => {
                return Err(AddError::PRInfo(GitHubError::NoOpenPR));
            }
        }
    }
    // If no PR number was provided, try to get open PR
    return match get_open_pr(git_info.clone()).await {
        Ok(pr) => {
            pr_info = extract_pr_info(config, &pr)?;
            Ok((pr_info, true))
        }
        Err(_) => {
            Ok((PRInfo::default(), false))
        }
    }
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

    let mut selectable_change_types: Vec<String> =
        config.change_types.clone().into_keys().collect();
    selectable_change_types.sort();

    let (mut pr_info, retrieved) = get_pr_info(&config, &git_info, pr_number).await?;

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
