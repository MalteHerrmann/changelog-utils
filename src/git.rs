use crate::config::Config;
use crate::errors::GitError;
use regex::Regex;
use std::process::Command;

/// Retrieves the name of the current branch if the working directory
/// is a Git repository.
pub fn get_current_local_branch() -> Result<String, GitError> {
    let output = Command::new("git")
        .args(vec!["branch", "--show-current"])
        .output()?;

    match output.status.success() {
        true => Ok(String::from_utf8(output.stdout)?.trim().to_string()),
        false => Err(GitError::CurrentBranch),
    }
}

/// Commits the current changes with the given commit message and pushes to the origin.
pub fn commit_and_push(config: &Config, message: &str) -> Result<(), GitError> {
    stage_changelog_changes(config)?;

    match Command::new("git")
        .args(vec!["commit", "-a", "-m", message])
        .status()?
        .success()
    {
        true => Ok(push()?),
        false => Err(GitError::FailedToCommit),
    }
}

/// Commits the current changes with the given commit message.
pub fn commit(config: &Config, message: &str) -> Result<(), GitError> {
    stage_changelog_changes(config)?;

    if !Command::new("git")
        .args(vec!["commit", "-m", message])
        .status()?
        .success()
    {
        return Err(GitError::FailedToCommit);
    }

    Ok(())
}

/// Gets the diff between the two defined branches.
pub fn get_diff(branch: &str, target: &str) -> Result<String, GitError> {
    let diff_str = format!("{}...{}", target, branch);
    let out = Command::new("git")
        .args(vec!["diff", diff_str.as_str()])
        .output()?;

    if !out.status.success() {
        return Err(GitError::Diff);
    }

    let diff = String::from_utf8(out.stdout)?;
    let diff_trimmed = diff.trim();
    match diff_trimmed.is_empty() {
        true => Err(GitError::EmptyDiff(branch.into(), target.into())),
        false => Ok(diff_trimmed.into()),
    }
}

/// Adds the changelog to the staged changes in Git.
fn stage_changelog_changes(config: &Config) -> Result<(), GitError> {
    if !Command::new("git")
        .args(vec!["add", config.changelog_path.as_str()])
        .status()?
        .success()
    {
        return Err(GitError::FailedToCommit);
    }

    Ok(())
}

/// Tries to push the latest commits on the current branch.
pub fn push() -> Result<(), GitError> {
    match Command::new("git").args(vec!["push"]).status()?.success() {
        true => Ok(()),
        false => Err(GitError::FailedToPush),
    }
}

/// Tries to push the current branch to the origin repository.
pub fn push_to_origin(branch_name: &str) -> Result<(), GitError> {
    match Command::new("git")
        .args(vec!["push", "-u", "origin", branch_name])
        .status()?
        .success()
    {
        true => Ok(()),
        false => Err(GitError::FailedToPush),
    }
}

/// Checks if there is a origin repository defined and returns the name
/// if that's the case.
pub fn get_origin() -> Result<String, GitError> {
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        return Err(GitError::Origin);
    };

    let origin = String::from_utf8(output.stdout)?;
    match Regex::new(r"(https://github.com/[^.\s]+/[^.\s]+)(\.git)?")?.captures(origin.as_str()) {
        Some(o) => Ok(o
            .get(1)
            .expect("unexpected matching condition")
            .as_str()
            .to_string()),
        None => Err(GitError::RegexMatch(origin)),
    }
}

/// Holds the relevant information for the Git configuration.
#[derive(Clone)]
pub struct GitInfo {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

/// Retrieves the Git information like the currently checked out branch and
/// repository owner and name.
pub fn get_git_info(config: &Config) -> Result<GitInfo, GitError> {
    let captures = match Regex::new(r"github.com/(?P<owner>[\w-]+)/(?P<repo>[\w-]+)\.*")
        .expect("failed to build regular expression")
        .captures(config.target_repo.as_str())
    {
        Some(r) => r,
        None => return Err(GitError::RegexMatch(config.target_repo.clone())),
    };

    let owner = captures.name("owner").unwrap().as_str().to_string();
    let repo = captures.name("repo").unwrap().as_str().to_string();
    let branch = get_current_local_branch()?;

    Ok(GitInfo {
        owner,
        repo,
        branch,
    })
}

// Ignore these tests when running on CI because there won't be a local branch
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "remote"))]
    #[test]
    fn test_current_branch() {
        let branch = get_current_local_branch().expect("failed to get current branch");
        assert_ne!(branch, "", "expected non-empty current branch")
    }

    #[test]
    fn test_get_origin() {
        let origin = get_origin().expect("failed to get origin");
        assert_eq!(
            origin, "https://github.com/MalteHerrmann/changelog-utils",
            "expected different origin"
        )
    }
}