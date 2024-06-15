use crate::entry::check_category;
use crate::errors::GitHubError;
use crate::{config::Config, entry::check_description};
use octocrab;
use octocrab::models::pulls::PullRequest;
use regex::{Regex, RegexBuilder};
use std::process::Command;

/// Holds the relevant information for a given PR.
pub struct PRInfo {
    pub change_type: String,
    pub category: String,
    pub description: String,
    pub number: String,
}

/// Extracts the pull request information from the given
/// instance.
fn extract_pr_info(config: &Config, pr: &PullRequest) -> Result<PRInfo, GitHubError> {
    let mut change_type = String::new();
    let mut category = String::new();
    let mut description = String::new();

    let pr_title = pr.title.clone().unwrap_or("".to_string());

    if let Some(i) = RegexBuilder::new(r"^(?P<ct>\w+)?\s*(\((?P<cat>\w+)\))?[:\s]*(?P<desc>.+)$")
        .build()?
        .captures(pr_title.as_str())
    {
        if let Some(ct) = i.name("ct") {
            match ct.as_str() {
                "fix" => change_type = "Bug Fixes".to_string(),
                "imp" => change_type = "Improvements".to_string(),
                "feat" => change_type = "Features".to_string(),
                _ => (),
            }
        };

        if let Some(cat) = i.name("cat") {
            (category, _) = check_category(config, cat.as_str());
        };

        if let Some(desc) = i.name("desc") {
            (description, _) = check_description(config, desc.as_str());
        };
    };

    Ok(PRInfo {
        number: format!("{}", pr.number),
        change_type,
        category,
        description,
    })
}

/// Returns an option for an open PR from the current local branch in the configured target
/// repository if it exists.
pub async fn get_open_pr(config: &Config) -> Result<PRInfo, GitHubError> {
    let captures = match Regex::new(r"github.com/(?P<owner>[\w-]+)/(?P<repo>[\w-]+)\.*")
        .expect("failed to build regular expression")
        .captures(config.target_repo.as_str())
    {
        Some(r) => r,
        None => return Err(GitHubError::NoGitHubRepo),
    };

    let owner = captures.name("owner").unwrap().as_str();
    let repo = captures.name("repo").unwrap().as_str();
    let branch = get_current_local_branch()?;

    let octocrab = octocrab::instance();
    let pulls = octocrab.pulls(owner, repo).list().send().await?.items;
    match pulls.iter().find(|pr| {
        pr.head.label.as_ref().is_some_and(|l| {
            let branch_parts: Vec<&str> = l.split(':').collect();
            let got_branch = branch_parts
                .get(1..)
                .expect("unexpected branch identifier format")
                .join("/");
            got_branch.eq(branch.as_str())
        })
    }) {
        Some(pr) => Ok(extract_pr_info(config, pr)?),
        None => Err(GitHubError::NoOpenPR),
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
        false => Err(GitHubError::CurrentBranch),
    }
}

// Ignore these tests when running on CI because there won't be a local branch
#[cfg(not(feature = "remote"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_branch() {
        let branch = get_current_local_branch().expect("failed to get current branch");
        assert_ne!(branch, "", "expected non-empty current branch")
    }
}
