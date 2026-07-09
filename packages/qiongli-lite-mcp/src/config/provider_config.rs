use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const PROVIDERS: [&str; 5] = [
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
];

const OPENALEX_API_KEY_ALIASES: &[&str] = &[
    "QIONGLI_OPENALEX_API_KEY",
    "OPENALEX_API_KEY",
    "QIONGLI_MCPB_OPENALEX_API_KEY",
];
const OPENALEX_EMAIL_ALIASES: &[&str] = &[
    "QIONGLI_OPENALEX_EMAIL",
    "OPENALEX_EMAIL",
    "QIONGLI_MCPB_OPENALEX_EMAIL",
];
const SEMANTIC_SCHOLAR_API_KEY_ALIASES: &[&str] = &[
    "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
    "SEMANTIC_SCHOLAR_API_KEY",
    "S2_API_KEY",
    "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
];
const CROSSREF_EMAIL_ALIASES: &[&str] = &[
    "QIONGLI_CROSSREF_EMAIL",
    "CROSSREF_EMAIL",
    "QIONGLI_MCPB_CROSSREF_EMAIL",
];
const PUBMED_API_KEY_ALIASES: &[&str] = &[
    "QIONGLI_NCBI_API_KEY",
    "NCBI_API_KEY",
    "PUBMED_API_KEY",
    "QIONGLI_MCPB_PUBMED_API_KEY",
];

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unsupported provider field: {0}.{1}")]
    UnsupportedField(String, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFieldRole {
    ActivationRequired,
    Optional,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderFieldSpec {
    pub provider: &'static str,
    pub field: &'static str,
    pub role: ProviderFieldRole,
    pub env_aliases: &'static [&'static str],
}

const PROVIDER_FIELD_SPECS: &[ProviderFieldSpec] = &[
    ProviderFieldSpec {
        provider: "openalex",
        field: "api_key",
        role: ProviderFieldRole::ActivationRequired,
        env_aliases: OPENALEX_API_KEY_ALIASES,
    },
    ProviderFieldSpec {
        provider: "openalex",
        field: "email",
        role: ProviderFieldRole::Optional,
        env_aliases: OPENALEX_EMAIL_ALIASES,
    },
    ProviderFieldSpec {
        provider: "semantic_scholar",
        field: "api_key",
        role: ProviderFieldRole::ActivationRequired,
        env_aliases: SEMANTIC_SCHOLAR_API_KEY_ALIASES,
    },
    ProviderFieldSpec {
        provider: "crossref",
        field: "email",
        role: ProviderFieldRole::ActivationRequired,
        env_aliases: CROSSREF_EMAIL_ALIASES,
    },
    ProviderFieldSpec {
        provider: "pubmed",
        field: "api_key",
        role: ProviderFieldRole::ActivationRequired,
        env_aliases: PUBMED_API_KEY_ALIASES,
    },
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProviderFile {
    #[serde(default = "default_version")]
    version: u64,
    #[serde(default)]
    providers: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    #[serde(default, flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Default)]
struct ResolvedProvider {
    enabled: bool,
    configured: bool,
    values: BTreeMap<String, String>,
}

/// Resolved provider state. This type deliberately does not implement `Debug`
/// or `Serialize` because it contains credential values.
#[derive(Clone, Default)]
pub struct ResolvedProviderConfig {
    providers: BTreeMap<String, ResolvedProvider>,
}

impl ResolvedProviderConfig {
    pub fn is_enabled(&self, provider: &str) -> bool {
        self.provider(provider)
            .map(|entry| entry.enabled)
            .unwrap_or(false)
    }

    pub fn is_configured(&self, provider: &str) -> bool {
        self.provider(provider)
            .map(|entry| entry.configured)
            .unwrap_or(false)
    }

    pub fn is_active(&self, provider: &str) -> bool {
        self.provider(provider)
            .map(|entry| entry.enabled && entry.configured)
            .unwrap_or(false)
    }

    pub fn value(&self, provider: &str, field: &str) -> Option<&str> {
        let provider = normalize_provider(provider);
        let field = normalize_key(field);
        self.providers
            .get(&provider)?
            .values
            .get(&field)
            .map(String::as_str)
    }

    /// Construct resolved provider state without reading process environment.
    /// This exists for deterministic integration tests and internal adapters;
    /// the production MCP runtime uses [`resolve_provider_config`].
    #[doc(hidden)]
    pub fn from_values(
        values: &[(&str, &str, &str)],
    ) -> Result<ResolvedProviderConfig, ConfigError> {
        let mut file = ProviderFile {
            version: 1,
            providers: BTreeMap::new(),
            extra: BTreeMap::new(),
        };
        for (provider, field, value) in values {
            let provider = normalize_provider(provider);
            let field = normalize_key(field);
            if !is_supported_field(&provider, &field) {
                return Err(ConfigError::UnsupportedField(provider, field));
            }
            let entry = file.providers.entry(provider).or_default();
            entry.insert("enabled".to_string(), serde_json::Value::Bool(true));
            entry.insert(field, serde_json::Value::String((*value).to_string()));
        }
        Ok(resolve_provider_file(&file, |_| None))
    }

    fn provider(&self, provider: &str) -> Option<&ResolvedProvider> {
        self.providers.get(&normalize_provider(provider))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderSummary {
    pub status: String,
    pub config_path: PathBuf,
    pub capability_mode: String,
    pub providers: BTreeMap<String, String>,
    pub missing: Vec<String>,
    pub redacted_config: RedactedProviderConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<ProviderNextAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactedProviderConfig {
    pub providers: BTreeMap<String, RedactedProvider>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactedProvider {
    pub enabled: bool,
    pub configured: bool,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderNextAction {
    pub tool: String,
    pub args: BTreeMap<String, String>,
    pub message: String,
}

fn default_version() -> u64 {
    1
}

pub fn provider_config_path() -> PathBuf {
    if let Ok(root) = std::env::var("QIONGLI_CONFIG_HOME") {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("providers.json");
        }
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("qiongli")
        .join("providers.json")
}

pub fn provider_field_specs() -> &'static [ProviderFieldSpec] {
    PROVIDER_FIELD_SPECS
}

pub fn resolve_provider_config() -> Result<ResolvedProviderConfig, ConfigError> {
    let file = read_provider_file()?;
    Ok(resolve_provider_file(&file, |name| {
        std::env::var(name).ok()
    }))
}

pub fn save_provider_value(
    provider: &str,
    field: &str,
    value: &str,
) -> Result<PathBuf, ConfigError> {
    let provider = normalize_provider(provider);
    let field = normalize_key(field);
    if !is_supported_field(&provider, &field) {
        return Err(ConfigError::UnsupportedField(provider, field));
    }

    let path = provider_config_path();
    let mut file = read_provider_file()?;
    file.version = 1;
    let entry = file.providers.entry(provider).or_default();
    entry.insert("enabled".to_string(), serde_json::Value::Bool(true));
    entry.insert(field, serde_json::Value::String(value.to_string()));
    write_provider_file(&path, &file)?;
    Ok(path)
}

pub fn summary() -> Result<ProviderSummary, ConfigError> {
    let resolved = resolve_provider_config()?;
    let providers = PROVIDERS
        .into_iter()
        .map(|provider| {
            let status = if resolved.is_configured(provider) {
                "configured"
            } else {
                "missing"
            };
            (provider.to_string(), status.to_string())
        })
        .collect();
    let missing: Vec<String> = PROVIDER_FIELD_SPECS
        .iter()
        .filter(|spec| spec.role == ProviderFieldRole::ActivationRequired)
        .filter(|spec| resolved.value(spec.provider, spec.field).is_none())
        .map(|spec| format!("{}.{}", spec.provider, spec.field))
        .collect();
    let capability_mode = if PROVIDERS
        .into_iter()
        .any(|provider| resolved.is_active(provider))
    {
        "provider_connected"
    } else {
        "strategy_only"
    };
    let redacted_providers = PROVIDERS
        .into_iter()
        .map(|provider| {
            let fields = PROVIDER_FIELD_SPECS
                .iter()
                .filter(|spec| spec.provider == provider)
                .map(|spec| {
                    let status = if resolved.value(provider, spec.field).is_some() {
                        "configured"
                    } else {
                        "missing"
                    };
                    (spec.field.to_string(), status.to_string())
                })
                .collect();
            (
                provider.to_string(),
                RedactedProvider {
                    enabled: resolved.is_enabled(provider),
                    configured: resolved.is_configured(provider),
                    fields,
                },
            )
        })
        .collect();
    let next_provider = if missing.iter().any(|field| field == "openalex.api_key") {
        Some("openalex")
    } else if missing
        .iter()
        .any(|field| field == "semantic_scholar.api_key")
    {
        Some("semantic_scholar")
    } else {
        None
    };
    let next_action = next_provider.map(|provider| ProviderNextAction {
        tool: "qiongli_configure_provider".to_string(),
        args: BTreeMap::from([("provider".to_string(), provider.to_string())]),
        message: ("Run qiongli_configure_provider to start a local setup page and return its URL. "
            .to_string() + "Do not paste API keys in chat."),
    });

    Ok(ProviderSummary {
        status: "ok".to_string(),
        config_path: provider_config_path(),
        capability_mode: capability_mode.to_string(),
        providers,
        missing,
        redacted_config: RedactedProviderConfig {
            providers: redacted_providers,
        },
        next_action,
    })
}

pub fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

pub fn normalize_provider(value: &str) -> String {
    let normalized = normalize_key(value);
    match normalized.as_str() {
        "s2" | "semanticscholar" => "semantic_scholar".to_string(),
        "ncbi" => "pubmed".to_string(),
        _ => normalized,
    }
}

fn read_provider_file() -> Result<ProviderFile, ConfigError> {
    let path = provider_config_path();
    if !path.is_file() {
        return Ok(ProviderFile::default());
    }
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn write_provider_file(path: &PathBuf, file: &ProviderFile) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = format!("{}\n", serde_json::to_string_pretty(file)?);
    write_owner_only(path, text.as_bytes()).map_err(ConfigError::Io)
}

#[cfg(unix)]
fn write_owner_only(path: &PathBuf, contents: &[u8]) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("providers.json");
    let temporary = path.with_file_name(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&temporary)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    let result = (|| {
        file.write_all(contents)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(unix))]
fn write_owner_only(path: &PathBuf, contents: &[u8]) -> Result<(), std::io::Error> {
    fs::write(path, contents)
}

fn resolve_provider_file<F>(file: &ProviderFile, env_value: F) -> ResolvedProviderConfig
where
    F: Fn(&str) -> Option<String>,
{
    let mut resolved = ResolvedProviderConfig::default();
    for provider in PROVIDERS {
        let persisted = file.providers.get(provider);
        let mut values = BTreeMap::new();
        for spec in PROVIDER_FIELD_SPECS
            .iter()
            .filter(|spec| spec.provider == provider)
        {
            if let Some(value) = resolve_field_value(persisted, spec, &env_value) {
                values.insert(spec.field.to_string(), value);
            }
        }
        let activation_fields: Vec<&ProviderFieldSpec> = PROVIDER_FIELD_SPECS
            .iter()
            .filter(|spec| {
                spec.provider == provider && spec.role == ProviderFieldRole::ActivationRequired
            })
            .collect();
        let configured = activation_fields.is_empty()
            || activation_fields
                .iter()
                .all(|spec| values.contains_key(spec.field));
        let explicitly_enabled = persisted
            .and_then(|entry| entry.get("enabled"))
            .and_then(serde_json::Value::as_bool);
        let enabled = explicitly_enabled.unwrap_or(configured);
        resolved.providers.insert(
            provider.to_string(),
            ResolvedProvider {
                enabled,
                configured,
                values,
            },
        );
    }
    resolved
}

fn resolve_field_value<F>(
    persisted: Option<&BTreeMap<String, serde_json::Value>>,
    spec: &ProviderFieldSpec,
    env_value: &F,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    spec.env_aliases
        .iter()
        .filter_map(|alias| env_value(alias))
        .find_map(non_empty)
        .or_else(|| {
            persisted
                .and_then(|entry| entry.get(spec.field))
                .and_then(serde_json::Value::as_str)
                .and_then(|value| non_empty(value.to_string()))
        })
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn is_supported_field(provider: &str, field: &str) -> bool {
    PROVIDER_FIELD_SPECS
        .iter()
        .any(|spec| spec.provider == provider && spec.field == field)
}
