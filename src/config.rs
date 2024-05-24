use crate::errors::ConfigError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

/// Holds the configuration of the application
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The map of expected spellings.
    ///
    /// Note: The key is the correct spelling and the value
    /// is a string representing a RegEx pattern of possible
    /// (mis-)spellings, that should be associated with the correct
    /// version.
    pub expected_spellings: HashMap<String, String>,
}

impl Config {
    // Loads a configuration from a given raw string.
    pub fn load(contents: &str) -> Result<Config, ConfigError> {
        let config: Config = serde_json::from_str(contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config: Config = serde_json::from_str(include_str!("testdata/example_config.json"))
            .expect("failed to parse config");
        println!("{:?}", config);

        assert!(
            config.expected_spellings.len() > 0,
            "expected non-zero length of example configuration spellings"
        );
        assert_eq!(config.expected_spellings.get("API").unwrap(), "api");
    }
}
