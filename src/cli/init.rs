use crate::{
    config::Config, errors::InitError,
    single_file::changelog::get_settings_from_existing_changelog, utils::git::get_origin,
};
use std::{fs, path::PathBuf};

/// Runs the logic to initialize the changelog utilities
/// in the current working directory.
pub fn run() -> Result<(), InitError> {
    init_in_folder(std::env::current_dir()?)
}

/// Runs the logic to initialize the changelog utilities in
/// the given directory.
pub fn init_in_folder(target: PathBuf) -> Result<(), InitError> {
    let config_path = target.join(".clconfig.json");
    if std::fs::symlink_metadata(&config_path).is_ok() {
        return Err(InitError::ConfigAlreadyFound);
    };

    let mut config = Config::default();

    if let Ok(origin) = get_origin() {
        config.target_repo.clone_from(&origin);
    };

    // TODO: check for available changelog file names and adjust config if something else other than the default is found
    let changelog_path = target.join("CHANGELOG.md");
    match fs::read_to_string(changelog_path.clone()) {
        Ok(contents) => {
            get_settings_from_existing_changelog(&mut config, contents.as_str());
        }
        Err(_) => {
            fs::write(changelog_path.as_path(), create_empty_changelog())?;
            println!(
                "created empty changelog at {}",
                changelog_path.as_os_str().to_string_lossy()
            );
        }
    }

    println!(
        "created new configuration at {}:\n{}",
        &config_path.as_os_str().to_string_lossy(),
        &config
    );
    Ok(config.export(config_path.as_path())?)
}

/// Creates an empty skeleton for a changelog.
pub fn create_empty_changelog() -> String {
    [
        "<!--",
        "This changelog was created using the `clu` binary",
        "(https://github.com/MalteHerrmann/changelog-utils).",
        "-->",
        "",
        "# Changelog",
        "",
        "## Unreleased",
        "",
    ]
    .join("\n")
}
