use crate::entry::check_category;
use crate::errors::GitHubError;
use crate::{config::Config, entry::check_description};
use octocrab::models::pulls::PullRequest;
use octocrab::params::repos::Reference::Branch;
use octocrab::{self, Octocrab};
use regex::{Regex, RegexBuilder};
use std::process::Command;

/// Holds the relevant information for a given PR.
#[derive(Default)]
pub struct PRInfo {
    pub change_type: String,
    pub category: String,
    pub description: String,
    pub number: u16,
}

/// Extracts the pull request information from the given
/// instance.
pub fn extract_pr_info(config: &Config, pr: &PullRequest) -> Result<PRInfo, GitHubError> {
    let mut change_type = String::new();
    let mut category = String::new();
    let mut description = String::new();

    let pr_title = pr.title.clone().unwrap_or("".to_string());

    if let Some(i) = RegexBuilder::new(r"^(?P<ct>\w+)?\s*(\((?P<cat>\w+)\))?[:\s]*(?P<desc>.+)$")
        .build()?
        .captures(pr_title.as_str())
    {
        if let Some(ct) = i.name("ct") {
            if let Some((name, _)) = config
                .change_types
                .iter()
                .find(|&(_, abbrev)| abbrev.eq(ct.into()))
            {
                change_type.clone_from(name);
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
        number: pr
            .number
            .try_into()
            .expect("failed to convert PR number to u16"),
        change_type,
        category,
        description,
    })
}

/// Returns an authenticated Octocrab instance if possible.
pub fn get_authenticated_github_client() -> Result<Octocrab, GitHubError> {
    let token = std::env::var("GITHUB_TOKEN")?;

    Ok(octocrab::OctocrabBuilder::new()
        .personal_token(token)
        .build()?)
}

/// Checks if the given branch exists on the GitHub repository.
pub async fn branch_exists_on_remote(client: &Octocrab, git_info: &GitInfo) -> bool {
    client
        .repos(&git_info.owner, &git_info.repo)
        .get_ref(&Branch(git_info.branch.clone()))
        .await
        .is_err()
}

/// Returns an option for an open PR from the current local branch in the configured target
/// repository if it exists.
pub async fn get_open_pr(git_info: GitInfo) -> Result<PullRequest, GitHubError> {
    let octocrab = match get_authenticated_github_client() {
        Ok(oc) => oc,
        _ => octocrab::Octocrab::default(),
    };

    let pulls = octocrab
        .pulls(git_info.owner, git_info.repo)
        .list()
        .send()
        .await?
        .items;
    match pulls.iter().find(|pr| {
        pr.head.label.as_ref().is_some_and(|l| {
            let branch_parts: Vec<&str> = l.split(':').collect();
            let got_branch = branch_parts
                .get(1..)
                .expect("unexpected branch identifier format")
                .join("/");
            got_branch.eq(git_info.branch.as_str())
        })
    }) {
        Some(pr) => Ok(pr.to_owned()),
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

/// Tries to push the current branch to the origin repository.
pub fn push_to_origin(branch_name: &str) -> Result<(), GitHubError> {
    match Command::new("git")
        .args(vec!["push", "-u", "origin", branch_name])
        .status()?
        .success()
    {
        true => Ok(()),
        false => Err(GitHubError::FailedToPush),
    }
}

/// Checks if there is a origin repository defined and returns the name
/// if that's the case.
pub fn get_origin() -> Result<String, GitHubError> {
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        return Err(GitHubError::Origin);
    };

    let origin = String::from_utf8(output.stdout)?;
    match Regex::new(r"(https://github.com/[^.\s]+/[^.\s]+)(\.git)?")?.captures(origin.as_str()) {
        Some(o) => Ok(o
            .get(1)
            .expect("unexpected matching condition")
            .as_str()
            .to_string()),
        None => Err(GitHubError::RegexMatch(origin)),
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
pub fn get_git_info(config: &Config) -> Result<GitInfo, GitHubError> {
    let captures = match Regex::new(r"github.com/(?P<owner>[\w-]+)/(?P<repo>[\w-]+)\.*")
        .expect("failed to build regular expression")
        .captures(config.target_repo.as_str())
    {
        Some(r) => r,
        None => return Err(GitHubError::NoGitHubRepo),
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
