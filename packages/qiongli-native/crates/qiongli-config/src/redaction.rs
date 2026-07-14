use qiongli_content::ProfileId;
use serde::Serialize;

use crate::{
    ConfigError, ConfigRoot, ConfigRootSource, GlobalSettings, LoadedGlobalSettings,
    ProviderReadiness,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigState {
    Missing,
    Ready,
    Invalid,
    FutureSchema,
    Insecure,
    Busy,
    RecoveryRequired,
    WriteUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedProviderStatus {
    pub enabled: bool,
    pub readiness: ProviderReadiness,
    pub secret_ref_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedProviderStatuses {
    pub openalex: RedactedProviderStatus,
    pub semantic_scholar: RedactedProviderStatus,
    pub crossref: RedactedProviderStatus,
    pub pubmed: RedactedProviderStatus,
    pub arxiv: RedactedProviderStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedConfigStatus {
    pub root_source: ConfigRootSource,
    pub symbolic_state_root: &'static str,
    pub state: ConfigState,
    pub revision: Option<u64>,
    pub default_profile: Option<ProfileId>,
    pub providers: Option<RedactedProviderStatuses>,
    pub secret_store: &'static str,
    pub remediation_code: &'static str,
    pub cleanup_required: bool,
}

impl RedactedConfigStatus {
    pub(crate) fn loaded(
        root: &ConfigRoot,
        state: ConfigState,
        loaded: &LoadedGlobalSettings,
        cleanup_required: bool,
    ) -> Self {
        Self {
            root_source: root.source(),
            symbolic_state_root: root.symbolic_state_root(),
            state,
            revision: Some(loaded.revision),
            default_profile: Some(loaded.settings.default_profile),
            providers: Some(RedactedProviderStatuses::from_settings(&loaded.settings)),
            secret_store: "unavailable",
            remediation_code: "secure-store-not-implemented",
            cleanup_required,
        }
    }

    pub(crate) fn failed(root: &ConfigRoot, error: &ConfigError) -> Self {
        Self {
            root_source: root.source(),
            symbolic_state_root: root.symbolic_state_root(),
            state: error_state(error),
            revision: None,
            default_profile: None,
            providers: None,
            secret_store: "unavailable",
            remediation_code: "secure-store-not-implemented",
            cleanup_required: matches!(error, ConfigError::RecoveryRequired),
        }
    }
}

impl RedactedProviderStatuses {
    fn from_settings(settings: &GlobalSettings) -> Self {
        Self {
            openalex: RedactedProviderStatus {
                enabled: settings.providers.openalex.enabled,
                readiness: settings.providers.openalex.readiness(),
                secret_ref_present: settings.providers.openalex.api_key_ref.is_some(),
            },
            semantic_scholar: RedactedProviderStatus {
                enabled: settings.providers.semantic_scholar.enabled,
                readiness: settings.providers.semantic_scholar.readiness(),
                secret_ref_present: settings.providers.semantic_scholar.api_key_ref.is_some(),
            },
            crossref: RedactedProviderStatus {
                enabled: settings.providers.crossref.enabled,
                readiness: settings.providers.crossref.readiness(),
                secret_ref_present: false,
            },
            pubmed: RedactedProviderStatus {
                enabled: settings.providers.pubmed.enabled,
                readiness: settings.providers.pubmed.readiness(),
                secret_ref_present: settings.providers.pubmed.api_key_ref.is_some(),
            },
            arxiv: RedactedProviderStatus {
                enabled: settings.providers.arxiv.enabled,
                readiness: settings.providers.arxiv.readiness(),
                secret_ref_present: false,
            },
        }
    }
}

const fn error_state(error: &ConfigError) -> ConfigState {
    match error {
        ConfigError::UnsupportedSchema { .. } => ConfigState::FutureSchema,
        ConfigError::UnsafeManagedPath | ConfigError::InsecurePermissions => ConfigState::Insecure,
        ConfigError::LockBusy => ConfigState::Busy,
        ConfigError::RecoveryRequired => ConfigState::RecoveryRequired,
        ConfigError::UnsupportedPlatformSecurity => ConfigState::WriteUnsupported,
        ConfigError::InvalidConfigHome
        | ConfigError::HomeUnavailable
        | ConfigError::InvalidDocumentKind
        | ConfigError::InvalidDocument
        | ConfigError::DocumentTooLarge
        | ConfigError::RevisionConflict { .. }
        | ConfigError::RevisionExhausted
        | ConfigError::PersistenceFailed { .. } => ConfigState::Invalid,
    }
}
