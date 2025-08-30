use inquire::InquireError;
use regex::Error;
use rig::completion::PromptError;
use serde_json;
use std::{env::VarError, io, num::ParseIntError, string::FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckDiffError {
    #[error("failed to get changelog: {0}")]
    Changelog(#[from] ChangelogError),
    #[error("failed to get config: {0}")]
    Config(#[from] ConfigError),
    #[error("failed to get git info: {0}")]
    Git(#[from] GitError),
    #[error("failed to interact with github: {0}")]
    GitHub(#[from] GitHubError),
    #[error("no unreleased entry found for pr")]
    NoEntry,
    #[error("no unreleased section in changelog")]
    NoUnreleased,
}

#[derive(Error, Debug)]
pub enum CLIError {
    #[error("failed to add changelog entry: {0}")]
    AddError(#[from] AddError),
    #[error("failed to check diff: {0}")]
    CheckDiff(#[from] CheckDiffError),
    #[error("failed to create pr: {0}")]
    CreateError(#[from] CreateError),
    #[error("failed to get release contents: {0}")]
    GetError(#[from] GetError),
    #[error("failed to initialize the changelog settings: {0}")]
    InitError(#[from] InitError),
    #[error("failed to run linter: {0}")]
    LintError(#[from] LintError),
    #[error("failed to read configuration: {0}")]
    Config(#[from] ConfigError),
    #[error("failed to adjust configuration: {0}")]
    ConfigAdjustment(#[from] ConfigAdjustError),
    #[error("failed to read/write: {0}")]
    IOError(#[from] io::Error),
    #[error("failed to create new release in changelog: {0}")]
    ReleaseCLIError(#[from] ReleaseCLIError),
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("branch not found on remote: {0}")]
    BranchNotOnRemote(String),
    #[error("changelog error: {0}")]
    Changelog(#[from] ChangelogError),
    #[error("failed to read configuration: {0}")]
    Config(#[from] ConfigError),
    #[error("found an existing PR for this branch: {0}")]
    ExistingPR(u64),
    #[error("failed to create PR: {0}")]
    FailedToCreatePR(#[from] octocrab::Error),
    #[error("failed to match: {0}")]
    FailedToMatch(String),
    #[error("failed to parse llm suggestions: {0}")]
    FailedToParse(#[from] serde_json::Error),
    #[error("error interacting with Git: {0}")]
    Git(#[from] GitError),
    #[error("error interacting with GitHub: {0}")]
    GitHub(#[from] GitHubError),
    #[error("error getting user input: {0}")]
    Input(#[from] InputError),
    #[error("failed to prompt llm: {0}")]
    Prompt(#[from] PromptError),
}

#[derive(Error, Debug)]
pub enum InputError {
    #[error("failed to prompt user: {0}")]
    InquireError(#[from] InquireError),
    #[error("failed to parse integer: {0}")]
    ParseError(#[from] ParseIntError),
    #[error("invalid selection")]
    InvalidSelection,
}

#[derive(Error, Debug)]
pub enum AddError {
    #[error("failed to load config: {0}")]
    Config(#[from] ConfigError),
    #[error("failed to get user input: {0}")]
    Input(#[from] InputError),
    #[error("first release is not unreleased section: {0}")]
    FirstReleaseNotUnreleased(String),
    #[error("failed to get git information: {0}")]
    Git(#[from] GitError),
    #[error("failed to get pull request information: {0}")]
    PRInfo(#[from] GitHubError),
    #[error("failed to parse changelog: {0}")]
    InvalidChangelog(#[from] ChangelogError),
    #[error("failed to read/write: {0}")]
    ReadWriteError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum InitError {
    #[error("failed to write: {0}")]
    FailedToWrite(#[from] io::Error),
    #[error("config already created")]
    ConfigAlreadyFound,
    #[error("error exporting config: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("failed to get origin")]
    OriginError(#[from] GitError),
}

#[derive(Error, Debug)]
pub enum LintError {
    #[error("failed to parse changelog: {0}")]
    InvalidChangelog(#[from] ChangelogError),
    #[error("invalid configuration: {0}")]
    InvalidConfig(#[from] ConfigError),
    #[error("found problems in changelog")]
    ProblemsInChangelog,
    #[error("failed to read file system: {0}")]
    Read(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ChangelogError {
    #[error("failed to parse change type: {0}")]
    InvalidChangeType(#[from] ChangeTypeError),
    #[error("failed to parse entry: {0}")]
    InvalidEntry(#[from] EntryError),
    #[error("failed to build regex: {0}")]
    InvalidRegex(#[from] Error),
    #[error("failed to parse release: {0}")]
    InvalidRelease(#[from] ReleaseError),
    #[error("invalid version: {0}")]
    InvalidVersion(#[from] VersionError),
    #[error("failed to find changelog in directory")]
    NoChangelogFound,
    #[error("failed to parse changelog: {0}")]
    Parse(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum EntryError {
    #[error("invalid entry: {0}")]
    InvalidEntry(String),
}

#[derive(Error, Debug)]
pub enum GetError {
    #[error("failed to get config: {0}")]
    Config(#[from] ConfigError),
    #[error("failed to get changelog: {0}")]
    Changelog(#[from] ChangelogError),
    #[error("version not found: {0}")]
    VersionNotFound(String),
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("failed to get current branch")]
    CurrentBranch,
    #[error("failed to get diff")]
    Diff,
    #[error("empty diff found between '{0}' and '{1}'")]
    EmptyDiff(String, String),
    #[error("failed to commit changes")]
    FailedToCommit,
    #[error("failed to push to origin")]
    FailedToPush,
    #[error("failed to build regex: {0}")]
    InvalidRegex(#[from] Error),
    #[error("failed to get origin")]
    Origin,
    #[error("failed to decode output: {0}")]
    OutputDecoding(#[from] FromUtf8Error),
    #[error("failed to match GitHub repo: {0}")]
    RegexMatch(String),
    #[error("failed to execute command: {0}")]
    StdCommand(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("failed to call GitHub API: {0}")]
    GitHub(#[from] octocrab::Error),
    #[error("failed to build regex: {0}")]
    InvalidRegex(#[from] Error),
    #[error("target repository in configuration is no GitHub repository")]
    NoGitHubRepo,
    #[error("no pull request open for branch")]
    NoOpenPR,
    #[error("failed to decode output: {0}")]
    OutputDecoding(#[from] FromUtf8Error),
    #[error("GITHUB_TOKEN environment variable not found")]
    Token(#[from] VarError),
}

#[derive(Error, Debug, PartialEq)]
pub enum MatchError {
    #[error("match is nested inside of code block")]
    MatchInCodeblock,
    #[error("invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("no match found")]
    NoMatchFound,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read/write configuration: {0}")]
    FailedToReadWrite(#[from] io::Error),
    #[error("failed to parse configuration")]
    FailedToParse(#[from] serde_json::Error),
}

#[derive(Error, Debug, PartialEq)]
pub enum ConfigAdjustError {
    #[error("category already found")]
    CategoryAlreadyFound,
    #[error("key is already present in hash map")]
    KeyAlreadyFound,
    #[error("duplicate change type: {0}")]
    DuplicateChangeType(String),
    #[error("Invalid URL")]
    InvalidURL(#[from] url::ParseError),
    #[error("expected value not found")]
    NotFound,
    #[error("target repository should be a GitHub link")]
    NoGitHubRepository,
}

#[derive(Error, Debug, PartialEq)]
pub enum ChangeTypeError {
    #[error("invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("no matches found")]
    NoMatchesFound,
}

#[derive(Error, Debug, PartialEq)]
pub enum ReleaseError {
    #[error("invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("invalid version: {0}")]
    InvalidVersion(#[from] VersionError),
    #[error("no release pattern found in line")]
    NoMatchFound,
}

#[derive(Error, Debug)]
pub enum ReleaseCLIError {
    #[error("failed to load config: {0}")]
    Config(#[from] ConfigError),
    #[error("duplicate version: {0}")]
    DuplicateVersion(String),
    #[error("input error: {0}")]
    Input(#[from] InputError),
    #[error("failed to parse changelog: {0}")]
    InvalidChangelog(#[from] ChangelogError),
    #[error("invalid version: {0}")]
    InvalidVersion(#[from] VersionError),
    #[error("no unreleased features")]
    NoUnreleased,
}

#[derive(Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("failed to parse version integer: {0}")]
    NoInteger(#[from] ParseIntError),
    #[error("invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("version does not follow semantic versioning")]
    NoMatchFound,
}
