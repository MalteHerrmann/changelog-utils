use clap::{Args, Parser};

#[derive(Parser, Debug)]
pub enum ChangelogCLI {
    Lint(LintArgs),
    Init,
}

#[derive(Args, Debug)]
pub struct LintArgs {
    #[arg(short, long)]
    pub fix: bool,
}
