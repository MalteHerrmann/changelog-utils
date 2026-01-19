use crate::{
    config::{self, Config},
    errors::ChangelogError,
    multi_file,
    single_file,
};
use std::path::{Path, PathBuf};

/// Common trait for changelog operations, implemented by both single-file and multi-file changelogs
pub trait Changelog {
    /// Returns the path to the changelog (file or directory)
    fn get_path(&self) -> &Path;

    /// Returns the list of problems found in the changelog
    fn get_problems(&self) -> &[String];

    /// Returns all PR numbers found across all releases
    fn get_all_pr_numbers(&self) -> Vec<u64>;

    /// Writes the changelog to the given path
    /// Multi-file implementations may return NotImplemented error
    fn write(&self, config: &Config, export_path: &Path) -> Result<(), ChangelogError>;

    /// Returns the fixed contents as a String
    /// Multi-file implementations may return NotImplemented error
    fn get_fixed_contents(&self, config: &Config) -> Result<String, ChangelogError>;

    /// Returns the path to the changelog as a PathBuf
    fn path(&self) -> PathBuf {
        self.get_path().to_path_buf()
    }
}

/// Loads the changelog based on the mode specified in the configuration
pub fn load(config: &Config) -> Result<Box<dyn Changelog>, ChangelogError> {
    match config.mode {
        config::Mode::Single => Ok(Box::new(single_file::load(config)?)),
        config::Mode::Multi => Ok(Box::new(multi_file::load(config)?)),
    }
}
