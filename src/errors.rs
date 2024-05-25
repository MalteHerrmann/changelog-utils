use serde_json;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LintError {
    #[error("failed to parse changelog: {0}")]
    InvalidChangelog(#[from] ChangelogError),
    #[error("invalid configuration: {0}")]
    InvalidConfig(#[from] ConfigError),
    #[error("failed to find changelog in directory")]
    NoChangelogFound,
    #[error("failed to read file system: {0}")]
    Read(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ChangelogError {
    // TODO: is this error necessary?
    #[error("failed to parse changelog: {0}")]
    Parse(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum EntryError {
    #[error("invalid entry: {0}")]
    InvalidEntry(String),
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
    #[error("failed to parse configuration")]
    FailedToParse(#[from] serde_json::Error)
}
