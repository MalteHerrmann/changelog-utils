use crate::config;
use eyre::WrapErr;
use std::path::Path;

pub fn run() -> eyre::Result<()> {
    // TODO: this is currently requiring a successful load, but actually down the line it will be required to
    // load the config with past versions of the config.
    let mut cfg = config::load()
        .wrap_err("Failed to load configuration")?;

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

    config::migrate_to_current(&mut cfg)
        .wrap_err("Failed to migrate configuration")?;

    cfg.export(Path::new(".clconfig.json"))
        .wrap_err("Failed to export migrated configuration")?;

    println!(
        "✓ Configuration migrated successfully to version {}",
        config::CURRENT_CONFIG_VERSION
    );
    println!("\nUpdated configuration:");
    println!("{}", cfg);

    Ok(())
}
