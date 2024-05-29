use serde_json;
use std::io;
use std::num::ParseIntError;
use regex::Error;
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
    FailedToParse(#[from] serde_json::Error),
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

#[derive(Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("failed to parse version integer: {0}")]
    NoInteger(#[from] ParseIntError),
    #[error("invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("version does not follow semantic versioning")]
    NoMatchFound,
}
