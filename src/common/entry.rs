use crate::{config, errors::MatchError};
use regex::{Error, Regex, RegexBuilder};

/// Check if the category is valid and return a fixed version that addresses
/// well-known problems.
pub fn check_category(config: &config::Config, category: &str) -> (String, Vec<String>) {
    let mut problems: Vec<String> = Vec::new();
    let fixed = category.to_lowercase();
    if category.to_lowercase() != category {
        problems.push(format!("category should be lowercase: ({})", category));
    }

    if !config.categories.contains(&fixed) {
        problems.push(format!("invalid change category: ({})", category));
    }

    (fixed, problems)
}

/// Check if the link is valid
pub fn check_link(config: &config::Config, link: &str, pr_number: u64) -> (String, Vec<String>) {
    let mut problems: Vec<String> = Vec::new();

    let fixed = format!("{}/pull/{}", config.target_repo, pr_number);

    if !link.starts_with(&config.target_repo) {
        problems.push(format!("PR link points to wrong repository: {}", link))
    }

    let split_link: Vec<&str> = link.split('/').collect();
    let contained_pr_number = split_link
        .last()
        .expect("this should never be empty")
        .parse::<u64>()
        .expect("this should always be a u64");

    if contained_pr_number != pr_number {
        problems.push(format!(
            "PR link is not matching PR number {}: '{}'",
            pr_number, link
        ));
    }

    (fixed, problems)
}

pub fn check_description(config: &config::Config, desc: &str) -> (String, Vec<String>) {
    let mut fixed = desc.to_string();
    let mut problems: Vec<String> = Vec::new();

    let first_letter = desc.chars().next().expect("no character in description");
    if first_letter.is_alphabetic() && !first_letter.is_uppercase() {
        fixed = first_letter.to_ascii_uppercase().to_string() + desc.to_owned()[1..].as_ref();
        problems.push(format!(
            "PR description should start with capital letter: '{}'",
            desc
        ))
    }

    let last_letter = fixed
        .chars()
        .last()
        .expect("no characters found in description");
    if last_letter.to_string() != '.'.to_string() {
        fixed = fixed.to_string() + ".";
        problems.push(format!("PR description should end with a dot: '{}'", desc))
    }

    let (fixed, spelling_problems) = check_spelling(config, &fixed);
    spelling_problems.into_iter().for_each(|p| problems.push(p));

    (fixed, problems)
}

/// Checks the spelling of entries according to the given configuration.
fn check_spelling(config: &config::Config, text: &str) -> (String, Vec<String>) {
    let mut fixed = text.to_string();
    let mut problems: Vec<String> = Vec::new();

    for (correct_spelling, pattern) in config.expected_spellings.iter() {
        match get_spelling_match(pattern, text) {
            Ok(m) => {
                if m.eq(correct_spelling) {
                    continue;
                };

                fixed = compile_regex(pattern)
                    .unwrap_or_else(|_| {
                        panic!(
                            "failed to compile regex for '{}'; check spelling configuration",
                            pattern
                        )
                    })
                    .replace(&fixed, correct_spelling)
                    .to_string();

                problems.push(format!(
                    "'{correct_spelling}' should be used instead of '{m}'",
                ))
            }
            Err(_) => continue,
        }
    }

    (fixed, problems)
}

/// Compiles the regular expression pattern with the common settings
/// used in this crate.
fn compile_regex(pattern: &str) -> Result<Regex, Error> {
    RegexBuilder::new(pattern).case_insensitive(true).build()
}

/// Returns the first match of the given pattern in the text.
/// Matching patterns inside of code blocks, links or within another word are ignored.
fn get_spelling_match(pattern: &str, text: &str) -> Result<String, MatchError> {
    // Check if pattern is inside a code block
    if RegexBuilder::new(format!(r"`[^`]*({pattern})[^`]*`").as_str())
        .case_insensitive(true)
        .build()?
        .find(text)
        .is_some()
    {
        return Err(MatchError::MatchInCodeblock);
    }

    // Check isolated words (i.e. pattern is not included in another word)
    match RegexBuilder::new(format!(r"(^|\s)({pattern})($|[\s.])").as_str())
        .case_insensitive(true)
        .build()?
        .captures(text)
    {
        Some(m) => match m.get(2) {
            Some(m) => Ok(m.as_str().to_string()),
            None => Err(MatchError::NoMatchFound),
        },
        None => Err(MatchError::NoMatchFound),
    }
}

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::unpack_config(include_str!("../testdata/example_config.json"))
        .expect("failed to load example config")
}

#[cfg(test)]
mod category_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let (fixed, problems) = check_category(&load_test_config(), "cli");
        assert_eq!(fixed, "cli");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_fail_invalid_category() {
        let (fixed, problems) = check_category(&load_test_config(), "invalid");
        assert_eq!(fixed, "invalid");
        assert_eq!(problems, ["invalid change category: (invalid)"]);
    }

    #[test]
    fn test_fail_non_lower_category() {
        let (fixed, problems) = check_category(&load_test_config(), "cLi");
        assert_eq!(fixed, "cli");
        assert_eq!(problems, ["category should be lowercase: (cLi)"]);
    }
}

#[cfg(test)]
mod link_tests {
    use super::*;

    fn load_test_config() -> config::Config {
        config::unpack_config(include_str!("../testdata/example_config.json"))
            .expect("failed to load example config")
    }

    #[test]
    fn test_pass() {
        let example = r"https://github.com/MalteHerrmann/changelog-utils/pull/1";
        let (fixed, problems) = check_link(&load_test_config(), example, 1);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_wrong_base_url() {
        let example = r"https://github.com/MalteHerrmann/changelg-utils/pull/1";
        let (fixed, problems) = check_link(&load_test_config(), example, 1);
        assert_eq!(fixed, example.replace("changelg", "changelog"));
        assert_eq!(
            problems,
            vec![format!("PR link points to wrong repository: {}", example)]
        );
    }

    #[test]
    fn test_wrong_pr_number() {
        let example = r"https://github.com/MalteHerrmann/changelog-utils/pull/2";
        let (fixed, problems) = check_link(&load_test_config(), example, 1);
        assert_eq!(fixed, example.replace("2", "1"));
        assert_eq!(
            problems,
            vec![format!(
                "PR link is not matching PR number {}: '{}'",
                1, example
            )]
        );
    }
}

#[cfg(test)]
mod description_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example = "Add Python implementation.";
        let (fixed, problems) = check_description(&load_test_config(), example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_pass_start_with_codeblock_instead_of_capital_letter() {
        let example = "`add` method implemented.";
        let (fixed, problems) = check_description(&load_test_config(), example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty(), "expected no problems: {:?}", problems);
    }

    #[test]
    fn test_fail_start_with_lowercase() {
        let example = "add Python implementation.";
        let (fixed, problems) = check_description(&load_test_config(), example);
        assert_eq!(fixed, "Add Python implementation.");
        assert_eq!(
            problems,
            vec![format!(
                "PR description should start with capital letter: '{}'",
                example
            )]
        );
    }

    #[test]
    fn test_fail_does_not_end_with_dot() {
        let example = "Add Python implementation";
        let (fixed, problems) = check_description(&load_test_config(), example);
        assert_eq!(fixed, example.to_string() + ".");
        assert_eq!(
            problems,
            vec![format!(
                "PR description should end with a dot: '{}'",
                example
            )]
        );
    }
}

#[cfg(test)]
/// Creates an empty config to be filled in test setups.
fn empty_config() -> config::Config {
    use std::collections::BTreeMap;

    config::Config {
        categories: vec![],
        change_types: vec![],
        commit_message: "".into(),
        changelog_path: "".into(),
        changelog_dir: None,
        expected_spellings: BTreeMap::new(),
        legacy_version: None,
        mode: config::Mode::Single,
        target_repo: "".into(),
        use_categories: false,
    }
}

#[cfg(test)]
mod spelling_tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_pass() {
        let mut test_config = empty_config();

        let exp_spellings = BTreeMap::from([("API".to_string(), "api".to_string())]);
        test_config.expected_spellings = exp_spellings;

        let example = "Fix API.";
        let (fixed, problems) = check_spelling(&test_config, example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_wrong_spelling() {
        let example = "Fix web--SdK.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, "Fix Web-SDK.");
        assert_eq!(problems, ["'Web-SDK' should be used instead of 'web--SdK'"])
    }

    #[test]
    fn test_multiple_problems() {
        let example = "Fix aPi and ClI.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, "Fix API and CLI.");
        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0], "'API' should be used instead of 'aPi'");
        assert_eq!(problems[1], "'CLI' should be used instead of 'ClI'");
    }

    #[test]
    fn test_pass_codeblocks() {
        let example = "Fix `ApI in codeblocks`.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_pass_nested_word() {
        let example = "FixApI in another word.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    fn load_multi_file_config() -> config::Config {
        config::unpack_config(include_str!(
            "../../tests/testdata/multi_file/fail/.clconfig.json"
        ))
        .expect("failed to load multi file config")
    }

    #[test]
    fn test_fail_usdn() {
        let example = "- Integrate our custom Dollar module, that enables the issuance of Noble's stablecoin $UsDN. ([#448](https://github.com/noble-assets/noble/pull/448))";
        let (_, problems) = check_spelling(&load_multi_file_config(), example);
        assert_eq!(problems, vec!["'$USDN' should be used instead of '$UsDN'"]);
    }

    #[test]
    fn test_fail_usdn2() {
        let mut test_config = empty_config();

        test_config.expected_spellings =
            BTreeMap::from([("$USDN".to_string(), r#"\$*usdn"#.to_string())]);

        let example = "- Integrate our custom Dollar module, that enables the issuance of Noble's stablecoin $UsDN. ([#448](https://github.com/noble-assets/noble/pull/448))";
        let (_, problems) = check_spelling(&test_config, example);
        assert_eq!(problems, vec!["'$USDN' should be used instead of '$UsDN'"]);
    }
}

#[cfg(test)]
mod match_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let found_res = get_spelling_match("api", "Fix API.");
        assert!(found_res.is_ok());
        let found = found_res.unwrap();
        assert_eq!(found, "API");
    }

    #[test]
    fn test_ignore_inside_codeblocks() {
        let found_err = get_spelling_match("api", "Fix `aPi in codeblocks`.")
            .expect_err("expected match in code block");
        assert_eq!(found_err, MatchError::MatchInCodeblock);
    }

    #[test]
    fn test_ignore_in_word() {
        let found_err = get_spelling_match("api", "FixApI in word.")
            .expect_err("expected no match found error");
        assert_eq!(found_err, MatchError::NoMatchFound);
    }

    #[test]
    fn test_ignore_in_link() {
        let found_err = get_spelling_match("api", "Fix [abcdef](https://example/aPi.com)")
            .expect_err("expected no match found error");
        assert_eq!(found_err, MatchError::NoMatchFound);
    }
}
