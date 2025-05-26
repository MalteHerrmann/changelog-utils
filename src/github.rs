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
fn extract_pr_info(config: &Config, pr: &PullRequest) -> Result<PRInfo, GitHubError> {
    let mut change_type = String::new();
    let mut category = String::new();
    let mut description = String::new();

    let pr_title = pr.title.clone().unwrap_or_default();

    if let Some(i) = RegexBuilder::new(r"^(?P<ct>\w+)?\s*(\((?P<cat>\w+)\))?[:\s]*(?P<desc>.+)$")
        .build()?
        .captures(pr_title.as_str())
    {
        if let Some(ct) = i.name("ct") {
            if let Some(found_ct) = config.get_short_change_type(ct.as_str()) {
                change_type.clone_from(&found_ct.short);
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
    // NOTE: make sure to export the token and not only define using GITHUB_TOKEN=... because Rust executes
    // in a child process, that cannot pick it up without using `export`
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
        .is_ok()
}

/// Returns an option for an open PR from the current local branch in the configured target
/// repository if it exists.
pub async fn get_open_pr(git_info: GitInfo) -> Result<PullRequest, GitHubError> {
    let octocrab = get_authenticated_github_client().unwrap_or_default();

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

/// Commits the current changes with the given commit message and pushes to the origin.
pub fn commit_and_push(config: &Config, message: &str) -> Result<(), GitHubError> {
    stage_changelog_changes(config)?;

    match Command::new("git")
        .args(vec!["commit", "-a", "-m", message])
        .status()?
        .success()
    {
        true => Ok(push()?),
        false => Err(GitHubError::FailedToCommit),
    }
}

/// Commits the current changes with the given commit message and pushes to the origin.
pub fn commit(config: &Config, message: &str) -> Result<(), GitHubError> {
    stage_changelog_changes(config)?;

    if !Command::new("git")
        .args(vec!["commit", "-m", message])
        .status()?
        .success()
    {
        return Err(GitHubError::FailedToCommit);
    }

    Ok(())
}

/// Gets the diff between the two defined branches.
pub fn get_diff(branch: &str, target: &str) -> Result<String, GitHubError> {
    let diff_str = format!("{}...{}", target, branch);
    let out = Command::new("git")
        .args(vec!["diff", diff_str.as_str()])
        .output()?;

    if !out.status.success() {
        return Err(GitHubError::Diff);
    }

    let diff = String::from_utf8(out.stdout)?;
    let diff_trimmed = diff.trim();
    match diff_trimmed.is_empty() {
        true => Err(GitHubError::EmptyDiff(branch.into(), target.into())),
        false => Ok(diff_trimmed.into()),
    }
}

/// Adds the changelog to the staged changes in Git.
fn stage_changelog_changes(config: &Config) -> Result<(), GitHubError> {
    if !Command::new("git")
        .args(vec!["add", config.changelog_path.as_str()])
        .status()?
        .success()
    {
        return Err(GitHubError::FailedToCommit);
    }

    Ok(())
}

/// Tries to push the latest commits on the current branch.
pub fn push() -> Result<(), GitHubError> {
    match Command::new("git").args(vec!["push"]).status()?.success() {
        true => Ok(()),
        false => Err(GitHubError::FailedToPush),
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

/// Returns a PR from the repository by its number.
async fn get_pr_by_number(git_info: &GitInfo, pr_number: u16) -> Result<PullRequest, GitHubError> {
    let client = get_authenticated_github_client()?;
    client
        .pulls(&git_info.owner, &git_info.repo)
        .get(pr_number as u64)
        .await
        .map_err(|_| GitHubError::NoOpenPR)
}

/// Retrieves PR information either from a specific PR number or from an open PR.
/// If a PR number is provided but no PR is found, returns an error.
pub async fn get_pr_info(
    config: &Config,
    git_info: &GitInfo,
    pr_number: Option<u16>,
) -> Result<PRInfo, GitHubError> {
    if let Some(pr_number) = pr_number {
        // Try to fetch PR information using the provided PR number
        let pr = get_pr_by_number(git_info, pr_number).await?;
        return extract_pr_info(config, &pr);
    }

    // If no PR number was provided, try to get open PR for current branch
    if let Ok(pr) = get_open_pr(git_info.clone()).await {
        return extract_pr_info(config, &pr);
    }

    Ok(PRInfo::default())
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
