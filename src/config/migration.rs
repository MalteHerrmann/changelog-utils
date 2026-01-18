use super::{Config, CURRENT_CONFIG_VERSION};
use crate::errors::ConfigError;

/// Represents a migration function that transforms a config from one version to another
type MigrationFn = fn(&mut Config) -> Result<(), ConfigError>;

/// Represents a migration between two versions
struct Migration {
    from_version: u16,
    to_version: u16,
    migrate: MigrationFn,
}

/// Registry of all available migrations
fn get_migrations() -> Vec<Migration> {
    vec![
        // Future migrations would be added here
        // Example:
        // Migration {
        //     from_version: 1,
        //     to_version: 2,
        //     migrate: migrate_1_to_2,
        // },
    ]
}

/// Applies all necessary migrations to bring a config to the current version
pub fn migrate_to_current(config: &mut Config) -> Result<(), ConfigError> {
    let current_version = config.get_version();

    if current_version == CURRENT_CONFIG_VERSION {
        return Ok(());
    }

    let migrations = get_migrations();
    let mut version = current_version;

    // Find and apply migration chain
    loop {
        if version == CURRENT_CONFIG_VERSION {
            break;
        }

        let migration = migrations
            .iter()
            .find(|m| m.from_version == version)
            .ok_or_else(|| {
                ConfigError::InvalidConfig(format!(
                    "No migration path found from version {} to {}",
                    version, CURRENT_CONFIG_VERSION
                ))
            })?;

        (migration.migrate)(config)?;
        version = migration.to_version;
    }

    // Update version to current
    config.update_to_current_version();

    Ok(())
}

/// Checks if a migration is needed for the config
pub fn needs_migration(config: &Config) -> bool {
    assert!(
        config.config_version <= CURRENT_CONFIG_VERSION,
        "invalid config version; cannot be higher than {}",
        CURRENT_CONFIG_VERSION
    );
    config.config_version < CURRENT_CONFIG_VERSION
}

/// Gets a description of what would be migrated
pub fn get_migration_info(config: &Config) -> String {
    let from = config.get_version();
    let to = CURRENT_CONFIG_VERSION;

    if from == to {
        return "Configuration is already at the current version.".to_string();
    }

    format!(
        "Configuration will be migrated from version {} to version {}.\nNew fields will be added with default values.",
        from, to
    )
}
