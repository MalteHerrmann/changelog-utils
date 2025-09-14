use std::path::Path;

/// Used for formatting the problem statements in the changelog.
///
/// NOTE: The line ID will be incremented by one based on the loop enumeration where it is used.
pub fn add_to_problems(
    problems: &mut Vec<String>,
    fp: &Path,
    line: Option<usize>,
    problem: impl Into<String>,
) {
    let added_line = match line {
        Some(l) => format!("{}:{}: {}", fp.to_string_lossy(), l + 1, problem.into()),
        None => format!("{}: {}", fp.to_string_lossy(), problem.into()),
    };
    problems.push(added_line)
}
