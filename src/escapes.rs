use regex::Regex;

/// Enum for the available linter escapes.
#[derive(Debug, PartialEq)]
pub enum LinterEscape {
    FullLine,
    DuplicatePR,
}

/// Checks the given comment for an escape pattern.
pub fn check_escape_pattern(line: &str) -> Option<LinterEscape> {
    // TODO: improve handling here with associated traits for the linter escapes?
    match Regex::new(r"<!--\s*clu-disable-next-line-duplicate-pr(:.+)?\s*-->").unwrap().is_match(line) {
        true => Some(LinterEscape::DuplicatePR),
        false => match Regex::new(r"<!--\s*clu-disable-next-line(:.+)?\s*-->").unwrap().is_match(line) {
            true => Some(LinterEscape::FullLine),
            false => None,
        }
    }
}

#[cfg(test)]
mod escape_tests {
    use super::*;

    #[test]
    fn test_no_escape() {
        assert!(check_escape_pattern("line without escape pattern").is_none());
    }

    #[test]
    fn test_escape_full_line() {
        assert_eq!(check_escape_pattern("<!-- clu-disable-next-line -->"), Some(LinterEscape::FullLine));
    }

    #[test]
    fn test_escape_full_line_with_comment() {
        assert_eq!(check_escape_pattern("<!-- clu-disable-next-line: optional description -->"), Some(LinterEscape::FullLine));
    }

    #[test]
    fn test_escape_duplicate() {
        assert_eq!(check_escape_pattern("<!-- clu-disable-next-line-duplicate-pr -->"), Some(LinterEscape::DuplicatePR));
    }

    #[test]
    fn test_escape_duplicate_with_comment() {
        assert_eq!(check_escape_pattern("<!-- clu-disable-next-line-duplicate-pr: optional description -->"), Some(LinterEscape::DuplicatePR));
    }
}