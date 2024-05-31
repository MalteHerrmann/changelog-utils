use std::collections::HashMap;
use regex::Regex;
use crate::{change_type, config::Config, entry, errors::ChangelogError, release};

/// Represents the changelog contents.
#[derive(Debug)]
pub struct Changelog {
    pub fixed: Vec<String>,
    pub releases: HashMap<String, release::Release>,
    pub problems: Vec<String>
}

/// Parses the given changelog contents.
///
/// TODO: implement fix functionality
pub fn parse_changelog(config: Config, contents: &str) -> Result<Changelog, ChangelogError> {
    let mut fixed: Vec<String> = Vec::new();
    let mut releases: HashMap<String, release::Release> = HashMap::new();
    let mut problems: Vec<String> = Vec::new();

    let mut current_release = release::new_empty_release();
    let mut current_change_type= change_type::new_empty_change_type();
    let mut seen_change_types: Vec<String> = Vec::new();
    let mut seen_prs: Vec<u16> = Vec::new();

    let mut is_comment = false;
    let mut is_legacy = false;

    let enter_comment_regex = Regex::new("<!--")?;
    let exit_comment_regex = Regex::new("-->")?;

    for line in contents.lines() {
        let trimmed_line = line.trim();

        // TODO: improve this?
        if enter_comment_regex.is_match(trimmed_line) {
            is_comment = true;
            fixed.push(line.to_string());
            continue
        }

        if is_comment && exit_comment_regex.is_match(trimmed_line) {
            is_comment = false;
            fixed.push(line.to_string());
            continue
        }

        if is_comment {
            fixed.push(line.to_string());
            continue
        }

        if trimmed_line.starts_with("## ") {
            current_release = release::parse(&config, line)?;
            // FIXME: this pushes a copy of the empty release to the hashmap
            // It would be better to push the reference into the hashmap but that requires lifetime
            // handling.
            // Alternatively, the logic should be adjusted to only insert into the hashmap, once
            // the next release is found but that makes the logic more complicated,
            // so we'll keep this for now.
            match releases.insert(current_release.version.clone(), current_release.clone()) {
                Some(_) => problems.push(
                    format!("duplicate release: {}", &current_release.version)
                ),
                _ => ()
            };

            seen_change_types = Vec::new();

            if current_release.is_legacy(&config).expect("failed to check legacy") && !is_legacy {
                is_legacy = true;
            }

            for rel_prob in &current_release.problems {
                problems.push(rel_prob.to_string())
            }

            fixed.push(current_release.fixed);

            continue
        }

        if trimmed_line.starts_with("### ") {
            current_change_type = change_type::parse(config.clone(), line)?;

            // TODO: this handling should definitely be improved.
            // It's only a quick and dirty implementation for now.
            if seen_change_types.contains(&current_change_type.name) {
                problems.push(format!("duplicate change type in release {}: {}",
                    current_release.version.clone(),
                    current_change_type.name.clone(),
                ))
            } else {
                seen_change_types.push(current_change_type.name.clone());
            }

            for ct_prob in &current_change_type.problems {
                problems.push(ct_prob.to_string())
            }

            fixed.push(current_change_type.fixed);

            continue
        }

        if !trimmed_line.starts_with("-") || is_legacy {
            fixed.push(line.to_string());
            continue
        }

        // TODO: remove clone?
        let current_entry = match entry::parse(config.clone(), line) {
            Ok(e) => e,
            Err(ee) => {
                problems.push(ee.to_string());
                fixed.push(line.to_string());
                continue
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

        fixed.push(current_entry.fixed)
    }

    Ok(Changelog { fixed, releases, problems })
}
