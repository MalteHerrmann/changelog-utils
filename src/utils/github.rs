use super::git::GitInfo;
use crate::{
    common::entry::{check_category, check_description},
    config::Config,
};
use eyre::WrapErr;
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
fn extract_pr_info(config: &Config, pr: &PullRequest) -> eyre::Result<PRInfo> {
    let mut change_type = String::new();
    let mut category = String::new();
    let mut description = String::new();

    let pr_title = pr.title.clone().unwrap_or_default();

    let regex = RegexBuilder::new(r"^(?P<ct>\w+)?\s*(\((?P<cat>\w+)\))?[:\s]*(?P<desc>.+)$")
        .build()
        .wrap_err("Failed to compile PR title regex pattern")?;

    if let Some(i) = regex.captures(pr_title.as_str()) {
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

/// Reads the GITHUB_TOKEN from the environment, returning `None` when it is
/// unset or empty.
///
/// The token is trimmed of surrounding whitespace because tools commonly used
/// to inject it (e.g. `GITHUB_TOKEN=$(op read ...)`) can leave a trailing
/// newline or spaces that would otherwise corrupt the `Authorization` header
/// and make GitHub respond with a misleading `404 Not Found` on private repos.
fn read_github_token() -> Option<String> {
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) if !token.trim().is_empty() => Some(token.trim().to_string()),
        _ => None,
    }
}

/// Returns an authenticated Octocrab instance, requiring a GITHUB_TOKEN.
pub fn get_authenticated_github_client() -> eyre::Result<Octocrab> {
    let token = read_github_token().ok_or_else(|| {
        eyre::eyre!(
            "GITHUB_TOKEN environment variable not found or empty - set it with: export GITHUB_TOKEN=your_token"
        )
    })?;

    octocrab::OctocrabBuilder::new()
        .personal_token(token)
        .build()
        .wrap_err("Failed to build authenticated GitHub client")
}

/// Returns a GitHub client, authenticated if GITHUB_TOKEN is available, otherwise unauthenticated.
///
/// Unlike a silent fallback, a build failure while a token *is* present is
/// surfaced as an error instead of degrading to an unauthenticated client,
/// which would otherwise make private repositories appear to not exist.
/// Note: Unauthenticated clients have lower rate limits and cannot access
/// private repositories.
pub fn get_github_client() -> eyre::Result<Octocrab> {
    match read_github_token() {
        Some(token) => octocrab::OctocrabBuilder::new()
            .personal_token(token)
            .build()
            .wrap_err("Failed to build authenticated GitHub client"),
        None => Ok(Octocrab::default()),
    }
}

/// Checks if the given branch exists on the GitHub repository.
///
/// A `404 Not Found` is interpreted as the branch being absent, while any other
/// error (authentication, network, etc.) is propagated so it is not silently
/// mistaken for a missing branch - the previous behavior made pushed branches
/// on private repositories appear to never exist.
pub async fn branch_exists_on_remote(
    client: &Octocrab,
    git_info: &GitInfo,
) -> eyre::Result<bool> {
    match client
        .repos(&git_info.owner, &git_info.repo)
        .get_ref(&Branch(git_info.branch.clone()))
        .await
    {
        Ok(_) => Ok(true),
        Err(octocrab::Error::GitHub { source, .. })
            if source.status_code.as_u16() == 404 =>
        {
            Ok(false)
        }
        Err(e) => Err(e).wrap_err_with(|| {
            format!(
                "Failed to check whether branch '{}' exists in {}/{} - \
                 ensure GITHUB_TOKEN is set and has access to this (private) repository",
                git_info.branch, git_info.owner, git_info.repo
            )
        }),
    }
}

/// Returns an option for an open PR from the current local branch in the configured target
/// repository if it exists.
pub async fn get_open_pr(git_info: &GitInfo) -> eyre::Result<PullRequest> {
    let octocrab = get_github_client()?;

    let pulls = octocrab
        .pulls(git_info.owner.to_owned(), git_info.repo.to_owned())
        .list()
        .send()
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to fetch pull requests from {}/{}",
                git_info.owner, git_info.repo
            )
        })?
        .items;

    pulls
        .iter()
        .find(|pr| {
            pr.head.label.as_ref().is_some_and(|l| {
                let branch_parts: Vec<&str> = l.split(':').collect();
                let got_branch = branch_parts
                    .get(1..)
                    .expect("unexpected branch identifier format")
                    .join("/");
                got_branch.eq(git_info.branch.as_str())
            })
        })
        .cloned()
        .ok_or_else(|| {
            eyre::eyre!(
                "No open pull request found for branch '{}' in repository {}/{}",
                git_info.branch,
                git_info.owner,
                git_info.repo
            )
        })
}

/// Returns a PR from the repository by its number.
async fn get_pr_by_number(git_info: &GitInfo, pr_number: u64) -> eyre::Result<PullRequest> {
    let client = get_github_client()?;
    client
        .pulls(&git_info.owner, &git_info.repo)
        .get(pr_number)
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to fetch PR #{} from {}/{} - verify the PR exists and GITHUB_TOKEN has correct permissions",
                pr_number, git_info.owner, git_info.repo
            )
        })
}

/// Retrieves PR information either from a specific PR number or from an open PR.
/// If a PR number is provided but no PR is found, returns an error.
pub async fn get_pr_info(
    config: &Config,
    git_info: &GitInfo,
    pr_number: Option<u64>,
) -> eyre::Result<PRInfo> {
    if let Some(pr_number) = pr_number {
        // Try to fetch PR information using the provided PR number
        let pr = get_pr_by_number(git_info, pr_number)
            .await
            .wrap_err_with(|| format!("Failed to fetch information for PR #{}", pr_number))?;
        return extract_pr_info(config, &pr)
            .wrap_err("Failed to extract PR information from pull request");
    }

    // If no PR number was provided, try to get open PR for current branch
    if let Ok(pr) = get_open_pr(git_info).await {
        return extract_pr_info(config, &pr)
            .wrap_err("Failed to extract PR information from pull request");
    }

    Ok(PRInfo::default())
}

/// Gets all merged PR numbers from the repository's default branch.
/// Returns a sorted, deduplicated list of PR numbers.
pub async fn get_merged_pr_numbers(git_info: &GitInfo) -> eyre::Result<Vec<u64>> {
    let client = get_github_client()?;

    // Get the default branch for the repository
    let repo = client
        .repos(&git_info.owner, &git_info.repo)
        .get()
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to fetch repository information for {}/{}",
                git_info.owner, git_info.repo
            )
        })?;

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
            .await
            .wrap_err_with(|| {
                format!(
                    "Failed to fetch merged PRs from {}/{} (page {})",
                    git_info.owner, git_info.repo, page
                )
            })?;

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
