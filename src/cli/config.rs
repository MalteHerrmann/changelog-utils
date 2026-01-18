use super::commands::{
    CategoryOperation, ChangeTypeConfigOperation,
    ConfigSubcommands::{
        self, Category, ChangeType, ChangelogDir, LegacyVersion, Migrate, Mode, Show, Spelling,
        TargetRepo, UseCategories,
    },
    KeyValueOperation, OptionalOperation,
};
use crate::{config, errors};
use std::path::Path;

// Handles the CLI subcommands to adjust the configuration file.
pub fn adjust_config(config_subcommand: ConfigSubcommands) -> Result<(), errors::CLIError> {
    match config_subcommand {
        // Handle Migrate separately without loading config first
        Migrate => super::migrate::run(),
        Category(args) => {
            let mut configuration = config::load()?;
            match args.command {
                CategoryOperation::Add { value } => configuration.add_category(value)?,
                CategoryOperation::Remove { value } => configuration.remove_category(value)?,
            }
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        ChangeType(args) => {
            let mut configuration = config::load()?;
            match args.command {
                ChangeTypeConfigOperation::Add { long, short } => {
                    configuration.add_change_type(long, short)?
                }
                ChangeTypeConfigOperation::Remove { short } => {
                    configuration.remove_change_type(short)?
                }
            }
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        Show => {
            let configuration = config::load()?;
            println!("{}", configuration);
            Ok(())
        }
        Spelling(args) => {
            let mut configuration = config::load()?;
            match args.command {
                KeyValueOperation::Add { key, value } => {
                    configuration.add_expected_spelling(key, value)?
                }
                KeyValueOperation::Remove { key } => configuration.remove_expected_spelling(key)?,
            }
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        LegacyVersion(args) => {
            let mut configuration = config::load()?;
            match args.command {
                OptionalOperation::Set { value } => configuration.legacy_version = Some(value),
                OptionalOperation::Unset => configuration.legacy_version = None,
            }
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        TargetRepo(args) => {
            let mut configuration = config::load()?;
            config::set_target_repo(&mut configuration, args.value)?;
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        ChangelogDir(args) => {
            let mut configuration = config::load()?;
            match args.command {
                OptionalOperation::Set { value } => configuration.set_changelog_dir(Some(value)),
                OptionalOperation::Unset => configuration.set_changelog_dir(None),
            }
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        Mode(args) => {
            let mut configuration = config::load()?;
            let mode = args.value.parse::<config::Mode>().map_err(|e| {
                errors::CLIError::ConfigAdjustError(errors::ConfigAdjustError::InvalidMode(e))
            })?;
            configuration.set_mode(mode);
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
        UseCategories(args) => {
            let mut configuration = config::load()?;
            match args.command {
                OptionalOperation::Set { value } => {
                    let use_categories = value.parse::<bool>().map_err(|_| {
                        errors::CLIError::ConfigAdjustError(
                            errors::ConfigAdjustError::InvalidBoolean(value),
                        )
                    })?;
                    configuration.set_use_categories(use_categories);
                }
                OptionalOperation::Unset => configuration.set_use_categories(false),
            }
            configuration.export(Path::new(".clconfig.json"))?;
            Ok(())
        }
    }
}
