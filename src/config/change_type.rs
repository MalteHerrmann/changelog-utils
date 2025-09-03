use serde::{Deserialize, Serialize};

/*
 * This type defines the information about a change type.
 * It consists of a short version of the long-form change type.
 *
 * Examples: short: imp; long: Improvements
 */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeTypeConfig {
    pub short: String,
    pub long: String,
}
