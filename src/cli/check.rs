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
    match config::load() {
        Ok(c) => {
            config = c;
            println!(" ✅ valid config");
        }
        // TODO: if config not found suggest running init
        Err(ConfigError::FailedToReadWrite(_)) => println!(" ❌ config not found"),
        Err(ConfigError::FailedToParse(e)) => println!(" 🚧 invalid config: {}", e),
        // TODO: return error here?
        Err(e) => panic!("unexpected condition: {}", e),
    };

    if fs::exists(config.changelog_path).expect("failed to check changelog path") {
        println!(" ✅ changelog exists");
    } else {
        println!(" ❌ no changelog found");
    }

    if let Some(cd) = config.changelog_dir {
        if fs::exists(cd).expect("failed to check multifile changelog dir") {
            println!(" ✅ multi-file changelog directory exists");
        } else {
            println!(" ❌ multi-file changelog selected, but configured directory not found");
        }
    }

    // // TODO: check for AI support?
    // let model = parrot::anthropic::load()?;

    Ok(())
}
