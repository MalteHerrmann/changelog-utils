use super::change_type::{self, ChangeType};
use crate::{config::Config, errors::ReleaseError, utils::version};
use regex::RegexBuilder;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Holds the information about a given release in the changelog.
///
/// TODO: abstract common interface / traits between single and multi-line implementations.
#[derive(Clone, Debug)]
pub struct Release {
    pub change_types: Vec<ChangeType>,
    pub path: PathBuf,
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
        self.version.eq_ignore_ascii_case("unreleased")
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

// TODO: remove the config passing here?
pub fn parse(config: &Config, dir: &Path) -> Result<Release, ReleaseError> {
    let base_name = dir
        .file_name()
        .expect("failed to get base name")
        .to_str()
        .expect("failed to get base name string");

    let version = base_name.to_string();
    let mut problems: Vec<String> = Vec::new();

    if !is_unreleased(base_name)
        && !RegexBuilder::new(r#"v\d+\.\d+\.\d+(-rc\d+)?"#)
            .build()?
            .is_match(&version)
    {
        problems.push(format!("invalid version string: {version}"));
    };

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

    let change_types = fs::read_dir(dir)
        .expect("failed to read directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|p| change_type::parse(config, p.as_path()).ok())
        .collect();

    Ok(Release {
        version,
        change_types,
        path: dir.into(),
        problems,
        // TODO: add summary parsing?
        summary: None,
    })
}

// TODO: abstract to common util? Currently similar to single file implementation
fn is_unreleased(dir_name: &str) -> bool {
    RegexBuilder::new(r"unreleased\s*$")
        .case_insensitive(true)
        .build()
        .expect("failed to build regex")
        .is_match(dir_name)
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
