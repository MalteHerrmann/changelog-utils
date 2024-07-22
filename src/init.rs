use crate::{config::Config, errors::InitError, github::get_origin};
use std::{collections::BTreeMap, fs, path::PathBuf};

/// Runs the logic to initialize the changelog utilities
/// in the current working directory.
pub fn run() -> Result<(), InitError> {
    init_in_folder(std::env::current_dir()?)
}

/// Runs the logic to initialize the changelog utilities in
/// the given directory.
pub fn init_in_folder(target: PathBuf) -> Result<(), InitError> {
    let changelog_path = target.join("CHANGELOG.md");
    match fs::read_to_string(changelog_path.clone()) {
        Ok(_) => println!("changelog file found"),
        Err(_) => fs::write(changelog_path.as_path(), create_empty_changelog())?,
    }

    let config_path = target.join(".clconfig.json");
    let mut config = create_default_config();

    if let Ok(origin) = get_origin() {
        config.target_repo.clone_from(&origin);
        println!("configured target repository: {}", origin);
    };

    if fs::read_to_string(&config_path).is_ok() {
        return Err(InitError::ConfigAlreadyFound);
    };

    Ok(config.export(config_path.as_path())?)
}

/// Creates a new default configuration file for the tool.
fn create_default_config() -> Config {
    let mut default_change_types: BTreeMap<String, String> = BTreeMap::new();
    default_change_types.insert("Bug Fixes".into(), "bug\\s*fixes".into());
    default_change_types.insert("Features".into(), "features".into());
    default_change_types.insert("Improvements".into(), "improvements".into());

    Config {
        categories: Vec::new(),
        change_types: default_change_types,
        expected_spellings: BTreeMap::new(),
        legacy_version: None,
        target_repo: "".to_string(),
    }
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
