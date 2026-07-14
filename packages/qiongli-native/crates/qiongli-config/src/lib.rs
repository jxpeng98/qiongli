//! Versioned native Qiongli configuration boundary.

mod error;
mod path;

pub use error::{ConfigError, PersistenceStage};
pub use path::{ConfigRoot, ConfigRootSource, resolve_config_root};
