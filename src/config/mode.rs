use serde::{Deserialize, Serialize};
use std::fmt;

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
