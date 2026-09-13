use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display, Formatter};

use serde::Serialize;
use zeroize::Zeroizing;

#[cfg(feature = "native-config")]
use qiongli_config::{GlobalSettings, SecretRef, SecretStore};

pub const PROVIDER_ORDER: [ProviderId; 5] = [
    ProviderId::OpenAlex,
    ProviderId::SemanticScholar,
    ProviderId::Crossref,
    ProviderId::PubMed,
    ProviderId::Arxiv,
];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    OpenAlex,
    SemanticScholar,
    Crossref,
    PubMed,
    Arxiv,
}

impl ProviderId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAlex => "openalex",
            Self::SemanticScholar => "semantic_scholar",
            Self::Crossref => "crossref",
            Self::PubMed => "pubmed",
            Self::Arxiv => "arxiv",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProviderIdError> {
        let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
        match normalized.as_str() {
            "openalex" => Ok(Self::OpenAlex),
            "semantic_scholar" | "semanticscholar" | "s2" => Ok(Self::SemanticScholar),
            "crossref" => Ok(Self::Crossref),
            "pubmed" | "ncbi" => Ok(Self::PubMed),
            "arxiv" => Ok(Self::Arxiv),
            _ => Err(ProviderIdError),
        }
    }
}

impl Display for ProviderId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderIdError;

impl Display for ProviderIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported-provider")
    }
}

impl std::error::Error for ProviderIdError {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProviderField {
    ApiKey,
    Email,
}

impl ProviderField {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::Email => "email",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProviderFieldError> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "api_key" => Ok(Self::ApiKey),
            "email" => Ok(Self::Email),
            _ => Err(ProviderFieldError),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderFieldError;

impl Display for ProviderFieldError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported-provider-field")
    }
}

impl std::error::Error for ProviderFieldError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailability {
    Disabled,
    Ready,
    NeedsSecret,
    NeedsPublicSetting,
    SecretStoreUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderStatus {
    pub provider: ProviderId,
    pub enabled: bool,
    pub readiness: ProviderAvailability,
}

#[derive(Clone)]
struct ProviderEntry {
    availability: ProviderAvailability,
    configured_fields: BTreeSet<ProviderField>,
    values: BTreeMap<ProviderField, Zeroizing<String>>,
}

impl Default for ProviderEntry {
    fn default() -> Self {
        Self {
            availability: ProviderAvailability::Disabled,
            configured_fields: BTreeSet::new(),
            values: BTreeMap::new(),
        }
    }
}

/// In-memory provider access. This type intentionally has no derived `Debug`
/// or serialization implementation because it can contain credentials.
#[derive(Clone, Default)]
pub struct ProviderAccess {
    providers: BTreeMap<ProviderId, ProviderEntry>,
}

impl Debug for ProviderAccess {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderAccess")
            .field("status", &self.status())
            .finish()
    }
}

impl ProviderAccess {
    #[must_use]
    pub fn builder() -> ProviderAccessBuilder {
        ProviderAccessBuilder::default()
    }

    #[must_use]
    pub fn is_enabled(&self, provider: ProviderId) -> bool {
        !matches!(self.availability(provider), ProviderAvailability::Disabled)
    }

    #[must_use]
    pub fn is_active(&self, provider: ProviderId) -> bool {
        matches!(self.availability(provider), ProviderAvailability::Ready)
    }

    #[must_use]
    pub fn availability(&self, provider: ProviderId) -> ProviderAvailability {
        self.providers
            .get(&provider)
            .map_or(ProviderAvailability::Disabled, |entry| entry.availability)
    }

    #[must_use]
    pub fn value(&self, provider: ProviderId, field: ProviderField) -> Option<&str> {
        self.providers
            .get(&provider)?
            .values
            .get(&field)
            .map(|value| value.as_str())
    }

    /// Report whether a field is configured without requiring its value to be
    /// loaded into memory. This keeps protocol/status paths independent from
    /// native credential stores.
    #[must_use]
    pub fn is_field_configured(&self, provider: ProviderId, field: ProviderField) -> bool {
        self.providers
            .get(&provider)
            .is_some_and(|entry| entry.configured_fields.contains(&field))
    }

    #[must_use]
    pub fn status(&self) -> Vec<ProviderStatus> {
        PROVIDER_ORDER
            .into_iter()
            .map(|provider| {
                let readiness = self.availability(provider);
                ProviderStatus {
                    provider,
                    enabled: !matches!(readiness, ProviderAvailability::Disabled),
                    readiness,
                }
            })
            .collect()
    }

    #[cfg(feature = "native-config")]
    #[must_use]
    pub fn from_global_settings(settings: &GlobalSettings, secret_store: &dyn SecretStore) -> Self {
        let mut builder = Self::builder();

        if settings.providers.openalex.enabled {
            match resolve_secret(
                settings.providers.openalex.api_key_ref.as_ref(),
                secret_store,
            ) {
                Ok(value) => {
                    builder.set_availability(ProviderId::OpenAlex, ProviderAvailability::Ready);
                    builder.set_value(ProviderId::OpenAlex, ProviderField::ApiKey, value);
                }
                Err(availability) => {
                    builder.set_availability(ProviderId::OpenAlex, availability);
                }
            }
            if let Some(email) = settings.providers.openalex.email.as_ref() {
                builder.set_value(ProviderId::OpenAlex, ProviderField::Email, email.as_str());
            }
        }

        if settings.providers.semantic_scholar.enabled {
            match resolve_secret(
                settings.providers.semantic_scholar.api_key_ref.as_ref(),
                secret_store,
            ) {
                Ok(value) => {
                    builder
                        .set_availability(ProviderId::SemanticScholar, ProviderAvailability::Ready);
                    builder.set_value(ProviderId::SemanticScholar, ProviderField::ApiKey, value);
                }
                Err(availability) => {
                    builder.set_availability(ProviderId::SemanticScholar, availability);
                }
            }
        }

        if settings.providers.crossref.enabled {
            if let Some(email) = settings.providers.crossref.email.as_ref() {
                builder.set_availability(ProviderId::Crossref, ProviderAvailability::Ready);
                builder.set_value(ProviderId::Crossref, ProviderField::Email, email.as_str());
            } else {
                builder.set_availability(
                    ProviderId::Crossref,
                    ProviderAvailability::NeedsPublicSetting,
                );
            }
        }

        if settings.providers.pubmed.enabled {
            match resolve_secret(settings.providers.pubmed.api_key_ref.as_ref(), secret_store) {
                Ok(value) => {
                    builder.set_availability(ProviderId::PubMed, ProviderAvailability::Ready);
                    builder.set_value(ProviderId::PubMed, ProviderField::ApiKey, value);
                }
                Err(availability) => {
                    builder.set_availability(ProviderId::PubMed, availability);
                }
            }
        }

        if settings.providers.arxiv.enabled {
            builder.set_availability(ProviderId::Arxiv, ProviderAvailability::Ready);
        }

        builder.build()
    }

    /// Build a redacted provider projection without resolving secret refs.
    ///
    /// This is safe for MCP initialization, tool discovery, and status calls:
    /// configured secret fields are represented only as booleans. Call
    /// [`Self::from_global_settings`] on the bounded execution path immediately
    /// before a provider request needs the credential value.
    #[cfg(feature = "native-config")]
    #[must_use]
    pub fn from_global_settings_metadata(settings: &GlobalSettings) -> Self {
        let mut builder = Self::builder();

        if settings.providers.openalex.enabled {
            if settings.providers.openalex.api_key_ref.is_some() {
                builder
                    .set_availability(ProviderId::OpenAlex, ProviderAvailability::Ready)
                    .set_field_configured(ProviderId::OpenAlex, ProviderField::ApiKey);
            } else {
                builder.set_availability(ProviderId::OpenAlex, ProviderAvailability::NeedsSecret);
            }
            if let Some(email) = settings.providers.openalex.email.as_ref() {
                builder.set_value(ProviderId::OpenAlex, ProviderField::Email, email.as_str());
            }
        }

        if settings.providers.semantic_scholar.enabled {
            if settings.providers.semantic_scholar.api_key_ref.is_some() {
                builder
                    .set_availability(ProviderId::SemanticScholar, ProviderAvailability::Ready)
                    .set_field_configured(ProviderId::SemanticScholar, ProviderField::ApiKey);
            } else {
                builder.set_availability(
                    ProviderId::SemanticScholar,
                    ProviderAvailability::NeedsSecret,
                );
            }
        }

        if settings.providers.crossref.enabled {
            if let Some(email) = settings.providers.crossref.email.as_ref() {
                builder
                    .set_availability(ProviderId::Crossref, ProviderAvailability::Ready)
                    .set_value(ProviderId::Crossref, ProviderField::Email, email.as_str());
            } else {
                builder.set_availability(
                    ProviderId::Crossref,
                    ProviderAvailability::NeedsPublicSetting,
                );
            }
        }

        if settings.providers.pubmed.enabled {
            if settings.providers.pubmed.api_key_ref.is_some() {
                builder
                    .set_availability(ProviderId::PubMed, ProviderAvailability::Ready)
                    .set_field_configured(ProviderId::PubMed, ProviderField::ApiKey);
            } else {
                builder.set_availability(ProviderId::PubMed, ProviderAvailability::NeedsSecret);
            }
        }

        if settings.providers.arxiv.enabled {
            builder.set_availability(ProviderId::Arxiv, ProviderAvailability::Ready);
        }

        builder.build()
    }
}

#[derive(Clone, Default)]
pub struct ProviderAccessBuilder {
    access: ProviderAccess,
}

impl ProviderAccessBuilder {
    pub fn set_availability(
        &mut self,
        provider: ProviderId,
        availability: ProviderAvailability,
    ) -> &mut Self {
        self.access
            .providers
            .entry(provider)
            .or_default()
            .availability = availability;
        self
    }

    pub fn set_value(
        &mut self,
        provider: ProviderId,
        field: ProviderField,
        value: impl AsRef<str>,
    ) -> &mut Self {
        let entry = self.access.providers.entry(provider).or_default();
        entry.configured_fields.insert(field);
        entry
            .values
            .insert(field, Zeroizing::new(value.as_ref().to_owned()));
        self
    }

    pub fn set_field_configured(
        &mut self,
        provider: ProviderId,
        field: ProviderField,
    ) -> &mut Self {
        self.access
            .providers
            .entry(provider)
            .or_default()
            .configured_fields
            .insert(field);
        self
    }

    #[must_use]
    pub fn build(self) -> ProviderAccess {
        self.access
    }
}

#[cfg(feature = "native-config")]
fn resolve_secret(
    reference: Option<&SecretRef>,
    secret_store: &dyn SecretStore,
) -> Result<Zeroizing<String>, ProviderAvailability> {
    let Some(reference) = reference else {
        return Err(ProviderAvailability::NeedsSecret);
    };
    let secret = secret_store
        .resolve(reference)
        .map_err(|_| ProviderAvailability::SecretStoreUnavailable)?;
    let value = Zeroizing::new(
        String::from_utf8(secret.as_bytes().to_vec())
            .map_err(|_| ProviderAvailability::NeedsSecret)?,
    );
    let value = value.trim();
    if value.is_empty() {
        Err(ProviderAvailability::NeedsSecret)
    } else {
        Ok(Zeroizing::new(value.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_resolve_to_canonical_provider_ids() {
        assert_eq!(
            ProviderId::parse("semantic-scholar"),
            Ok(ProviderId::SemanticScholar)
        );
        assert_eq!(ProviderId::parse("s2"), Ok(ProviderId::SemanticScholar));
        assert_eq!(ProviderId::parse("ncbi"), Ok(ProviderId::PubMed));
        assert_eq!(ProviderId::parse("unknown"), Err(ProviderIdError));
    }

    #[test]
    fn debug_and_status_do_not_render_access_values() {
        let mut builder = ProviderAccess::builder();
        builder
            .set_availability(ProviderId::OpenAlex, ProviderAvailability::Ready)
            .set_value(
                ProviderId::OpenAlex,
                ProviderField::ApiKey,
                "provider-secret-canary",
            );
        let access = builder.build();

        let rendered = format!(
            "{access:?} {}",
            serde_json::to_string(&access.status()).unwrap()
        );
        assert!(!rendered.contains("provider-secret-canary"));
        assert!(access.is_active(ProviderId::OpenAlex));
    }

    #[cfg(feature = "native-config")]
    #[test]
    fn native_defaults_enable_only_arxiv_without_secret_fallback() {
        use qiongli_config::UnavailableSecretStore;

        let access = ProviderAccess::from_global_settings(
            &GlobalSettings::default(),
            &UnavailableSecretStore,
        );

        assert!(access.is_active(ProviderId::Arxiv));
        assert!(!access.is_active(ProviderId::OpenAlex));
        assert_eq!(
            access.availability(ProviderId::OpenAlex),
            ProviderAvailability::Disabled
        );
    }

    #[cfg(feature = "native-config")]
    #[test]
    fn unavailable_store_is_redacted_and_distinct_from_missing_secret() {
        use qiongli_config::{SecretRef, UnavailableSecretStore};

        let secret_ref = "qsr1_0123456789abcdef0123456789abcdef";
        let mut settings = GlobalSettings::default();
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.api_key_ref = Some(SecretRef::parse(secret_ref).unwrap());
        settings.providers.semantic_scholar.enabled = true;
        let access = ProviderAccess::from_global_settings(&settings, &UnavailableSecretStore);
        let rendered = format!(
            "{access:?} {}",
            serde_json::to_string(&access.status()).unwrap()
        );

        assert_eq!(
            access.availability(ProviderId::OpenAlex),
            ProviderAvailability::SecretStoreUnavailable
        );
        assert_eq!(
            access.availability(ProviderId::SemanticScholar),
            ProviderAvailability::NeedsSecret
        );
        assert!(!rendered.contains(secret_ref));
    }

    #[cfg(feature = "native-config")]
    #[test]
    fn available_store_activates_provider_without_exposing_secret() {
        use qiongli_config::{
            SecretRef, SecretStore, SecretStoreError, SecretStoreStatus, SecretValue,
        };

        struct TestSecretStore;

        impl SecretStore for TestSecretStore {
            fn status(&self) -> SecretStoreStatus {
                SecretStoreStatus::Available
            }

            fn resolve(&self, _secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
                Ok(SecretValue::new(b"resolved-secret-canary".to_vec()).unwrap())
            }
        }

        let secret_ref = "qsr1_0123456789abcdef0123456789abcdef";
        let mut settings = GlobalSettings::default();
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.api_key_ref = Some(SecretRef::parse(secret_ref).unwrap());
        let access = ProviderAccess::from_global_settings(&settings, &TestSecretStore);
        let rendered = format!(
            "{access:?} {}",
            serde_json::to_string(&access.status()).unwrap()
        );

        assert!(access.is_active(ProviderId::OpenAlex));
        assert_eq!(
            access.value(ProviderId::OpenAlex, ProviderField::ApiKey),
            Some("resolved-secret-canary")
        );
        assert!(!rendered.contains("resolved-secret-canary"));
        assert!(!rendered.contains(secret_ref));
    }

    #[cfg(feature = "native-config")]
    #[test]
    fn metadata_projection_marks_secret_refs_without_resolving_values() {
        let secret_ref = "qsr1_0123456789abcdef0123456789abcdef";
        let mut settings = GlobalSettings::default();
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.api_key_ref = Some(SecretRef::parse(secret_ref).unwrap());

        let access = ProviderAccess::from_global_settings_metadata(&settings);

        assert!(access.is_active(ProviderId::OpenAlex));
        assert!(access.is_field_configured(ProviderId::OpenAlex, ProviderField::ApiKey));
        assert_eq!(
            access.value(ProviderId::OpenAlex, ProviderField::ApiKey),
            None
        );
        assert!(!format!("{access:?}").contains(secret_ref));
    }
}
