use crate::{
    changelog::{self, Changelog},
    config,
    errors::ReleaseCLIError,
    inputs::get_release_type,
    release::Release,
    version
};
use chrono::offset::Local;

/// Creates a new release with the given version based on the given version.
pub fn run(version_option: Option<String>) -> Result<(), ReleaseCLIError> {
    let config = config::load()?;
    let mut changelog = changelog::load(config.clone())?;

    let version = match version_option {
        Some(v) => version::parse(v.as_str())?,
        None => get_release_version(&changelog)?,
    };

    if changelog.releases.iter().any(|x| x.version.eq(&version.to_string())) {
        return Err(ReleaseCLIError::DuplicateVersion(version.to_string()));
    }

    let unreleased = match changelog.releases.iter_mut().find(|x| x.is_unreleased()) {
        Some(r) => r,
        None => return Err(ReleaseCLIError::NoUnreleased),
    };

    let today = Local::now();

    unreleased.version.clone_from(&version.to_string());
    unreleased.fixed = format!(
        "## [{0}]({1}/releases/tag/{0}) - {2}",
        version,
        &config.target_repo,
        today.date_naive()
    );

    Ok(changelog.write(&changelog.path)?)
}

/// Queries the user for the desired release type and then derives the required
/// upgraded version from the existing releases.
///
/// Example: If a user selects a patch release with the latest version being `1.2.3`,
/// the released version would be `1.2.4`.
fn get_release_version(changelog: &Changelog) -> Result<version::Version, ReleaseCLIError> {
    let mut prior_releases: Vec<&Release> = changelog.releases.iter().filter(|x| !x.is_unreleased()).collect();

    // TODO: this should be done when saving the changelog
    prior_releases.sort_by(|a, b| a.version.cmp(&b.version));

    let latest_release = prior_releases.last().unwrap();
    let latest_version = version::parse(&latest_release.version)?;

    let release_type = get_release_type()?;

    let new_version = version::bump_version(&latest_version, &release_type);

    Ok(new_version)
}
