/*
Main file to run the changelog utils application.
*/
use clap::Parser;
use clu::cli::{
    add, check, check_diff, commands::ChangelogCLI, config, create_pr, get, init, lint, release,
};
use eyre::WrapErr;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    match ChangelogCLI::parse() {
        ChangelogCLI::Add(add_args) => {
            add::run(add_args.number, add_args.yes, add_args.all_previous)
                .await
                .wrap_err("Failed to add changelog entry")?;
            Ok(())
        }
        ChangelogCLI::Check => {
            check::run()
                .await
                .wrap_err("Failed to check changelog configuration")?;
            Ok(())
        }
        ChangelogCLI::CheckDiff => {
            check_diff::run()
                .await
                .wrap_err("Failed to check changelog diff")?;
            Ok(())
        }
        ChangelogCLI::CreatePR => {
            create_pr::run()
                .await
                .wrap_err("Failed to create pull request")?;
            Ok(())
        }
        ChangelogCLI::Fix => {
            lint::run(true)
                .wrap_err("Failed to fix changelog formatting")?;
            Ok(())
        }
        ChangelogCLI::Get(get_args) => {
            get::run(get_args)
                .wrap_err("Failed to get changelog entries")?;
            Ok(())
        }
        ChangelogCLI::Lint => {
            lint::run(false)
                .wrap_err("Failed to lint changelog")?;
            Ok(())
        }
        ChangelogCLI::Init => {
            init::run()
                .wrap_err("Failed to initialize changelog configuration")?;
            Ok(())
        }
        ChangelogCLI::Config(config_subcommand) => {
            config::adjust_config(config_subcommand)
                .wrap_err("Failed to adjust changelog configuration")?;
            Ok(())
        }
        ChangelogCLI::Release(args) => {
            release::run(args.version)
                .wrap_err("Failed to create release")?;
            Ok(())
        }
    }
}
