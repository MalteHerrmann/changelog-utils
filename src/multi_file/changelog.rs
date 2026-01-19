use crate::{
    common::add_to_problems,
    config::Config,
    errors::{ChangelogError, ConfigError},
    multi_file::release,
};

use super::release::Release;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct MultiFileChangelog {
    pub releases: Vec<Release>,
    pub problems: Vec<String>,
    pub path: PathBuf,
}

impl crate::common::changelog::Changelog for MultiFileChangelog {
    fn get_path(&self) -> &std::path::Path {
        &self.path
    }

    fn get_problems(&self) -> &[String] {
        &self.problems
    }

    fn get_all_pr_numbers(&self) -> Vec<u64> {
        self.releases
            .iter()
            .flat_map(|release| &release.change_types)
            .flat_map(|change_type| &change_type.entries)
            .map(|entry| entry.pr_number)
            .collect()
    }

    fn write(
        &self,
        _config: &Config,
        _export_path: &std::path::Path,
    ) -> Result<(), crate::errors::ChangelogError> {
        unimplemented!("write operation not yet implemented for multi-file changelogs")
    }

    fn get_fixed_contents(
        &self,
        _config: &Config,
    ) -> Result<String, crate::errors::ChangelogError> {
        unimplemented!("get_fixed_contents not yet implemented for multi-file changelogs")
    }
}

pub fn load(config: &Config) -> Result<MultiFileChangelog, ChangelogError> {
    let expected_path = config
        .changelog_dir
        .as_ref()
        .ok_or_else(|| ConfigError::InvalidConfig("changelog_dir must be set".to_string()))?;

    let changelog_path = match fs::read_dir(Path::new("./"))?.find(|e| {
        e.as_ref()
            .is_ok_and(|e| e.file_name().eq_ignore_ascii_case(expected_path))
    }) {
        Some(d) => d.unwrap(),
        None => {
            println!("could not find the changelog subdirectory in the current directory");
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

    // Gather all problems from the individual entries
    let mut problems: Vec<String> = Vec::new();
    releases.iter().for_each(|r| {
        r.problems
            .iter()
            .for_each(|p| add_to_problems(&mut problems, &r.path, None, p));
        r.change_types.iter().for_each(|ct| {
            ct.problems
                .iter()
                .for_each(|p| add_to_problems(&mut problems, &ct.path, None, p));
            ct.entries.iter().for_each(|e| {
                e.problems
                    .iter()
                    .for_each(|p| add_to_problems(&mut problems, &e.path, Some(0), p))
            });
        })
    });

    // NOTE: sorting entries here to ensure deterministic order
    // even with parallel handling of entries
    problems.sort();

    Ok(MultiFileChangelog {
        releases,
        problems,
        path: dir_path.to_path_buf(),
    })
}
