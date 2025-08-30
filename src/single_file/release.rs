use super::change_type::ChangeType;
use crate::{
    errors::ReleaseError,
    utils::{config, version},
};
use regex::RegexBuilder;

/// Holds the information about a release section in the changelog.
#[derive(Clone, Debug)]
pub struct Release {
    pub line: String,
    pub fixed: String,
    pub version: String,
    pub change_types: Vec<ChangeType>,
    pub problems: Vec<String>,
}

impl Release {
    pub fn get_fixed_contents(&self) -> String {
        let mut exported_string = String::new();

        exported_string.push_str(&self.fixed);
        exported_string.push('\n');

        self.change_types.iter().for_each(|change_type| {
            exported_string.push('\n');
            exported_string.push_str(change_type.get_fixed_contents().as_str());
        });

        exported_string
    }

    /// Returns a boolean value if the given release has the unreleased tag.
    pub fn is_unreleased(&self) -> bool {
        self.version == "Unreleased"
    }

    /// Returns a boolean value whether the release version is lower than or equal to the
    /// legacy version defined in the configuration.
    ///
    /// If no legacy version is defined, it returns false.
    pub fn is_legacy(&self, config: &config::Config) -> Result<bool, ReleaseError> {
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
        line: "## Unreleased".to_string(),
        fixed: "## Unreleased".to_string(),
        version: "Unreleased".to_string(),
        change_types: Vec::new(),
        problems: Vec::new(),
    }
}

pub fn new_empty_release() -> Release {
    Release {
        line: "".to_string(),
        fixed: "".to_string(),
        version: "".to_string(),
        change_types: Vec::new(),
        problems: Vec::new(),
    }
}

/// Parses the contents of a release line in the changelog.
pub fn parse(config: &config::Config, line: &str) -> Result<Release, ReleaseError> {
    let change_types: Vec<ChangeType> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    // Check unreleased pattern
    if let Some(r) = check_unreleased(line) {
        return Ok(r);
    }

    let captures = match RegexBuilder::new(concat!(
        r#"^\s*##\s*\[(?P<version>v\d+\.\d+\.\d+(-rc\d+)?)]"#,
        r#"(?P<link>\(.*\))?\s*-\s*(?P<date>\d{4}-\d{2}-\d{2})$"#,
    ))
    .case_insensitive(true)
    .build()?
    .captures(line)
    {
        Some(c) => c,
        None => return Err(ReleaseError::NoMatchFound),
    };

    let version = captures.name("version").unwrap().as_str().to_string();

    let link = match captures.name("link") {
        Some(c) => {
            let mut cleaned_link = c.as_str().to_string();
            // remove brackets from (link) -> link
            cleaned_link.remove(0);
            cleaned_link.pop();
            cleaned_link
        }
        None => "".to_string(),
    };
    let (fixed_link, link_problems) = check_link(config, link.as_str(), version.as_str());
    link_problems.into_iter().for_each(|p| problems.push(p));

    let date = captures.name("date").unwrap().as_str();
    let fixed = format!("## [{version}]({fixed_link}) - {date}");

    Ok(Release {
        line: line.to_string(),
        fixed,
        version,
        change_types,
        problems,
    })
}

fn check_unreleased(line: &str) -> Option<Release> {
    if RegexBuilder::new(r"\s*##\s*unreleased\s*$")
        .case_insensitive(true)
        .build()
        .expect("failed to build regex")
        .is_match(line)
    {
        let fixed = "## Unreleased".to_string();
        let mut problems: Vec<String> = Vec::new();
        let change_types: Vec<ChangeType> = Vec::new();

        if fixed.ne(line) {
            problems.push(format!(
                "Unreleased header is malformed; expected: '{fixed}'; got: '{line}'"
            ))
        }

        return Some(Release {
            line: line.to_string(),
            fixed,
            version: "Unreleased".to_string(),
            change_types,
            problems,
        });
    }

    None
}

#[cfg(test)]
mod release_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example = "## [v0.1.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.1.0) - 2024-04-27";
        let release = parse(&load_test_config(), example).expect("failed to parse release");
        assert_eq!(release.fixed, example);
        assert_eq!(release.version, "v0.1.0");
        assert!(release.problems.is_empty());
    }

    #[test]
    fn test_pass_unreleased() {
        let example = "## Unreleased";
        let release = parse(&load_test_config(), example).expect("failed to parse release");
        assert_eq!(release.fixed, example);
        assert_eq!(release.version, "Unreleased");
        assert!(release.problems.is_empty());
    }

    #[test]
    fn test_unreleased_too_much_whitespace() {
        let example = " ##  Unreleased";
        let fixed = "## Unreleased";
        let release = parse(&load_test_config(), example).expect("failed to parse release");
        assert_eq!(release.fixed, fixed);
        assert_eq!(release.version, "Unreleased");
        assert_eq!(
            release.problems,
            vec![format!(
                "Unreleased header is malformed; expected: '{fixed}'; got: '{example}'"
            )]
        );
    }

    #[test]
    fn test_fail_malformed() {
        let example = "## invalid entry";
        let err = parse(&load_test_config(), example).expect_err("expected parsing to fail");
        assert_eq!(err, ReleaseError::NoMatchFound);
    }

    #[test]
    fn test_missing_link() {
        let example = "## [v0.1.0] - 2024-04-27";
        let release = parse(&load_test_config(), example).expect("failed to parse release");
        assert_eq!(release.version, "v0.1.0");
        assert_eq!(
            release.problems,
            vec!["Release link is missing for version v0.1.0"]
        );
    }

    #[test]
    fn test_wrong_link() {
        let example = "## [v0.1.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.2.0) - 2024-04-27";
        let fixed = example.replace("0.2.0", "0.1.0");
        let release = parse(&load_test_config(), example).expect("failed to parse release");
        assert_eq!(release.version, "v0.1.0");
        assert_eq!(release.fixed, fixed);
        assert_eq!(release.problems,
            vec![concat!(
                "Release link should point to the GitHub release for v0.1.0; ",
                "expected: 'https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.1.0'; ",
                "got: 'https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.2.0'"
            )]
        );
    }
}

fn check_link(config: &config::Config, link: &str, version: &str) -> (String, Vec<String>) {
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

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::unpack_config(include_str!("../testdata/example_config.json"))
        .expect("failed to load example config")
}

#[cfg(test)]
mod link_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example = "https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.1.0";
        let (fixed, problems) = check_link(&load_test_config(), example, "v0.1.0");
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_no_link() {
        let (fixed, problems) = check_link(&load_test_config(), "", "v0.1.0");
        assert_eq!(
            fixed,
            "https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.1.0"
        );
        assert_eq!(problems, vec!["Release link is missing for version v0.1.0"]);
    }

    #[test]
    fn test_wrong_base_url() {
        let example = "https://github.com/MalteHerrmann/changelg-utils/releases/tag/v0.1.0";
        let (fixed, problems) = check_link(&load_test_config(), example, "v0.1.0");
        assert_eq!(fixed, example.replace("changelg", "changelog"));
        assert_eq!(problems, vec![
            format!("Release link should point to the GitHub release for v0.1.0; expected: '{fixed}'; got: '{example}'")
        ]);
    }

    #[test]
    fn test_wrong_version() {
        let example = "https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.2.0";
        let (fixed, problems) = check_link(&load_test_config(), example, "v0.1.0");
        assert_eq!(fixed, example.replace("2", "1"));
        assert_eq!(problems, vec![
            format!("Release link should point to the GitHub release for v0.1.0; expected: '{fixed}'; got: '{example}'")
        ]);
    }

    #[test]
    fn test_link_is_correct_version_and_base_url_but_more_elements() {
        let example =
            "https://github.com/MalteHerrmann/changelog-utils/releases/tag/otherElement/v0.1.0";
        let (fixed, problems) = check_link(&load_test_config(), example, "v0.1.0");
        assert_eq!(fixed, example.replace("otherElement/", ""));
        assert_eq!(problems, vec![
            format!("Release link should point to the GitHub release for v0.1.0; expected: '{fixed}'; got: '{example}'")
        ]);
    }
}
