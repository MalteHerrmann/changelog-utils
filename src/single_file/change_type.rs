use super::entry::Entry;
use crate::{errors::ChangeTypeError, utils::config};
use regex::{Regex, RegexBuilder};

#[derive(Clone, Debug)]
pub struct ChangeType {
    pub name: String,
    pub fixed: String,
    pub problems: Vec<String>,
    pub entries: Vec<Entry>,
}

impl ChangeType {
    pub fn get_fixed_contents(&self) -> String {
        let mut exported_string = String::new();

        exported_string.push_str(&self.fixed);
        exported_string.push_str("\n\n");

        self.entries.iter().for_each(|entry| {
            exported_string.push_str(format!("{}\n", entry.fixed).as_str());
        });

        exported_string
    }
}

// Creates a new instance of a change type.
pub fn new(name: String, entries: Option<Vec<Entry>>) -> ChangeType {
    ChangeType {
        name: name.clone(),
        fixed: format!("### {name}"),
        problems: Vec::new(),
        entries: entries.unwrap_or_default(),
    }
}

pub fn parse(config: config::Config, line: &str) -> Result<ChangeType, ChangeTypeError> {
    let captures = match Regex::new(r"^\s*###\s*(?P<name>[a-zA-Z0-9\- ]+)\s*$")
        .expect("regex pattern should be valid")
        .captures(line)
    {
        Some(c) => c,
        None => return Err(ChangeTypeError::NoMatchesFound),
    };

    // NOTE: calling unwrap here is okay, because the match was checked above
    let name = captures.name("name").unwrap().as_str();
    let mut fixed_name = name.to_string();
    let mut problems: Vec<String> = Vec::new();

    // Check the correctness of the current change type.
    if !config.change_types.iter().any(|ct| {
        // derive the generalized pattern with case insensitivity and whitespace
        // matching from the given change type
        let pattern = RegexBuilder::new(r"\s+")
            .case_insensitive(true)
            .build()
            .unwrap()
            .replace_all(&ct.long, r"\s*")
            .into_owned();

        if !RegexBuilder::new(pattern.as_str())
            .case_insensitive(true)
            .build()
            .unwrap()
            .is_match(name)
        {
            return false;
        }

        if name != ct.long {
            problems.push(format!(
                "'{}' should be used instead of '{}'",
                ct.long, name
            ));
            fixed_name.clone_from(&ct.long);
        }

        true
    }) {
        problems.push(format!("'{name}' is not a valid change type"))
    };

    let fixed = format!("### {fixed_name}");

    if format!("### {name}").ne(line) {
        problems.push(format!(
            "Change type line is malformed; should be: '{fixed}'"
        ));
    }

    Ok(ChangeType {
        name: fixed_name,
        fixed,
        problems,
        entries: Vec::new(),
    })
}

#[cfg(test)]
mod change_type_tests {
    use super::*;

    fn load_test_config() -> config::Config {
        config::unpack_config(include_str!("../testdata/example_config.json"))
            .expect("failed to load config")
    }

    #[test]
    fn test_pass() {
        let example = "### Bug Fixes";
        let change_type =
            parse(load_test_config(), example).expect("unexpected error parsing change type");
        assert_eq!(change_type.fixed, example);
        assert_eq!(change_type.name, "Bug Fixes");
        assert!(change_type.problems.is_empty());
    }

    #[test]
    fn test_wrong_whitespace() {
        let example = " ###  Bug Fixes";
        let change_type =
            parse(load_test_config(), example).expect("unexpected error parsing change type");
        assert_eq!(change_type.fixed, "### Bug Fixes");
        assert_eq!(change_type.name, "Bug Fixes");
        assert_eq!(
            change_type.problems,
            vec!["Change type line is malformed; should be: '### Bug Fixes'"]
        );
    }

    #[test]
    fn test_fail_malformed_entry() {
        let example = "##jeaf";
        let err = parse(load_test_config(), example).expect_err("expected parsing to fail");
        assert_eq!(err, ChangeTypeError::NoMatchesFound);
    }

    #[test]
    fn test_wrong_spelling() {
        let example = "### BugFixes";
        let change_type =
            parse(load_test_config(), example).expect("unexpected error parsing change type");
        assert_eq!(change_type.fixed, "### Bug Fixes");
        assert_eq!(change_type.name, "Bug Fixes");
        assert_eq!(
            change_type.problems,
            vec!["'Bug Fixes' should be used instead of 'BugFixes'"]
        );
    }

    #[test]
    fn test_invalid_change_type() {
        let example = "### invalid type";
        let change_type =
            parse(load_test_config(), example).expect("unexpected error parsing change type");
        assert_eq!(change_type.fixed, example);
        assert_eq!(change_type.name, "invalid type");
        assert_eq!(
            change_type.problems,
            vec!["'invalid type' is not a valid change type"]
        );
    }
}
