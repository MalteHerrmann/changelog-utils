/*
Main file to run the changelog utils application.
*/
use clap::Parser;
use clu::{add, cli::ChangelogCLI, cli_config, errors::CLIError, init, lint};

fn main() -> Result<(), CLIError> {
    match ChangelogCLI::parse() {
        ChangelogCLI::Add => Ok(add::run()?),
        ChangelogCLI::Fix => Ok(lint::run(true)?),
        ChangelogCLI::Lint => Ok(lint::run(false)?),
        ChangelogCLI::Init => Ok(init::run()?),
        ChangelogCLI::Config(config_subcommand) => {
            Ok(cli_config::adjust_config(config_subcommand)?)
        }
    }
}
