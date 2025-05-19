use crate::{changelog, config, errors::GetError, cli::GetArgs};

/// Executes the get command to display a specific version's release notes.
pub fn run(args: GetArgs) -> Result<(), GetError> {
    let config = config::load()?;
    let changelog = changelog::load(config)?;

    match get(&changelog, &args) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Version {} not found in changelog: {}", args.version, e);
            Err(e)
        }
    }
}

fn get(changelog: &changelog::Changelog, args: &GetArgs) -> Result<(), GetError> {
    let release = changelog.releases.iter().find(|r| {
        println!("checking {} against {}", r.version, args.version);
        r.version == args.version
    });
    
    match release {
        Some(release) => {
            // Print the release header
            //
            // TODO: add a method to the release struct to print the contents
            println!("{}", release.fixed);
            println!();

            // Print each change type and its entries
            for change_type in &release.change_types {
                println!("{}", change_type.fixed);
                println!();
                
                for entry in &change_type.entries {
                    println!("{}", entry.fixed);
                }
                println!();
            }
            Ok(())
        },
        None => {
            Err(GetError::VersionNotFound(args.version.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ChangelogError;
    use std::path::Path;

    /// Creates a test config from the example config file
    fn load_test_config() -> config::Config {
        config::unpack_config(include_str!("testdata/example_config.json"))
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
        
        let changelog = match changelog::parse_changelog(config, changelog_path) {
            Ok(cl) => cl,
            Err(ChangelogError::NoChangelogFound) => {
                // Skip test if changelog not found
                return;
            },
            Err(e) => panic!("Failed to parse changelog: {:?}", e),
        };
        
        // The v1.0.0 version should exist in the changelog
        let result = get(&changelog, &GetArgs { version: "v1.0.0".to_string() });
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
        
        let changelog = match changelog::parse_changelog(config, changelog_path) {
            Ok(cl) => cl,
            Err(ChangelogError::NoChangelogFound) => {
                // Skip test if changelog not found
                return;
            },
            Err(e) => panic!("Failed to parse changelog: {:?}", e),
        };
        
        // A version that definitely doesn't exist
        let result = get(&changelog, &GetArgs { version: "v999.999.999".to_string() });
        assert!(result.is_err());
        
        // Check specific error type
        match result {
            Err(GetError::VersionNotFound(version)) => {
                assert_eq!(version, "v999.999.999");
            },
            _ => panic!("Expected VersionNotFound error"),
        }
    }
} 