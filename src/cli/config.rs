use super::commands::{
    CategoryOperation, ChangeTypeConfigOperation,
    ConfigSubcommands::{
        self, Category, ChangeType, ChangelogDir, LegacyVersion, Migrate, Mode, Show, Spelling,
        TargetRepo, UseCategories,
    },
    KeyValueOperation, OptionalOperation,
};
use crate::config;
use eyre::WrapErr;
use std::path::Path;

// Handles the CLI subcommands to adjust the configuration file.
pub fn adjust_config(config_subcommand: ConfigSubcommands) -> eyre::Result<()> {
    match config_subcommand {
        // Handle Migrate separately without loading config first
        Migrate => super::migrate::run(),
        Category(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            match args.command {
                CategoryOperation::Add { value } => {
                    let val = value.clone();
                    configuration.add_category(value)
                        .wrap_err_with(|| format!("Failed to add category '{}'", val))?
                }
                CategoryOperation::Remove { value } => {
                    let val = value.clone();
                    configuration.remove_category(value)
                        .wrap_err_with(|| format!("Failed to remove category '{}'", val))?
                }
            }
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        ChangeType(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            match args.command {
                ChangeTypeConfigOperation::Add { long, short } => {
                    let long_copy = long.clone();
                    let short_copy = short.clone();
                    configuration.add_change_type(long, short)
                        .wrap_err_with(|| format!("Failed to add change type '{}' ({})", long_copy, short_copy))?
                }
                ChangeTypeConfigOperation::Remove { short } => {
                    let short_copy = short.clone();
                    configuration.remove_change_type(short)
                        .wrap_err_with(|| format!("Failed to remove change type '{}'", short_copy))?
                }
            }
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        Show => {
            let configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            println!("{}", configuration);
            Ok(())
        }
        Spelling(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            match args.command {
                KeyValueOperation::Add { key, value } => {
                    let key_copy = key.clone();
                    let value_copy = value.clone();
                    configuration.add_expected_spelling(key, value)
                        .wrap_err_with(|| format!("Failed to add expected spelling '{}' -> '{}'", key_copy, value_copy))?
                }
                KeyValueOperation::Remove { key } => {
                    let key_copy = key.clone();
                    configuration.remove_expected_spelling(key)
                        .wrap_err_with(|| format!("Failed to remove expected spelling '{}'", key_copy))?
                }
            }
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        LegacyVersion(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            match args.command {
                OptionalOperation::Set { value } => configuration.legacy_version = Some(value),
                OptionalOperation::Unset => configuration.legacy_version = None,
            }
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        TargetRepo(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            let value_copy = args.value.clone();
            config::set_target_repo(&mut configuration, args.value)
                .wrap_err_with(|| format!("Failed to set target repo to '{}'", value_copy))?;
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        ChangelogDir(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            match args.command {
                OptionalOperation::Set { value } => configuration.set_changelog_dir(Some(value)),
                OptionalOperation::Unset => configuration.set_changelog_dir(None),
            }
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        Mode(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            let mode = args.value.parse::<config::Mode>()
                .map_err(|e| eyre::eyre!("Invalid mode value '{}': {}", args.value, e))?;
            configuration.set_mode(mode);
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
        UseCategories(args) => {
            let mut configuration = config::load()
                .wrap_err("Failed to load configuration")?;
            match args.command {
                OptionalOperation::Set { value } => {
                    let use_categories = value.parse::<bool>()
                        .wrap_err_with(|| format!("Invalid boolean value '{}'. Expected 'true' or 'false'", value))?;
                    configuration.set_use_categories(use_categories);
                }
                OptionalOperation::Unset => configuration.set_use_categories(false),
            }
            configuration.export(Path::new(".clconfig.json"))
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
    }
}
