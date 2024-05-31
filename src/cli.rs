use clap::{Args, Parser};

#[derive(Parser, Debug)]
pub enum ChangelogCLI {
    Lint(LintArgs),
}

#[derive(Args, Debug)]
pub struct LintArgs {
    #[arg(short, long)]
    pub fix: bool
}
