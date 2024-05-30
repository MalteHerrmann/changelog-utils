/*
Main file to run the changelog utils application.
*/
use clu::{cli::ChangelogCLI, errors::CLIError, lint};
use clap::Parser;

fn main() -> Result<(), CLIError>{
    match ChangelogCLI::parse() {
        ChangelogCLI::Lint => {
            Ok(lint::run()?)
        }
    }
}
