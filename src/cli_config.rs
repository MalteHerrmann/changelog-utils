use crate::{
    cli::{
        CategoryOperation, ChangeTypeConfigOperation,
        ConfigSubcommands::{
            self, Category, ChangeType, LegacyVersion, Show, Spelling, TargetRepo,
        },
        KeyValueOperation, OptionalOperation,
    },
    config, errors,
};
use std::path::Path;

// Handles the CLI subcommands to adjust the configuration file.
pub fn adjust_config(config_subcommand: ConfigSubcommands) -> Result<(), errors::CLIError> {
    let mut configuration = config::load()?;

    match config_subcommand {
        Category(args) => match args.command {
            CategoryOperation::Add { value } => config::add_category(&mut configuration, value)?,
            CategoryOperation::Remove { value } => {
                config::remove_category(&mut configuration, value)?
            }
        },
        ChangeType(args) => match args.command {
            ChangeTypeConfigOperation::Add { long, short } => {
                config::add_change_type(&mut configuration, &long, &short)?
            }
            ChangeTypeConfigOperation::Remove { short } => {
                config::remove_change_type(&mut configuration, &short)?
            }
        },
        Show => println!("{}", configuration),
        Spelling(args) => match args.command {
            KeyValueOperation::Add { key, value } => {
                config::add_into_collection(&mut configuration.expected_spellings, key, value)?
            }
            KeyValueOperation::Remove { key } => {
                config::remove_from_collection(&mut configuration.expected_spellings, key)?
            }
        },
        LegacyVersion(args) => match args.command {
            OptionalOperation::Set { value } => configuration.legacy_version = Some(value),
            OptionalOperation::Unset => configuration.legacy_version = None,
        },
        TargetRepo(args) => config::set_target_repo(&mut configuration, args.value)?,
    }

    Ok(configuration.export(Path::new(".clconfig.json"))?)
}
