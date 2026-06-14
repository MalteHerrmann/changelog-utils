use crate::config::Config;
use eyre::{ensure, WrapErr};
use regex::Regex;
use std::process::Command;

/// Retrieves the name of the current branch if the working directory
/// is a Git repository.
pub fn get_current_local_branch() -> eyre::Result<String> {
    let output = Command::new("git")
        .args(vec!["branch", "--show-current"])
        .output()
        .wrap_err("Failed to execute git command to get current branch")?;

    ensure!(
        output.status.success(),
        "Git command failed - ensure you're in a git repository"
    );

    let branch = String::from_utf8(output.stdout)
        .wrap_err("Failed to parse git branch name as UTF-8")?
        .trim()
        .to_string();

    Ok(branch)
}

/// Commits the current changes with the given commit message and pushes to the origin.
pub fn commit_and_push(config: &Config, message: &str) -> eyre::Result<()> {
    stage_changelog_changes(config)
        .wrap_err("Failed to stage changelog changes")?;

    let status = Command::new("git")
        .args(vec!["commit", "-a", "-m", message])
        .status()
        .wrap_err("Failed to execute git commit command")?;

    ensure!(
        status.success(),
        "Git commit failed - check for uncommitted changes or pre-commit hooks"
    );

    push().wrap_err("Failed to push commits to remote")?;
    Ok(())
}

/// Commits the current changes with the given commit message.
pub fn commit(config: &Config, message: &str) -> eyre::Result<()> {
    stage_changelog_changes(config)
        .wrap_err("Failed to stage changelog changes")?;

    let status = Command::new("git")
        .args(vec!["commit", "-m", message])
        .status()
        .wrap_err("Failed to execute git commit command")?;

    ensure!(
        status.success(),
        "Git commit failed - check for uncommitted changes or pre-commit hooks"
    );

    Ok(())
}

/// Extracts the added lines from the git diff.
pub fn get_additions(diff: &str) -> Vec<String> {
    let addition_prefix = "+";

    diff.lines()
        .filter_map(|l| l.strip_prefix(addition_prefix))
        .map(|l| l.to_string())
        .collect()
}

/// Gets the diff between the two defined branches.
pub fn get_diff(branch: &str, target: &str) -> eyre::Result<String> {
    let diff_str = format!("{}...{}", target, branch);
    let out = Command::new("git")
        .args(vec!["diff", diff_str.as_str()])
        .output()
        .wrap_err_with(|| {
            format!(
                "Failed to execute git diff between '{}' and '{}'",
                target, branch
            )
        })?;

    ensure!(
        out.status.success(),
        "Git diff command failed - ensure you're in a git repository with valid branches"
    );

    let diff = String::from_utf8(out.stdout)
        .wrap_err("Failed to parse git diff output as UTF-8")?;
    let diff_trimmed = diff.trim();

    ensure!(
        !diff_trimmed.is_empty(),
        "Empty diff found between '{}' and '{}' - branches may be in sync",
        branch,
        target
    );

    Ok(diff_trimmed.into())
}

/// Adds the changelog to the staged changes in Git.
fn stage_changelog_changes(config: &Config) -> eyre::Result<()> {
    let status = Command::new("git")
        .args(vec!["add", config.changelog_path.as_str()])
        .status()
        .wrap_err_with(|| {
            format!(
                "Failed to execute git add for changelog at '{}'",
                config.changelog_path
            )
        })?;

    ensure!(
        status.success(),
        "Git add failed for changelog file at '{}' - ensure file exists and is in a git repository",
        config.changelog_path
    );

    Ok(())
}

/// Tries to push the latest commits on the current branch.
pub fn push() -> eyre::Result<()> {
    let status = Command::new("git")
        .args(vec!["push"])
        .status()
        .wrap_err("Failed to execute git push command")?;

    ensure!(
        status.success(),
        "Git push failed - ensure you have push access and the remote branch is set up"
    );

    Ok(())
}

/// Tries to push the current branch to the origin repository.
pub fn push_to_origin(branch_name: &str) -> eyre::Result<()> {
    let status = Command::new("git")
        .args(vec!["push", "-u", "origin", branch_name])
        .status()
        .wrap_err_with(|| format!("Failed to execute git push for branch '{}'", branch_name))?;

    ensure!(
        status.success(),
        "Git push failed for branch '{}' - ensure you have push access to the remote repository",
        branch_name
    );

    Ok(())
}

/// Parses the owner and repository name from a GitHub URL or git remote.
///
/// This intentionally supports the most common URL forms so that private
/// repositories cloned via SSH work just as well as ones cloned via HTTPS:
///   - `https://github.com/owner/repo` (optionally suffixed with `.git`)
///   - `git@github.com:owner/repo.git` (SCP-like SSH syntax)
///   - `ssh://git@github.com/owner/repo.git`
///   - `github.com/owner/repo`
pub fn parse_github_owner_repo(url: &str) -> eyre::Result<(String, String)> {
    let regex = Regex::new(
        r"(?:https?://|ssh://)?(?:git@)?github\.com[/:](?P<owner>[\w.-]+)/(?P<repo>[\w.-]+?)(?:\.git)?/?$",
    )
    .wrap_err("Failed to compile GitHub URL regex pattern")?;

    let trimmed = url.trim();
    let captures = regex.captures(trimmed).ok_or_else(|| {
        eyre::eyre!(
            "'{}' does not match a supported GitHub URL format \
             (e.g. https://github.com/owner/repo or git@github.com:owner/repo)",
            trimmed
        )
    })?;

    let owner = captures
        .name("owner")
        .expect("regex should have owner capture group")
        .as_str()
        .to_string();
    let repo = captures
        .name("repo")
        .expect("regex should have repo capture group")
        .as_str()
        .to_string();

    Ok((owner, repo))
}

/// Checks if there is a origin repository defined and returns the canonical
/// `https://github.com/owner/repo` URL if that's the case.
///
/// The origin remote may use either HTTPS or SSH; both are normalized to the
/// canonical HTTPS form used for `target_repo` and for building PR/release links.
pub fn get_origin() -> eyre::Result<String> {
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .output()
        .wrap_err("Failed to execute git remote get-url command")?;

    ensure!(
        output.status.success(),
        "Failed to get git origin - ensure a remote origin is configured"
    );

    let origin = String::from_utf8(output.stdout)
        .wrap_err("Failed to parse git origin URL as UTF-8")?;

    let (owner, repo) = parse_github_owner_repo(&origin)
        .wrap_err("Failed to parse GitHub owner/repo from the origin remote URL")?;

    Ok(format!("https://github.com/{}/{}", owner, repo))
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
pub fn get_git_info(config: &Config) -> eyre::Result<GitInfo> {
    let (owner, repo) = parse_github_owner_repo(&config.target_repo).wrap_err_with(|| {
        format!(
            "Failed to parse owner/repo from target repository '{}'",
            config.target_repo
        )
    })?;

    let branch = get_current_local_branch()
        .wrap_err("Failed to get current git branch")?;

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

    #[test]
    fn test_parse_github_owner_repo() {
        let cases = vec![
            "https://github.com/MalteHerrmann/changelog-utils",
            "https://github.com/MalteHerrmann/changelog-utils.git",
            "https://github.com/MalteHerrmann/changelog-utils/",
            "git@github.com:MalteHerrmann/changelog-utils.git",
            "git@github.com:MalteHerrmann/changelog-utils",
            "ssh://git@github.com/MalteHerrmann/changelog-utils.git",
            "github.com/MalteHerrmann/changelog-utils",
            "  git@github.com:MalteHerrmann/changelog-utils.git\n",
        ];

        for case in cases {
            let (owner, repo) =
                parse_github_owner_repo(case).unwrap_or_else(|_| panic!("failed to parse '{}'", case));
            assert_eq!(owner, "MalteHerrmann", "unexpected owner for '{}'", case);
            assert_eq!(repo, "changelog-utils", "unexpected repo for '{}'", case);
        }
    }

    #[test]
    fn test_parse_github_owner_repo_with_dots() {
        let (owner, repo) = parse_github_owner_repo("git@github.com:noble-assets/my.repo.git")
            .expect("failed to parse repo name containing dots");
        assert_eq!(owner, "noble-assets");
        assert_eq!(repo, "my.repo");
    }

    #[test]
    fn test_parse_github_owner_repo_invalid() {
        assert!(
            parse_github_owner_repo("https://gitlab.com/owner/repo").is_err(),
            "expected non-GitHub URL to fail parsing"
        );
    }
}
