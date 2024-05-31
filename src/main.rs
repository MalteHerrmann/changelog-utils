/*
Main file to run the changelog utils application.
*/
use clu::{cli::ChangelogCLI, errors::CLIError, lint};
use clap::Parser;

fn main() -> Result<(), CLIError>{
    let cli = ChangelogCLI::parse();
    match cli {
        ChangelogCLI::Lint(args) => {
            Ok(lint::run(args.fix)?)
        }
    }
}
