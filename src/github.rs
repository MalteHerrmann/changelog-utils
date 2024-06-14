use crate::errors::GitHubError;
use octocrab;
use regex::Regex;
use std::process::Command;
use crate::config::Config;

/// Returns an option for an open PR from the current local branch in the configured target
/// repository if it exists.
pub async fn check_for_open_pr(config: &Config) -> Result<String, GitHubError> {
    let captures = match Regex::new(r"github.com/(?P<owner>[\w-]+)/(?P<repo>[\w-]+)\.*")
        .expect("failed to build regular expression")
        .captures(config.target_repo.as_str()) {
        Some(r) => r,
        None => return Ok("".to_string()),
    };
    let owner = captures.name("owner").unwrap().as_str();
    let repo = captures.name("repo").unwrap().as_str();
    let branch = get_current_local_branch()?;

    let octocrab = octocrab::instance();
    let pulls = octocrab.pulls(owner, repo).list().send().await?.items;
    match pulls
        .iter()
        .find(|pr| pr.head.label.as_ref().is_some_and(
            |l| {
                let branch_parts: Vec<&str> = l.split(":").collect();
                let got_branch = branch_parts.get(1..)
                    .expect("unexpected branch identifier format")
                    .join("/");
                got_branch.eq(branch.as_str())
            })
        )
    {
        Some(pr) => Ok(format!("{}", pr.number)),
        None => Ok("".to_string())
    }
}

/// Retrieves the name of the current branch if the working directory
/// is a Git repository.
fn get_current_local_branch() -> Result<String, GitHubError> {
    let output = Command::new("git")
        .args(vec!["branch", "--show-current"])
        .output()?;

    match output.status.success() {
        true => Ok(String::from_utf8(output.stdout)?.trim().to_string()),
        false => Err(GitHubError::CurrentBranch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_branch() {
        let branch = get_current_local_branch().expect("failed to get current branch");
        assert_ne!(branch, "", "expected non-empty current branch")
    }
}
