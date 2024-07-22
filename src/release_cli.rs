use crate::{changelog, config, errors::ReleaseCLIError, version};
use chrono::offset::Local;

/// Creates a new release with the given version based on the given version.
pub fn run(version: String) -> Result<(), ReleaseCLIError> {
    let config = config::load()?;
    let mut changelog = changelog::load(config.clone())?;

    version::parse(version.as_str())?;

    if changelog.releases.iter().any(|x| x.version.eq(&version)) {
        return Err(ReleaseCLIError::DuplicateVersion(version.to_string()));
    }

    let unreleased = match changelog.releases.iter_mut().find(|x| x.is_unreleased()) {
        Some(r) => r,
        None => return Err(ReleaseCLIError::NoUnreleased),
    };

    let today = Local::now();

    unreleased.version.clone_from(&version);
    unreleased.fixed = format!(
        "## [{0}]({1}/releases/tag/{0}) - {2}",
        version,
        &config.target_repo,
        today.date_naive()
    );

    Ok(changelog.write(&changelog.path)?)
}
