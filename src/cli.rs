use clap::Parser;

#[derive(Parser, Debug)]
pub enum ChangelogCLI {
    #[command(about = "Lint the changelog contents")]
    Lint,
}
