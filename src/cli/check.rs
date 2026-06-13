use eyre::{bail, WrapErr};
use std::fs;

use crate::config::{self, Config};

/// Runs the logic to check the tool's state in the current
/// working directory.
pub async fn run() -> eyre::Result<()> {
    check().await
}

/// Checks the state of the tool in the current working directory.
async fn check() -> eyre::Result<()> {
    let mut config = Config::default();
    let mut has_critical_error = false;

    // Check configuration
    match config::load() {
        Ok(c) => {
            config = c;
            println!(" ✅ valid config");
        }
        Err(e) => {
            // Check if it's a file not found error
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    println!(" ❌ config not found");
                    println!(" 💡 run 'clu init' to create configuration");
                    has_critical_error = true;
                } else {
                    println!(" ❌ invalid config: {}", e);
                    has_critical_error = true;
                }
            } else {
                println!(" ❌ invalid config: {}", e);
                has_critical_error = true;
            }
        }
    };

    // If config failed, we can't proceed with other checks
    if has_critical_error {
        println!("\n ❌ Check unsuccessful - see errors above");
        bail!("Configuration validation failed");
    }

    // Check changelog file
    if fs::exists(&config.changelog_path)
        .wrap_err_with(|| {
            format!(
                "Failed to check if changelog exists at {}",
                config.changelog_path
            )
        })?
    {
        println!(" ✅ changelog exists");
    } else {
        println!(" ❌ no changelog found at: {}", config.changelog_path);
        has_critical_error = true;
    }

    // Check multi-file changelog directory if configured
    if let Some(cd) = &config.changelog_dir {
        if fs::exists(cd)
            .wrap_err_with(|| format!("Failed to check if changelog directory exists at {}", cd))?
        {
            println!(" ✅ multi-file changelog directory exists");
        } else {
            println!(" ❌ multi-file changelog directory not found at: {}", cd);
            has_critical_error = true;
        }
    }

    // Check LLM availability (non-critical)
    match parrot::llm::get_available_models() {
        Ok(models) => {
            if models.is_empty() {
                println!(" ⚠️  no LLM provider available (AI features unavailable)");
            } else {
                let provider_names: Vec<String> = models.iter().map(|m| m.get_name()).collect();
                println!(" ✅ LLM available: {}", provider_names.join(", "));
            }
        }
        Err(e) => {
            println!(" ⚠️  failed to check LLM providers: {}", e);
        }
    }

    // Return appropriate result
    if has_critical_error {
        println!("\n ❌ Check unsuccessful - see errors above");
        bail!("Check found critical errors");
    }

    println!("\n ✅ Check complete");
    Ok(())
}
