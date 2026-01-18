mod change_type;
pub mod config;
mod migration;
mod mode;

pub use change_type::ChangeTypeConfig;
pub use config::{load, set_target_repo, unpack_config, Config, CURRENT_CONFIG_VERSION};
pub use migration::{get_migration_info, migrate_to_current, needs_migration};
pub use mode::Mode;
