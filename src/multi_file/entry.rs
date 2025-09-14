use crate::{common, config::Config, errors::EntryError};
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct MultiFileEntry {
    pub category: Option<String>,
    pub fixed: String,
    pub path: PathBuf,
    pub pr_number: u64,
    pub problems: Vec<String>,
}

pub fn parse(config: &Config, path: &Path) -> Result<MultiFileEntry, EntryError> {
    let contents = std::fs::read_to_string(path)?;

    // TODO: parse the contents for the following structure
    let mut pattern_string = r"^(?P<ws0>\s*)-(?P<ws1>\s*)".to_string();
    if config.use_categories {
        pattern_string.push_str(r"\((?P<category>[a-zA-Z0-9\-]+)\)");
    }
    pattern_string.push_str(r"(?P<ws2>\s*)(?P<desc>.+\S)");
    pattern_string.push_str(r"(?P<ws3>\s*)\(\[#(?P<pr>\d+)]");
    pattern_string.push_str(r"(?P<ws4>\s*)\((?P<link>[^)]*)\)\)\s*$");

    let entry_pattern = Regex::new(&pattern_string).expect("invalid regex pattern");

    let matches = match entry_pattern.captures(&contents) {
        Some(c) => c,
        None => return Err(EntryError::InvalidEntry(contents)),
    };

    // NOTE: calling unwrap here is okay because we checked that the pattern matched above
    let description = matches.name("desc").unwrap().as_str();
    let link = matches.name("link").unwrap().as_str();
    let pr_number = matches.name("pr").unwrap().as_str().parse::<u64>().unwrap();
    let mut spaces = [
        matches.name("ws0").unwrap().as_str(),
        matches.name("ws1").unwrap().as_str(),
        matches.name("ws2").unwrap().as_str(),
        matches.name("ws3").unwrap().as_str(),
        matches.name("ws4").unwrap().as_str(),
    ];

    let category: &str;
    if config.use_categories {
        category = matches.name("category").unwrap().as_str();
    } else {
        // NOTE: here we are adjusting the spaces slice in a way that
        // the expected spaces align if not using the categories.
        category = "";
        spaces[2] = spaces[1];
        spaces[1] = " ";
    }

    let mut problems: Vec<String> = Vec::new();

    if !path
        .file_name()
        .expect("failed to get base name")
        .to_str()
        .expect("failed to convert base name")
        .starts_with(&format!("{}", pr_number))
    {
        problems.push("The filename should be prefixed with the PR number".to_string());
    };

    let mut fixed_category: Option<String> = None;
    if config.use_categories {
        let (fixed_cat, category_problems) = common::entry::check_category(config, category);
        category_problems.into_iter().for_each(|p| problems.push(p));
        fixed_category = Some(fixed_cat);
    }

    let (fixed_link, link_problems) = common::entry::check_link(config, link, pr_number);
    link_problems.into_iter().for_each(|p| problems.push(p));

    let (fixed_desc, desc_problems) = common::entry::check_description(config, description);
    desc_problems.into_iter().for_each(|p| problems.push(p));

    let fixed = build_fixed(fixed_category.clone(), &fixed_link, &fixed_desc, pr_number);

    check_whitespace(spaces)
        .into_iter()
        .for_each(|p| problems.push(p));

    Ok(MultiFileEntry {
        category: fixed_category,
        fixed,
        path: path.into(),
        pr_number,
        problems,
    })
}

/// Returns the fixed entry string based on the given building parts.
fn build_fixed(cat: Option<String>, link: &str, desc: &str, pr: u64) -> String {
    // TODO: remove category?
    match cat {
        Some(c) => format!("- ({}) {} [#{}]({})", c, desc, pr, link),
        None => format!("- {} [#{}]({})", desc, pr, link),
    }
}

/// Checks the used whitespace in the entry.
fn check_whitespace(spaces: [&str; 5]) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();

    let expected_whitespace = ["", " ", " ", " ", ""];
    let errors = [
        "There should be no leading whitespace before the dash",
        "There should be exactly one space between the leading dash and the category",
        "There should be exactly one space between the category and the description",
        "There should be exactly one space between the description and the PR link",
        "There should be no whitespace inside of the markdown link",
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

// TODO: tests should be added
#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::unpack_config;

    fn load_example_config() -> Config {
        unpack_config(include_str!(
            "../../tests/testdata/multi_file/ok/.clconfig.json"
        ))
        .expect("failed to load example config")
    }

    #[test]
    fn test_pass() {
        let res = parse(
            &load_example_config(),
            Path::new(
                "tests/testdata/multi_file/ok/.changelog/v9.0.0/features/448-integrate-dollar.md",
            ),
        );
        assert!(res.is_ok());

        let entry = res.unwrap();
        let empty_problems: Vec<String> = Vec::new();
        assert_eq!(entry.pr_number, 448);
        assert_eq!(entry.problems, empty_problems);
    }

    #[test]
    fn test_fail() {
        let res = parse(
            &load_example_config(),
            Path::new(
                "tests/testdata/multi_file/fail/.changelog/v9.0.0/features/448-integrate-dollar.md",
            ),
        );
        assert!(res.is_ok());

        let entry = res.unwrap();
        let expected = vec!["'$USDN' should be used instead of '$UsDN'"];
        assert_eq!(entry.problems, expected);
    }
}
