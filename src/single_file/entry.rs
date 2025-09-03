use crate::{common, config, errors::EntryError};
use regex::Regex;

/// Represents an individual entry in the changelog.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The category of the entry.
    pub category: String,
    /// The fixed line adhering to all standards.
    pub fixed: String,
    /// The PR number for the given change.
    pub pr_number: u64,
    /// The list of problems with the given line.
    pub problems: Vec<String>,
}

impl Entry {
    // TODO: remove? or apply everywhere?
    pub fn new(
        config: &config::Config,
        category: &str,
        description: &str,
        pr_number: u64,
    ) -> Entry {
        let link = format!("{}/pull/{}", config.target_repo, pr_number);
        let fixed = build_fixed(category, &link, description, pr_number);

        Entry {
            category: category.to_string(),
            fixed,
            pr_number,
            problems: Vec::new(),
        }
    }
}

pub fn parse(config: &config::Config, line: &str) -> Result<Entry, EntryError> {
    let entry_pattern = Regex::new(concat!(
        r"^(?P<ws0>\s*)-(?P<ws1>\s*)\((?P<category>[a-zA-Z0-9\-]+)\)",
        r"(?P<ws2>\s*)\[(?P<bs>\\)?#(?P<pr>\d+)]",
        r"(?P<ws3>\s*)\((?P<link>[^)]*)\)(?P<ws4>\s*)(?P<desc>.+)$"
    ))
    .expect("invalid regex pattern");

    let matches = match entry_pattern.captures(line) {
        Some(c) => c,
        None => return Err(EntryError::InvalidEntry(line.to_string())),
    };

    // NOTE: calling unwrap here is okay because we checked that the pattern matched above
    let category = matches.name("category").unwrap().as_str();
    let description = matches.name("desc").unwrap().as_str();
    let link = matches.name("link").unwrap().as_str();
    let pr_number = matches.name("pr").unwrap().as_str().parse::<u64>().unwrap();
    let spaces = [
        matches.name("ws0").unwrap().as_str(),
        matches.name("ws1").unwrap().as_str(),
        matches.name("ws2").unwrap().as_str(),
        matches.name("ws3").unwrap().as_str(),
        matches.name("ws4").unwrap().as_str(),
    ];

    let mut problems: Vec<String> = Vec::new();

    check_whitespace(spaces)
        .into_iter()
        .for_each(|p| problems.push(p));

    let (fixed_category, category_problems) = common::entry::check_category(config, category);
    category_problems.into_iter().for_each(|p| problems.push(p));

    let (fixed_link, link_problems) = common::entry::check_link(config, link, pr_number);
    link_problems.into_iter().for_each(|p| problems.push(p));

    let (fixed_desc, desc_problems) = common::entry::check_description(config, description);
    desc_problems.into_iter().for_each(|p| problems.push(p));

    let fixed = build_fixed(&fixed_category, &fixed_link, &fixed_desc, pr_number);

    Ok(Entry {
        category: fixed_category.to_string(),
        fixed,
        pr_number,
        problems,
    })
}

/// Returns the fixed entry string based on the given building parts.
fn build_fixed(cat: &str, link: &str, desc: &str, pr: u64) -> String {
    format!("- ({}) [#{}]({}) {}", cat, pr, link, desc,)
}

/// Checks the used whitespace in the entry.
fn check_whitespace(spaces: [&str; 5]) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();

    let expected_whitespace = ["", " ", " ", "", " "];
    let errors = [
        "There should be no leading whitespace before the dash",
        "There should be exactly one space between the leading dash and the category",
        "There should be exactly one space between the category and the PR link",
        "There should be no whitespace inside of the markdown link",
        "There should be exactly one space between the PR link and the description",
    ];

    spaces
        .into_iter()
        .zip(expected_whitespace)
        .zip(errors)
        .for_each(|((got, expected), error)| {
            if (*got).ne(expected) {
                problems.push(error.to_string())
            }
        });

    problems
}

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::unpack_config(include_str!("../testdata/example_config.json"))
        .expect("failed to load example config")
}

#[cfg(test)]
mod entry_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example = concat!(
            "- (cli) [#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) ",
            "Add initial Python implementation."
        );
        let entry_res = parse(&load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.fixed, example); // NOTE: since line is okay there are no changes to it in the fixed version
        assert_eq!(entry.pr_number, 1);
        assert!(entry.problems.is_empty());
    }

    #[test]
    fn test_fail_has_backslash_in_link() {
        let example =
            r"- (cli) [\#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) Test.";
        let entry_res = parse(&load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.fixed, example.replace(r"\", ""));
        assert_eq!(entry.pr_number, 1);
        assert_eq!(entry.problems.len(), 1);
        assert_eq!(
            entry.problems[0],
            "There should be no backslash in front of the # in the PR link"
        );
    }

    #[test]
    fn test_fail_wrong_pr_link_and_missing_dot() {
        let example = r"- (cli) [#2](https://github.com/MalteHerrmann/changelog-utils/pull/1) Test";
        let fixed = r"- (cli) [#2](https://github.com/MalteHerrmann/changelog-utils/pull/2) Test.";
        let entry_res = parse(&load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.fixed, fixed);
        assert_eq!(entry.pr_number, 2);
        assert_eq!(entry.problems.len(), 2);
        assert_eq!(
            entry.problems,
            vec![
                concat!(
                    r"PR link is not matching PR number 2: ",
                    "'https://github.com/MalteHerrmann/changelog-utils/pull/1'"
                ),
                "PR description should end with a dot: 'Test'"
            ]
        );
    }

    #[test]
    fn test_malformed_entry() {
        let example = r"- (cli) [#13tps://github.com/Ma/2";
        assert!(parse(&load_test_config(), example).is_err());
    }

    #[test]
    fn test_fail_wrong_whitespace() {
        let example =
            r"- (cli)   [#1] (https://github.com/MalteHerrmann/changelog-utils/pull/1) Run test.";
        let expected =
            r"- (cli) [#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) Run test.";
        let entry_res = parse(&load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.fixed, expected);
        assert_eq!(entry.pr_number, 1);
        assert_eq!(entry.problems.len(), 2);
        assert_eq!(
            entry.problems,
            [
                "There should be exactly one space between the category and the PR link",
                "There should be no whitespace inside of the markdown link",
            ]
        );
    }
}

#[cfg(test)]
mod whitespace_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example_spaces = ["", " ", " ", "", " "];
        assert!(check_whitespace(example_spaces).is_empty());
    }

    #[test]
    fn test_fail_leading_space() {
        let example_spaces = [" ", " ", " ", "", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            ["There should be no leading whitespace before the dash"]
        );
    }

    #[test]
    fn test_fail_space_between_category_and_link() {
        let example_spaces = ["", " ", "", "", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            ["There should be exactly one space between the category and the PR link"]
        );
    }

    #[test]
    fn test_fail_multiple_spaces() {
        let example_spaces = ["", "", " ", "", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            ["There should be exactly one space between the leading dash and the category"]
        );
    }

    #[test]
    fn test_fail_multiple_spaces_before_description() {
        let example_spaces = ["", " ", " ", "", "  "];
        assert_eq!(
            check_whitespace(example_spaces),
            ["There should be exactly one space between the PR link and the description"]
        );
    }

    #[test]
    fn test_fail_space_in_link() {
        let example_spaces = ["", " ", " ", " ", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            ["There should be no whitespace inside of the markdown link"]
        );
    }
}
