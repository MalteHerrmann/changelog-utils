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
    legacy_contents: Vec<String>,
    pub releases: Vec<release::Release>,
    pub problems: Vec<String>,
}

impl Changelog {
    /// Exports the changelog contents to the given filepath.
    pub fn write(&self, export_path: &Path) -> Result<(), ChangelogError> {
        Ok(fs::write(export_path, self.get_fixed())?)
    }

    /// Returns the fixed contents as a String to be exported.
    pub fn get_fixed(&self) -> String {
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

        self.legacy_contents
            .iter()
            .for_each(|l| exported_string.push_str(format!("{}\n", l).as_str()));

        exported_string
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
    let mut legacy_contents: Vec<String> = Vec::new();
    let mut releases: Vec<release::Release> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    let mut current_release = release::new_empty_release();
    let mut seen_releases: Vec<String> = Vec::new();
    let mut current_change_type: change_type::ChangeType;
    let mut seen_change_types: Vec<String> = Vec::new();
    let mut seen_prs: Vec<u16> = Vec::new();

    let mut is_comment = false;
    let mut is_legacy = false;

    let enter_comment_regex = Regex::new("<!--")?;
    let exit_comment_regex = Regex::new("-->")?;

    for (i, line) in contents.lines().enumerate() {
        if is_legacy {
            legacy_contents.push(line.to_string());
            continue;
        }

        let trimmed_line = line.trim();

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

            releases.push(current_release.clone());
            n_releases += 1;
            match seen_releases.contains(&current_release.version) {
                true => add_to_problems(&mut problems, file_path, i, format!("duplicate release: {}", &current_release.version)),
                false => seen_releases.push((current_release.version).to_string()),
            };

            // reset the seen change types for the current release
            seen_change_types = Vec::new();
            n_change_types = 0;

            if current_release
                .is_legacy(&config)
                .expect("failed to check legacy")
            {
                is_legacy = true;
            }

            current_release
                .problems
                .into_iter()
                .for_each(|p| add_to_problems(&mut problems, file_path, i, p.to_string()));

            fixed.push(current_release.fixed);

            continue;
        }

        if trimmed_line.starts_with("### ") {
            current_change_type = change_type::parse(config.clone(), line)?;

            // TODO: this handling should definitely be improved.
            // It's only a quick and dirty implementation for now.
            n_change_types += 1;
            if seen_change_types.contains(&current_change_type.name) {
                add_to_problems(&mut problems, file_path, i,
                    format!(
                        "duplicate change type in release {}: {}",
                        current_release.version.clone(),
                        current_change_type.name.clone(),
                    ),
                )
            } else {
                seen_change_types.push(current_change_type.name.clone());
            }

            fixed.push(current_change_type.fixed.clone());

            current_change_type
                .problems
                .iter()
                .for_each(|p| add_to_problems(&mut problems, file_path, i, p.to_string()));

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

        let current_entry = match entry::parse(&config, line) {
            Ok(e) => e,
            Err(err) => {
                add_to_problems(&mut problems, file_path, i, err.to_string());
                fixed.push(line.to_string());
                continue;
            }
        };

        // TODO: ditto, handling could be improved here like with change types, etc.
        if seen_prs.contains(&current_entry.pr_number) {
            add_to_problems(&mut problems, file_path, i, format!(
                "duplicate PR: #{}", &current_entry.pr_number,
            ));
        } else {
            seen_prs.push(current_entry.pr_number)
        }

        current_entry
            .problems
            .iter()
            .for_each(|p| add_to_problems(&mut problems, file_path, i, p.to_string()));

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
        legacy_contents,
    })
}

/// Used for formatting the problem statements in the changelog.
/// 
/// NOTE: The line ID will be incremented by one based on the loop enumeration where it is used.
fn add_to_problems(problems: &mut Vec<String>, fp: &Path, line: usize, problem: impl Into<String>) {
    problems.push(format!("{}:{}: {}", fp.to_string_lossy(), line+1, problem.into()))
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
            legacy_contents: Vec::new(),
            problems: Vec::new(),
        };
        let e = entry::parse(&cfg, example).expect("failed to parse entry");
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
