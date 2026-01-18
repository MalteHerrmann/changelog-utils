use std::fs;

use crate::{
    config::{self, Config},
    errors::{CheckError, ConfigError},
};

/// Runs the logic to check the tool's state in the current
/// working directory.
pub async fn run() -> Result<(), CheckError> {
    check().await
}

/// Checks the state of the tool in the current working directory.
async fn check() -> Result<(), CheckError> {
    let mut config = Config::default();
    let mut has_critical_error = false;

    // Check configuration
    match config::load() {
        Ok(c) => {
            config = c;
            println!(" ✅ valid config");
        }
        Err(ConfigError::FailedToReadWrite(_)) => {
            println!(" ❌ config not found");
            println!(" 💡 run 'clu init' to create configuration");
            has_critical_error = true;
        }
        Err(ConfigError::FailedToParse(e)) => {
            println!(" ❌ invalid config: {}", e);
            has_critical_error = true;
        }
        Err(ConfigError::InvalidConfig(e)) => {
            println!(" ❌ invalid config: {}", e);
            has_critical_error = true;
        }
    };

    // If config failed, we can't proceed with other checks
    if has_critical_error {
        println!("\n ❌ Check unsuccessful - see errors above");
        return Err(CheckError::ConfigInvalid);
    }

    // Check changelog file
    if fs::exists(&config.changelog_path)? {
        println!(" ✅ changelog exists");
    } else {
        println!(" ❌ no changelog found at: {}", config.changelog_path);
        has_critical_error = true;
    }

    // Check multi-file changelog directory if configured
    if let Some(cd) = &config.changelog_dir {
        if fs::exists(cd)? {
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
                let provider_names: Vec<String> =
                    models.iter().map(|m| m.get_name()).collect();
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
        if !fs::exists(&config.changelog_path)? {
            return Err(CheckError::ChangelogNotFound);
        }
        if let Some(cd) = &config.changelog_dir {
            if !fs::exists(cd)? {
                return Err(CheckError::MultiFileDirNotFound);
            }
        }
        // Fallback error if we have critical error but didn't match specific cases
        return Err(CheckError::ConfigInvalid);
    }

    println!("\n ✅ Check complete");
    Ok(())
}
