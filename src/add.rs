use crate::errors::AddError;
use crate::{changelog, config, entry};
use inquire::Select;
use std::path::Path;

// Runs the logic to add an entry to the unreleased section of the changelog.
pub fn run() -> Result<(), AddError> {
    let config = config::load()?;
    let mut changelog = changelog::load(config.clone())?;

    // TODO: don't assume first release is unreleased but check for it and if missing add the unreleased section
    let unreleased = changelog
        .releases
        .get_mut(0)
        .expect("failed to get first release");
    if &unreleased.version != "Unreleased" {
        return Err(AddError::FirstReleaseNotUnreleased(
            unreleased.version.clone(),
        ));
    }

    let selectable_change_types: Vec<String> = unreleased
        .clone()
        .change_types
        .into_iter()
        .map(|x| x.name)
        .collect();

    let change_type =
        Select::new("Select change type to add into", selectable_change_types).prompt()?;

    println!("selected change type: {}", change_type);
    let mut idx = 0;
    let mut found = false;
    for (i, ct) in unreleased.clone().change_types.into_iter().enumerate() {
        if ct.name.eq(&change_type) {
            println!("found at idx: {}", i);
            idx = i;
            found = true;
        }
    }

    if !found {
        // TODO: remove, should be handled better above
        return Err(AddError::Generic);
    }

    let mut_ct = unreleased
        .change_types
        .get_mut(idx)
        .expect("failed to get change type");

    // TODO: collect info to add
    mut_ct
        .entries
        .push(entry::Entry::new(config, "cat", "desc", 15));

    Ok(changelog.write(Path::new("CHANGELOG.md"))?)
}
