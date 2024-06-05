use crate::errors::AddError;
use crate::{changelog, config};

// Runs the logic to add an entry to the unreleased section of the changelog.
pub fn run() -> Result<(), AddError> {
    let mut changelog = changelog::load(config::load()?)?;

    Ok(())
}