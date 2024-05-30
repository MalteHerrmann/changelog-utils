use crate::errors::ConfigError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

/// Holds the configuration of the application
///
/// TODO: check if clone is actually necessary?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The list of categories for a given entry,
    /// that can be used.
    pub categories: Vec<String>,
    /// The map of allowed change types.
    ///
    /// Note: The key is the correct spelling and the value is
    /// a regular expression matching all possible (mis-)spellings
    /// of the given category.
    pub change_types: HashMap<String, String>,
    /// The map of expected spellings.
    ///
    /// Note: The key is the correct spelling and the value
    /// is a string representing a RegEx pattern of possible
    /// (mis-)spellings, that should be associated with the correct
    /// version.
    pub expected_spellings: HashMap<String, String>,
    /// Optional Version to specify legacy entries, that
    /// don't need to adhere to the given linter standards.
    ///
    /// TODO: use Version type directly instead
    pub legacy_version: Option<String>,
    /// The target repository, that represents the base url
    /// enforced to occur in PR links.
    pub target_repo: String,
}

impl Config {
    pub fn has_legacy_version(&self) -> bool {
        self.legacy_version.is_some()
    }
}

// Loads a configuration from a given raw string.
pub fn load(contents: &str) -> Result<Config, ConfigError> {
    let config: Config = serde_json::from_str(contents)?;
    Ok(config)
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = load(include_str!("testdata/example_config.json"))
            .expect("failed to parse config");
        println!("{:?}", config);

        assert!(
            config.expected_spellings.len() > 0,
            "expected non-zero length of example configuration spellings"
        );
        assert_eq!(config.expected_spellings.get("API").unwrap(), "api");

        assert!(
            config.change_types.len() > 0,
            "expected non-zero length of change types in example config"
        );
        assert_eq!(
            config.change_types.get("Bug Fixes").unwrap(),
            "bug\\s*fixes"
        );

        assert!(
            config.categories.len() > 0,
            "expected non-zero length of categories in example config",
        );
        assert!(
            config.categories.contains(&"cli".to_string()),
            "expected cli to be in list of allowed categories"
        );

        assert!(
            config.legacy_version.is_some(),
            "expected legacy version to be found"
        )
    }

    #[test]
    fn test_load_config_no_optionals() {
        let config = load(include_str!("testdata/example_config_without_optionals.json"))
            .expect("failed to load config without optionals");
        assert!(config.legacy_version.is_none(), "expected legacy version not to be set")
    }
}
