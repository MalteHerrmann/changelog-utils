use crate::errors::{ConfigAdjustError, ConfigError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::BTreeMap, fmt, fs, path::Path};
use url::Url;

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
    pub change_types: BTreeMap<String, String>,
    /// The map of expected spellings.
    ///
    /// Note: The key is the correct spelling and the value
    /// is a string representing a RegEx pattern of possible
    /// (mis-)spellings, that should be associated with the correct
    /// version.
    pub expected_spellings: BTreeMap<String, String>,
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
    pub fn export(&self, path: &Path) -> Result<(), ConfigError> {
        Ok(fs::write(path, format!("{}", self))?)
    }

    pub fn has_legacy_version(&self) -> bool {
        self.legacy_version.is_some()
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

// Unpacks the configuration from a given raw string.
pub fn unpack_config(contents: &str) -> Result<Config, ConfigError> {
    let config: Config = serde_json::from_str(contents)?;
    Ok(config)
}

// Tries to open the configuration file in the expected location
// and load the configuration.
pub fn load() -> Result<Config, ConfigError> {
    unpack_config(fs::read_to_string(".clconfig.json")?.as_str())
}

// Adds a category to the list of allowed categories.
pub fn add_category(config: &mut Config, value: String) -> Result<(), ConfigAdjustError> {
    if config.categories.contains(&value) {
        return Err(ConfigAdjustError::CategoryAlreadyFound);
    }

    config.categories.push(value);
    config.categories.sort_unstable();

    Ok(())
}

// Removes a category from the list of allowed categories.
pub fn remove_category(config: &mut Config, value: String) -> Result<(), ConfigAdjustError> {
    let index = match config.categories.iter().position(|x| x == &value) {
        Some(i) => i,
        None => return Err(ConfigAdjustError::NotFound),
    };
    config.categories.remove(index);

    Ok(())
}

// Adds a new key-value pair into the given collection in case the key is not
// already present.
pub fn add_into_collection(
    hm: &mut BTreeMap<String, String>,
    key: String,
    value: String,
) -> Result<(), ConfigAdjustError> {
    if let Some(_) = hm.insert(key, value) {
        return Err(ConfigAdjustError::KeyAlreadyFound);
    };

    Ok(())
}

// Removes a key from the given collection in case it is found.
pub fn remove_from_collection(
    hm: &mut BTreeMap<String, String>,
    key: String,
) -> Result<(), ConfigAdjustError> {
    match hm.remove(&key) {
        Some(_) => Ok(()),
        None => Err(ConfigAdjustError::NotFound),
    }
}

// Checks if the given value is a valid GitHub URL and sets the target
// repository field if it is the case.
pub fn set_target_repo(config: &mut Config, value: String) -> Result<(), ConfigAdjustError> {
    match Url::parse(value.as_str())?.domain() {
        Some(d) => {
            if d != "github.com" {
                return Err(ConfigAdjustError::NoGitHubRepository);
            }
        }
        None => return Err(ConfigAdjustError::NoGitHubRepository),
    }

    config.target_repo = value;
    Ok(())
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = unpack_config(include_str!("testdata/example_config.json"))
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
        let config = unpack_config(include_str!(
            "testdata/example_config_without_optionals.json"
        ))
        .expect("failed to load config without optionals");
        assert!(
            config.legacy_version.is_none(),
            "expected legacy version not to be set"
        )
    }
}

#[cfg(test)]
mod config_adjustment_tests {
    use super::*;

    fn load_example_config() -> Config {
        unpack_config(include_str!("testdata/example_config.json"))
            .expect("failed to load example config")
    }

    #[test]
    fn test_add_category_pass() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        assert!(!config.categories.contains(&"new".to_string()));
        assert!(add_category(&mut config, "new".into()).is_ok());
        assert_eq!(config.categories.len(), 3);
        assert!(config.categories.contains(&"new".to_string()));
    }

    #[test]
    fn test_add_category_duplicate() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        // TODO: get value from existing categories instead of hardcoding here
        assert_eq!(
            add_category(&mut config, "test".to_string()).unwrap_err(),
            ConfigAdjustError::CategoryAlreadyFound
        );
        assert_eq!(config.categories.len(), 2);
    }

    #[test]
    fn test_remove_category() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        // TODO: get value from existing categories instead of hardcoding here
        assert!(remove_category(&mut config, "test".to_string()).is_ok());
        assert_eq!(config.categories.len(), 1);
    }

    #[test]
    fn test_remove_category_not_found() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        // TODO: get value from existing categories instead of hardcoding here
        assert_eq!(
            remove_category(&mut config, "not-found".to_string()).unwrap_err(),
            ConfigAdjustError::NotFound
        );
        assert_eq!(config.categories.len(), 2);
    }

    #[test]
    fn test_add_into_collection() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.keys().len(), 3);
        assert!(!config.change_types.contains_key("newkey"));
        assert!(add_into_collection(
            &mut config.change_types,
            "newkey".to_string(),
            "newvalue".to_string()
        )
        .is_ok());
        assert_eq!(config.change_types.keys().len(), 4);
        assert!(config.change_types.contains_key("newkey"));
    }

    #[test]
    fn test_add_into_collection_already_present() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.keys().len(), 3);
        assert!(config.change_types.contains_key("Bug Fixes"));
        assert_eq!(
            add_into_collection(
                &mut config.change_types,
                "Bug Fixes".to_string(),
                "newvalue".to_string()
            )
            .unwrap_err(),
            ConfigAdjustError::KeyAlreadyFound
        );
        assert_eq!(config.change_types.keys().len(), 3);
    }

    #[test]
    fn test_remove_from_collection() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.keys().len(), 3);
        assert!(config.change_types.contains_key("Bug Fixes"));
        assert!(remove_from_collection(&mut config.change_types, "Bug Fixes".to_string()).is_ok());
        assert_eq!(config.change_types.keys().len(), 2);
        assert!(!config.change_types.contains_key("Bug Fixes"));
    }

    #[test]
    fn test_remove_from_collection_not_found() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.keys().len(), 3);
        assert_eq!(
            remove_from_collection(&mut config.change_types, "not found".to_string()).unwrap_err(),
            ConfigAdjustError::NotFound
        );
        assert_eq!(config.change_types.keys().len(), 3);
    }

    #[test]
    fn test_set_target_repo_fail() {
        let mut config = load_example_config();
        let new_target = "https://other-link.com/MalteHerrmann/other-repo";
        assert_eq!(
            set_target_repo(&mut config, new_target.to_string()).unwrap_err(),
            ConfigAdjustError::NoGitHubRepository
        );
        assert_ne!(config.target_repo, new_target);
    }

    #[test]
    fn test_set_target_repo_pass() {
        let mut config = load_example_config();
        let new_target = "https://github.com/MalteHerrmann/other-repo";
        assert_ne!(config.target_repo, new_target);
        assert!(set_target_repo(&mut config, new_target.to_string()).is_ok());
        assert_eq!(config.target_repo, new_target);
    }
}
