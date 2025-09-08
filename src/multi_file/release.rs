use super::change_type::ChangeType;
use crate::{config::Config, errors::ReleaseError, utils::version};
use regex::RegexBuilder;
use std::path::Path;

/// Holds the information about a given release in the changelog.
///
/// TODO: abstract common interface / traits between single and multi-line implementations.
#[derive(Clone, Debug)]
pub struct Release {
    pub change_types: Vec<ChangeType>,
    pub problems: Vec<String>,
    pub summary: Option<String>,
    pub version: String,
}

impl Release {
    // TODO: this should rather go into the generate function -- to get fixed contents the lower
    // level entries and changetypes should be adjusted
    //
    // TODO: rework?
    pub fn get_fixed_contents(&self) -> String {
        let mut exported_string = String::new();

        // TODO: fix
        exported_string.push_str(&format!("## {}", &self.version));
        exported_string.push('\n');

        self.change_types.iter().for_each(|change_type| {
            exported_string.push('\n');
            exported_string.push_str(change_type.get_fixed_contents().as_str());
        });

        exported_string
    }

    // TODO: implement
    pub fn add_summary(&self, summary: &str) -> Result<(), ReleaseError> {
        Ok(())
    }

    /// Returns a boolean value if the given release has the unreleased tag.
    pub fn is_unreleased(&self) -> bool {
        self.version.to_ascii_lowercase() == "unreleased"
    }

    /// Returns a boolean value whether the release version is lower than or equal to the
    /// legacy version defined in the configuration.
    ///
    /// If no legacy version is defined, it returns false.
    // TODO: can be removed?
    pub fn is_legacy(&self, config: &Config) -> Result<bool, ReleaseError> {
        if self.is_unreleased() || !config.has_legacy_version() {
            return Ok(false);
        }

        let legacy_version = version::parse(config.legacy_version.as_ref().unwrap())?;
        let parsed_version = version::parse(self.version.as_str())?;

        Ok(!parsed_version.gt(&legacy_version))
    }
}

/// Returns a new Release instance for the unreleased section without any contained blocks.
pub fn new_unreleased() -> Release {
    Release {
        version: "Unreleased".to_string(),
        change_types: Vec::new(),
        problems: Vec::new(),
        summary: None,
    }
}

pub fn new_empty_release() -> Release {
    Release {
        version: "".to_string(),
        change_types: Vec::new(),
        problems: Vec::new(),
        summary: None,
    }
}

// TODO: remove the config passing here?
pub fn parse(config: &Config, dir: &Path) -> Result<Release, ReleaseError> {
    let base_name = dir
        .file_name()
        .expect("failed to get base name")
        .to_str()
        .expect("failed to get base name string");

    let change_types: Vec<ChangeType> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    // Check unreleased pattern
    if let Some(r) = check_unreleased(base_name) {
        return Ok(r);
    }

    // TODO: check version and add to problems if invalid
    let version = dir.to_str().unwrap().to_string();

    if !RegexBuilder::new(concat!(r#"v\d+\.\d+\.\d+(-rc\d+)?"#))
        .build()?
        .is_match(&version)
    {
        problems.push(format!("invalid version string: {version}"));
    };

    // // TODO: this needs to be adjusted for sure! remove the ##, that should only go into the
    // // generated one
    // let captures = match RegexBuilder::new(concat!(
    //     r#"^\s*##\s*\[(?P<version>v\d+\.\d+\.\d+(-rc\d+)?)]"#,
    //     r#"(?P<link>\(.*\))?\s*-\s*(?P<date>\d{4}-\d{2}-\d{2})$"#,
    // ))
    // .case_insensitive(true)
    // .build()?
    // .captures(base_name)
    // {
    //     Some(c) => c,
    //     None => return Err(ReleaseError::NoMatchFound),
    // };
    //
    // let version = captures.name("version").unwrap().as_str().to_string();

    // // TODO: I guess this whole thing rather applies to the Summary.md which should contain the link etc.
    //
    // let link = match captures.name("link") {
    //     Some(c) => {
    //         let mut cleaned_link = c.as_str().to_string();
    //         // remove brackets from (link) -> link
    //         cleaned_link.remove(0);
    //         cleaned_link.pop();
    //         cleaned_link
    //     }
    //     None => "".to_string(),
    // };
    // let (_, link_problems) = check_link(config, link.as_str(), version.as_str());
    // link_problems.into_iter().for_each(|p| problems.push(p));
    //
    // // TODO: what to do with the date etc. here? That should only be part of the generated complete
    // // changelog?
    // let date = captures.name("date").unwrap().as_str();
    // let fixed = format!("## [{version}]({fixed_link}) - {date}");

    Ok(Release {
        version,
        change_types,
        problems,
        summary: None,
    })
}

// TODO: abstract to common util? Currently similar to single file implementation
fn check_unreleased(dir_name: &str) -> Option<Release> {
    if RegexBuilder::new(r"unreleased\s*$")
        .case_insensitive(true)
        .build()
        .expect("failed to build regex")
        .is_match(dir_name)
    {
        let fixed = "unreleased".to_string();
        let mut problems: Vec<String> = Vec::new();
        let change_types: Vec<ChangeType> = Vec::new(); // TODO: parse contents

        if fixed.ne(dir_name) {
            problems.push(format!(
                "Unreleased directory name is wrong; expected: '{fixed}'; got: '{dir_name}'"
            ))
        }

        return Some(Release {
            version: "Unreleased".to_string(),
            change_types,
            problems,
            summary: None, // TODO: check for existing summary
        });
    }

    None
}

// TODO: remove? or use in Summary?
fn check_link(config: &Config, link: &str, version: &str) -> (String, Vec<String>) {
    let mut problems: Vec<String> = Vec::new();

    let fixed_link = format!("{}/releases/tag/{}", &config.target_repo, version);

    if link.is_empty() {
        // NOTE: returning here because the following checks are not relevant without a link
        return (
            fixed_link,
            vec![format!("Release link is missing for version {version}")],
        );
    }

    if link != fixed_link {
        problems.push(format!("Release link should point to the GitHub release for {version}; expected: '{fixed_link}'; got: '{link}'"))
    }

    (fixed_link, problems)
}
