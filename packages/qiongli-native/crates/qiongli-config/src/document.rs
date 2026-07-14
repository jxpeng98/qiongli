use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use qiongli_content::ProfileId;
use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};

use crate::{ConfigError, SecretRef};

pub const GLOBAL_SETTINGS_DOCUMENT_KIND: &str = "qiongli-global-settings";
pub const GLOBAL_SETTINGS_SCHEMA_VERSION: u64 = 1;
pub const MAX_GLOBAL_SETTINGS_BYTES: usize = 64 * 1024;
pub const MAX_GLOBAL_SETTINGS_REVISION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Eq, PartialEq)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse(value: &str) -> Result<Self, EmailAddressError> {
        let value = value.trim();
        let characters = value.chars().count();
        if !(1..=320).contains(&characters) || value.chars().any(char::is_control) {
            return Err(EmailAddressError);
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for EmailAddress {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-email>")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailAddressError;

impl Display for EmailAddressError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid-email-setting")
    }
}

impl Error for EmailAddressError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderReadiness {
    Disabled,
    Ready,
    NeedsSecret,
    NeedsPublicSetting,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OpenAlexSettings {
    pub enabled: bool,
    pub email: Option<EmailAddress>,
    pub api_key_ref: Option<SecretRef>,
}

impl OpenAlexSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.api_key_ref.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsSecret
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticScholarSettings {
    pub enabled: bool,
    pub api_key_ref: Option<SecretRef>,
}

impl SemanticScholarSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.api_key_ref.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsSecret
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CrossrefSettings {
    pub enabled: bool,
    pub email: Option<EmailAddress>,
}

impl CrossrefSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.email.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsPublicSetting
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PubmedSettings {
    pub enabled: bool,
    pub api_key_ref: Option<SecretRef>,
}

impl PubmedSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.api_key_ref.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsSecret
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArxivSettings {
    pub enabled: bool,
}

impl ArxivSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if self.enabled {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::Disabled
        }
    }
}

impl Default for ArxivSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderSettings {
    pub openalex: OpenAlexSettings,
    pub semantic_scholar: SemanticScholarSettings,
    pub crossref: CrossrefSettings,
    pub pubmed: PubmedSettings,
    pub arxiv: ArxivSettings,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSettings {
    pub default_profile: ProfileId,
    pub providers: ProviderSettings,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            default_profile: ProfileId::MarketplaceLite,
            providers: ProviderSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedGlobalSettings {
    pub revision: u64,
    pub settings: GlobalSettings,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GlobalSettingsDocumentV1 {
    document_kind: String,
    schema_version: u64,
    revision: u64,
    default_profile: ProfileId,
    providers: ProviderDocumentV1,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProviderDocumentV1 {
    openalex: OpenAlexDocumentV1,
    semantic_scholar: SemanticScholarDocumentV1,
    crossref: CrossrefDocumentV1,
    pubmed: PubmedDocumentV1,
    arxiv: ArxivDocumentV1,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OpenAlexDocumentV1 {
    enabled: bool,
    email: Option<String>,
    api_key_ref: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SemanticScholarDocumentV1 {
    enabled: bool,
    api_key_ref: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CrossrefDocumentV1 {
    enabled: bool,
    email: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PubmedDocumentV1 {
    enabled: bool,
    api_key_ref: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArxivDocumentV1 {
    enabled: bool,
}

#[cfg(any(unix, test))]
pub(crate) fn encode_global_settings(
    settings: &GlobalSettings,
    revision: u64,
) -> Result<Vec<u8>, ConfigError> {
    validate_revision(revision)?;
    let document = GlobalSettingsDocumentV1::from_settings(settings, revision);
    let mut bytes =
        serde_json::to_vec_pretty(&document).map_err(|_| ConfigError::InvalidDocument)?;
    bytes.push(b'\n');
    let loaded = decode_global_settings(&bytes)?;
    if loaded.revision != revision || loaded.settings != *settings {
        return Err(ConfigError::InvalidDocument);
    }
    Ok(bytes)
}

pub(crate) fn decode_global_settings(bytes: &[u8]) -> Result<LoadedGlobalSettings, ConfigError> {
    if bytes.len() > MAX_GLOBAL_SETTINGS_BYTES {
        return Err(ConfigError::DocumentTooLarge);
    }
    let value = parse_unique_json(bytes)?;
    validate_envelope(&value)?;
    validate_exact_shape(&value)?;
    let document: GlobalSettingsDocumentV1 =
        serde_json::from_value(value).map_err(|_| ConfigError::InvalidDocument)?;
    validate_revision(document.revision)?;
    let revision = document.revision;
    let settings = document.into_settings()?;
    Ok(LoadedGlobalSettings { revision, settings })
}

fn validate_revision(revision: u64) -> Result<(), ConfigError> {
    if revision == 0 {
        Err(ConfigError::InvalidDocument)
    } else if revision > MAX_GLOBAL_SETTINGS_REVISION {
        Err(ConfigError::RevisionExhausted)
    } else {
        Ok(())
    }
}

impl GlobalSettingsDocumentV1 {
    #[cfg(any(unix, test))]
    fn from_settings(settings: &GlobalSettings, revision: u64) -> Self {
        Self {
            document_kind: GLOBAL_SETTINGS_DOCUMENT_KIND.to_owned(),
            schema_version: GLOBAL_SETTINGS_SCHEMA_VERSION,
            revision,
            default_profile: settings.default_profile,
            providers: ProviderDocumentV1 {
                openalex: OpenAlexDocumentV1 {
                    enabled: settings.providers.openalex.enabled,
                    email: settings
                        .providers
                        .openalex
                        .email
                        .as_ref()
                        .map(|email| email.as_str().to_owned()),
                    api_key_ref: settings
                        .providers
                        .openalex
                        .api_key_ref
                        .as_ref()
                        .map(|reference| reference.raw().to_owned()),
                },
                semantic_scholar: SemanticScholarDocumentV1 {
                    enabled: settings.providers.semantic_scholar.enabled,
                    api_key_ref: settings
                        .providers
                        .semantic_scholar
                        .api_key_ref
                        .as_ref()
                        .map(|reference| reference.raw().to_owned()),
                },
                crossref: CrossrefDocumentV1 {
                    enabled: settings.providers.crossref.enabled,
                    email: settings
                        .providers
                        .crossref
                        .email
                        .as_ref()
                        .map(|email| email.as_str().to_owned()),
                },
                pubmed: PubmedDocumentV1 {
                    enabled: settings.providers.pubmed.enabled,
                    api_key_ref: settings
                        .providers
                        .pubmed
                        .api_key_ref
                        .as_ref()
                        .map(|reference| reference.raw().to_owned()),
                },
                arxiv: ArxivDocumentV1 {
                    enabled: settings.providers.arxiv.enabled,
                },
            },
        }
    }

    fn into_settings(self) -> Result<GlobalSettings, ConfigError> {
        if self.document_kind != GLOBAL_SETTINGS_DOCUMENT_KIND
            || self.schema_version != GLOBAL_SETTINGS_SCHEMA_VERSION
        {
            return Err(ConfigError::InvalidDocument);
        }
        Ok(GlobalSettings {
            default_profile: self.default_profile,
            providers: ProviderSettings {
                openalex: OpenAlexSettings {
                    enabled: self.providers.openalex.enabled,
                    email: parse_email(self.providers.openalex.email)?,
                    api_key_ref: parse_secret_ref(self.providers.openalex.api_key_ref)?,
                },
                semantic_scholar: SemanticScholarSettings {
                    enabled: self.providers.semantic_scholar.enabled,
                    api_key_ref: parse_secret_ref(self.providers.semantic_scholar.api_key_ref)?,
                },
                crossref: CrossrefSettings {
                    enabled: self.providers.crossref.enabled,
                    email: parse_email(self.providers.crossref.email)?,
                },
                pubmed: PubmedSettings {
                    enabled: self.providers.pubmed.enabled,
                    api_key_ref: parse_secret_ref(self.providers.pubmed.api_key_ref)?,
                },
                arxiv: ArxivSettings {
                    enabled: self.providers.arxiv.enabled,
                },
            },
        })
    }
}

fn parse_email(value: Option<String>) -> Result<Option<EmailAddress>, ConfigError> {
    value
        .map(|email| EmailAddress::parse(&email).map_err(|_| ConfigError::InvalidDocument))
        .transpose()
}

fn parse_secret_ref(value: Option<String>) -> Result<Option<SecretRef>, ConfigError> {
    value
        .map(|reference| SecretRef::parse(&reference).map_err(|_| ConfigError::InvalidDocument))
        .transpose()
}

fn validate_envelope(value: &Value) -> Result<(), ConfigError> {
    let object = value.as_object().ok_or(ConfigError::InvalidDocument)?;
    if object.get("document_kind").and_then(Value::as_str) != Some(GLOBAL_SETTINGS_DOCUMENT_KIND) {
        return Err(ConfigError::InvalidDocumentKind);
    }
    match object.get("schema_version").and_then(Value::as_u64) {
        Some(GLOBAL_SETTINGS_SCHEMA_VERSION) => Ok(()),
        observed => Err(ConfigError::UnsupportedSchema { observed }),
    }
}

fn validate_exact_shape(value: &Value) -> Result<(), ConfigError> {
    let root = exact_object(
        value,
        &[
            "document_kind",
            "schema_version",
            "revision",
            "default_profile",
            "providers",
        ],
    )?;
    let providers = exact_object(
        root.get("providers").ok_or(ConfigError::InvalidDocument)?,
        &[
            "openalex",
            "semantic_scholar",
            "crossref",
            "pubmed",
            "arxiv",
        ],
    )?;
    exact_object(
        providers
            .get("openalex")
            .ok_or(ConfigError::InvalidDocument)?,
        &["enabled", "email", "api_key_ref"],
    )?;
    exact_object(
        providers
            .get("semantic_scholar")
            .ok_or(ConfigError::InvalidDocument)?,
        &["enabled", "api_key_ref"],
    )?;
    exact_object(
        providers
            .get("crossref")
            .ok_or(ConfigError::InvalidDocument)?,
        &["enabled", "email"],
    )?;
    exact_object(
        providers
            .get("pubmed")
            .ok_or(ConfigError::InvalidDocument)?,
        &["enabled", "api_key_ref"],
    )?;
    exact_object(
        providers.get("arxiv").ok_or(ConfigError::InvalidDocument)?,
        &["enabled"],
    )?;
    Ok(())
}

fn exact_object<'a>(
    value: &'a Value,
    expected: &[&str],
) -> Result<&'a Map<String, Value>, ConfigError> {
    let object = value.as_object().ok_or(ConfigError::InvalidDocument)?;
    if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
        return Err(ConfigError::InvalidDocument);
    }
    Ok(object)
}

fn parse_unique_json(bytes: &[u8]) -> Result<Value, ConfigError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let UniqueValue(value) =
        UniqueValue::deserialize(&mut deserializer).map_err(|_| ConfigError::InvalidDocument)?;
    deserializer
        .end()
        .map_err(|_| ConfigError::InvalidDocument)?;
    Ok(value)
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(UniqueValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut entries: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(A::Error::custom("duplicate JSON object key"));
            }
            let UniqueValue(value) = entries.next_value()?;
            object.insert(key, value);
        }
        Ok(UniqueValue(Value::Object(object)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConfigError;

    const SECRET_REF: &str = "qsr1_0123456789abcdef0123456789abcdef";

    #[test]
    fn document_round_trip_has_the_exact_v1_envelope() {
        let bytes = encode_global_settings(&GlobalSettings::default(), 1).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.ends_with('\n'));
        assert!(text.contains("\"document_kind\": \"qiongli-global-settings\""));
        assert!(text.contains("\"schema_version\": 1"));
        assert!(text.contains("\"revision\": 1"));
        let loaded = decode_global_settings(&bytes).unwrap();
        assert_eq!(loaded.revision, 1);
        assert_eq!(loaded.settings, GlobalSettings::default());
    }

    #[test]
    fn private_settings_round_trip_only_through_the_private_adapter() {
        let mut settings = GlobalSettings {
            default_profile: ProfileId::Full,
            ..GlobalSettings::default()
        };
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.email =
            Some(EmailAddress::parse("researcher@example.org").unwrap());
        settings.providers.openalex.api_key_ref = Some(SecretRef::parse(SECRET_REF).unwrap());
        let bytes = encode_global_settings(&settings, 7).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("researcher@example.org"));
        assert!(text.contains(SECRET_REF));
        assert!(!text.contains("raw-credential-canary"));
        let loaded = decode_global_settings(&bytes).unwrap();
        assert_eq!(loaded.revision, 7);
        assert_eq!(loaded.settings, settings);
    }

    #[test]
    fn duplicate_keys_are_rejected_at_every_depth() {
        let root_duplicate = br#"{"document_kind":"qiongli-global-settings","document_kind":"qiongli-global-settings","schema_version":1,"revision":1}"#;
        assert!(matches!(
            decode_global_settings(root_duplicate),
            Err(ConfigError::InvalidDocument)
        ));
        let nested_duplicate = valid_document().replace(
            "\"openalex\": {\n      \"enabled\": false,",
            "\"openalex\": {\n      \"enabled\": false,\n      \"enabled\": false,",
        );
        assert!(matches!(
            decode_global_settings(nested_duplicate.as_bytes()),
            Err(ConfigError::InvalidDocument)
        ));
    }

    #[test]
    fn wrong_kind_future_schema_and_unknown_fields_have_closed_semantics() {
        let wrong_kind =
            valid_document().replace("qiongli-global-settings", "qiongli-project-settings");
        assert!(matches!(
            decode_global_settings(wrong_kind.as_bytes()),
            Err(ConfigError::InvalidDocumentKind)
        ));

        let future = valid_document().replace("\"schema_version\": 1", "\"schema_version\": 2");
        assert!(matches!(
            decode_global_settings(future.as_bytes()),
            Err(ConfigError::UnsupportedSchema { observed: Some(2) })
        ));

        let unknown =
            valid_document().replace("\"revision\": 1,", "\"revision\": 1,\n  \"unknown\": true,");
        assert!(matches!(
            decode_global_settings(unknown.as_bytes()),
            Err(ConfigError::InvalidDocument)
        ));
    }

    #[test]
    fn document_size_revision_and_private_values_are_bounded() {
        assert!(matches!(
            decode_global_settings(&vec![b' '; MAX_GLOBAL_SETTINGS_BYTES + 1]),
            Err(ConfigError::DocumentTooLarge)
        ));
        assert!(matches!(
            encode_global_settings(&GlobalSettings::default(), 0),
            Err(ConfigError::InvalidDocument)
        ));
        assert!(matches!(
            encode_global_settings(&GlobalSettings::default(), MAX_GLOBAL_SETTINGS_REVISION + 1),
            Err(ConfigError::RevisionExhausted)
        ));

        let invalid_ref =
            valid_document().replace("\"api_key_ref\": null", "\"api_key_ref\": \"raw-key\"");
        assert!(matches!(
            decode_global_settings(invalid_ref.as_bytes()),
            Err(ConfigError::InvalidDocument)
        ));
        let invalid_email =
            valid_document().replace("\"email\": null", "\"email\": \"bad\\nmail\"");
        assert!(matches!(
            decode_global_settings(invalid_email.as_bytes()),
            Err(ConfigError::InvalidDocument)
        ));
    }

    fn valid_document() -> String {
        String::from_utf8(encode_global_settings(&GlobalSettings::default(), 1).unwrap()).unwrap()
    }
}
