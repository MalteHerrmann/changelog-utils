use crate::errors::{ConfigAdjustError, ConfigError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::BTreeMap, fmt, fs, path::Path};
use url::Url;

/// Holds the configuration of the application
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The list of categories for a given entry,
    /// that can be used.
    pub categories: Vec<String>,
    /// The list of allowed change types.
    pub change_types: Vec<ChangeTypeConfig>,
    /// The default commit message to be used when committing
    /// the new changelog entry.
    pub commit_message: String,
    /// The relative path of the changelog file.
    pub changelog_path: String,
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

    pub fn get_long_change_type(&self, long: &str) -> Option<ChangeTypeConfig> {
        self.change_types
            .iter()
            .find(|&ct| ct.long.eq(long))
            .cloned()
    }

    pub fn get_short_change_type(&self, short: &str) -> Option<ChangeTypeConfig> {
        self.change_types
            .iter()
            .find(|&ct| ct.short.eq(short))
            .cloned()
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

impl Default for Config {
    fn default() -> Config {
        let default_change_types = vec![
            ChangeTypeConfig {
                short: "feat".into(),
                long: "Features".into(),
            },
            ChangeTypeConfig {
                short: "imp".into(),
                long: "Improvements".into(),
            },
            ChangeTypeConfig {
                short: "fix".into(),
                long: "Bug Fixes".into(),
            },
        ];

        let commit_message = "add changelog entry".to_string();
        let changelog_path = "CHANGELOG.md".to_string();

        Config {
            categories: Vec::default(),
            change_types: default_change_types,
            commit_message,
            changelog_path,
            expected_spellings: BTreeMap::default(),
            legacy_version: None,
            target_repo: String::default(),
        }
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

pub fn add_change_type(
    config: &mut Config,
    long: &str,
    short: &str,
) -> Result<(), ConfigAdjustError> {
    if config.get_long_change_type(long).is_some() {
        return Err(ConfigAdjustError::DuplicateChangeType(long.into()));
    };

    config.change_types.push(ChangeTypeConfig {
        short: short.into(),
        long: long.into(),
    });
    Ok(())
}

pub fn remove_change_type(config: &mut Config, short: &str) -> Result<(), ConfigAdjustError> {
    let i = match config.change_types.iter().position(|ct| ct.short.eq(short)) {
        Some(i) => i,
        None => return Err(ConfigAdjustError::NotFound),
    };

    config.change_types.remove(i);
    Ok(())
}

// This type defines the information about a change type.
// This consists of a short version of the long-form change type.
//
// Examples: short: imp; long: Improvements
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeTypeConfig {
    pub short: String,
    pub long: String,
}

// Adds a new key-value pair into the given collection in case the key is not
// already present.
pub fn add_into_collection(
    hm: &mut BTreeMap<String, String>,
    key: String,
    value: String,
) -> Result<(), ConfigAdjustError> {
    if hm.insert(key, value).is_some() {
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
            config.get_long_change_type("Bug Fixes").unwrap(),
            ChangeTypeConfig {
                short: "fix".into(),
                long: "Bug Fixes".into()
            }
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
        assert!(remove_category(&mut config, "test".to_string()).is_ok());
        assert_eq!(config.categories.len(), 1);
    }

    #[test]
    fn test_remove_category_not_found() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        assert_eq!(
            remove_category(&mut config, "not-found".to_string()).unwrap_err(),
            ConfigAdjustError::NotFound
        );
        assert_eq!(config.categories.len(), 2);
    }

    #[test]
    fn test_add_change_type() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.len(), 3);
        assert!(add_change_type(&mut config, "LONG CHANGE TYPE", "SHORT").is_ok());
        assert_eq!(config.change_types.len(), 4);
        assert_eq!(
            config.change_types[3],
            ChangeTypeConfig {
                short: "SHORT".into(),
                long: "LONG CHANGE TYPE".into()
            }
        );
    }

    #[test]
    fn test_add_change_type_duplicate() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.len(), 3);
        assert!(add_change_type(&mut config, "Bug Fixes", "fix").is_err());
        assert_eq!(config.change_types.len(), 3);
    }

    #[test]
    fn test_get_short_change_type() {
        let config = load_example_config();
        assert!(config.get_short_change_type("fix").is_some());
        assert!(config.get_short_change_type("abcde").is_none());
    }

    #[test]
    fn test_get_long_change_type() {
        let config = load_example_config();
        assert!(config.get_long_change_type("Bug Fixes").is_some());
        assert!(config.get_long_change_type("non-existente").is_none());
    }

    #[test]
    fn test_remove_change_type() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.len(), 3);
        assert!(config.get_long_change_type("Bug Fixes").is_some());

        assert!(remove_change_type(&mut config, "fix").is_ok());
        assert_eq!(config.change_types.len(), 2);
        assert!(config.get_long_change_type("Bug Fixes").is_none());

        assert!(remove_change_type(&mut config, "abcde").is_err());
        assert_eq!(config.change_types.len(), 2);
    }

    #[test]
    fn test_add_into_collection() {
        let mut config = load_example_config();
        assert_eq!(config.expected_spellings.keys().len(), 3);
        assert!(!config.expected_spellings.contains_key("newkey"));
        assert!(add_into_collection(
            &mut config.expected_spellings,
            "newkey".to_string(),
            "newvalue".to_string()
        )
        .is_ok());
        assert_eq!(config.expected_spellings.keys().len(), 4);
        assert!(config.expected_spellings.contains_key("newkey"));
    }

    #[test]
    fn test_add_into_collection_already_present() {
        let mut config = load_example_config();
        assert_eq!(config.expected_spellings.keys().len(), 3);
        assert!(config.expected_spellings.contains_key("API"));
        assert_eq!(
            add_into_collection(
                &mut config.expected_spellings,
                "API".to_string(),
                "newvalue".to_string()
            )
            .unwrap_err(),
            ConfigAdjustError::KeyAlreadyFound
        );
        assert_eq!(config.expected_spellings.keys().len(), 3);
    }

    #[test]
    fn test_remove_from_collection() {
        let mut config = load_example_config();
        assert_eq!(config.expected_spellings.keys().len(), 3);
        assert!(config.expected_spellings.contains_key("API"));
        assert!(remove_from_collection(&mut config.expected_spellings, "API".to_string()).is_ok());
        assert_eq!(config.expected_spellings.keys().len(), 2);
        assert!(!config.expected_spellings.contains_key("API"));
    }

    #[test]
    fn test_remove_from_collection_not_found() {
        let mut config = load_example_config();
        assert_eq!(config.expected_spellings.keys().len(), 3);
        assert_eq!(
            remove_from_collection(&mut config.expected_spellings, "not found".to_string())
                .unwrap_err(),
            ConfigAdjustError::NotFound
        );
        assert_eq!(config.expected_spellings.keys().len(), 3);
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
