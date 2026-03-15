use super::inputs::get_release_type;
use crate::{
    common::changelog::Changelog,
    config,
    single_file::{
        changelog::{self, SingleFileChangelog},
        release::Release,
    },
    utils::version,
};
use chrono::offset::Local;
use eyre::{bail, WrapErr};

/// Creates a new release with the given version based on the given version.
pub fn run(version_option: Option<String>) -> eyre::Result<()> {
    let config = config::load()
        .wrap_err("Failed to load configuration")?;
    let mut changelog = changelog::load(&config)
        .wrap_err("Failed to load changelog")?;

    let version = match version_option {
        Some(v) => version::parse(v.as_str())
            .wrap_err_with(|| format!("Failed to parse version string '{}'", v))?,
        None => get_next_release_version(&changelog)
            .wrap_err("Failed to determine next release version")?,
    };

    if changelog
        .releases
        .iter()
        .any(|x| x.version.eq(&version.to_string()))
    {
        bail!("Version '{}' already exists in changelog", version);
    }

    let unreleased = match changelog.releases.iter_mut().find(|x| x.is_unreleased()) {
        Some(r) => r,
        None => bail!("No unreleased section found in changelog"),
    };

    let today = Local::now();

    unreleased.version.clone_from(&version.to_string());
    unreleased.fixed = format!(
        "## [{0}]({1}/releases/tag/{0}) - {2}",
        version,
        &config.target_repo,
        today.date_naive()
    );

    changelog.write(&config, &changelog.path)
        .wrap_err("Failed to write changelog with new release")
}

/// Queries the user for the desired release type and then derives the required
/// upgraded version from the existing releases.
///
/// Example: If a user selects a patch release with the latest version being `1.2.3`,
/// the released version would be `1.2.4`.
fn get_next_release_version(
    changelog: &SingleFileChangelog,
) -> eyre::Result<version::Version> {
    let mut prior_releases: Vec<&Release> = changelog
        .releases
        .iter()
        .filter(|x| !x.is_unreleased())
        .collect();

    // TODO: this should be done when saving the changelog
    prior_releases.sort_by(|a, b| a.version.cmp(&b.version));

    let latest_release = prior_releases.last().unwrap();
    let latest_version = version::parse(&latest_release.version)
        .wrap_err_with(|| format!("Failed to parse latest release version '{}'", latest_release.version))?;

    let release_type = get_release_type()
        .wrap_err("Failed to get release type from user")?;

    let new_version = version::bump_version(&latest_version, &release_type);

    Ok(new_version)
}
