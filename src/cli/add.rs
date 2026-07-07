use super::{commands::AddArgs, inputs};
use crate::{
    common::changelog::Changelog,
    config,
    single_file::{change_type, changelog, entry, release},
    utils::{
        git::{commit, get_git_info},
        github::{get_merged_pr_numbers, get_pr_info, PRInfo},
    },
};
use eyre::{ensure, WrapErr};
use std::collections::HashMap;

/// Holds the fully-resolved, validated inputs needed to add a changelog entry
/// without any interactive prompts or GitHub lookups.
#[derive(Debug)]
struct NonInteractiveInputs {
    change_type: String,
    category: String,
    description: String,
    pr_number: u64,
}

/// Checks whether any of the non-interactive flags were provided.
fn any_non_interactive_flags_set(args: &AddArgs) -> bool {
    args.change_type.is_some() || args.category.is_some() || args.description.is_some()
}

/// Resolves and validates the non-interactive CLI flags against the given configuration.
///
/// Returns `Ok(None)` if none of the non-interactive flags were passed, so the caller can
/// fall back to the regular interactive flow. Returns an error if only some of the required
/// flags were passed, if the changelog is not in single-file mode, or if a given value does
/// not match the configuration.
fn resolve_non_interactive_inputs(
    config: &config::Config,
    args: &AddArgs,
) -> eyre::Result<Option<NonInteractiveInputs>> {
    if !any_non_interactive_flags_set(args) {
        return Ok(None);
    }

    ensure!(
        matches!(config.mode, config::Mode::Single),
        "Non-interactive mode (--change-type, --category, --description) is not supported for \
         multi-file changelogs yet; run 'clu config mode single' or use the interactive flow instead"
    );

    ensure!(
        args.number.is_some()
            && args.change_type.is_some()
            && args.category.is_some()
            && args.description.is_some(),
        "Non-interactive mode requires all of the following to be set: the PR number, \
         --change-type, --category and --description"
    );

    let change_type = args.change_type.clone().unwrap();
    ensure!(
        config.get_long_change_type(&change_type).is_some(),
        "Invalid change type '{}'; allowed values are: {}",
        change_type,
        config
            .change_types
            .iter()
            .map(|ct| ct.long.clone())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let category = args.category.clone().unwrap().to_lowercase();
    ensure!(
        config.categories.contains(&category),
        "Invalid category '{}'; allowed values are: {}",
        category,
        config.categories.join(", ")
    );

    let description = args.description.clone().unwrap();
    ensure!(
        !description.trim().is_empty(),
        "Description must not be empty"
    );

    Ok(Some(NonInteractiveInputs {
        change_type,
        category,
        description,
        pr_number: args.number.unwrap(),
    }))
}

/// Adds the changelog entry and commits the changes using the given fully-resolved
/// inputs, without any interactive prompts or GitHub lookups.
fn run_non_interactive(
    config: &config::Config,
    inputs: NonInteractiveInputs,
    commit_message: Option<String>,
) -> eyre::Result<()> {
    let mut changelog = changelog::load(config).wrap_err("Failed to load changelog")?;

    add_entry(
        config,
        &mut changelog,
        &inputs.change_type,
        &inputs.category,
        &inputs.description,
        inputs.pr_number,
    );

    changelog
        .write(config, &changelog.path)
        .wrap_err("Failed to write changelog")?;

    let cm = commit_message.unwrap_or_else(|| config.commit_message.clone());
    commit(config, &cm).wrap_err("Failed to commit changes")
}

/// Determines if user input is required based on the accept flag and whether PR info was retrieved.
fn should_get_user_input(accept: bool, retrieved: bool) -> bool {
    !accept || !retrieved
}

/// Handles all user input for the changelog entry, either using existing PR info or prompting for input.
fn get_entry_inputs(
    config: &config::Config,
    pr_info: &mut PRInfo,
    accept: bool,
    retrieved: bool,
) -> eyre::Result<(String, u64, String, String)> {
    let selectable_change_types: Vec<String> = config
        .change_types
        .iter()
        .map(|ct| ct.long.to_owned())
        .collect();

    // populate the map with false if user input is not required, otherwise true
    let mut get_inputs: HashMap<&str, bool> =
        ["change_type", "pr_number", "category", "description"]
            .into_iter()
            .map(|key| (key, should_get_user_input(accept, retrieved)))
            .collect();

    let mut selected_change_type = pr_info.change_type.clone();
    if !selectable_change_types.contains(&pr_info.change_type) {
        get_inputs.insert("change_type", true);
    }

    if get_inputs["change_type"] {
        selected_change_type = inputs::get_change_type(config, &pr_info.change_type)
            .wrap_err("Failed to get change type for entry")?;
    }

    let mut pr_number = pr_info.number;
    if get_inputs["pr_number"] {
        pr_number = inputs::get_pr_number(pr_info.number)
            .wrap_err("Failed to get PR number for entry")?;
    }

    let mut cat = pr_info.category.clone();
    if !config.categories.contains(&cat) {
        get_inputs.insert("category", true);
    }

    if get_inputs["category"] {
        cat = inputs::get_category(config, &pr_info.category)
            .wrap_err("Failed to get category for entry")?;
    }

    let mut desc = pr_info.description.clone();
    if get_inputs["description"] {
        desc = inputs::get_description(pr_info.description.as_str())
            .wrap_err("Failed to get description for entry")?;
    }

    Ok((selected_change_type, pr_number, cat, desc))
}

// Runs the logic to add an entry to the unreleased section of the changelog.
//
// After adding the new entry, the user is queried for a commit message to use
// to commit the changes.
//
// NOTE: the changes are NOT pushed to the origin when running the `add` command.
pub async fn run(args: AddArgs) -> eyre::Result<()> {
    let config = config::load()
        .wrap_err("Failed to load configuration")?;

    ensure!(
        !(args.all_previous && any_non_interactive_flags_set(&args)),
        "Cannot combine --all-previous with --change-type, --category or --description"
    );

    if let Some(inputs) = resolve_non_interactive_inputs(&config, &args)
        .wrap_err("Failed to resolve non-interactive inputs")?
    {
        return run_non_interactive(&config, inputs, args.commit_message);
    }

    let git_info = get_git_info(&config)
        .wrap_err("Failed to get git information")?;

    if args.all_previous {
        ensure!(
            args.number.is_none(),
            "Cannot specify both a PR number and --all-previous flag"
        );
        return run_batch(config, git_info, args.yes).await;
    }

    let mut pr_info = get_pr_info(&config, &git_info, args.number)
        .await
        .wrap_err("Failed to get PR information")?;
    let retrieved = pr_info.number != 0;

    let (selected_change_type, pr_number, cat, desc) =
        get_entry_inputs(&config, &mut pr_info, args.yes, retrieved)?;

    let mut changelog = changelog::load(&config)
        .wrap_err("Failed to load changelog")?;
    add_entry(
        &config,
        &mut changelog,
        &selected_change_type,
        &cat,
        &desc,
        pr_number,
    );

    changelog.write(&config, &changelog.path)
        .wrap_err("Failed to write changelog")?;

    let cm = inputs::get_commit_message(&config)
        .wrap_err("Failed to get commit message")?;
    commit(&config, &cm)
        .wrap_err("Failed to commit changes")
}

/// Runs batch processing to add changelog entries for all previous merged PRs
/// that don't yet have changelog entries.
async fn run_batch(
    config: config::Config,
    git_info: crate::utils::git::GitInfo,
    accept: bool,
) -> eyre::Result<()> {
    // Inform user about authentication status
    if std::env::var("GITHUB_TOKEN").is_err() {
        println!("⚠ No GITHUB_TOKEN found. Using unauthenticated GitHub API with rate limiting.");
        println!("  Set GITHUB_TOKEN environment variable for higher rate limits.\n");
    }

    println!("Fetching merged PRs from repository...");
    let merged_prs = get_merged_pr_numbers(&git_info)
        .await
        .wrap_err("Failed to fetch merged PR numbers")?;
    println!("Found {} merged PRs", merged_prs.len());

    let mut changelog = changelog::load(&config)
        .wrap_err("Failed to load changelog")?;
    let existing_prs = changelog.get_all_pr_numbers();

    let missing_prs: Vec<u64> = merged_prs
        .into_iter()
        .filter(|pr| !existing_prs.contains(pr))
        .collect();

    if missing_prs.is_empty() {
        println!("All merged PRs already have changelog entries!");
        return Ok(());
    }

    println!(
        "\nFound {} PRs without changelog entries",
        missing_prs.len()
    );
    println!("Fetching PR details...\n");

    // Fetch PR info for all missing PRs to show titles
    let mut pr_details = Vec::new();
    for pr_number in &missing_prs {
        match get_pr_info(&config, &git_info, Some(*pr_number)).await {
            Ok(pr_info) => {
                pr_details.push((*pr_number, pr_info.description));
            }
            Err(_) => {
                pr_details.push((*pr_number, String::from("(unable to fetch title)")));
            }
        }
    }

    // Let user select which PRs to add
    let selected_prs = inputs::select_prs_to_add(pr_details)
        .wrap_err("Failed to select PRs to add")?;

    if selected_prs.is_empty() {
        println!("No PRs selected. Aborted.");
        return Ok(());
    }

    println!("\nProcessing {} selected PRs...", selected_prs.len());

    let mut added_count = 0;
    let mut skipped_count = 0;

    for pr_number in selected_prs {
        print!("Processing PR #{}... ", pr_number);

        match get_pr_info(&config, &git_info, Some(pr_number)).await {
            Ok(mut pr_info) => {
                let retrieved = pr_info.number != 0;

                match get_entry_inputs(&config, &mut pr_info, accept, retrieved) {
                    Ok((selected_change_type, pr_num, cat, desc)) => {
                        add_entry(
                            &config,
                            &mut changelog,
                            &selected_change_type,
                            &cat,
                            &desc,
                            pr_num,
                        );
                        added_count += 1;
                        println!("✓ added");
                    }
                    Err(e) => {
                        skipped_count += 1;
                        println!("⚠ skipped ({})", e);
                    }
                }
            }
            Err(e) => {
                skipped_count += 1;
                println!("⚠ skipped ({})", e);
            }
        }
    }

    if added_count > 0 {
        changelog.write(&config, &changelog.path)
            .wrap_err("Failed to write changelog with new entries")?;

        let commit_message = format!(
            "chore: Add changelog entries for {} previous PRs",
            added_count
        );
        commit(&config, &commit_message)
            .wrap_err("Failed to commit changelog changes")?;

        println!("\n✓ Successfully added {} entries", added_count);
    }

    if skipped_count > 0 {
        println!("⚠ Skipped {} PRs", skipped_count);
    }

    Ok(())
}

/// Adds the given contents into a new entry in the unreleased section
/// of the changelog.
pub fn add_entry(
    config: &config::Config,
    // TODO: implement support for multi file changelog
    changelog: &mut changelog::SingleFileChangelog,
    change_type: &str,
    cat: &str,
    desc: &str,
    pr: u64,
) {
    let unreleased = match changelog.releases.iter_mut().find(|r| r.is_unreleased()) {
        Some(r) => r,
        None => {
            let mut new_releases = vec![release::new_unreleased()];
            new_releases.append(changelog.releases.as_mut());

            changelog.releases = new_releases;
            changelog.releases.get_mut(0).unwrap()
        }
    };

    let mut idx = 0;
    let mut change_type_is_found = false;
    for (i, ct) in unreleased.clone().change_types.into_iter().enumerate() {
        if ct.name.eq(&change_type) {
            idx = i;
            change_type_is_found = true;
        }
    }

    let new_entry = entry::Entry::new(config, cat, desc, pr);
    // NOTE: we're re-parsing the entry from the fixed version to incorporate all possible fixes
    let new_fixed_entry = entry::parse(config, new_entry.fixed.as_str()).unwrap();

    // Get the mutable change type to add the entry into.
    // NOTE: If it's not found yet, we add a new section to the changelog.
    if change_type_is_found {
        let mut_ct = unreleased
            .change_types
            .get_mut(idx)
            .expect("failed to get change type");

        mut_ct.entries.insert(0, new_fixed_entry);
    } else {
        let new_ct = change_type::new(change_type.to_owned(), Some(vec![new_fixed_entry]));
        unreleased.change_types.push(new_ct);
    }
}

#[cfg(test)]
mod non_interactive_tests {
    use super::*;

    fn load_test_config() -> config::Config {
        config::unpack_config(include_str!(
            "../../tests/testdata/single_file/evmos_config.json"
        ))
        .expect("failed to load example config")
    }

    fn base_args() -> AddArgs {
        AddArgs {
            number: None,
            yes: false,
            all_previous: false,
            change_type: None,
            category: None,
            description: None,
            commit_message: None,
        }
    }

    #[test]
    fn test_none_when_no_flags_set() {
        let config = load_test_config();
        let result = resolve_non_interactive_inputs(&config, &base_args())
            .expect("should not error when no flags are set");
        assert!(result.is_none());
    }

    #[test]
    fn test_error_on_partial_flags() {
        let config = load_test_config();
        let args = AddArgs {
            change_type: Some("Bug Fixes".to_string()),
            ..base_args()
        };
        let err = resolve_non_interactive_inputs(&config, &args).unwrap_err();
        assert!(err.to_string().contains("requires all of"));
    }

    #[test]
    fn test_error_on_multi_mode() {
        let mut config = load_test_config();
        config.set_mode(config::Mode::Multi);

        let args = AddArgs {
            number: Some(1),
            change_type: Some("Bug Fixes".to_string()),
            category: Some("go".to_string()),
            description: Some("Fixed a bug.".to_string()),
            ..base_args()
        };
        let err = resolve_non_interactive_inputs(&config, &args).unwrap_err();
        assert!(err.to_string().contains("multi-file changelogs"));
    }

    #[test]
    fn test_error_on_invalid_change_type() {
        let config = load_test_config();
        let args = AddArgs {
            number: Some(1),
            change_type: Some("Not A Real Change Type".to_string()),
            category: Some("go".to_string()),
            description: Some("Fixed a bug.".to_string()),
            ..base_args()
        };
        let err = resolve_non_interactive_inputs(&config, &args).unwrap_err();
        assert!(err.to_string().contains("Invalid change type"));
    }

    #[test]
    fn test_error_on_invalid_category() {
        let config = load_test_config();
        let args = AddArgs {
            number: Some(1),
            change_type: Some("Bug Fixes".to_string()),
            category: Some("not-a-category".to_string()),
            description: Some("Fixed a bug.".to_string()),
            ..base_args()
        };
        let err = resolve_non_interactive_inputs(&config, &args).unwrap_err();
        assert!(err.to_string().contains("Invalid category"));
    }

    #[test]
    fn test_error_on_empty_description() {
        let config = load_test_config();
        let args = AddArgs {
            number: Some(1),
            change_type: Some("Bug Fixes".to_string()),
            category: Some("go".to_string()),
            description: Some("   ".to_string()),
            ..base_args()
        };
        let err = resolve_non_interactive_inputs(&config, &args).unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn test_success_lowercases_category() {
        let config = load_test_config();
        let args = AddArgs {
            number: Some(42),
            change_type: Some("Bug Fixes".to_string()),
            category: Some("GO".to_string()),
            description: Some("Fixed a bug.".to_string()),
            ..base_args()
        };
        let resolved = resolve_non_interactive_inputs(&config, &args)
            .expect("should resolve successfully")
            .expect("should return Some inputs");

        assert_eq!(resolved.change_type, "Bug Fixes");
        assert_eq!(resolved.category, "go");
        assert_eq!(resolved.description, "Fixed a bug.");
        assert_eq!(resolved.pr_number, 42);
    }
}
