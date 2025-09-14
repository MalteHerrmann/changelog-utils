use super::{change_type::ChangeTypeConfig, mode::Mode};
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
    /// In multi mode, this defines the directory where entries are created.
    pub changelog_dir: Option<String>,
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
    /// Controls whethere a single or multi file changelog is used.
    pub mode: Mode,
    /// The target repository, that represents the base url
    /// enforced to occur in PR links.
    pub target_repo: String,
    /// Sets whether categories are enforced in entries or are left out.
    pub use_categories: bool,
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

    pub fn add_category(&mut self, value: String) -> Result<(), ConfigAdjustError> {
        if self.categories.contains(&value) {
            return Err(ConfigAdjustError::CategoryAlreadyFound);
        }

        self.categories.push(value);
        self.categories.sort_unstable();

        Ok(())
    }

    pub fn remove_category(&mut self, value: String) -> Result<(), ConfigAdjustError> {
        let i = match self.categories.iter().position(|cat| cat.eq(&value)) {
            Some(i) => i,
            None => return Err(ConfigAdjustError::NotFound),
        };

        self.categories.remove(i);
        Ok(())
    }

    pub fn add_change_type(
        &mut self,
        long: String,
        short: String,
    ) -> Result<(), ConfigAdjustError> {
        if self.get_long_change_type(&long).is_some() {
            return Err(ConfigAdjustError::DuplicateChangeType(long));
        }

        if self.get_short_change_type(&short).is_some() {
            return Err(ConfigAdjustError::DuplicateChangeType(short));
        }

        self.change_types.push(ChangeTypeConfig { short, long });
        Ok(())
    }

    pub fn remove_change_type(&mut self, short: String) -> Result<(), ConfigAdjustError> {
        let i = match self.change_types.iter().position(|ct| ct.short.eq(&short)) {
            Some(i) => i,
            None => return Err(ConfigAdjustError::NotFound),
        };

        self.change_types.remove(i);
        Ok(())
    }

    pub fn add_expected_spelling(
        &mut self,
        key: String,
        value: String,
    ) -> Result<(), ConfigAdjustError> {
        if self.expected_spellings.contains_key(&key) {
            return Err(ConfigAdjustError::KeyAlreadyFound);
        };

        self.expected_spellings.insert(key, value);
        Ok(())
    }

    pub fn remove_expected_spelling(&mut self, key: String) -> Result<(), ConfigAdjustError> {
        match self.expected_spellings.remove(&key) {
            Some(_) => Ok(()),
            None => Err(ConfigAdjustError::NotFound),
        }
    }

    pub fn set_changelog_dir(&mut self, value: Option<String>) {
        self.changelog_dir = value;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn set_use_categories(&mut self, value: bool) {
        self.use_categories = value;
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
            // TODO: add config method to set this.
            changelog_dir: None,
            expected_spellings: BTreeMap::default(),
            legacy_version: None,
            // TODO: add config method to set this.
            mode: Mode::Single,
            target_repo: String::default(),
            // TODO: add config method to set this.
            use_categories: true,
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
        let config = unpack_config(include_str!("../testdata/example_config.json"))
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
            "../testdata/example_config_without_optionals.json"
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
        unpack_config(include_str!("../testdata/example_config.json"))
            .expect("failed to load example config")
    }

    #[test]
    fn test_add_category_pass() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        assert!(!config.categories.contains(&"new".to_string()));
        assert!(config.add_category("new".into()).is_ok());
        assert_eq!(config.categories.len(), 3);
        assert!(config.categories.contains(&"new".to_string()));
    }

    #[test]
    fn test_add_category_duplicate() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        assert_eq!(
            config.add_category("test".to_string()).unwrap_err(),
            ConfigAdjustError::CategoryAlreadyFound
        );
        assert_eq!(config.categories.len(), 2);
    }

    #[test]
    fn test_remove_category() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        assert!(config.remove_category("test".to_string()).is_ok());
        assert_eq!(config.categories.len(), 1);
    }

    #[test]
    fn test_remove_category_not_found() {
        let mut config = load_example_config();
        assert_eq!(config.categories.len(), 2);
        assert_eq!(
            config.remove_category("not-found".to_string()).unwrap_err(),
            ConfigAdjustError::NotFound
        );
        assert_eq!(config.categories.len(), 2);
    }

    #[test]
    fn test_add_change_type() {
        let mut config = load_example_config();
        assert_eq!(config.change_types.len(), 3);
        assert!(config
            .add_change_type("LONG CHANGE TYPE".into(), "SHORT".into())
            .is_ok());
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
        assert!(config
            .add_change_type("Bug Fixes".into(), "fix".into())
            .is_err());
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

        assert!(config.remove_change_type("fix".into()).is_ok());
        assert_eq!(config.change_types.len(), 2);
        assert!(config.get_long_change_type("Bug Fixes").is_none());

        assert!(config.remove_change_type("abcde".into()).is_err());
        assert_eq!(config.change_types.len(), 2);
    }

    #[test]
    fn test_add_into_collection() {
        let mut config = load_example_config();
        assert_eq!(config.expected_spellings.keys().len(), 3);
        assert!(!config.expected_spellings.contains_key("newkey"));
        assert!(config
            .add_expected_spelling("newkey".to_string(), "newvalue".to_string())
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
            config
                .add_expected_spelling("API".to_string(), "newvalue".to_string())
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
        assert!(config.remove_expected_spelling("API".to_string()).is_ok());
        assert_eq!(config.expected_spellings.keys().len(), 2);
        assert!(!config.expected_spellings.contains_key("API"));
    }

    #[test]
    fn test_remove_from_collection_not_found() {
        let mut config = load_example_config();
        assert_eq!(config.expected_spellings.keys().len(), 3);
        assert_eq!(
            config
                .remove_expected_spelling("not found".to_string())
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
