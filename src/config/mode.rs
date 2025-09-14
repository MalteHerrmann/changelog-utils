use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/*
 * The ChangelogType sets whether a single or multi file approach is
 * used for the changelog generation.
 */
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // this allows lowercase parsing
pub enum Mode {
    Single,
    Multi,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ct = match &self {
            Self::Single => "single",
            Self::Multi => "multi",
        };
        write!(f, "{}", ct)
    }
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "single" => Ok(Mode::Single),
            "multi" => Ok(Mode::Multi),
            _ => Err(format!("Invalid mode: {}. Expected 'single' or 'multi'", s)),
        }
    }
}
