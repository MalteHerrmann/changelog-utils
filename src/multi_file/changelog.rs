use crate::{config::Config, errors::ChangelogError, multi_file::release};

use super::release::Release;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct MultiFileChangelog {
    // TODO: implement comments?
    pub comments: Vec<String>,
    pub releases: Vec<Release>,
    pub problems: Vec<String>,
    pub path: PathBuf,
}

pub fn load(config: &Config) -> Result<MultiFileChangelog, ChangelogError> {
    let changelog_path = match fs::read_dir(Path::new("./"))?.find(|e| {
        e.as_ref()
            .is_ok_and(|e| e.file_name().eq_ignore_ascii_case(".changelog"))
    }) {
        Some(d) => d.unwrap(),
        None => {
            println!("could not find a changelog subdirectory in the current directory");
            return Err(ChangelogError::NoChangelogFound);
        }
    };

    parse_changelog(config, changelog_path.path().as_path())
}

// TODO: support escapes in multi file implementation?
pub fn parse_changelog(
    config: &Config,
    dir_path: &Path,
) -> Result<MultiFileChangelog, ChangelogError> {
    let dir_contents = fs::read_dir(dir_path)?;

    let releases: Vec<Release> = dir_contents
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|p| release::parse(config, &p).ok())
        .collect();

    println!("found {} subdirs", releases.len());

    releases.iter().for_each(|r| println!("release: {:?}", r));

    Ok(MultiFileChangelog {
        comments: Vec::new(),
        releases: Vec::new(),
        problems: Vec::new(),
        path: dir_path.to_path_buf(),
    })
}
