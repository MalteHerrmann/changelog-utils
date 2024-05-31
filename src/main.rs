/*
Main file to run the changelog utils application.
*/
use clap::Parser;
use clu::{cli::ChangelogCLI, errors::CLIError, init, lint};

fn main() -> Result<(), CLIError> {
    let cli = ChangelogCLI::parse();
    match cli {
        ChangelogCLI::Lint(args) => Ok(lint::run(args.fix)?),
        ChangelogCLI::Init => Ok(init::run()?),
    }
}
