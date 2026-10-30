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
}

impl fmt::Display for LintErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            LintErrorType::File => "file",
            LintErrorType::Category => "category",
            LintErrorType::ChangeType => "change type",
            LintErrorType::Description => "description",
            LintErrorType::PullRequest => "pr",
            LintErrorType::Release => "release",
            LintErrorType::Whitespace => "whitespace",
            LintErrorType::Duplicate => "duplicate",
        };

        write!(f, "{}", name)
    }
}

/// A typed problem message produced by a leaf lint check, before path and line
/// information are attached at the aggregation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProblemDetail {
    /// The type of lint error.
    pub error_type: LintErrorType,
    /// The description of the problem.
    pub message: String,
}

impl ProblemDetail {
    /// Creates a new problem detail with an explicit lint error type.
    pub fn new(error_type: LintErrorType, message: impl Into<String>) -> ProblemDetail {
        ProblemDetail {
            error_type,
            message: message.into(),
        }
    }
}

impl fmt::Display for ProblemDetail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
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
    /// Creates a new problem for the given location and typed detail.
    ///
    /// NOTE: The line ID will be incremented by one based on the loop
    /// enumeration where it is used.
    pub fn new(path: &Path, line: Option<usize>, detail: ProblemDetail) -> Problem {
        Problem {
            error_type: detail.error_type,
            path: path.to_path_buf(),
            line: line.map(|l| l + 1),
            message: detail.message,
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
mod problem_tests {
    use super::*;

    #[test]
    fn test_display_with_line() {
        let problem = Problem::new(
            Path::new("CHANGELOG.md"),
            Some(10),
            ProblemDetail::new(
                LintErrorType::Description,
                "PR description should end with a dot: 'Test'",
            ),
        );

        assert_eq!(
            problem.to_string(),
            "CHANGELOG.md:11 [description]: PR description should end with a dot: 'Test'"
        );
    }

    #[test]
    fn test_display_without_line() {
        let problem = Problem::new(
            Path::new("CHANGELOG.md"),
            None,
            ProblemDetail::new(LintErrorType::Duplicate, "duplicate PR: #1"),
        );

        assert_eq!(
            problem.to_string(),
            "CHANGELOG.md [duplicate]: duplicate PR: #1"
        );
    }
}
