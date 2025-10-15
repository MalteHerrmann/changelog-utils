use crate::{common, config::Config, errors::EntryError};
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct MultiFileEntry {
    pub category: String,
    pub fixed: String,
    pub path: PathBuf,
    pub pr_number: u64,
    pub problems: Vec<String>,
}

pub fn parse(config: &Config, path: &Path) -> Result<MultiFileEntry, EntryError> {
    let contents = std::fs::read_to_string(path)?;

    let entry_pattern = Regex::new(concat!(
        // TODO: have category as optional?
        r"^(?P<ws0>\s*)-(?P<ws1>\s*)\((?P<category>[a-zA-Z0-9\-]+)\)",
        r"(?P<ws2>\s*)(?P<desc>.+)",
        r"(?P<ws3>\s*)\(\[#(?P<pr>\d+)]",
        r"(?P<ws4>\s*)\((?P<link>[^)]*)\)\)\s*$"
    ))
    .expect("invalid regex pattern");

    let matches = match entry_pattern.captures(&contents) {
        Some(c) => c,
        None => return Err(EntryError::InvalidEntry(contents)),
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

    if !path
        .to_str()
        .expect("failed to convert path into string")
        .starts_with(format!("{}", pr_number).as_str())
    {
        problems.push("The filename should be prefixed with the PR number".to_string());
    };

    let (fixed_category, category_problems) = common::entry::check_category(config, category);
    category_problems.into_iter().for_each(|p| problems.push(p));

    let (fixed_link, link_problems) = common::entry::check_link(config, link, pr_number);
    link_problems.into_iter().for_each(|p| problems.push(p));

    let (fixed_desc, desc_problems) = common::entry::check_description(config, description);
    desc_problems.into_iter().for_each(|p| problems.push(p));

    let fixed = build_fixed(&fixed_category, &fixed_link, &fixed_desc, pr_number);

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
fn build_fixed(cat: &str, link: &str, desc: &str, pr: u64) -> String {
    // TODO: remove category?
    format!("- ({}) {} [#{}]({})", cat, desc, pr, link)
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
