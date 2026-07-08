use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unsupported provider field: {0}.{1}")]
    UnsupportedField(String, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProviderFile {
    #[serde(default = "default_version")]
    version: u64,
    #[serde(default)]
    providers: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderSummary {
    pub status: String,
    pub capability_mode: String,
    pub providers: BTreeMap<String, String>,
    pub missing: Vec<String>,
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

pub fn save_provider_value(
    provider: &str,
    field: &str,
    value: &str,
) -> Result<PathBuf, ConfigError> {
    let provider = normalize_key(provider);
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
    let file = read_provider_file()?;
    let mut providers = BTreeMap::from([
        ("openalex".to_string(), "missing".to_string()),
        ("semantic_scholar".to_string(), "missing".to_string()),
        ("crossref".to_string(), "missing".to_string()),
        ("pubmed".to_string(), "missing".to_string()),
        ("arxiv".to_string(), "configured".to_string()),
    ]);
    let mut missing = Vec::new();

    for (provider, field, aliases) in provider_fields() {
        if configured_field(&file, provider, field, &aliases) {
            providers.insert(provider.to_string(), "configured".to_string());
        } else {
            missing.push(format!("{provider}.{field}"));
        }
    }

    let capability_mode = if providers.values().any(|status| status == "configured") {
        "provider_connected"
    } else {
        "strategy_only"
    };

    Ok(ProviderSummary {
        status: "ok".to_string(),
        capability_mode: capability_mode.to_string(),
        providers,
        missing,
    })
}

pub fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
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
    fs::write(path, text)?;
    set_owner_only_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &PathBuf) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &PathBuf) -> Result<(), std::io::Error> {
    Ok(())
}

fn configured_field(file: &ProviderFile, provider: &str, field: &str, aliases: &[&str]) -> bool {
    if let Some(entry) = file.providers.get(provider) {
        if entry
            .get(field)
            .and_then(serde_json::Value::as_str)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return true;
        }
    }
    aliases.iter().any(|alias| {
        std::env::var(alias)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}

fn is_supported_field(provider: &str, field: &str) -> bool {
    provider_fields()
        .iter()
        .any(|(known_provider, known_field, _)| {
            *known_provider == provider && *known_field == field
        })
}

fn provider_fields() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
    vec![
        (
            "openalex",
            "api_key",
            vec![
                "QIONGLI_OPENALEX_API_KEY",
                "OPENALEX_API_KEY",
                "QIONGLI_MCPB_OPENALEX_API_KEY",
            ],
        ),
        (
            "semantic_scholar",
            "api_key",
            vec![
                "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
                "SEMANTIC_SCHOLAR_API_KEY",
                "S2_API_KEY",
                "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
            ],
        ),
        (
            "crossref",
            "email",
            vec![
                "QIONGLI_CROSSREF_EMAIL",
                "CROSSREF_EMAIL",
                "QIONGLI_MCPB_CROSSREF_EMAIL",
            ],
        ),
        (
            "pubmed",
            "api_key",
            vec![
                "QIONGLI_NCBI_API_KEY",
                "NCBI_API_KEY",
                "PUBMED_API_KEY",
                "QIONGLI_MCPB_PUBMED_API_KEY",
            ],
        ),
    ]
}
