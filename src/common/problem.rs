use std::{
    fmt,
    path::{Path, PathBuf},
};

/// The type of a lint error, used to give more context about what part of the
/// changelog a problem relates to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LintErrorType {
    /// Problems with the file itself, like a wrong file name.
    File,
    /// Problems with the directory structure of a multi file changelog.
    Dir,
    /// Problems with the change category of an entry.
    Category,
    /// Problems with the change type of an entry.
    ChangeType,
    /// Problems with the description of an entry.
    Description,
    /// Problems with the PR reference or link of an entry.
    PullRequest,
    /// Problems with a release header, version or release link.
    Release,
    /// Problems with the whitespace used in an entry.
    Whitespace,
    /// Duplicated releases, change types or entries.
    Duplicate,
    /// Everything that cannot be assigned to one of the other types.
    Other,
}

impl LintErrorType {
    /// Returns the type of lint error for the given problem description.
    pub fn match_problem(problem: &str) -> LintErrorType {
        let lowercase = problem.to_lowercase();

        if lowercase.starts_with("duplicate ") {
            return LintErrorType::Duplicate;
        }

        if lowercase.contains("filename") || lowercase.contains("found in file") {
            return LintErrorType::File;
        }

        if lowercase.contains("directory") {
            return LintErrorType::Dir;
        }

        if lowercase.contains("whitespace") || lowercase.contains(" space ") {
            return LintErrorType::Whitespace;
        }

        if lowercase.contains("pr link") || lowercase.contains("pr number") {
            return LintErrorType::PullRequest;
        }

        if lowercase.contains("change type") {
            return LintErrorType::ChangeType;
        }

        if lowercase.contains("category") {
            return LintErrorType::Category;
        }

        if lowercase.contains("description")
            || lowercase.contains("should be used instead of")
            || lowercase.contains("malformed entry")
        {
            return LintErrorType::Description;
        }

        if lowercase.contains("version string")
            || lowercase.contains("release link")
            || lowercase.contains("unreleased header")
        {
            return LintErrorType::Release;
        }

        LintErrorType::Other
    }
}

impl fmt::Display for LintErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            LintErrorType::File => "file",
            LintErrorType::Dir => "dir",
            LintErrorType::Category => "category",
            LintErrorType::ChangeType => "change type",
            LintErrorType::Description => "description",
            LintErrorType::PullRequest => "pr",
            LintErrorType::Release => "release",
            LintErrorType::Whitespace => "whitespace",
            LintErrorType::Duplicate => "duplicate",
            LintErrorType::Other => "other",
        };

        write!(f, "{}", name)
    }
}

/// Represents an individual problem found while linting a changelog,
/// including its location and the type of lint error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Problem {
    /// The type of lint error.
    pub error_type: LintErrorType,
    /// The path to the file the problem was found in.
    pub path: PathBuf,
    /// The line in the file the problem was found in, if applicable.
    pub line: Option<usize>,
    /// The description of the problem.
    pub message: String,
}

impl Problem {
    /// Creates a new problem for the given location and description
    /// and derives the type of lint error from the description.
    ///
    /// NOTE: The line ID will be incremented by one based on the loop
    /// enumeration where it is used.
    pub fn new(path: &Path, line: Option<usize>, message: impl Into<String>) -> Problem {
        let message = message.into();

        Problem {
            error_type: LintErrorType::match_problem(&message),
            path: path.to_path_buf(),
            line: line.map(|l| l + 1),
            message,
        }
    }

    /// Returns the location of the problem in the shape of `path[:line]`.
    pub fn location(&self) -> String {
        match self.line {
            Some(l) => format!("{}:{}", self.path.to_string_lossy(), l),
            None => self.path.to_string_lossy().to_string(),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} [{}]: {}",
            self.location(),
            self.error_type,
            self.message
        )
    }
}

#[cfg(test)]
mod match_problem_tests {
    use super::*;

    #[test]
    fn test_matching_types() {
        let cases = [
            ("duplicate PR: #1862", LintErrorType::Duplicate),
            (
                "duplicate change type in release Unreleased: Bug Fixes",
                LintErrorType::Duplicate,
            ),
            (
                "The filename should be prefixed with the PR number",
                LintErrorType::File,
            ),
            (
                "invalid entry found in file: some/path.md",
                LintErrorType::File,
            ),
            (
                "There should be no whitespace inside of the markdown link",
                LintErrorType::Whitespace,
            ),
            (
                "There should be exactly one space between the category and the PR link",
                LintErrorType::Whitespace,
            ),
            (
                "PR link is not matching PR number 2: 'https://github.com/org/repo/pull/1'",
                LintErrorType::PullRequest,
            ),
            (
                "'Invalid Category' is not a valid change type",
                LintErrorType::ChangeType,
            ),
            ("category should be lowercase: (CLI)", LintErrorType::Category),
            (
                "PR description should end with a dot: 'Test'",
                LintErrorType::Description,
            ),
            ("'ABI' should be used instead of 'ABi'", LintErrorType::Description),
            ("invalid version string: v1", LintErrorType::Release),
            (
                "Unreleased header is malformed; expected: '## Unreleased'; got: '## unreleased'",
                LintErrorType::Release,
            ),
            ("some unknown problem", LintErrorType::Other),
        ];

        cases.into_iter().for_each(|(problem, expected)| {
            assert_eq!(
                LintErrorType::match_problem(problem),
                expected,
                "unexpected error type for '{}'",
                problem
            )
        });
    }
}

#[cfg(test)]
mod problem_tests {
    use super::*;

    #[test]
    fn test_display_with_line() {
        let problem = Problem::new(
            Path::new("CHANGELOG.md"),
            Some(10),
            "PR description should end with a dot: 'Test'",
        );

        assert_eq!(
            problem.to_string(),
            "CHANGELOG.md:11 [description]: PR description should end with a dot: 'Test'"
        );
    }

    #[test]
    fn test_display_without_line() {
        let problem = Problem::new(Path::new("CHANGELOG.md"), None, "some unknown problem");

        assert_eq!(
            problem.to_string(),
            "CHANGELOG.md [other]: some unknown problem"
        );
    }
}
