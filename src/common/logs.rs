use std::path::Path;

/// Used for formatting the problem statements in the changelog.
///
/// NOTE: The line ID will be incremented by one based on the loop enumeration where it is used.
pub fn add_to_problems(
    problems: &mut Vec<String>,
    fp: &Path,
    line: usize,
    problem: impl Into<String>,
) {
    problems.push(format!(
        "{}:{}: {}",
        fp.to_string_lossy(),
        line + 1,
        problem.into()
    ))
}
