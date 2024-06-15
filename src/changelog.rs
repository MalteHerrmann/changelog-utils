use crate::{change_type, config::Config, entry, errors::ChangelogError, release};
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Represents the changelog contents.
#[derive(Debug)]
pub struct Changelog {
    pub path: PathBuf,
    pub fixed: Vec<String>,
    comments: Vec<String>,
    pub releases: Vec<release::Release>,
    pub problems: Vec<String>,
}

impl Changelog {
    /// Exports the changelog contents to the given filepath.
    pub fn write(&self, export_path: &Path) -> Result<(), ChangelogError> {
        let mut exported_string = "".to_string();

        self.comments
            .iter()
            .for_each(|x| exported_string.push_str(format!("{x}\n").as_str()));
        exported_string.push_str("# Changelog\n");

        for release in &self.releases {
            exported_string.push('\n');
            exported_string.push_str(release.fixed.as_str());
            exported_string.push('\n');

            for change_type in &release.change_types {
                exported_string.push('\n');
                exported_string.push_str(change_type.fixed.as_str());
                exported_string.push_str("\n\n");

                for entry in &change_type.entries {
                    exported_string.push_str(entry.fixed.as_str());
                    exported_string.push('\n');
                }
            }
        }

        Ok(fs::write(export_path, exported_string)?)
    }
}

/// Loads the changelog from the default changelog path.
pub fn load(config: Config) -> Result<Changelog, ChangelogError> {
    let changelog_file = match fs::read_dir(Path::new("./"))?.find(|e| {
        e.as_ref()
            .is_ok_and(|e| e.file_name().to_ascii_lowercase() == "changelog.md")
    }) {
        Some(f) => f.unwrap(),
        None => {
            println!("could not find the changelog in the current directory");
            return Err(ChangelogError::NoChangelogFound);
        }
    };

    parse_changelog(config, changelog_file.path().as_path())
}

/// Parses the given changelog contents.
pub fn parse_changelog(config: Config, file_path: &Path) -> Result<Changelog, ChangelogError> {
    let contents = fs::read_to_string(file_path)?;

    let mut n_releases = 0;
    let mut n_change_types = 0;

    let mut comments: Vec<String> = Vec::new();
    let mut fixed: Vec<String> = Vec::new();
    let mut releases: Vec<release::Release> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    let mut current_release = release::new_empty_release();
    let mut seen_releases: Vec<String> = Vec::new();
    let mut current_change_type = change_type::new_empty_change_type();
    let mut seen_change_types: Vec<String> = Vec::new();
    let mut seen_prs: Vec<u16> = Vec::new();

    let mut is_comment = false;
    let mut is_legacy = false;

    let enter_comment_regex = Regex::new("<!--")?;
    let exit_comment_regex = Regex::new("-->")?;

    for line in contents.lines() {
        let trimmed_line = line.trim();

        // TODO: improve this?
        if enter_comment_regex.is_match(trimmed_line) && !exit_comment_regex.is_match(trimmed_line)
        {
            is_comment = true;
            comments.push(line.to_string());
            fixed.push(line.to_string());
            continue;
        }

        if is_comment && exit_comment_regex.is_match(trimmed_line) {
            is_comment = false;
            comments.push(line.to_string());
            fixed.push(line.to_string());
            continue;
        }

        if is_comment {
            fixed.push(line.to_string());
            comments.push(line.to_string());
            continue;
        }

        if trimmed_line.starts_with("## ") {
            current_release = release::parse(&config, line)?;
            // FIXME: this pushes a copy of the empty release to the hashmap
            // It would be better to push the reference into the hashmap but that requires lifetime
            // handling.
            // Alternatively, the logic should be adjusted to only insert into the hashmap, once
            // the next release is found but that makes the logic more complicated,
            // so we'll keep this for now.
            releases.push(current_release.clone());
            n_releases += 1;
            match seen_releases.contains(&current_release.version) {
                true => problems.push(format!("duplicate release: {}", &current_release.version)),
                false => seen_releases.push((current_release.version).to_string()),
            };

            // reset the seen change types for the current release
            seen_change_types = Vec::new();
            n_change_types = 0;

            if current_release
                .is_legacy(&config)
                .expect("failed to check legacy")
                && !is_legacy
            {
                is_legacy = true;
            }

            for rel_prob in &current_release.problems {
                problems.push(rel_prob.to_string())
            }

            fixed.push(current_release.fixed);

            continue;
        }

        if trimmed_line.starts_with("### ") {
            current_change_type = change_type::parse(config.clone(), line)?;

            // TODO: this handling should definitely be improved.
            // It's only a quick and dirty implementation for now.
            n_change_types += 1;
            if seen_change_types.contains(&current_change_type.name) {
                problems.push(format!(
                    "duplicate change type in release {}: {}",
                    current_release.version.clone(),
                    current_change_type.name.clone(),
                ))
            } else {
                seen_change_types.push(current_change_type.name.clone());
            }

            for ct_prob in &current_change_type.problems {
                problems.push(ct_prob.to_string())
            }

            fixed.push(current_change_type.fixed.clone());

            // TODO: improve this? can this handling be made "more rustic"?
            let last_release = releases
                .get_mut(n_releases - 1)
                .expect("failed to get last release");
            last_release.change_types.push(current_change_type.clone());

            continue;
        }

        // TODO: check how to handle legacy content with the type based export?
        // TODO: this can actually be removed now with the new type-based exports
        if !trimmed_line.starts_with('-') || is_legacy {
            fixed.push(line.to_string());
            continue;
        }

        // TODO: remove clone?
        let current_entry = match entry::parse(config.clone(), line) {
            Ok(e) => e,
            Err(err) => {
                problems.push(err.to_string());
                fixed.push(line.to_string());
                continue;
            }
        };

        // TODO: ditto, handling could be improved here like with change types, etc.
        if seen_prs.contains(&current_entry.pr_number) {
            problems.push(format!(
                "duplicate PR in {}->{}: {}",
                &current_release.version.clone(),
                current_change_type.name.clone(),
                &current_entry.pr_number,
            ));
        } else {
            seen_prs.push(current_entry.pr_number)
        }

        for entry_prob in &current_entry.problems {
            problems.push(entry_prob.to_string());
        }

        // TODO: can be removed with new type-based exports
        fixed.push(current_entry.clone().fixed);

        // TODO: improve this, seems not ideal because it's also being retrieved in the statements above
        let last_release = releases
            .get_mut(n_releases - 1)
            .expect("failed to get last release");

        let last_change_type = last_release
            .change_types
            .get_mut(n_change_types - 1)
            .expect("failed to get last change type");
        last_change_type.entries.push(current_entry);
    }

    Ok(Changelog {
        path: file_path.to_path_buf(),
        fixed,
        releases,
        comments,
        problems,
    })
}

#[cfg(test)]
mod changelog_tests {
    use std::str::FromStr;

    use crate::config;

    use super::*;

    fn load_test_config() -> Config {
        config::unpack_config(include_str!("testdata/example_config.json"))
            .expect("failed to load example configuration")
    }

    #[test]
    fn test_pass() {
        let cfg = load_test_config();
        let example = concat!(
            "- (cli) [#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) ",
            "Add initial Python implementation."
        );

        let mut cl = Changelog {
            path: PathBuf::from_str("test").unwrap(),
            fixed: Vec::new(),
            releases: Vec::new(),
            comments: Vec::new(),
            problems: Vec::new(),
        };
        let e = entry::parse(cfg.clone(), example).expect("failed to parse entry");
        let ct =
            change_type::parse(cfg.clone(), "### Bug Fixes").expect("failed to parse change type");

        let er = "## [v0.1.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v0.1.0) - 2024-04-27";
        let r = release::parse(&cfg, er).expect("failed to parse release");

        cl.releases.push(r.clone());
        let mut_cr = cl.releases.get_mut(0).expect("failed to get last release");
        mut_cr.change_types.push(ct.clone());

        let mut_ct = mut_cr
            .change_types
            .get_mut(0)
            .expect("failed to get last change type");
        mut_ct.entries.push(e);
        assert_eq!(
            mut_cr
                .change_types
                .get(0)
                .expect("failed to get first change type in assert")
                .entries
                .len(),
            1
        );
        assert_eq!(
            cl.releases
                .get(0)
                .expect("failed to get first release")
                .change_types
                .get(0)
                .expect("failed to get first change type in changelog")
                .entries
                .len(),
            1
        );
    }
}
