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
        .expect("failed to convert base name to string")
        .contains(&format!("{}", pr_number))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::unpack_config;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn load_example_config() -> Config {
        unpack_config(include_str!(
            "../../tests/testdata/multi_file/ok/.clconfig.json"
        ))
        .expect("failed to load example config")
    }

    fn create_test_config_with_categories() -> Config {
        let mut config = load_example_config();
        config.use_categories = true;
        config.categories = vec!["feat".to_string(), "fix".to_string(), "imp".to_string()];
        config
    }

    // Custom struct to hold a temp file with a specific name
    struct NamedTestFile {
        _temp_dir: TempDir,
        path: PathBuf,
    }

    impl NamedTestFile {
        fn new(content: &str, filename: &str) -> Self {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).unwrap();

            Self {
                _temp_dir: temp_dir,
                path: file_path,
            }
        }

        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    fn create_temp_file(content: &str, filename: &str) -> NamedTestFile {
        NamedTestFile::new(content, filename)
    }

    #[test]
    fn test_parse_valid_entry_without_categories() {
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
        assert_eq!(entry.category, None);
        assert_eq!(entry.problems, empty_problems);
        assert!(entry.fixed.contains("Integrate our custom Dollar module"));
        assert!(entry.fixed.contains("[#448]"));
    }

    #[test]
    fn test_parse_entry_with_spelling_error() {
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

    #[test]
    fn test_parse_valid_entry_with_categories() {
        let content =
            "- (feat) Add new feature. ([#123](https://github.com/noble-assets/noble/pull/123))";
        let file = create_temp_file(content, "123-add-feature.md");

        let result = parse(&create_test_config_with_categories(), file.path());
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.pr_number, 123);
        assert_eq!(entry.category, Some("feat".to_string()));
        // No filename mismatch error since we now use the correct filename
        assert_eq!(entry.problems.len(), 0);
    }

    #[test]
    fn test_parse_entry_no_problems() {
        let content = "- Add feature with proper formatting. ([#123](https://github.com/noble-assets/noble/pull/123))";
        let file = create_temp_file(content, "123-add-feature.md");

        let result = parse(&load_example_config(), file.path());
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.pr_number, 123);
        assert_eq!(entry.category, None);

        let exp: Vec<String> = Vec::new();
        assert_eq!(entry.problems, exp);
    }

    #[test]
    fn test_filename_mismatch_error() {
        let content = "- Add feature ([#456](https://github.com/example/repo/pull/456))";
        let file = create_temp_file(content, "123-wrong-number.md");

        let result = parse(&load_example_config(), file.path());
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.pr_number, 456);
        assert!(entry
            .problems
            .iter()
            .any(|p| p.contains("The filename should be prefixed with the PR number")));
    }

    #[test]
    fn test_invalid_entry_format() {
        let content = "This is not a valid changelog entry format";
        let file = create_temp_file(content, "123-invalid.md");

        let result = parse(&load_example_config(), file.path());
        assert!(result.is_err());

        if let Err(EntryError::InvalidEntry(invalid_content)) = result {
            assert_eq!(invalid_content, content);
        } else {
            panic!("Expected InvalidEntry error");
        }
    }

    #[test]
    fn test_whitespace_validation() {
        let content = "  - Extra spaces here ([#123](https://github.com/example/repo/pull/123))";
        let file = create_temp_file(content, "123-whitespace.md");

        let result = parse(&load_example_config(), file.path());
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert!(entry
            .problems
            .contains(&"There should be no leading whitespace before the dash".to_string()));
    }

    #[test]
    fn test_build_fixed_without_category() {
        let fixed = build_fixed(
            None,
            "https://github.com/example/repo/pull/123",
            "Add feature",
            123,
        );
        assert_eq!(
            fixed,
            "- Add feature [#123](https://github.com/example/repo/pull/123)"
        );
    }

    #[test]
    fn test_build_fixed_with_category() {
        let fixed = build_fixed(
            Some("feat".to_string()),
            "https://github.com/example/repo/pull/123",
            "Add feature",
            123,
        );
        assert_eq!(
            fixed,
            "- (feat) Add feature [#123](https://github.com/example/repo/pull/123)"
        );
    }

    #[test]
    fn test_check_whitespace_perfect() {
        let spaces = ["", " ", " ", " ", ""];
        let problems = check_whitespace(spaces);
        assert_eq!(problems.len(), 0);
    }

    #[test]
    fn test_check_whitespace_leading_space() {
        let spaces = [" ", " ", " ", " ", ""];
        let problems = check_whitespace(spaces);
        assert!(
            problems.contains(&"There should be no leading whitespace before the dash".to_string())
        );
    }

    #[test]
    fn test_check_whitespace_multiple_errors() {
        let spaces = ["  ", "  ", "", " ", " "];
        let problems = check_whitespace(spaces);
        assert_eq!(problems.len(), 4);
        assert!(
            problems.contains(&"There should be no leading whitespace before the dash".to_string())
        );
        assert!(problems.contains(
            &"There should be exactly one space between the leading dash and the category"
                .to_string()
        ));
        assert!(problems.contains(
            &"There should be exactly one space between the category and the description"
                .to_string()
        ));
        assert!(problems
            .contains(&"There should be no whitespace inside of the markdown link".to_string()));
    }

    #[test]
    fn test_parse_entry_with_multiline_content() {
        let content =
            "- Add feature with\nmultiple lines ([#123](https://github.com/example/repo/pull/123))";
        let file = create_temp_file(content, "123-multiline.md");

        let result = parse(&load_example_config(), file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_entry_missing_pr_number() {
        let content = "- Add feature without PR number";
        let file = create_temp_file(content, "123-no-pr.md");

        let result = parse(&load_example_config(), file.path());
        assert!(result.is_err());
    }
}
