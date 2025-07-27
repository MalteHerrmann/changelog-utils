/*
Main file to run the changelog utils application.
*/
use clap::Parser;
use clu::{
    add, check_diff, cli::ChangelogCLI, cli_config, create_pr, errors::CLIError, get, init, lint,
    release_cli,
};

#[tokio::main]
async fn main() -> Result<(), CLIError> {
    match ChangelogCLI::parse() {
        ChangelogCLI::Add(add_args) => Ok(add::run(add_args.number, add_args.yes).await?),
        ChangelogCLI::CheckDiff => Ok(check_diff::run().await?),
        ChangelogCLI::CreatePR => Ok(create_pr::run().await?),
        ChangelogCLI::Fix => Ok(lint::run(true)?),
        ChangelogCLI::Get(get_args) => Ok(get::run(get_args)?),
        ChangelogCLI::Lint => Ok(lint::run(false)?),
        ChangelogCLI::Init => Ok(init::run()?),
        ChangelogCLI::Config(config_subcommand) => {
            Ok(cli_config::adjust_config(config_subcommand)?)
        }
        ChangelogCLI::Release(args) => Ok(release_cli::run(args.version)?),
    }
}
