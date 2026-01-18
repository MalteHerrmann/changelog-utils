use crate::{config, errors::CLIError};
use std::path::Path;

pub fn run() -> Result<(), CLIError> {
    // TODO: this is currently requiring a successful load, but actually down the line it will be required to
    // load the config with past versions of the config.
    let mut cfg = config::load()?;

    if !config::needs_migration(&cfg) {
        println!(
            "Configuration is already at version {}",
            config::CURRENT_CONFIG_VERSION
        );
        println!("No migration needed.");
        return Ok(());
    }

    println!("{}", config::get_migration_info(&cfg));
    println!("\nMigrating configuration...");

    config::migrate_to_current(&mut cfg)?;

    cfg.export(Path::new(".clconfig.json"))?;

    println!(
        "✓ Configuration migrated successfully to version {}",
        config::CURRENT_CONFIG_VERSION
    );
    println!("\nUpdated configuration:");
    println!("{}", cfg);

    Ok(())
}
