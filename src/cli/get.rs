use super::commands::GetArgs;
use crate::{config, multi_file, single_file};
use eyre::{bail, WrapErr};

/// Executes the get command to display a specific version's release notes.
pub fn run(args: GetArgs) -> eyre::Result<()> {
    let config = config::load()
        .wrap_err("Failed to load configuration")?;

    match config.mode {
        config::Mode::Single => {
            let changelog = single_file::changelog::load(&config)
                .wrap_err("Failed to load single-file changelog")?;
            get_single(&config, &changelog, &args)?;
        }
        config::Mode::Multi => {
            let changelog = multi_file::changelog::load(&config)
                .wrap_err("Failed to load multi-file changelog")?;
            get_multi(&config, &changelog, &args)?;
        }
    }

    Ok(())
}

/// TODO: this should be refactored to use a common function. From what I can see it's required to
/// have common traits for the release, change type and entry impementations as well?
///
/// Will have to investigate more..
fn get_single(
    config: &config::Config,
    changelog: &single_file::changelog::SingleFileChangelog,
    args: &GetArgs,
) -> eyre::Result<()> {
    if let Some(release) = changelog
        .releases
        .iter()
        .find(|&r| r.version == args.version)
    {
        println!("{}", release.get_fixed_contents(config));
        return Ok(());
    }

    bail!("Version '{}' not found in changelog", args.version)
}

fn get_multi(
    config: &config::Config,
    changelog: &multi_file::changelog::MultiFileChangelog,
    args: &GetArgs,
) -> eyre::Result<()> {
    if let Some(release) = changelog
        .releases
        .iter()
        .find(|&r| r.version == args.version)
    {
        println!("{}", release.get_fixed_contents(config));
        return Ok(());
    }

    bail!("Version '{}' not found in changelog", args.version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Creates a test config from the example config file
    fn load_test_config() -> config::Config {
        config::unpack_config(include_str!(
            "../testdata/example_config_without_optionals.json"
        ))
        .expect("failed to load example config")
    }

    /// Test that we can successfully find an existing version
    #[test]
    fn test_get_existing_version() {
        // Load the test config
        let config = load_test_config();

        // Parse the actual changelog from this repo
        let changelog_path = Path::new("CHANGELOG.md");
        if !changelog_path.exists() {
            // Skip test if changelog doesn't exist
            return;
        }

        let changelog = match single_file::changelog::parse_changelog(&config, changelog_path) {
            Ok(cl) => cl,
            Err(e) => {
                // Check if it's a "no changelog found" error
                if e.to_string().contains("No changelog found") {
                    return; // Skip test
                }
                panic!("Failed to parse changelog: {:?}", e);
            }
        };

        // The v1.0.0 version should exist in the changelog
        let result = get_single(
            &config,
            &changelog,
            &GetArgs {
                version: "v1.0.0".to_string(),
            },
        );
        assert!(result.is_ok());
    }

    /// Test handling of a non-existent version
    #[test]
    fn test_get_nonexistent_version() {
        // Load the test config
        let config = load_test_config();

        // Parse the actual changelog from this repo
        let changelog_path = Path::new("CHANGELOG.md");
        if !changelog_path.exists() {
            // Skip test if changelog doesn't exist
            return;
        }

        let changelog = match single_file::changelog::parse_changelog(&config, changelog_path) {
            Ok(cl) => cl,
            Err(e) => {
                // Check if it's a "no changelog found" error
                if e.to_string().contains("No changelog found") {
                    return; // Skip test
                }
                panic!("Failed to parse changelog: {:?}", e);
            }
        };

        // A version that definitely doesn't exist
        let result = get_single(
            &config,
            &changelog,
            &GetArgs {
                version: "v999.999.999".to_string(),
            },
        );
        assert!(result.is_err());

        // Check specific error message
        let err = result.unwrap_err();
        assert!(err.to_string().contains("v999.999.999"));
        assert!(err.to_string().contains("not found"));
    }
}
