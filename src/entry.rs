use crate::{config, errors::{EntryError, MatchError}};
use regex::{Error, Regex, RegexBuilder};

/// Represents an individual entry in the changelog.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The original line from the parsed changelog.
    line: String,
    /// The fixed line adhering to all standards.
    pub fixed: String,
    /// The category of the given entry, e.g. (tests).
    category: String,
    /// The description of the changes.
    description: String,
    /// The PR number for the given change.
    pub pr_number: u16,
    /// The link to the PR
    link: String,
    /// The list of problems with the given line.
    ///
    /// TODO: Should this rather be a Vec<a' str>?
    pub problems: Vec<String>,
}

pub fn parse(config: config::Config, line: &str) -> Result<Entry, EntryError> {
    let entry_pattern = Regex::new(concat!(
        r"^(?P<ws0>\s*)-(?P<ws1>\s*)\((?P<category>[a-zA-Z0-9\-]+)\)",
        r"(?P<ws2>\s*)\[(?P<bs>\\)?#(?P<pr>\d+)]",
        r"(?P<ws3>\s*)\((?P<link>[^)]*)\)(?P<ws4>\s*)(?P<desc>.+)$"
    )).expect("invalid regex pattern");

    let matches = match entry_pattern.captures(line) {
        Some(c) => c,
        None => return Err(EntryError::InvalidEntry(line.to_string())),
    };

    // NOTE: calling unwrap here is okay because we checked that the pattern matched above
    //
    // TODO: This should definitely improved if possible, using some iterator stuff maybe?
    let category = matches.name("category").unwrap().as_str();
    let description = matches.name("desc").unwrap().as_str();
    let link = matches.name("link").unwrap().as_str();
    let pr_number = matches.name("pr").unwrap().as_str().parse::<u16>().unwrap();
    let spaces = vec![
        matches.name("ws0").unwrap().as_str(),
        matches.name("ws1").unwrap().as_str(),
        matches.name("ws2").unwrap().as_str(),
        matches.name("ws3").unwrap().as_str(),
        matches.name("ws4").unwrap().as_str(),
    ];

    let mut problems: Vec<String> = Vec::new();
    for whitespace_problem in check_whitespace(spaces) {
        problems.push(whitespace_problem)
    }

    let (fixed_category, category_problems) = check_category(&config, category);
    for category_problem in category_problems {
        problems.push(category_problem)
    }

    match matches.name("bs") {
        Some(_) => problems
            .push("There should be no backslash in front of the # in the PR link".to_string()),
        _ => (),
    }

    let (fixed_link, link_problems) = check_link(&config, link, pr_number);
    for link_problem in link_problems {
        problems.push(link_problem)
    }

    let (fixed_desc, desc_problems) = check_description(&config, description);
    for desc_problem in desc_problems {
        problems.push(desc_problem)
    }

    let fixed = format!(
        "- ({}) [#{}]({}) {}",
        fixed_category, pr_number, fixed_link, fixed_desc,
    );

    Ok(Entry {
        line: line.to_string(),
        fixed, // TODO: why is it not possible to have this as &'a str too?
        category: category.to_string(),
        description: description.to_string(),
        link: link.to_string(),
        pr_number,
        // TODO: implement describing problems in line
        problems,
    })
}

/// Check if the category is valid and return a fixed version that addresses
/// well-known problems.
fn check_category(config: &config::Config, category: &str) -> (String, Vec<String>) {
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
fn check_link(config: &config::Config, link: &str, pr_number: u16) -> (String, Vec<String>) {
    let mut problems: Vec<String> = Vec::new();

    // TODO: check the base url of the used Git repository automatically
    let fixed = format!("{}/pull/{}", config.target_repo, pr_number);

    if !link.starts_with(config.target_repo.as_str()) {
        problems.push(format!("PR link points to wrong repository: {}", link))
    }

    let split_link: Vec<&str> = link.split("/").collect();
    let contained_pr_number = split_link
        .last()
        .expect("this should never be empty")
        .parse::<u16>()
        .expect("this should always be a u16");

    if contained_pr_number != pr_number {
        problems.push(format!(
            "PR link is not matching PR number {}: '{}'",
            pr_number, link
        ));
    }

    (fixed, problems)
}

fn check_description(config: &config::Config, desc: &str) -> (String, Vec<String>) {
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

    let last_letter = desc
        .chars()
        .last()
        .expect("no characters found in description");
    if last_letter.to_string() != ".".to_string() {
        fixed = desc.to_string() + ".";
        problems.push(format!("PR description should end with a dot: '{}'", desc))
    }

    let (fixed, spelling_problems) = check_spelling(config, fixed.as_str());
    for prob in spelling_problems {
        problems.push(prob)
    }

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
                    .expect("failed to compile regex") // TODO: return Result rather than use expect here?
                    .replace(fixed.as_str(), correct_spelling)
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
    match RegexBuilder::new(format!(r"`[^`]*({pattern})[^`]*`").as_str())
        .case_insensitive(true)
        .build()?
        .find(text)
    {
        Some(_) => return Err(MatchError::MatchInCodeblock),
        None => (),
    }

    // Check isolated words (i.e. pattern is not included in another word)
    let found = match RegexBuilder::new(format!(r"(^|\s)({pattern})($|[\s.])").as_str())
        .case_insensitive(true)
        .build()?
        .captures(text)
    {
        Some(m) => m,
        None => return Err(MatchError::NoMatchFound),
    };

    // TODO: merge with match above to avoid double matching?
    match found.get(2) {
        Some(m) => Ok(m.as_str().to_string()),
        None => return Err(MatchError::NoMatchFound),
    }
}

/// Checks the used whitespace in the entry.
fn check_whitespace(spaces: Vec<&str>) -> Vec<String> {
    if spaces.len() != 5 {
        panic!(
            "unexpected amount of whitespace values passed; expected 5; got {}",
            spaces.len()
        )
    }

    let mut problems: Vec<String> = Vec::new();
    let separator = " ";

    let errors = vec![
        "There should be no leading whitespace before the dash",
        "There should be exactly one space between the leading dash and the category",
        "There should be exactly one space between the category and the PR link",
        "There should be no whitespace inside of the markdown link",
        "There should be exactly one space between the PR link and the description",
    ];

    for (i, val) in spaces.iter().enumerate() {
        match i {
            // The whitespace at these indices should be empty instead of the separator
            0 | 3 => {
                if (*val).ne("") {
                    problems.push(errors[i].to_string())
                }
            }
            _ => {
                if (*val).ne(separator) {
                    problems.push(errors[i].to_string())
                }
            }
        }
    }

    problems
}

#[cfg(test)]
fn load_test_config() -> config::Config {
    config::load(include_str!("testdata/example_config.json"))
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
        let entry_res = parse(load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.line, example);
        assert_eq!(entry.fixed, example); // NOTE: since line is okay there are no changes to it in the fixed version
        assert_eq!(entry.category, "cli");
        assert_eq!(entry.pr_number, 1);
        assert_eq!(
            entry.link,
            "https://github.com/MalteHerrmann/changelog-utils/pull/1"
        );
        assert_eq!(entry.description, "Add initial Python implementation.");
        assert!(entry.problems.is_empty());
    }

    #[test]
    fn test_fail_has_backslash_in_link() {
        let example =
            r"- (cli) [\#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) Test.";
        let entry_res = parse(load_test_config(), example);
        // TODO: should this actually return an error? Not really, because parsing has worked??
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.line, example);
        assert_eq!(entry.fixed, example.replace(r"\", ""));
        assert_eq!(entry.category, "cli");
        assert_eq!(entry.pr_number, 1);
        assert_eq!(
            entry.link,
            "https://github.com/MalteHerrmann/changelog-utils/pull/1"
        );
        assert_eq!(entry.description, "Test.");
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
        let entry_res = parse(load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.line, example);
        assert_eq!(entry.fixed, fixed);
        assert_eq!(entry.category, "cli");
        assert_eq!(entry.pr_number, 2);
        assert_eq!(
            entry.link,
            "https://github.com/MalteHerrmann/changelog-utils/pull/1"
        );
        assert_eq!(entry.description, "Test");
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
        // TODO: figure how to still return an entry but with the corresponding array of problems filled
        assert!(parse(load_test_config(), example).is_err());
    }

    #[test]
    fn test_fail_wrong_whitespace() {
        let example =
            r"- (cli)   [#1] (https://github.com/MalteHerrmann/changelog-utils/pull/1) Run test.";
        let expected =
            r"- (cli) [#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) Run test.";
        let entry_res = parse(load_test_config(), example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert_eq!(entry.line, example);
        assert_eq!(entry.fixed, expected);
        assert_eq!(entry.category, "cli");
        assert_eq!(entry.pr_number, 1);
        assert_eq!(
            entry.link,
            "https://github.com/MalteHerrmann/changelog-utils/pull/1"
        );
        assert_eq!(entry.description, "Run test.");
        assert_eq!(entry.problems.len(), 2);
        assert_eq!(
            entry.problems,
            vec![
                "There should be exactly one space between the category and the PR link",
                "There should be no whitespace inside of the markdown link",
            ]
        );
    }
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
        assert_eq!(problems, vec!["invalid change category: (invalid)"]);
    }

    #[test]
    fn test_fail_non_lower_category() {
        let (fixed, problems) = check_category(&load_test_config(), "cLi");
        assert_eq!(fixed, "cli");
        assert_eq!(problems, vec!["category should be lowercase: (cLi)"]);
    }
}

#[cfg(test)]
mod link_tests {
    use super::*;

    fn load_test_config() -> config::Config {
        config::load(include_str!("testdata/example_config.json"))
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
mod spelling_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example = "Fix API.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_wrong_spelling() {
        let example = "Fix web--SdK.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, "Fix Web-SDK.");
        assert_eq!(
            problems,
            vec!["'Web-SDK' should be used instead of 'web--SdK'"]
        )
    }

    #[test]
    fn test_multiple_problems() {
        let example = "Fix aPi and ClI.";
        let (fixed, problems) = check_spelling(&load_test_config(), example);
        assert_eq!(fixed, "Fix API and CLI.");
        assert_eq!(problems.len(), 2);
        // TODO: this is currently not deterministically in the same order
        assert!(problems.contains(&"'API' should be used instead of 'aPi'".to_string()));
        assert!(problems.contains(&"'CLI' should be used instead of 'ClI'".to_string()));
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

#[cfg(test)]
mod whitespace_tests {
    use super::*;

    #[test]
    fn test_pass() {
        // TODO: rather pass as &str?
        let example_spaces = vec!["", " ", " ", "", " "];
        assert!(check_whitespace(example_spaces).is_empty());
    }

    #[test]
    fn test_fail_leading_space() {
        let example_spaces = vec![" ", " ", " ", "", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            vec!["There should be no leading whitespace before the dash"]
        );
    }

    #[test]
    fn test_fail_space_between_category_and_link() {
        let example_spaces = vec!["", " ", "", "", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            vec!["There should be exactly one space between the category and the PR link"]
        );
    }

    #[test]
    fn test_fail_multiple_spaces() {
        let example_spaces = vec!["", "", " ", "", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            vec!["There should be exactly one space between the leading dash and the category"]
        );
    }

    #[test]
    fn test_fail_multiple_spaces_before_description() {
        let example_spaces = vec!["", " ", " ", "", "  "];
        assert_eq!(
            check_whitespace(example_spaces),
            vec!["There should be exactly one space between the PR link and the description"]
        );
    }

    #[test]
    fn test_fail_space_in_link() {
        let example_spaces = vec!["", " ", " ", " ", " "];
        assert_eq!(
            check_whitespace(example_spaces),
            vec!["There should be no whitespace inside of the markdown link"]
        );
    }
}
