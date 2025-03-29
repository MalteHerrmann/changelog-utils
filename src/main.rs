/*
Main file to run the changelog utils application.
*/
use clap::Parser;
use clu::{
    add, cli::ChangelogCLI, cli_config, create_pr, errors::CLIError, init, lint, release_cli,
};

#[tokio::main]
async fn main() -> Result<(), CLIError> {
    match ChangelogCLI::parse() {
        ChangelogCLI::Add(add_args) => Ok(add::run(add_args.number, add_args.yes).await?),
        ChangelogCLI::CreatePR => Ok(create_pr::run().await?),
        ChangelogCLI::Fix => Ok(lint::run(true)?),
        ChangelogCLI::Lint => Ok(lint::run(false)?),
        ChangelogCLI::Init => Ok(init::run()?),
        ChangelogCLI::Config(config_subcommand) => {
            Ok(cli_config::adjust_config(config_subcommand)?)
        }
        ChangelogCLI::Release(args) => Ok(release_cli::run(args.version)?),
    }
}
