use std::str;
use std::sync::Arc;

use qiongli_config::{GlobalSettings, SecretRef, SecretStore, SecretStoreError, SecretStoreStatus};
use serde::{Deserialize, Serialize};

use crate::{
    AgentBackendError, AgentBackendErrorCode, CancellationToken, OpenAiBackendConfigV1,
    OpenAiResponsesBackend,
};

pub const BACKEND_CONTROL_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendReadinessV1 {
    Disabled,
    NeedsSecretReference,
    SecretStoreUnavailable,
    CredentialUnverified,
    CredentialMissing,
    CredentialInvalid,
    Ready,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackendStatusV1 {
    pub schema_version: u32,
    pub backend_id: String,
    pub model: String,
    pub enabled: bool,
    pub readiness: BackendReadinessV1,
    pub test_available: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendConnectionTestOutcomeV1 {
    Passed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackendConnectionTestV1 {
    pub schema_version: u32,
    pub backend_id: String,
    pub model: String,
    pub outcome: BackendConnectionTestOutcomeV1,
    pub provider_storage: String,
    pub hosted_tools: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendControlError {
    Disabled,
    SecretReferenceMissing,
    SecretStoreUnavailable,
    CredentialUnverified,
    CredentialMissing,
    CredentialInvalid,
    AuthenticationUnavailable,
    TransportUnavailable,
    ProviderRejected,
    ResponseInvalid,
    Cancelled,
}

impl BackendControlError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Disabled => "agent-backend-disabled",
            Self::SecretReferenceMissing => "agent-backend-secret-reference-missing",
            Self::SecretStoreUnavailable => "agent-backend-secret-store-unavailable",
            Self::CredentialUnverified => "agent-backend-credential-unverified",
            Self::CredentialMissing => "agent-backend-credential-missing",
            Self::CredentialInvalid => "agent-backend-credential-invalid",
            Self::AuthenticationUnavailable => "agent-backend-authentication-unavailable",
            Self::TransportUnavailable => "agent-backend-transport-unavailable",
            Self::ProviderRejected => "agent-backend-provider-rejected",
            Self::ResponseInvalid => "agent-backend-response-invalid",
            Self::Cancelled => "agent-backend-test-cancelled",
        }
    }
}

impl From<AgentBackendError> for BackendControlError {
    fn from(error: AgentBackendError) -> Self {
        match error.code {
            AgentBackendErrorCode::InvalidRequest
            | AgentBackendErrorCode::CapabilityUnavailable => Self::ResponseInvalid,
            AgentBackendErrorCode::AuthenticationUnavailable => Self::AuthenticationUnavailable,
            AgentBackendErrorCode::TransportUnavailable => Self::TransportUnavailable,
            AgentBackendErrorCode::ProviderRejected => Self::ProviderRejected,
            AgentBackendErrorCode::ResponseInvalid => Self::ResponseInvalid,
            AgentBackendErrorCode::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Clone)]
pub struct BackendControlService {
    enabled: bool,
    secret_ref: Option<SecretRef>,
    secrets: Arc<dyn SecretStore>,
}

impl BackendControlService {
    #[must_use]
    pub fn from_global_settings(settings: &GlobalSettings, secrets: Arc<dyn SecretStore>) -> Self {
        Self {
            enabled: settings.agent_backends.openai.enabled,
            secret_ref: settings.agent_backends.openai.api_key_ref.clone(),
            secrets,
        }
    }

    #[must_use]
    pub fn openai_status(&self) -> BackendStatusV1 {
        openai_backend_status_parts(
            self.enabled,
            self.secret_ref.as_ref(),
            self.secrets.as_ref(),
        )
    }

    pub fn test_openai_connection(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<BackendConnectionTestV1, BackendControlError> {
        match self.openai_readiness() {
            BackendReadinessV1::Disabled => return Err(BackendControlError::Disabled),
            BackendReadinessV1::NeedsSecretReference => {
                return Err(BackendControlError::SecretReferenceMissing);
            }
            BackendReadinessV1::SecretStoreUnavailable => {
                return Err(BackendControlError::SecretStoreUnavailable);
            }
            BackendReadinessV1::CredentialUnverified => {
                return Err(BackendControlError::CredentialUnverified);
            }
            BackendReadinessV1::CredentialMissing => {
                return Err(BackendControlError::CredentialMissing);
            }
            BackendReadinessV1::CredentialInvalid => {
                return Err(BackendControlError::CredentialInvalid);
            }
            BackendReadinessV1::Ready => {}
        }
        let reference = self
            .secret_ref
            .clone()
            .ok_or(BackendControlError::SecretReferenceMissing)?;
        let backend = OpenAiResponsesBackend::for_connection_test(
            OpenAiBackendConfigV1::gpt_5_6_sol(reference),
            Arc::clone(&self.secrets),
        )
        .map_err(BackendControlError::from)?;
        backend
            .test_connection(cancellation)
            .map_err(BackendControlError::from)?;
        Ok(BackendConnectionTestV1 {
            schema_version: BACKEND_CONTROL_SCHEMA_VERSION,
            backend_id: "openai-responses".to_owned(),
            model: OpenAiBackendConfigV1::model_id().to_owned(),
            outcome: BackendConnectionTestOutcomeV1::Passed,
            provider_storage: "disabled".to_owned(),
            hosted_tools: "disabled".to_owned(),
        })
    }

    fn openai_readiness(&self) -> BackendReadinessV1 {
        openai_backend_readiness(
            self.enabled,
            self.secret_ref.as_ref(),
            self.secrets.as_ref(),
        )
    }
}

#[must_use]
pub fn openai_backend_status(
    settings: &GlobalSettings,
    secrets: &dyn SecretStore,
) -> BackendStatusV1 {
    openai_backend_status_parts(
        settings.agent_backends.openai.enabled,
        settings.agent_backends.openai.api_key_ref.as_ref(),
        secrets,
    )
}

/// Reports startup-safe backend metadata without resolving the credential.
///
/// Native credential stores may display a blocking authorization prompt when
/// read. Snapshot and readiness surfaces must therefore remain metadata-only;
/// credential resolution belongs to an explicit connection test or run.
#[must_use]
pub fn openai_backend_metadata_status(
    settings: &GlobalSettings,
    secret_store_status: SecretStoreStatus,
) -> BackendStatusV1 {
    let enabled = settings.agent_backends.openai.enabled;
    let readiness = if !enabled {
        BackendReadinessV1::Disabled
    } else if settings.agent_backends.openai.api_key_ref.is_none() {
        BackendReadinessV1::NeedsSecretReference
    } else if secret_store_status != SecretStoreStatus::Available {
        BackendReadinessV1::SecretStoreUnavailable
    } else {
        BackendReadinessV1::CredentialUnverified
    };
    BackendStatusV1 {
        schema_version: BACKEND_CONTROL_SCHEMA_VERSION,
        backend_id: "openai-responses".to_owned(),
        model: OpenAiBackendConfigV1::model_id().to_owned(),
        enabled,
        readiness,
        test_available: readiness == BackendReadinessV1::CredentialUnverified,
    }
}

fn openai_backend_status_parts(
    enabled: bool,
    secret_ref: Option<&SecretRef>,
    secrets: &dyn SecretStore,
) -> BackendStatusV1 {
    let readiness = openai_backend_readiness(enabled, secret_ref, secrets);
    BackendStatusV1 {
        schema_version: BACKEND_CONTROL_SCHEMA_VERSION,
        backend_id: "openai-responses".to_owned(),
        model: OpenAiBackendConfigV1::model_id().to_owned(),
        enabled,
        readiness,
        test_available: readiness == BackendReadinessV1::Ready,
    }
}

fn openai_backend_readiness(
    enabled: bool,
    secret_ref: Option<&SecretRef>,
    secrets: &dyn SecretStore,
) -> BackendReadinessV1 {
    if !enabled {
        return BackendReadinessV1::Disabled;
    }
    let Some(reference) = secret_ref else {
        return BackendReadinessV1::NeedsSecretReference;
    };
    if secrets.status() != SecretStoreStatus::Available {
        return BackendReadinessV1::SecretStoreUnavailable;
    }
    match secrets.resolve(reference) {
        Ok(secret) if str::from_utf8(secret.as_bytes()).is_ok() => BackendReadinessV1::Ready,
        Ok(_) => BackendReadinessV1::CredentialInvalid,
        Err(SecretStoreError::NotFound) => BackendReadinessV1::CredentialMissing,
        Err(SecretStoreError::Unavailable | SecretStoreError::PersistenceFailed) => {
            BackendReadinessV1::SecretStoreUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use qiongli_config::{SecretValue, UnavailableSecretStore};

    use super::*;

    const SECRET_REF: &str = "qsr1_0123456789abcdef0123456789abcdef";

    #[derive(Clone)]
    struct FixedSecretStore(Result<Vec<u8>, SecretStoreError>);

    impl SecretStore for FixedSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(&self, _secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
            let bytes = self.0.clone()?;
            SecretValue::new(bytes).map_err(|_| SecretStoreError::PersistenceFailed)
        }
    }

    fn settings(enabled: bool, reference: bool) -> GlobalSettings {
        let mut settings = GlobalSettings::default();
        settings.agent_backends.openai.enabled = enabled;
        settings.agent_backends.openai.api_key_ref = reference
            .then(|| SecretRef::parse(SECRET_REF).expect("static reference must be valid"));
        settings
    }

    #[test]
    fn readiness_distinguishes_opt_in_reference_store_and_credential_state() {
        let disabled = BackendControlService::from_global_settings(
            &settings(false, false),
            Arc::new(UnavailableSecretStore),
        );
        assert_eq!(
            disabled.openai_status().readiness,
            BackendReadinessV1::Disabled
        );

        let no_reference = BackendControlService::from_global_settings(
            &settings(true, false),
            Arc::new(UnavailableSecretStore),
        );
        assert_eq!(
            no_reference.openai_status().readiness,
            BackendReadinessV1::NeedsSecretReference
        );

        let no_store = BackendControlService::from_global_settings(
            &settings(true, true),
            Arc::new(UnavailableSecretStore),
        );
        assert_eq!(
            no_store.openai_status().readiness,
            BackendReadinessV1::SecretStoreUnavailable
        );

        let missing = BackendControlService::from_global_settings(
            &settings(true, true),
            Arc::new(FixedSecretStore(Err(SecretStoreError::NotFound))),
        );
        assert_eq!(
            missing.openai_status().readiness,
            BackendReadinessV1::CredentialMissing
        );

        let ready = BackendControlService::from_global_settings(
            &settings(true, true),
            Arc::new(FixedSecretStore(Ok(b"test-key".to_vec()))),
        );
        assert_eq!(ready.openai_status().readiness, BackendReadinessV1::Ready);
        assert!(ready.openai_status().test_available);
    }

    #[test]
    fn metadata_status_defers_credential_resolution_until_an_explicit_operation() {
        let configured = settings(true, true);
        let status = openai_backend_metadata_status(&configured, SecretStoreStatus::Available);
        assert_eq!(status.readiness, BackendReadinessV1::CredentialUnverified);
        assert!(status.test_available);

        let unavailable =
            openai_backend_metadata_status(&configured, SecretStoreStatus::Unavailable);
        assert_eq!(
            unavailable.readiness,
            BackendReadinessV1::SecretStoreUnavailable
        );
        assert!(!unavailable.test_available);
    }

    #[test]
    fn unavailable_test_paths_fail_before_transport_construction() {
        let cancellation = CancellationToken::new();
        for (service, expected) in [
            (
                BackendControlService::from_global_settings(
                    &settings(false, false),
                    Arc::new(UnavailableSecretStore),
                ),
                BackendControlError::Disabled,
            ),
            (
                BackendControlService::from_global_settings(
                    &settings(true, false),
                    Arc::new(UnavailableSecretStore),
                ),
                BackendControlError::SecretReferenceMissing,
            ),
        ] {
            assert_eq!(service.test_openai_connection(&cancellation), Err(expected));
        }
    }
}
