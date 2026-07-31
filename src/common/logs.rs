use crate::common::problem::Problem;
use std::path::Path;

/// Used for collecting the problem statements found in the changelog.
///
/// NOTE: The line ID will be incremented by one based on the loop enumeration where it is used.
pub fn add_to_problems(
    problems: &mut Vec<Problem>,
    fp: &Path,
    line: Option<usize>,
    problem: impl Into<String>,
) {
    problems.push(Problem::new(fp, line, problem))
}
