use crate::{config::Config, errors::ChangelogError};

/// Represents the changelog contents.
pub struct Changelog {
    // TODO: implement Release struct
    pub releases: Vec<String>,
}

/// Parses the given changelog contents.
///
/// TODO: implement this fully!
pub fn parse_changelog(config: Config, contents: &str) -> Result<Changelog, ChangelogError> {
    let releases: Vec<String> = vec![];
    for line in contents.lines() {
        if line.trim().starts_with("## ") {}
    }
    Ok(Changelog { releases: vec![] })
}
