use crate::{
    cli::{
        CategoryOperation, ConfigSubcommands,
        ConfigSubcommands::{Category, ChangeType, LegacyVersion, Show, Spelling, TargetRepo},
        HashMapOperation, OptionalOperation,
    },
    config, errors,
};
use std::fs;

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
            HashMapOperation::Add { key, value } => {
                config::add_into_hashmap(&mut configuration.change_types, key, value)?
            }
            HashMapOperation::Remove { key } => {
                config::remove_from_hashmap(&mut configuration.change_types, key)?
            }
        },
        Show => println!("{}", configuration),
        Spelling(args) => match args.command {
            HashMapOperation::Add { key, value } => {
                config::add_into_hashmap(&mut configuration.expected_spellings, key, value)?
            }
            HashMapOperation::Remove { key } => {
                config::remove_from_hashmap(&mut configuration.expected_spellings, key)?
            }
        },
        LegacyVersion(args) => match args.command {
            OptionalOperation::Set { value } => configuration.legacy_version = Some(value),
            OptionalOperation::Unset => configuration.legacy_version = None,
        },
        TargetRepo(args) => config::set_target_repo(&mut configuration, args.value)?,
    }

    Ok(fs::write(".clconfig.json", format!("{}", configuration))?)
}
