use super::commands::{
    CategoryOperation, ChangeTypeConfigOperation,
    ConfigSubcommands::{self, Category, ChangeType, LegacyVersion, Show, Spelling, TargetRepo},
    KeyValueOperation, OptionalOperation,
};
use crate::{errors, utils::config};
use std::path::Path;

// Handles the CLI subcommands to adjust the configuration file.
pub fn adjust_config(config_subcommand: ConfigSubcommands) -> Result<(), errors::CLIError> {
    let mut configuration = config::load()?;

    match config_subcommand {
        Category(args) => match args.command {
            CategoryOperation::Add { value } => configuration.add_category(value)?,
            CategoryOperation::Remove { value } => configuration.remove_category(value)?,
        },
        ChangeType(args) => match args.command {
            ChangeTypeConfigOperation::Add { long, short } => {
                configuration.add_change_type(long, short)?
            }
            ChangeTypeConfigOperation::Remove { short } => {
                configuration.remove_change_type(short)?
            }
        },
        Show => println!("{}", configuration),
        Spelling(args) => match args.command {
            KeyValueOperation::Add { key, value } => {
                configuration.add_expected_spelling(key, value)?
            }
            KeyValueOperation::Remove { key } => configuration.remove_expected_spelling(key)?,
        },
        LegacyVersion(args) => match args.command {
            OptionalOperation::Set { value } => configuration.legacy_version = Some(value),
            OptionalOperation::Unset => configuration.legacy_version = None,
        },
        TargetRepo(args) => config::set_target_repo(&mut configuration, args.value)?,
    }

    Ok(configuration.export(Path::new(".clconfig.json"))?)
}
