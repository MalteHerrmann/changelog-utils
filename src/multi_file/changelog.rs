use crate::{config::Config, errors::ChangelogError};

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

    for (i, entry) in dir_contents.into_iter().enumerate() {
        // TODO: test this and then proceed parsing the entries
        println!("got entry {:?}: {:?}", i, entry.unwrap())
    }

    Ok(MultiFileChangelog {
        comments: Vec::new(),
        releases: Vec::new(),
        problems: Vec::new(),
        path: dir_path.to_path_buf(),
    })
}
