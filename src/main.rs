/*
Main file to run the changelog utils application.
*/
use changelog_utils::{cli::ChangelogCLI, lint};
use clap::Parser;
use std::process;

fn main() {
    match ChangelogCLI::parse() {
        ChangelogCLI::Lint => {
            if let Err(e) = lint::run() {
                println!("errors while linting changelog: {}", e);
                process::exit(1);
            };
        }
    };
}
