use crate::{config::Config, errors::InitError};
use serde_json::to_string_pretty;
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
    match fs::read_to_string(config_path.clone()) {
        Ok(_) => Err(InitError::ConfigAlreadyFound),
        Err(_) => Ok(fs::write(config_path.as_path(), create_default_config())?),
    }
}

/// Creates a new default configuration file for the tool.
fn create_default_config() -> String {
    let mut default_change_types: BTreeMap<String, String> = BTreeMap::new();
    default_change_types.insert("Bug Fixes".into(), "bug\\s*fixes".into());
    default_change_types.insert("Features".into(), "features".into());
    default_change_types.insert("Improvements".into(), "improvements".into());

    let default_config = Config {
        categories: Vec::new(),
        change_types: default_change_types,
        expected_spellings: BTreeMap::new(),
        legacy_version: None,
        target_repo: "".to_string(),
    };

    to_string_pretty(&default_config).expect("failed to print default configuration")
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
