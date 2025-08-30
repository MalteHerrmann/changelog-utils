use super::{change_type, entry, release};
use crate::{
    errors::ChangelogError,
    escapes,
    utils::config::{ChangeTypeConfig, Config},
};
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Represents the changelog contents.
#[derive(Debug)]
pub struct Changelog {
    pub path: PathBuf,
    comments: Vec<String>,
    legacy_contents: Vec<String>,
    pub releases: Vec<release::Release>,
    pub problems: Vec<String>,
}

impl Changelog {
    /// Exports the changelog contents to the given filepath.
    pub fn write(&self, export_path: &Path) -> Result<(), ChangelogError> {
        Ok(fs::write(export_path, self.get_fixed_contents())?)
    }

    /// Returns the fixed contents as a String to be exported.
    pub fn get_fixed_contents(&self) -> String {
        let mut exported_string = "".to_string();

        self.comments
            .iter()
            .for_each(|x| exported_string.push_str(format!("{x}\n").as_str()));
        exported_string.push_str("# Changelog\n");

        self.releases.iter().for_each(|release| {
            exported_string.push('\n');
            exported_string.push_str(release.get_fixed_contents().as_str());
        });

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
            .is_ok_and(|e| e.file_name().eq_ignore_ascii_case("changelog.md"))
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
    let mut legacy_contents: Vec<String> = Vec::new();
    let mut releases: Vec<release::Release> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    let mut current_release = release::new_empty_release();
    let mut seen_releases: Vec<String> = Vec::new();
    let mut current_change_type: change_type::ChangeType;
    let mut seen_change_types: Vec<String> = Vec::new();
    let mut seen_prs: Vec<u64> = Vec::new();

    let mut escapes: Vec<escapes::LinterEscape> = Vec::new();
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

        if enter_comment_regex.is_match(trimmed_line) {
            is_comment = true;
        }

        if is_comment && exit_comment_regex.is_match(trimmed_line) {
            is_comment = false;
            comments.push(line.to_string());

            // Check inline comments
            if let Some(e) = escapes::check_escape_pattern(trimmed_line) {
                escapes.push(e);
            }

            continue;
        }

        if is_comment {
            comments.push(line.to_string());
            continue;
        }

        if trimmed_line.starts_with("## ") {
            current_release = release::parse(&config, line)?;

            releases.push(current_release.clone());
            n_releases += 1;
            if seen_releases.contains(&current_release.version) {
                add_to_problems(
                    &mut problems,
                    file_path,
                    i,
                    format!("duplicate release: {}", &current_release.version),
                );
            } else {
                seen_releases.push((current_release.version).to_string());
            };

            // reset the seen change types for the current release
            seen_change_types.clear();
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

            continue;
        }

        if trimmed_line.starts_with("### ") {
            current_change_type = change_type::parse(config.clone(), line)?;

            n_change_types += 1;
            if seen_change_types.contains(&current_change_type.name) {
                add_to_problems(
                    &mut problems,
                    file_path,
                    i,
                    format!(
                        "duplicate change type in release {}: {}",
                        current_release.version.clone(),
                        current_change_type.name.clone(),
                    ),
                )
            } else {
                seen_change_types.push(current_change_type.name.clone());
            }

            current_change_type
                .problems
                .iter()
                .for_each(|p| add_to_problems(&mut problems, file_path, i, p.to_string()));

            let last_release = releases
                .get_mut(n_releases - 1)
                .expect("failed to get last release");

            last_release.change_types.push(current_change_type.clone());

            continue;
        }

        if !trimmed_line.starts_with('-') {
            continue;
        }

        let current_entry = match entry::parse(&config, line) {
            Ok(e) => e,
            Err(err) => {
                if !escapes.contains(&escapes::LinterEscape::FullLine) {
                    add_to_problems(&mut problems, file_path, i, err.to_string());
                }

                // reset escapes after processing entry
                escapes.clear();

                continue;
            }
        };

        if seen_prs.contains(&current_entry.pr_number)
            && (!escapes.contains(&escapes::LinterEscape::DuplicatePR)
                && !escapes.contains(&escapes::LinterEscape::FullLine))
        {
            add_to_problems(
                &mut problems,
                file_path,
                i,
                format!("duplicate PR: #{}", &current_entry.pr_number,),
            );
            escapes.retain(|e| e.ne(&escapes::LinterEscape::DuplicatePR));
        } else {
            seen_prs.push(current_entry.pr_number)
        }

        if !escapes.contains(&escapes::LinterEscape::FullLine) {
            current_entry
                .problems
                .iter()
                .for_each(|p| add_to_problems(&mut problems, file_path, i, p.to_string()));
        }

        let last_release = releases
            .get_mut(n_releases - 1)
            .expect("failed to get last release");

        let last_change_type = last_release
            .change_types
            .get_mut(n_change_types - 1)
            .expect("failed to get last change type");

        last_change_type.entries.push(current_entry);

        // Reset the escapes after an entry line
        escapes.clear();
    }

    Ok(Changelog {
        path: file_path.to_path_buf(),
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
    problems.push(format!(
        "{}:{}: {}",
        fp.to_string_lossy(),
        line + 1,
        problem.into()
    ))
}

#[cfg(test)]
mod changelog_tests {
    use std::str::FromStr;

    use crate::utils::config;

    use super::*;

    fn load_test_config() -> Config {
        config::unpack_config(include_str!("../testdata/example_config.json"))
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

// Tries to parse the individual entries of an existing changelog
// to derive a configuration from it.
//
// NOTE: Errors while parsing are ignored as the purpose of this method
// is to simply extract all available information.
pub fn get_settings_from_existing_changelog(config: &mut Config, contents: &str) {
    let mut seen_change_types: Vec<String> = Vec::new();
    let mut seen_categories: Vec<String> = Vec::new();

    for line in contents.lines() {
        let trimmed_line = line.trim();

        if trimmed_line.starts_with("### ") {
            if let Ok(ct) = change_type::parse(config.clone(), line) {
                if !seen_change_types.contains(&ct.name) {
                    seen_change_types.push(ct.name)
                }
            };

            continue;
        }

        if let Ok(e) = entry::parse(config, line) {
            if !seen_categories.contains(&e.category) {
                seen_categories.push(e.category)
            }
        }
    }

    let mut change_type_configs: Vec<ChangeTypeConfig> = Vec::new();
    seen_change_types.into_iter().for_each(|ct| {
        let pattern = ct[0..4].trim().to_ascii_lowercase();

        change_type_configs.push(ChangeTypeConfig {
            short: pattern,
            long: ct,
        });
    });

    seen_categories.sort();
    config.categories = seen_categories;
    config.change_types = change_type_configs;
}
