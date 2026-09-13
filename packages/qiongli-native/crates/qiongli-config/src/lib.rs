//! Versioned native Qiongli configuration boundary.

mod document;
mod error;
mod legacy;
mod path;
mod redaction;
mod secret;
mod store;
mod update;
mod workflow_variant;

pub use document::{
    AgentBackendSettings, ArxivSettings, CrossrefSettings, EmailAddress, EmailAddressError,
    GLOBAL_SETTINGS_DOCUMENT_KIND, GLOBAL_SETTINGS_SCHEMA_VERSION, GlobalSettings,
    LoadedGlobalSettings, MAX_GLOBAL_SETTINGS_BYTES, MAX_GLOBAL_SETTINGS_REVISION,
    OpenAiAgentBackendSettings, OpenAlexSettings, ProviderReadiness, ProviderSettings,
    PubmedSettings, SemanticScholarSettings,
};
pub use error::{ConfigError, PersistenceStage};
pub use legacy::{
    LEGACY_PROVIDER_CONFIG_FILE, LegacyProviderConfig, LegacyProviderConfigError,
    LegacyProviderConfigSummary, LegacyProviderConflict, LegacyProviderId,
    LegacyProviderResolution, LegacyProviderResolutionStrategy, LegacyProviderSecret,
    inspect_legacy_provider_config,
};
pub use path::{ConfigRoot, ConfigRootSource, resolve_config_root};
pub use redaction::{
    ConfigState, RedactedAgentBackendStatus, RedactedAgentBackendStatuses, RedactedConfigStatus,
    RedactedProviderStatus, RedactedProviderStatuses,
};
pub use secret::{
    MAX_SECRET_VALUE_BYTES, SecretRef, SecretRefError, SecretStore, SecretStoreError,
    SecretStoreStatus, SecretValue, SecretValueError, UnavailableSecretStore,
};
pub use store::{CommitOutcome, GLOBAL_SETTINGS_FILE, GlobalSettingsStore};
pub use update::{
    LoadedUpdateState, MAX_UPDATE_STATE_BYTES, UPDATE_STATE_DOCUMENT_KIND, UPDATE_STATE_FILE,
    UPDATE_STATE_SCHEMA_VERSION, UpdateActiveTransaction, UpdateLastKnownGood,
    UpdateReleaseChannel, UpdateState, UpdateStateStore, UpdateStreamPreference,
    UpdateTransactionPhase,
};
pub use workflow_variant::{
    LoadedWorkflowVariant, WORKFLOW_VARIANT_DIRECTORY, WORKFLOW_VARIANT_RECEIPT_FILE,
    WorkflowVariantCommit, WorkflowVariantError, WorkflowVariantPreview, WorkflowVariantStore,
};
