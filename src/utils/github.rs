use super::git::GitInfo;
use crate::{
    common::entry::{check_category, check_description},
    config::Config,
    errors::GitHubError,
};
use octocrab::models::pulls::PullRequest;
use octocrab::params::repos::Reference::Branch;
use octocrab::{self, Octocrab};
use regex::RegexBuilder;

/// Holds the relevant information for a given PR.
#[derive(Default)]
pub struct PRInfo {
    pub change_type: String,
    pub category: String,
    pub description: String,
    pub number: u64,
}

/// Extracts the pull request information from the given
/// instance.
///
/// TODO: instead of relying on the single file checker here, it should use some common utils?
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
        number: pr.number,
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

/// Returns a GitHub client, authenticated if GITHUB_TOKEN is available, otherwise unauthenticated.
/// Note: Unauthenticated clients have lower rate limits than authenticated ones.
pub fn get_github_client() -> Octocrab {
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) => octocrab::OctocrabBuilder::new()
            .personal_token(token)
            .build()
            .unwrap_or_default(),
        Err(_) => {
            // No token available, use unauthenticated client
            Octocrab::default()
        }
    }
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
pub async fn get_open_pr(git_info: &GitInfo) -> Result<PullRequest, GitHubError> {
    let octocrab = get_github_client();

    let pulls = octocrab
        .pulls(git_info.owner.to_owned(), git_info.repo.to_owned())
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

/// Returns a PR from the repository by its number.
async fn get_pr_by_number(git_info: &GitInfo, pr_number: u64) -> Result<PullRequest, GitHubError> {
    let client = get_github_client();
    client
        .pulls(&git_info.owner, &git_info.repo)
        .get(pr_number)
        .await
        .map_err(|_| GitHubError::NoOpenPR)
}

/// Retrieves PR information either from a specific PR number or from an open PR.
/// If a PR number is provided but no PR is found, returns an error.
pub async fn get_pr_info(
    config: &Config,
    git_info: &GitInfo,
    pr_number: Option<u64>,
) -> Result<PRInfo, GitHubError> {
    if let Some(pr_number) = pr_number {
        // Try to fetch PR information using the provided PR number
        let pr = get_pr_by_number(git_info, pr_number).await?;
        return extract_pr_info(config, &pr);
    }

    // If no PR number was provided, try to get open PR for current branch
    if let Ok(pr) = get_open_pr(git_info).await {
        return extract_pr_info(config, &pr);
    }

    Ok(PRInfo::default())
}

/// Gets all merged PR numbers from the repository's default branch.
/// Returns a sorted, deduplicated list of PR numbers.
pub async fn get_merged_pr_numbers(git_info: &GitInfo) -> Result<Vec<u64>, GitHubError> {
    let client = get_github_client();

    // Get the default branch for the repository
    let repo = client.repos(&git_info.owner, &git_info.repo).get().await?;

    let default_branch = repo.default_branch.unwrap_or_else(|| "main".to_string());

    let mut pr_numbers = Vec::new();
    let mut page = 1u32;

    loop {
        let pulls = client
            .pulls(&git_info.owner, &git_info.repo)
            .list()
            .state(octocrab::params::State::Closed)
            .base(&default_branch)
            .per_page(100)
            .page(page)
            .send()
            .await?;

        if pulls.items.is_empty() {
            break;
        }

        for pr in pulls.items {
            // Only include PRs that were actually merged
            if pr.merged_at.is_some() {
                pr_numbers.push(pr.number);
            }
        }

        page += 1;
    }

    // Sort and deduplicate
    pr_numbers.sort_unstable();
    pr_numbers.dedup();

    Ok(pr_numbers)
}
