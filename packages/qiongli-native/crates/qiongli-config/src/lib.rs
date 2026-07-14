//! Versioned native Qiongli configuration boundary.

mod document;
mod error;
mod path;
mod redaction;
mod secret;
mod store;

pub use document::{
    ArxivSettings, CrossrefSettings, EmailAddress, EmailAddressError,
    GLOBAL_SETTINGS_DOCUMENT_KIND, GLOBAL_SETTINGS_SCHEMA_VERSION, GlobalSettings,
    LoadedGlobalSettings, MAX_GLOBAL_SETTINGS_BYTES, MAX_GLOBAL_SETTINGS_REVISION,
    OpenAlexSettings, ProviderReadiness, ProviderSettings, PubmedSettings, SemanticScholarSettings,
};
pub use error::{ConfigError, PersistenceStage};
pub use path::{ConfigRoot, ConfigRootSource, resolve_config_root};
pub use redaction::{
    ConfigState, RedactedConfigStatus, RedactedProviderStatus, RedactedProviderStatuses,
};
pub use secret::{
    MAX_SECRET_VALUE_BYTES, SecretRef, SecretRefError, SecretStore, SecretStoreError,
    SecretStoreStatus, SecretValue, UnavailableSecretStore,
};
pub use store::{CommitOutcome, GLOBAL_SETTINGS_FILE, GlobalSettingsStore};
