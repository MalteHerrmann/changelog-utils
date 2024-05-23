use crate::errors::ChangelogError;

/// Represents the changelog contents.
pub struct Changelog {
    // TODO: implement Release struct
    pub releases: Vec<String>,
}

/// Parses the given changelog contents.
pub fn parse_changelog(contents: &str) -> Result<Changelog, ChangelogError> {
    println!("parsing contents: {}", contents);
    let releases: Vec<String> = vec![];
    for line in contents.lines() {
        if line.trim().starts_with("## ") {

        }
    }
    Ok(Changelog { releases: vec![] })
}
