use super::git::{self, GitInfo};
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

/// Returns the HTTP status code of a GitHub API error, if the error originated
/// from an API response rather than from the transport or deserialization.
fn github_status_code(error: &octocrab::Error) -> Option<u16> {
    match error {
        octocrab::Error::GitHub { source, .. } => Some(source.status_code.as_u16()),
        _ => None,
    }
}

/// Builds the list of likely reasons why the given repository is not visible
/// with the current credentials.
///
/// GitHub answers with `404 Not Found` instead of `403 Forbidden` for
/// repositories the caller may not see, so the status code alone cannot tell
/// "does not exist" apart from "not allowed to know it exists". The hint
/// therefore spells out the cases that need to be ruled out by hand.
fn access_hint(owner: &str, repo: &str, token_present: bool, origin: Option<&str>) -> String {
    let mut hints = Vec::new();

    if token_present {
        hints.push(
            "the GITHUB_TOKEN is set but may not grant access to this repository: \
             classic tokens need the 'repo' scope, fine-grained tokens need read access \
             to this exact repository (at least 'Contents' and 'Pull requests')"
                .to_string(),
        );
        hints.push(
            "if the repository belongs to an organization with SAML SSO, the token has to be \
             authorized for that organization"
                .to_string(),
        );
    } else {
        hints.push(
            "no GITHUB_TOKEN is set, so the request was sent unauthenticated - \
             private repositories are invisible without a token (export GITHUB_TOKEN=...)"
                .to_string(),
        );
    }

    let canonical = format!("https://github.com/{}/{}", owner, repo);
    match origin {
        Some(origin) if origin != canonical => hints.push(format!(
            "the configured target_repo points to {} while the origin remote points to {} - \
             adjust 'target_repo' in .clconfig.json if these should be the same repository",
            canonical, origin
        )),
        None => hints.push(
            "the origin remote could not be resolved, so 'target_repo' in .clconfig.json \
             could not be cross-checked against it"
                .to_string(),
        ),
        _ => {}
    }

    hints
        .iter()
        .map(|hint| format!("  - {}", hint))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Builds the access hint for the current environment.
fn current_access_hint(owner: &str, repo: &str) -> String {
    let origin = git::get_origin().ok();
    access_hint(
        owner,
        repo,
        read_github_token().is_some(),
        origin.as_deref(),
    )
}

/// Checks that the repository itself can be read with the current credentials.
///
/// This is used to disambiguate a `404` for a specific resource (e.g. a branch
/// that really does not exist yet) from a `404` for the whole repository, which
/// GitHub returns when the credentials do not grant access to it.
pub async fn ensure_repo_is_accessible(
    client: &Octocrab,
    owner: &str,
    repo: &str,
) -> eyre::Result<()> {
    let error = match client.repos(owner, repo).get().await {
        Ok(_) => return Ok(()),
        Err(e) => e,
    };

    let context = match github_status_code(&error) {
        Some(401) => format!(
            "GitHub rejected the provided GITHUB_TOKEN while accessing {}/{} (401 Unauthorized) - \
             the token is invalid, expired or revoked",
            owner, repo
        ),
        Some(403) => format!(
            "Access to {}/{} was forbidden (403) - this is either a rate limit or missing \
             permissions:\n{}",
            owner,
            repo,
            current_access_hint(owner, repo)
        ),
        Some(404) => format!(
            "Repository {}/{} was not found with the current credentials (404). \
             It either does not exist or is not accessible:\n{}",
            owner,
            repo,
            current_access_hint(owner, repo)
        ),
        _ => format!("Failed to verify access to repository {}/{}", owner, repo),
    };

    Err(error).wrap_err(context)
}

/// Wraps an API error with actionable context, checking repository access first
/// when GitHub answered with a status code that can indicate missing permissions.
async fn explain_api_error(
    client: &Octocrab,
    error: octocrab::Error,
    owner: &str,
    repo: &str,
    action: &str,
) -> eyre::Report {
    let context = format!("Failed to {} for {}/{}", action, owner, repo);

    if matches!(github_status_code(&error), Some(401 | 403 | 404)) {
        if let Err(access_error) = ensure_repo_is_accessible(client, owner, repo).await {
            return access_error.wrap_err(context);
        }
    }

    eyre::Report::new(error).wrap_err(context)
}

/// Checks if the given branch exists on the GitHub repository.
///
/// A `404 Not Found` is only interpreted as the branch being absent if the
/// repository itself can be read with the current credentials. Any other error
/// (authentication, network, etc.) is propagated so it is not silently mistaken
/// for a missing branch - that behavior made pushed branches on private
/// repositories appear to never exist.
pub async fn branch_exists_on_remote(client: &Octocrab, git_info: &GitInfo) -> eyre::Result<bool> {
    let error = match client
        .repos(&git_info.owner, &git_info.repo)
        .get_ref(&Branch(git_info.branch.clone()))
        .await
    {
        Ok(_) => return Ok(true),
        Err(e) => e,
    };

    if github_status_code(&error) == Some(404) {
        ensure_repo_is_accessible(client, &git_info.owner, &git_info.repo)
            .await
            .wrap_err_with(|| {
                format!(
                    "Failed to check whether branch '{}' exists",
                    git_info.branch
                )
            })?;

        return Ok(false);
    }

    Err(explain_api_error(
        client,
        error,
        &git_info.owner,
        &git_info.repo,
        &format!("check whether branch '{}' exists", git_info.branch),
    )
    .await)
}

/// Returns the open PR from the current local branch in the configured target
/// repository, or `None` if the repository is accessible but has no such PR.
///
/// Failures to reach or read the repository are returned as errors instead of
/// being reported as "no open PR", which used to hide missing permissions on
/// private repositories.
pub async fn find_open_pr(git_info: &GitInfo) -> eyre::Result<Option<PullRequest>> {
    let octocrab = get_github_client()?;

    let pulls = match octocrab
        .pulls(git_info.owner.to_owned(), git_info.repo.to_owned())
        .list()
        .head(format!("{}:{}", git_info.owner, git_info.branch))
        .send()
        .await
    {
        Ok(page) => page.items,
        Err(e) => {
            return Err(explain_api_error(
                &octocrab,
                e,
                &git_info.owner,
                &git_info.repo,
                "fetch pull requests",
            )
            .await)
        }
    };

    Ok(pulls
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
        .cloned())
}

/// Returns the open PR from the current local branch in the configured target
/// repository and fails if there is none.
pub async fn get_open_pr(git_info: &GitInfo) -> eyre::Result<PullRequest> {
    find_open_pr(git_info).await?.ok_or_else(|| {
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
    match client
        .pulls(&git_info.owner, &git_info.repo)
        .get(pr_number)
        .await
    {
        Ok(pr) => Ok(pr),
        Err(e) => Err(explain_api_error(
            &client,
            e,
            &git_info.owner,
            &git_info.repo,
            &format!("fetch PR #{}", pr_number),
        )
        .await),
    }
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
    match find_open_pr(git_info).await? {
        Some(pr) => extract_pr_info(config, &pr)
            .wrap_err("Failed to extract PR information from pull request"),
        None => Ok(PRInfo::default()),
    }
}

/// Gets all merged PR numbers from the repository's default branch.
/// Returns a sorted, deduplicated list of PR numbers.
pub async fn get_merged_pr_numbers(git_info: &GitInfo) -> eyre::Result<Vec<u64>> {
    let client = get_github_client()?;

    // Get the default branch for the repository
    let repo = match client.repos(&git_info.owner, &git_info.repo).get().await {
        Ok(repo) => repo,
        Err(e) => {
            return Err(explain_api_error(
                &client,
                e,
                &git_info.owner,
                &git_info.repo,
                "fetch repository information",
            )
            .await)
        }
    };

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

#[cfg(test)]
mod access_hint_tests {
    use super::*;

    const OWNER: &str = "MalteHerrmann";
    const REPO: &str = "changelog-utils";
    const ORIGIN: &str = "https://github.com/MalteHerrmann/changelog-utils";

    #[test]
    fn test_hint_without_token() {
        let hint = access_hint(OWNER, REPO, false, Some(ORIGIN));
        assert!(hint.contains("no GITHUB_TOKEN is set"), "got: {}", hint);
        assert!(!hint.contains("SAML SSO"), "got: {}", hint);
    }

    #[test]
    fn test_hint_with_token() {
        let hint = access_hint(OWNER, REPO, true, Some(ORIGIN));
        assert!(hint.contains("'repo' scope"), "got: {}", hint);
        assert!(hint.contains("SAML SSO"), "got: {}", hint);
        assert!(
            !hint.contains("origin remote points to"),
            "matching origin should not be reported, got: {}",
            hint
        );
    }

    #[test]
    fn test_hint_reports_origin_mismatch() {
        let hint = access_hint(
            OWNER,
            REPO,
            true,
            Some("https://github.com/MalteHerrmann/other-repo"),
        );
        assert!(
            hint.contains("origin remote points to https://github.com/MalteHerrmann/other-repo"),
            "got: {}",
            hint
        );
    }

    #[test]
    fn test_hint_without_origin() {
        let hint = access_hint(OWNER, REPO, true, None);
        assert!(
            hint.contains("origin remote could not be resolved"),
            "got: {}",
            hint
        );
    }
}
