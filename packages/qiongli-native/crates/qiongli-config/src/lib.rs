//! Versioned native Qiongli configuration boundary.

mod document;
mod error;
mod path;
mod secret;

pub use document::{
    ArxivSettings, CrossrefSettings, EmailAddress, EmailAddressError, GlobalSettings,
    OpenAlexSettings, ProviderReadiness, ProviderSettings, PubmedSettings, SemanticScholarSettings,
};
pub use error::{ConfigError, PersistenceStage};
pub use path::{ConfigRoot, ConfigRootSource, resolve_config_root};
pub use secret::{
    MAX_SECRET_VALUE_BYTES, SecretRef, SecretRefError, SecretStore, SecretStoreError,
    SecretStoreStatus, SecretValue, UnavailableSecretStore,
};
