use crate::errors::{EntryError, MatchError};
use regex::{Regex, RegexBuilder};

/// Represents an individual entry in the changelog.
struct Entry<'a> {
    /// The original line from the parsed changelog.
    line: &'a str,
    /// The fixed line adhering to all standards.
    fixed: String,
    /// The category of the given entry, e.g. (tests).
    category: &'a str,
    /// The description of the changes.
    description: &'a str,
    /// The PR number for the given change.
    pr_number: u16,
    /// The link to the PR
    link: &'a str,
    /// The list of problems with the given line.
    ///
    /// TODO: Should this rather be a Vec<a' str>?
    problems: Vec<String>,
}

fn parse(line: &str) -> Result<Entry, EntryError> {
    let mut regex_string = r"^-(?P<ws1>\s*)\((?P<category>[a-zA-Z0-9\-]+)\)".to_string();
    regex_string.push_str(r"(?P<ws2>\s*)\[(?P<bs>\\)?#(?P<pr>\d+)]");
    regex_string.push_str(r"(?P<ws3>\s*)\((?P<link>[^)]*)\)(?P<ws4>\s*)(?P<desc>.+)$");

    let entry_pattern = Regex::new(regex_string.as_str()).expect("invalid regex pattern");
    let matches = match entry_pattern.captures(line) {
        Some(c) => c,
        None => return Err(EntryError::InvalidEntry(line.to_string())),
    };

    // NOTE: calling unwrap here is okay because we checked that the pattern matched above
    let category = matches.name("category").unwrap().as_str();
    let description = matches.name("desc").unwrap().as_str();
    let link = matches.name("link").unwrap().as_str();
    let pr_number = matches.name("pr").unwrap().as_str().parse::<u16>().unwrap();

    // TODO: check whitespace in matches

    // TODO: check individual parts for problems like category, etc.
    let mut problems: Vec<String> = Vec::new();

    let fixed = format!(
        "- ({}) [#{}]({}) {}",
        category, pr_number, link, description,
    );

    Ok(Entry {
        line,
        fixed, // TODO: why is it not possible to have this as &'a str too?
        category,
        description,
        link,
        pr_number,
        // TODO: implement describing problems in line
        problems,
    })
}

/// Check if the category is valid and return a fixed version that addresses
/// well-known problems.
fn check_category(category: &str) -> (String, Vec<String>) {
    let mut problems: Vec<String> = Vec::new();
    let fixed = category.to_lowercase();
    if category.to_lowercase() != category {
        problems.push(format!("category should be lowercase: ({})", category));
    }

    // TODO: use config
    let allowed_categories: Vec<String> = vec!["cli".to_string(), "test".to_string()];
    if !allowed_categories.contains(&fixed) {
        problems.push(format!("invalid change category: ({})", category));
    }

    (fixed, problems)
}

/// Check if the link is valid
fn check_link(link: &str, pr_number: u16) -> (String, Vec<String>) {
    let mut problems: Vec<String> = Vec::new();

    // TODO: check the base url of the used Git repository automatically
    let expected_base_url = r"https://github.com/MalteHerrmann/changelog-utils";
    let fixed = format!("{}/pull/{}", expected_base_url, pr_number);

    if !link.starts_with(expected_base_url) {
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
            "PR link is not matching PR number {}: {}",
            pr_number, link
        ));
    }

    (fixed, problems)
}

fn check_description(desc: &str) -> (String, Vec<String>) {
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
        problems.push(format!(
            "PR description should end with a dot: '{}'",
            desc
        ))
    }

    (fixed, problems)
}

// /// Checks the spelling of entries according to the given configuration.
// fn check_spelling() -> (String, Vec<String>) {
//     // TODO: continue here
// }

/// Returns the first match of the given pattern in the text.
/// Matching patterns inside of code blocks, links or within another word are ignored.
fn get_spelling_match(pattern: &str, text: &str) -> Result<String, MatchError> {
    // Check if pattern is inside a code block
    match RegexBuilder::new(
        format!(r"`[^`]*({pattern})[^`]*`").as_str()
    )
        .case_insensitive(true)
        .build()?
        .find(text) {
        Some(_) => return Err(MatchError::MatchInCodeblock),
        None => (),
    }

    // Check isolated words (i.e. pattern is not included in another word)
    let found = match RegexBuilder::new(
        format!(r"(^|\s)({pattern})($|[\s.])").as_str()
    )
        .case_insensitive(true)
        .build()?
        .captures(text) {
        Some(m) => m,
        None => return Err(MatchError::NoMatchFound)
    };

    // TODO: merge with match above to avoid double matching?
    match found.get(2) {
        Some(m) => Ok(m.as_str().to_string()),
        None => return Err(MatchError::NoMatchFound)
    }
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
        let entry_res = parse(example);
        assert!(entry_res.is_ok());
        let entry = entry_res.unwrap();
        assert!(entry.line == example);
        assert!(entry.fixed == example); // NOTE: since line is okay there are no changes to it in the fixed version
        assert!(entry.category == "cli");
        assert!(entry.pr_number == 1);
        assert!(entry.link == "https://github.com/MalteHerrmann/changelog-utils/pull/1");
        assert!(entry.description == "Add initial Python implementation.");
        assert!(entry.problems.is_empty());
    }

    #[test]
    fn test_fail_has_backslash_in_link() {
        let example =
            r"- (cli) [\#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) Test.";
        let entry_res = parse(example);
        // TODO: should this actually return an error? Not really, because parsing has worked??
        assert!(entry_res.is_err());
        let entry = entry_res.unwrap();
        assert!(entry.line == example);
        assert!(entry.fixed == example.replace(r"\", ""));
        assert!(entry.category == "cli");
        assert!(entry.pr_number == 1851);
        assert!(entry.link == "https://github.com/MalteHerrmann/changelog-utils/pull/1");
        assert!(entry.description == "Test.");
        assert!(entry.problems.len() == 1);
        assert!(
            entry.problems[0] == "there should be no backslash in front of the # in the PR link"
        );
    }

    #[test]
    fn test_fail_wrong_pr_link_and_missing_dot() {
        let example = r"- (cli) [#2](https://github.com/MalteHerrmann/changelog-utils/pull/1) Test";
        let entry_res = parse(example);
        assert!(entry_res.is_err());
        let entry = entry_res.unwrap();
        assert!(entry.line == example);
        assert!(entry.fixed == example.replace(r"\", ""));
        assert!(entry.category == "cli");
        assert!(entry.pr_number == 1851);
        assert!(entry.link == "https://github.com/MalteHerrmann/changelog-utils/pull/1");
        assert!(entry.description == "Test.");
        assert!(entry.problems.len() == 2);
        assert!(
            entry.problems
                == vec![
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
        assert!(parse(example).is_err());
    }
}

#[cfg(test)]
mod category_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let (fixed, problems) = check_category("cli");
        assert!(fixed == "cli");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_fail_invalid_category() {
        let (fixed, problems) = check_category("invalid");
        assert!(fixed == "invalid");
        assert!(problems == vec!["invalid change category: (invalid)"]);
    }

    #[test]
    fn test_fail_non_lower_category() {
        let (fixed, problems) = check_category("cLi");
        assert!(fixed == "cli");
        assert!(problems == vec!["category should be lowercase: (cLi)"]);
    }
}

#[cfg(test)]
mod link_tests {
    use super::*;

    #[test]
    fn test_pass() {
        let example = r"https://github.com/MalteHerrmann/changelog-utils/pull/1";
        let (fixed, problems) = check_link(example, 1);
        assert!(fixed == example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_wrong_base_url() {
        let example = r"https://github.com/MalteHerrmann/changelg-utils/pull/1";
        let (fixed, problems) = check_link(example, 1);
        assert!(fixed == example.replace("changelg", "changelog"));
        assert!(problems == vec![format!("PR link points to wrong repository: {}", example)]);
    }

    #[test]
    fn test_wrong_pr_number() {
        let example = r"https://github.com/MalteHerrmann/changelog-utils/pull/2";
        let (fixed, problems) = check_link(example, 1);
        assert!(fixed == example.replace("2", "1"));
        assert!(
            problems
                == vec![format!(
                    "PR link is not matching PR number {}: {}",
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
        let (fixed, problems) = check_description(example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_pass_start_with_codeblock_instead_of_capital_letter() {
        let example = "`add` method implemented.";
        let (fixed, problems) = check_description(example);
        assert_eq!(fixed, example);
        assert!(problems.is_empty(), "expected no problems: {:?}", problems);
    }

    #[test]
    fn test_fail_start_with_lowercase() {
        let example = "add Python implementation.";
        let (fixed, problems) = check_description(example);
        assert_eq!(fixed, "Add Python implementation.");
        assert_eq!(problems, vec![format!(
            "PR description should start with capital letter: '{}'",
            example
        )]);
    }

    #[test]
    fn test_fail_does_not_end_with_dot() {
        let example = "Add Python implementation";
        let (fixed, problems) = check_description(example);
        assert_eq!(fixed, example.to_string() + ".");
        assert_eq!(problems, vec![format!("PR description should end with a dot: '{}'", example)]);
    }
}

// #[cfg(test)]
// mod spelling_tests {
//     use super::*;
//
//     #[test]
//     fn test_pass() {
//         let example = "Fix API.";
//         let (fixed, problems) = check_spelling(CONFIG, example)
//             .expect("unexpected error during spell check");
//         assert_eq!(fixed, example);
//         assert!(problems.is_empty());
//     }
//
//     #[test]
//     fn test_wrong_spelling() {
//         let example = "Fix aPi.";
//         let (fixed, problems) = check_spelling(CONFIG, example)
//             .expect("unexpected error during spell check");
//         assert_eq!(fixed, "Fix API.");
//         assert_eq!(problems, vec!["'API' should be used instead of 'aPi'"])
//     }
//
//     #[test]
//     fn test_multiple_problems() {
//         let example = "Fix aPi and ClI.";
//         let (fixed, problems) = check_spelling(CONFIG, example)
//             .expect("unexpected error during spell check");
//         assert_eq!(fixed, "Fix API and CLI.");
//         assert_eq!(problems, vec![
//             "'API' should be used instead of 'aPi'",
//             "'CLI' should be used instead of 'ClI'",
//         ])
//     }
//
//     #[test]
//     fn test_pass_codeblocks() {
//         let example = "Fix `ApI in codeblocks`.";
//         let (fixed, problems) = check_spelling(CONFIG, example)
//             .expect("unexpected error during spell check");
//         assert_eq!(fixed, example);
//         assert!(problems.is_empty());
//     }
//
//     #[test]
//     fn test_pass_nested_word() {
//         let example = "FixApI in another word.";
//         let (fixed, problems) = check_spelling(CONFIG, example)
//             .expect("unexpected error during spell check");
//         assert_eq!(fixed, example);
//         assert!(problems.is_empty());
//     }
// }
//
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