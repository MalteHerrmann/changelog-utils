use clap::Parser;

#[derive(Parser, Debug)]
pub enum ChangelogCLI {
    Lint,
}
