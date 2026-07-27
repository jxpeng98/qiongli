use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};

use crate::{
    ConfigRoot, EmailAddress, GlobalSettings, LoadedGlobalSettings, MAX_GLOBAL_SETTINGS_BYTES,
    MAX_SECRET_VALUE_BYTES, ProviderSettings, SecretRef, SecretValue, document::parse_unique_json,
};

pub const LEGACY_PROVIDER_CONFIG_FILE: &str = "providers.json";

const PROVIDERS: [&str; 5] = [
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyProviderSecret {
    OpenAlex,
    SemanticScholar,
    Pubmed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyProviderId {
    OpenAlex,
    SemanticScholar,
    Crossref,
    Pubmed,
    Arxiv,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyProviderResolutionStrategy {
    KeepV2,
    UseLegacy,
    MergeCompatible,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProviderResolution {
    pub provider: LegacyProviderId,
    pub strategy: LegacyProviderResolutionStrategy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProviderConflict {
    pub provider: LegacyProviderId,
    pub differing_fields: Vec<String>,
    pub legacy_secret_present: bool,
    pub current_secret_reference_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProviderConfigSummary {
    pub content_sha256: String,
    pub provider_count: usize,
    pub secret_count: usize,
    pub public_setting_count: usize,
}

#[derive(Clone, Default, Eq, PartialEq)]
struct LegacyProviderEntry {
    present: bool,
    enabled: bool,
    email: Option<EmailAddress>,
    api_key: Option<String>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct LegacyProviderConfig {
    summary: LegacyProviderConfigSummary,
    openalex: LegacyProviderEntry,
    semantic_scholar: LegacyProviderEntry,
    crossref: LegacyProviderEntry,
    pubmed: LegacyProviderEntry,
    arxiv: LegacyProviderEntry,
}

impl LegacyProviderConfig {
    #[must_use]
    pub const fn summary(&self) -> &LegacyProviderConfigSummary {
        &self.summary
    }

    pub fn secret_values(
        &self,
    ) -> Result<Vec<(LegacyProviderSecret, SecretValue)>, LegacyProviderConfigError> {
        [
            (
                LegacyProviderSecret::OpenAlex,
                self.openalex.api_key.as_ref(),
            ),
            (
                LegacyProviderSecret::SemanticScholar,
                self.semantic_scholar.api_key.as_ref(),
            ),
            (LegacyProviderSecret::Pubmed, self.pubmed.api_key.as_ref()),
        ]
        .into_iter()
        .filter_map(|(provider, value)| value.map(|value| (provider, value)))
        .map(|(provider, value)| {
            SecretValue::new(value.as_bytes().to_vec())
                .map(|value| (provider, value))
                .map_err(|_| LegacyProviderConfigError::InvalidValue)
        })
        .collect()
    }

    pub fn project_settings(
        &self,
        loaded: &LoadedGlobalSettings,
        secret_refs: &[(LegacyProviderSecret, SecretRef)],
    ) -> Result<GlobalSettings, LegacyProviderConfigError> {
        self.project_settings_with_resolutions(loaded, secret_refs, &[])
    }

    pub fn provider_conflicts(
        &self,
        loaded: &LoadedGlobalSettings,
        secret_refs: &[(LegacyProviderSecret, SecretRef)],
    ) -> Result<Vec<LegacyProviderConflict>, LegacyProviderConfigError> {
        let legacy = self.project_providers(secret_refs)?;
        let current = &loaded.settings.providers;
        let mut conflicts = Vec::new();
        if self.openalex.present
            && current.openalex != Default::default()
            && current.openalex != legacy.openalex
        {
            conflicts.push(LegacyProviderConflict {
                provider: LegacyProviderId::OpenAlex,
                differing_fields: differing_fields([
                    (
                        "enabled",
                        current.openalex.enabled != legacy.openalex.enabled,
                    ),
                    ("email", current.openalex.email != legacy.openalex.email),
                    (
                        "api-key-reference",
                        current.openalex.api_key_ref != legacy.openalex.api_key_ref,
                    ),
                ]),
                legacy_secret_present: self.openalex.api_key.is_some(),
                current_secret_reference_present: current.openalex.api_key_ref.is_some(),
            });
        }
        if self.semantic_scholar.present
            && current.semantic_scholar != Default::default()
            && current.semantic_scholar != legacy.semantic_scholar
        {
            conflicts.push(LegacyProviderConflict {
                provider: LegacyProviderId::SemanticScholar,
                differing_fields: differing_fields([
                    (
                        "enabled",
                        current.semantic_scholar.enabled != legacy.semantic_scholar.enabled,
                    ),
                    (
                        "api-key-reference",
                        current.semantic_scholar.api_key_ref != legacy.semantic_scholar.api_key_ref,
                    ),
                ]),
                legacy_secret_present: self.semantic_scholar.api_key.is_some(),
                current_secret_reference_present: current.semantic_scholar.api_key_ref.is_some(),
            });
        }
        if self.crossref.present
            && current.crossref != Default::default()
            && current.crossref != legacy.crossref
        {
            conflicts.push(LegacyProviderConflict {
                provider: LegacyProviderId::Crossref,
                differing_fields: differing_fields([
                    (
                        "enabled",
                        current.crossref.enabled != legacy.crossref.enabled,
                    ),
                    ("email", current.crossref.email != legacy.crossref.email),
                ]),
                legacy_secret_present: false,
                current_secret_reference_present: false,
            });
        }
        if self.pubmed.present
            && current.pubmed != Default::default()
            && current.pubmed != legacy.pubmed
        {
            conflicts.push(LegacyProviderConflict {
                provider: LegacyProviderId::Pubmed,
                differing_fields: differing_fields([
                    ("enabled", current.pubmed.enabled != legacy.pubmed.enabled),
                    (
                        "api-key-reference",
                        current.pubmed.api_key_ref != legacy.pubmed.api_key_ref,
                    ),
                ]),
                legacy_secret_present: self.pubmed.api_key.is_some(),
                current_secret_reference_present: current.pubmed.api_key_ref.is_some(),
            });
        }
        if self.arxiv.present
            && current.arxiv != Default::default()
            && current.arxiv != legacy.arxiv
        {
            conflicts.push(LegacyProviderConflict {
                provider: LegacyProviderId::Arxiv,
                differing_fields: differing_fields([(
                    "enabled",
                    current.arxiv.enabled != legacy.arxiv.enabled,
                )]),
                legacy_secret_present: false,
                current_secret_reference_present: false,
            });
        }
        Ok(conflicts)
    }

    pub fn project_settings_with_resolutions(
        &self,
        loaded: &LoadedGlobalSettings,
        secret_refs: &[(LegacyProviderSecret, SecretRef)],
        resolutions: &[LegacyProviderResolution],
    ) -> Result<GlobalSettings, LegacyProviderConfigError> {
        let legacy = self.project_providers(secret_refs)?;
        let conflicts = self.provider_conflicts(loaded, secret_refs)?;
        validate_resolutions(&conflicts, resolutions)?;
        let mut projected = loaded.settings.clone();
        if self.openalex.present {
            projected.providers.openalex = resolve_provider(
                LegacyProviderId::OpenAlex,
                &loaded.settings.providers.openalex,
                &legacy.openalex,
                &conflicts,
                resolutions,
                |current, legacy| crate::OpenAlexSettings {
                    enabled: current.enabled,
                    email: current.email.clone().or_else(|| legacy.email.clone()),
                    api_key_ref: current
                        .api_key_ref
                        .clone()
                        .or_else(|| legacy.api_key_ref.clone()),
                },
            )?;
        }
        if self.semantic_scholar.present {
            projected.providers.semantic_scholar = resolve_provider(
                LegacyProviderId::SemanticScholar,
                &loaded.settings.providers.semantic_scholar,
                &legacy.semantic_scholar,
                &conflicts,
                resolutions,
                |current, legacy| crate::SemanticScholarSettings {
                    enabled: current.enabled,
                    api_key_ref: current
                        .api_key_ref
                        .clone()
                        .or_else(|| legacy.api_key_ref.clone()),
                },
            )?;
        }
        if self.crossref.present {
            projected.providers.crossref = resolve_provider(
                LegacyProviderId::Crossref,
                &loaded.settings.providers.crossref,
                &legacy.crossref,
                &conflicts,
                resolutions,
                |current, legacy| crate::CrossrefSettings {
                    enabled: current.enabled,
                    email: current.email.clone().or_else(|| legacy.email.clone()),
                },
            )?;
        }
        if self.pubmed.present {
            projected.providers.pubmed = resolve_provider(
                LegacyProviderId::Pubmed,
                &loaded.settings.providers.pubmed,
                &legacy.pubmed,
                &conflicts,
                resolutions,
                |current, legacy| crate::PubmedSettings {
                    enabled: current.enabled,
                    api_key_ref: current
                        .api_key_ref
                        .clone()
                        .or_else(|| legacy.api_key_ref.clone()),
                },
            )?;
        }
        if self.arxiv.present {
            projected.providers.arxiv = resolve_provider(
                LegacyProviderId::Arxiv,
                &loaded.settings.providers.arxiv,
                &legacy.arxiv,
                &conflicts,
                resolutions,
                |current, _legacy| current.clone(),
            )?;
        }
        Ok(projected)
    }

    fn project_providers(
        &self,
        secret_refs: &[(LegacyProviderSecret, SecretRef)],
    ) -> Result<ProviderSettings, LegacyProviderConfigError> {
        let reference = |provider| {
            secret_refs.iter().find_map(|(candidate, reference)| {
                (*candidate == provider).then(|| reference.clone())
            })
        };
        if (self.openalex.api_key.is_some() && reference(LegacyProviderSecret::OpenAlex).is_none())
            || (self.semantic_scholar.api_key.is_some()
                && reference(LegacyProviderSecret::SemanticScholar).is_none())
            || (self.pubmed.api_key.is_some() && reference(LegacyProviderSecret::Pubmed).is_none())
            || secret_refs.len() != self.summary.secret_count
        {
            return Err(LegacyProviderConfigError::SecretReferenceMismatch);
        }
        let mut providers = ProviderSettings::default();
        if self.openalex.present {
            providers.openalex.enabled = self.openalex.enabled;
            providers.openalex.email = self.openalex.email.clone();
            providers.openalex.api_key_ref = reference(LegacyProviderSecret::OpenAlex);
        }
        if self.semantic_scholar.present {
            providers.semantic_scholar.enabled = self.semantic_scholar.enabled;
            providers.semantic_scholar.api_key_ref =
                reference(LegacyProviderSecret::SemanticScholar);
        }
        if self.crossref.present {
            providers.crossref.enabled = self.crossref.enabled;
            providers.crossref.email = self.crossref.email.clone();
        }
        if self.pubmed.present {
            providers.pubmed.enabled = self.pubmed.enabled;
            providers.pubmed.api_key_ref = reference(LegacyProviderSecret::Pubmed);
        }
        if self.arxiv.present {
            providers.arxiv.enabled = self.arxiv.enabled;
        }
        Ok(providers)
    }
}

impl Debug for LegacyProviderConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyProviderConfig")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyProviderConfigError {
    PathUnavailable,
    UnsafePath,
    DocumentTooLarge,
    InvalidDocument,
    UnsupportedData,
    InvalidValue,
    CurrentConfigConflict,
    ResolutionInvalid,
    SecretReferenceMismatch,
}

impl LegacyProviderConfigError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PathUnavailable => "legacy-provider-config-unavailable",
            Self::UnsafePath => "legacy-provider-config-path-unsafe",
            Self::DocumentTooLarge => "legacy-provider-config-too-large",
            Self::InvalidDocument => "legacy-provider-config-invalid",
            Self::UnsupportedData => "legacy-provider-config-review-required",
            Self::InvalidValue => "legacy-provider-value-invalid",
            Self::CurrentConfigConflict => "legacy-provider-v2-conflict",
            Self::ResolutionInvalid => "legacy-provider-resolution-invalid",
            Self::SecretReferenceMismatch => "legacy-provider-secret-reference-mismatch",
        }
    }
}

fn differing_fields<const N: usize>(fields: [(&str, bool); N]) -> Vec<String> {
    fields
        .into_iter()
        .filter_map(|(field, differs)| differs.then(|| field.to_owned()))
        .collect()
}

fn validate_resolutions(
    conflicts: &[LegacyProviderConflict],
    resolutions: &[LegacyProviderResolution],
) -> Result<(), LegacyProviderConfigError> {
    let conflict_providers = conflicts
        .iter()
        .map(|conflict| conflict.provider)
        .collect::<BTreeSet<_>>();
    let resolution_providers = resolutions
        .iter()
        .map(|resolution| resolution.provider)
        .collect::<BTreeSet<_>>();
    if resolution_providers.len() != resolutions.len() {
        return Err(LegacyProviderConfigError::ResolutionInvalid);
    }
    if !conflict_providers.is_subset(&resolution_providers) {
        return Err(if resolutions.is_empty() {
            LegacyProviderConfigError::CurrentConfigConflict
        } else {
            LegacyProviderConfigError::ResolutionInvalid
        });
    }
    Ok(())
}

fn resolve_provider<T: Clone + Default + PartialEq>(
    provider: LegacyProviderId,
    current: &T,
    legacy: &T,
    conflicts: &[LegacyProviderConflict],
    resolutions: &[LegacyProviderResolution],
    merge: impl FnOnce(&T, &T) -> T,
) -> Result<T, LegacyProviderConfigError> {
    if !conflicts
        .iter()
        .any(|conflict| conflict.provider == provider)
    {
        return Ok(legacy.clone());
    }
    let strategy = resolutions
        .iter()
        .find_map(|resolution| (resolution.provider == provider).then_some(resolution.strategy))
        .ok_or(LegacyProviderConfigError::CurrentConfigConflict)?;
    Ok(match strategy {
        LegacyProviderResolutionStrategy::KeepV2 => current.clone(),
        LegacyProviderResolutionStrategy::UseLegacy => legacy.clone(),
        LegacyProviderResolutionStrategy::MergeCompatible => merge(current, legacy),
    })
}

pub fn inspect_legacy_provider_config(
    root: &ConfigRoot,
) -> Result<Option<LegacyProviderConfig>, LegacyProviderConfigError> {
    let path = root.compatibility_root().join(LEGACY_PROVIDER_CONFIG_FILE);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(LegacyProviderConfigError::PathUnavailable),
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(LegacyProviderConfigError::UnsafePath);
    }
    validate_directory_chain(path.parent().ok_or(LegacyProviderConfigError::UnsafePath)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(LegacyProviderConfigError::UnsafePath);
        }
    }
    if metadata.len() > MAX_GLOBAL_SETTINGS_BYTES as u64 {
        return Err(LegacyProviderConfigError::DocumentTooLarge);
    }
    let bytes = fs::read(&path).map_err(|_| LegacyProviderConfigError::PathUnavailable)?;
    if bytes.len() > MAX_GLOBAL_SETTINGS_BYTES {
        return Err(LegacyProviderConfigError::DocumentTooLarge);
    }
    let value =
        parse_unique_json(&bytes).map_err(|_| LegacyProviderConfigError::InvalidDocument)?;
    parse_legacy_provider_config(&value, &bytes).map(Some)
}

fn validate_directory_chain(path: &Path) -> Result<(), LegacyProviderConfigError> {
    let mut current = std::path::PathBuf::new();
    for component in path.components() {
        current.push(component);
        if matches!(component, Component::RootDir | Component::Prefix(_)) {
            continue;
        }
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| LegacyProviderConfigError::PathUnavailable)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(LegacyProviderConfigError::UnsafePath);
        }
    }
    Ok(())
}

fn parse_legacy_provider_config(
    value: &Value,
    bytes: &[u8],
) -> Result<LegacyProviderConfig, LegacyProviderConfigError> {
    let root = value
        .as_object()
        .ok_or(LegacyProviderConfigError::InvalidDocument)?;
    if root
        .keys()
        .any(|key| !matches!(key.as_str(), "version" | "providers" | "search"))
    {
        return Err(LegacyProviderConfigError::UnsupportedData);
    }
    if root
        .get("version")
        .map(|version| version.as_u64() != Some(1))
        .unwrap_or(false)
    {
        return Err(LegacyProviderConfigError::InvalidDocument);
    }
    if root
        .get("search")
        .is_some_and(|search| !search.as_object().is_some_and(Map::is_empty))
    {
        return Err(LegacyProviderConfigError::UnsupportedData);
    }
    let providers = root
        .get("providers")
        .map(|providers| {
            providers
                .as_object()
                .ok_or(LegacyProviderConfigError::InvalidDocument)
        })
        .transpose()?
        .cloned()
        .unwrap_or_default();
    let providers = normalized_providers(providers)?;

    let openalex = parse_provider(
        providers.get("openalex"),
        &["enabled", "email", "api_key"],
        Some("api_key"),
    )?;
    let semantic_scholar = parse_provider(
        providers.get("semantic_scholar"),
        &["enabled", "api_key"],
        Some("api_key"),
    )?;
    let crossref = parse_provider(
        providers.get("crossref"),
        &["enabled", "email"],
        Some("email"),
    )?;
    let pubmed = parse_provider(
        providers.get("pubmed"),
        &["enabled", "api_key"],
        Some("api_key"),
    )?;
    let arxiv = parse_provider(providers.get("arxiv"), &["enabled"], None)?;
    let entries = [&openalex, &semantic_scholar, &crossref, &pubmed, &arxiv];
    let provider_count = entries.iter().filter(|entry| entry.present).count();
    let secret_count = [&openalex, &semantic_scholar, &pubmed]
        .into_iter()
        .filter(|entry| entry.api_key.is_some())
        .count();
    let public_setting_count = [&openalex, &crossref]
        .into_iter()
        .filter(|entry| entry.email.is_some())
        .count();
    Ok(LegacyProviderConfig {
        summary: LegacyProviderConfigSummary {
            content_sha256: sha256_hex(bytes),
            provider_count,
            secret_count,
            public_setting_count,
        },
        openalex,
        semantic_scholar,
        crossref,
        pubmed,
        arxiv,
    })
}

fn normalized_providers(
    providers: Map<String, Value>,
) -> Result<BTreeMap<String, Map<String, Value>>, LegacyProviderConfigError> {
    let mut normalized = BTreeMap::new();
    for (raw_provider, value) in providers {
        let provider = normalize_provider(&raw_provider);
        if !PROVIDERS.contains(&provider.as_str()) || normalized.contains_key(&provider) {
            return Err(LegacyProviderConfigError::UnsupportedData);
        }
        let raw_fields = value
            .as_object()
            .ok_or(LegacyProviderConfigError::InvalidDocument)?;
        let mut fields = Map::new();
        for (raw_field, value) in raw_fields {
            let field = normalize_key(raw_field);
            if fields.insert(field, value.clone()).is_some() {
                return Err(LegacyProviderConfigError::UnsupportedData);
            }
        }
        normalized.insert(provider, fields);
    }
    Ok(normalized)
}

fn parse_provider(
    fields: Option<&Map<String, Value>>,
    supported_fields: &[&str],
    activation_field: Option<&str>,
) -> Result<LegacyProviderEntry, LegacyProviderConfigError> {
    let Some(fields) = fields else {
        return Ok(LegacyProviderEntry::default());
    };
    let supported = supported_fields.iter().copied().collect::<BTreeSet<_>>();
    if fields
        .keys()
        .any(|field| !supported.contains(field.as_str()))
    {
        return Err(LegacyProviderConfigError::UnsupportedData);
    }
    let api_key = fields
        .get("api_key")
        .map(parse_nonempty_secret)
        .transpose()?
        .flatten();
    let email = fields.get("email").map(parse_email).transpose()?.flatten();
    let configured = match activation_field {
        Some("api_key") => api_key.is_some(),
        Some("email") => email.is_some(),
        Some(_) => return Err(LegacyProviderConfigError::InvalidDocument),
        None => true,
    };
    let enabled = fields
        .get("enabled")
        .map(|enabled| {
            enabled
                .as_bool()
                .ok_or(LegacyProviderConfigError::InvalidDocument)
        })
        .transpose()?
        .unwrap_or(configured);
    Ok(LegacyProviderEntry {
        present: true,
        enabled,
        email,
        api_key,
    })
}

fn parse_nonempty_secret(value: &Value) -> Result<Option<String>, LegacyProviderConfigError> {
    let value = value
        .as_str()
        .ok_or(LegacyProviderConfigError::InvalidDocument)?
        .trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() > MAX_SECRET_VALUE_BYTES {
        return Err(LegacyProviderConfigError::InvalidValue);
    }
    Ok(Some(value.to_owned()))
}

fn parse_email(value: &Value) -> Result<Option<EmailAddress>, LegacyProviderConfigError> {
    let value = value
        .as_str()
        .ok_or(LegacyProviderConfigError::InvalidDocument)?
        .trim();
    if value.is_empty() {
        return Ok(None);
    }
    EmailAddress::parse(value)
        .map(Some)
        .map_err(|_| LegacyProviderConfigError::InvalidValue)
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

fn normalize_provider(value: &str) -> String {
    match normalize_key(value).as_str() {
        "s2" | "semanticscholar" => "semantic_scholar".to_owned(),
        "ncbi" => "pubmed".to_owned(),
        normalized => normalized.to_owned(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::resolve_config_root;

    fn root(label: &str) -> (PathBuf, ConfigRoot) {
        let requested = std::env::temp_dir().join(format!(
            "qiongli-legacy-provider-{label}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&requested).unwrap();
        let canonical = fs::canonicalize(&requested).unwrap();
        let config = resolve_config_root(Some(canonical.as_os_str()), &canonical).unwrap();
        (requested, config)
    }

    fn write_private(path: &Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;

            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }

    #[test]
    fn recognized_provider_values_are_redacted_and_projected() {
        let (directory, root) = root("recognized");
        write_private(
            &root.compatibility_root().join(LEGACY_PROVIDER_CONFIG_FILE),
            br#"{
  "version": 1,
  "providers": {
    "openalex": {"enabled": true, "email": "person@example.org", "api_key": "openalex-secret"},
    "s2": {"api-key": "s2-secret"},
    "crossref": {"email": "crossref@example.org"},
    "ncbi": {"enabled": false, "api_key": "pubmed-secret"},
    "arxiv": {"enabled": false}
  }
}"#,
        );
        let legacy = inspect_legacy_provider_config(&root).unwrap().unwrap();
        assert_eq!(legacy.summary().provider_count, 5);
        assert_eq!(legacy.summary().secret_count, 3);
        assert!(!format!("{legacy:?}").contains("openalex-secret"));
        let refs = [
            (
                LegacyProviderSecret::OpenAlex,
                SecretRef::parse("qsr1_11111111111111111111111111111111").unwrap(),
            ),
            (
                LegacyProviderSecret::SemanticScholar,
                SecretRef::parse("qsr1_22222222222222222222222222222222").unwrap(),
            ),
            (
                LegacyProviderSecret::Pubmed,
                SecretRef::parse("qsr1_33333333333333333333333333333333").unwrap(),
            ),
        ];
        let projected = legacy
            .project_settings(
                &LoadedGlobalSettings {
                    revision: 1,
                    settings: GlobalSettings::default(),
                },
                &refs,
            )
            .unwrap();
        assert!(projected.providers.openalex.enabled);
        assert!(projected.providers.semantic_scholar.enabled);
        assert!(projected.providers.crossref.enabled);
        assert!(!projected.providers.pubmed.enabled);
        assert!(!projected.providers.arxiv.enabled);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unknown_data_and_existing_v2_provider_changes_require_review() {
        let (directory, root) = root("review");
        write_private(
            &root.compatibility_root().join(LEGACY_PROVIDER_CONFIG_FILE),
            br#"{"providers":{"unknown":{"api_key":"secret"}}}"#,
        );
        assert_eq!(
            inspect_legacy_provider_config(&root).unwrap_err(),
            LegacyProviderConfigError::UnsupportedData
        );

        write_private(
            &root.compatibility_root().join(LEGACY_PROVIDER_CONFIG_FILE),
            br#"{"providers":{"crossref":{"enabled":false}}}"#,
        );
        let legacy = inspect_legacy_provider_config(&root).unwrap().unwrap();
        let mut current = GlobalSettings::default();
        current.providers.crossref.enabled = true;
        let loaded = LoadedGlobalSettings {
            revision: 2,
            settings: current.clone(),
        };
        assert_eq!(
            legacy.project_settings(&loaded, &[]).unwrap_err(),
            LegacyProviderConfigError::CurrentConfigConflict
        );
        let conflicts = legacy.provider_conflicts(&loaded, &[]).unwrap();
        assert_eq!(
            conflicts,
            vec![LegacyProviderConflict {
                provider: LegacyProviderId::Crossref,
                differing_fields: vec!["enabled".to_owned()],
                legacy_secret_present: false,
                current_secret_reference_present: false,
            }]
        );
        let keep_v2 = legacy
            .project_settings_with_resolutions(
                &loaded,
                &[],
                &[LegacyProviderResolution {
                    provider: LegacyProviderId::Crossref,
                    strategy: LegacyProviderResolutionStrategy::KeepV2,
                }],
            )
            .unwrap();
        assert!(keep_v2.providers.crossref.enabled);
        let use_legacy = legacy
            .project_settings_with_resolutions(
                &loaded,
                &[],
                &[LegacyProviderResolution {
                    provider: LegacyProviderId::Crossref,
                    strategy: LegacyProviderResolutionStrategy::UseLegacy,
                }],
            )
            .unwrap();
        assert!(!use_legacy.providers.crossref.enabled);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;

            let path = root.compatibility_root().join(LEGACY_PROVIDER_CONFIG_FILE);
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            assert_eq!(
                inspect_legacy_provider_config(&root).unwrap_err(),
                LegacyProviderConfigError::UnsafePath
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
