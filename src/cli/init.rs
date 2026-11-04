use crate::{
    config::{self, Config},
    single_file::changelog::get_settings_from_existing_changelog,
    utils::git::parse_github_owner_repo,
};
use eyre::{bail, ensure, WrapErr};
use inquire::{validator::Validation, Confirm, MultiSelect, Text};
use std::{
    fs,
    io::{IsTerminal, Write},
    path::{Path, PathBuf},
    process::Command,
};

use super::commands::InitArgs;

/// Runs the logic to initialize the changelog utilities
/// in the current working directory.
pub fn run(args: InitArgs) -> eyre::Result<()> {
    initialize(
        std::env::current_dir().wrap_err("Failed to get current directory")?,
        !args.yes,
    )
}

/// Initializes the given directory with detected settings and defaults, without prompts.
pub fn init_in_folder(target: PathBuf) -> eyre::Result<()> {
    initialize(target, false)
}

fn initialize(target: PathBuf, interactive: bool) -> eyre::Result<()> {
    let config_path = target.join(".clconfig.json");
    if std::fs::symlink_metadata(&config_path).is_ok() {
        bail!(
            "Configuration file already exists at {}",
            config_path.display()
        );
    }

    ensure!(
        !interactive || (std::io::stdin().is_terminal() && std::io::stdout().is_terminal()),
        "Interactive initialization requires a terminal; use 'clu init --yes' to accept defaults"
    );

    let mut config = Config {
        target_repo: suggest_repository(&target),
        ..Config::default()
    };

    let changelog_path = target.join("CHANGELOG.md");
    let create_changelog = match fs::read_to_string(&changelog_path) {
        Ok(contents) => {
            get_settings_from_existing_changelog(&mut config, &contents);
            if config.change_types.is_empty() {
                config.change_types = Config::default().change_types;
            }
            false
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => return Err(error).wrap_err("Failed to read existing CHANGELOG.md"),
    };
    // A new project has no category names to select when adding entries.
    if config.categories.is_empty() {
        config.use_categories = false;
    }

    if interactive {
        configure_interactively(&mut config)?;
        println!("\nConfiguration to save:\n{config}");
        if !Confirm::new("Create changelog configuration?")
            .with_default(true)
            .prompt()?
        {
            println!("Initialization cancelled; no files were written.");
            return Ok(());
        }
    }

    if create_changelog {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&changelog_path)
            .and_then(|mut file| file.write_all(create_empty_changelog().as_bytes()))
            .wrap_err_with(|| {
                format!("Failed to create changelog at {}", changelog_path.display())
            })?;
        println!("created empty changelog at {}", changelog_path.display());
    }

    config.export(&config_path).wrap_err_with(|| {
        format!(
            "Failed to export configuration to {}",
            config_path.display()
        )
    })?;
    println!(
        "created new configuration at {}:\n{}",
        config_path.display(),
        config
    );
    if config.target_repo.is_empty() {
        println!("Set a repository before adding entries: clu config target-repo <GitHub URL>");
    }
    Ok(())
}

fn git_output(target: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(target)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn suggest_repository(target: &Path) -> String {
    let remotes = git_output(target, &["remote"]).unwrap_or_default();
    for remote in std::iter::once("origin").chain(remotes.lines().filter(|r| *r != "origin")) {
        if let Some(url) = git_output(target, &["remote", "get-url", remote]) {
            if let Ok((owner, repo)) = parse_github_owner_repo(&url) {
                return format!("https://github.com/{owner}/{repo}");
            }
        }
    }

    let folder = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    for key in ["github.user", "user.name"] {
        if let Some(user) = git_output(target, &["config", "--get", key]) {
            let suggestion = format!("https://github.com/{user}/{folder}");
            if parse_github_owner_repo(&suggestion).is_ok() {
                return suggestion;
            }
        }
    }
    String::new()
}

fn configure_interactively(config: &mut Config) -> eyre::Result<()> {
    let repository = Text::new("GitHub repository URL:")
        .with_initial_value(&config.target_repo)
        .with_help_message("Review the suggestion; leave empty to configure the repository later.")
        .with_validator(|value: &str| {
            Ok(
                if value.trim().is_empty() || parse_github_owner_repo(value).is_ok() {
                    Validation::Valid
                } else {
                    Validation::Invalid("Enter a GitHub repository URL (HTTPS or SSH).".into())
                },
            )
        })
        .prompt()?;
    if repository.trim().is_empty() {
        config.target_repo.clear();
    } else {
        config::set_target_repo(config, repository)?;
    }

    let options = config
        .change_types
        .iter()
        .map(|ct| format!("{} ({})", ct.long, ct.short))
        .collect();
    let selected = MultiSelect::new("Change types to keep:", options)
        .with_all_selected_by_default()
        .with_help_message("Space toggles a type; Enter continues. Custom types can be added next.")
        .raw_prompt()?;
    config.change_types = selected
        .iter()
        .map(|option| config.change_types[option.index].clone())
        .collect();

    loop {
        if !config.change_types.is_empty()
            && !Confirm::new("Add a custom change type?")
                .with_default(false)
                .prompt()?
        {
            break;
        }
        let existing = config.change_types.clone();
        let long = Text::new("Change type name:")
            .with_help_message("For example: Documentation")
            .with_validator(move |value: &str| {
                let value = value.trim();
                Ok(
                    if value.is_empty()
                        || !value
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ')
                    {
                        Validation::Invalid(
                            "Use letters, digits, spaces or hyphens; a name is required.".into(),
                        )
                    } else if existing
                        .iter()
                        .any(|ct| ct.long.eq_ignore_ascii_case(value))
                    {
                        Validation::Invalid("That change type name already exists.".into())
                    } else {
                        Validation::Valid
                    },
                )
            })
            .prompt()?;
        let existing = config.change_types.clone();
        let short = Text::new("Short identifier:")
            .with_help_message("For example: docs")
            .with_validator(move |value: &str| {
                let value = value.trim();
                Ok(
                    if value.is_empty()
                        || !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                    {
                        Validation::Invalid(
                            "Use letters, digits or hyphens; an identifier is required.".into(),
                        )
                    } else if existing
                        .iter()
                        .any(|ct| ct.short.eq_ignore_ascii_case(value))
                    {
                        Validation::Invalid("That short identifier already exists.".into())
                    } else {
                        Validation::Valid
                    },
                )
            })
            .prompt()?;
        config.add_change_type(long.trim().into(), short.trim().into())?;
    }

    config.use_categories = Confirm::new("Use categories?")
        .with_default(config.use_categories)
        .prompt()?;
    if config.use_categories {
        let categories = Text::new("Categories (comma-separated):")
            .with_initial_value(&config.categories.join(", "))
            .with_help_message(
                "For example: cli, api, docs. Edit the list to add or remove categories.",
            )
            .with_validator(|value: &str| {
                let mut seen = Vec::new();
                for category in value.split(',').map(str::trim) {
                    if category.is_empty()
                        || !category
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    {
                        return Ok(Validation::Invalid(
                            "Provide nonempty lowercase names using letters, digits or hyphens."
                                .into(),
                        ));
                    }
                    if seen.contains(&category) {
                        return Ok(Validation::Invalid("Category names must be unique.".into()));
                    }
                    seen.push(category);
                }
                Ok(Validation::Valid)
            })
            .prompt()?;
        config.categories.clear();
        for category in categories.split(',').map(str::trim) {
            config.add_category(category.into())?;
        }
    }
    Ok(())
}

/// Creates an empty skeleton for a changelog.
pub fn create_empty_changelog() -> String {
    [
        "<!--",
        "This changelog was created using the `clu` binary",
        "(https://github.com/MalteHerrmann/changelog-utils).",
        "-->",
        "",
        "# Changelog",
        "",
        "## Unreleased",
        "",
    ]
    .join("\n")
}
