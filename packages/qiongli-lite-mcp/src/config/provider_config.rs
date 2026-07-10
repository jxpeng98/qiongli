use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const PROVIDERS: [&str; 5] = [
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
];

const TEMP_CREATE_ATTEMPTS: usize = 128;
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    #[error("provider config home must be a fully qualified absolute path or start with ~/")]
    InvalidConfigHome,
    #[error("platform home directory is unavailable")]
    HomeUnavailable,
    #[error("invalid provider config: {0}")]
    InvalidConfig(&'static str),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderFile {
    #[serde(default = "default_version")]
    version: u64,
    #[serde(default)]
    providers: BTreeMap<String, serde_json::Value>,
    #[serde(default, flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Default for ProviderFile {
    fn default() -> Self {
        Self {
            version: default_version(),
            providers: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
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
            let entry = file
                .providers
                .entry(provider)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            let entry = entry.as_object_mut().ok_or(ConfigError::InvalidConfig(
                "known provider entry must be an object",
            ))?;
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

pub fn provider_config_path() -> Result<PathBuf, ConfigError> {
    let root = match trimmed_env_path("QIONGLI_CONFIG_HOME") {
        Some(configured) => resolve_config_home(configured)?,
        None => platform_home_dir()?.join(".config").join("qiongli"),
    };
    Ok(root.join("providers.json"))
}

fn resolve_config_home(configured: PathBuf) -> Result<PathBuf, ConfigError> {
    if configured.is_absolute() {
        return Ok(configured);
    }
    let configured = configured.to_str().ok_or(ConfigError::InvalidConfigHome)?;
    if configured == "~" {
        return platform_home_dir();
    }
    if let Some(suffix) = configured.strip_prefix("~/") {
        if !is_portable_relative_suffix(suffix) {
            return Err(ConfigError::InvalidConfigHome);
        }
        let suffix = Path::new(suffix);
        if suffix.has_root() || matches!(suffix.components().next(), Some(Component::Prefix(_))) {
            return Err(ConfigError::InvalidConfigHome);
        }
        return Ok(platform_home_dir()?.join(suffix));
    }
    Err(ConfigError::InvalidConfigHome)
}

fn is_portable_relative_suffix(suffix: &str) -> bool {
    let bytes = suffix.as_bytes();
    !matches!(bytes.first(), Some(b'/' | b'\\'))
        && !matches!(bytes, [drive, b':', ..] if drive.is_ascii_alphabetic())
}

fn trimmed_env_path(name: &str) -> Option<PathBuf> {
    let value = std::env::var_os(name)?;
    if let Some(value) = value.to_str() {
        let value = value.trim();
        (!value.is_empty()).then(|| PathBuf::from(value))
    } else if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

#[cfg(unix)]
fn platform_home_dir() -> Result<PathBuf, ConfigError> {
    trimmed_env_path("HOME")
        .filter(|path| path.is_absolute())
        .ok_or(ConfigError::HomeUnavailable)
}

#[cfg(windows)]
fn platform_home_dir() -> Result<PathBuf, ConfigError> {
    if let Some(home) = trimmed_env_path("USERPROFILE").filter(|path| path.is_absolute()) {
        return Ok(home);
    }

    let drive = trimmed_env_path("HOMEDRIVE").ok_or(ConfigError::HomeUnavailable)?;
    let home_path = trimmed_env_path("HOMEPATH").ok_or(ConfigError::HomeUnavailable)?;
    let mut combined = drive.into_os_string();
    combined.push(home_path.as_os_str());
    let home = PathBuf::from(combined);
    home.is_absolute()
        .then_some(home)
        .ok_or(ConfigError::HomeUnavailable)
}

#[cfg(not(any(unix, windows)))]
fn platform_home_dir() -> Result<PathBuf, ConfigError> {
    trimmed_env_path("HOME")
        .filter(|path| path.is_absolute())
        .ok_or(ConfigError::HomeUnavailable)
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

    let path = provider_config_path()?;
    save_provider_value_at(&path, &provider, &field, value)?;
    Ok(path)
}

pub(crate) fn save_provider_value_at(
    path: &Path,
    provider: &str,
    field: &str,
    value: &str,
) -> Result<(), ConfigError> {
    let provider = normalize_provider(provider);
    let field = normalize_key(field);
    if !is_supported_field(&provider, &field) {
        return Err(ConfigError::UnsupportedField(provider, field));
    }

    let mut file = read_provider_file_at(path)?;
    canonicalize_known_aliases(&mut file)?;
    file.version = 1;
    let entry = file
        .providers
        .entry(provider)
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let entry = entry.as_object_mut().ok_or(ConfigError::InvalidConfig(
        "known provider entry must be an object",
    ))?;
    entry.insert("enabled".to_string(), serde_json::Value::Bool(true));
    entry.insert(field, serde_json::Value::String(value.to_string()));
    write_provider_file(path, &file)
}

pub fn summary() -> Result<ProviderSummary, ConfigError> {
    let config_path = provider_config_path()?;
    let file = read_provider_file_at(&config_path)?;
    let resolved = resolve_provider_file(&file, |name| std::env::var(name).ok());
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
    let next_provider = [
        ("openalex.api_key", "openalex"),
        ("semantic_scholar.api_key", "semantic_scholar"),
        ("crossref.email", "crossref"),
        ("pubmed.api_key", "pubmed"),
    ]
    .into_iter()
    .find_map(|(field, provider)| missing.iter().any(|item| item == field).then_some(provider));
    let next_action = next_provider.map(|provider| ProviderNextAction {
        tool: "qiongli_configure_provider".to_string(),
        args: BTreeMap::from([("provider".to_string(), provider.to_string())]),
        message: ("Run qiongli_configure_provider to open a local setup page. ".to_string()
            + "Do not paste API keys in chat."),
    });

    Ok(ProviderSummary {
        status: "ok".to_string(),
        config_path,
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
    let path = provider_config_path()?;
    read_provider_file_at(&path)
}

fn read_provider_file_at(path: &Path) -> Result<ProviderFile, ConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ProviderFile::default());
        }
        Err(error) => return Err(ConfigError::Io(error)),
    };
    let payload: serde_json::Value = serde_json::from_str(&text)?;
    validate_provider_payload(&payload)?;
    Ok(serde_json::from_value(payload)?)
}

fn validate_provider_payload(payload: &serde_json::Value) -> Result<(), ConfigError> {
    let root = payload.as_object().ok_or(ConfigError::InvalidConfig(
        "provider config root must be an object",
    ))?;

    if let Some(version) = root.get("version") {
        let version = version.as_u64().ok_or(ConfigError::InvalidConfig(
            "version must be a positive integer",
        ))?;
        if version == 0 {
            return Err(ConfigError::InvalidConfig(
                "version must be a positive integer",
            ));
        }
        if version != default_version() {
            return Err(ConfigError::InvalidConfig("version is not supported"));
        }
    }

    if let Some(providers) = root.get("providers") {
        let providers = providers
            .as_object()
            .ok_or(ConfigError::InvalidConfig("providers must be an object"))?;
        let mut seen_known_providers = BTreeSet::new();
        for (raw_provider, raw_entry) in providers {
            let provider = normalize_provider(raw_provider);
            if !PROVIDERS.contains(&provider.as_str()) {
                continue;
            }
            if !seen_known_providers.insert(provider.clone()) {
                return Err(ConfigError::InvalidConfig(
                    "known provider aliases must not collide",
                ));
            }
            let entry = raw_entry.as_object().ok_or(ConfigError::InvalidConfig(
                "known provider entry must be an object",
            ))?;
            let mut seen_known_fields = BTreeSet::new();
            for (raw_field, value) in entry {
                let field = normalize_key(raw_field);
                let known_field = field == "enabled" || is_supported_field(&provider, &field);
                if known_field && !seen_known_fields.insert(field.clone()) {
                    return Err(ConfigError::InvalidConfig(
                        "known provider field aliases must not collide",
                    ));
                }
                if field == "enabled" && !value.is_boolean() {
                    return Err(ConfigError::InvalidConfig(
                        "known provider enabled field must be a boolean",
                    ));
                }
                if is_supported_field(&provider, &field) && !value.is_string() {
                    return Err(ConfigError::InvalidConfig(
                        "known provider credential field must be a string",
                    ));
                }
            }
        }
    }

    if let Some(search) = root.get("search") {
        let search = search.as_object().ok_or(ConfigError::InvalidConfig(
            "search settings must be an object",
        ))?;
        let mut seen_known_fields = BTreeSet::new();
        for (raw_field, value) in search {
            let field = normalize_key(raw_field);
            let known_field = matches!(
                field.as_str(),
                "minimum_productive_providers" | "allow_platform_search_supplement"
            );
            if known_field && !seen_known_fields.insert(field.clone()) {
                return Err(ConfigError::InvalidConfig(
                    "known search field aliases must not collide",
                ));
            }
            match field.as_str() {
                "minimum_productive_providers" => {
                    let positive_integer = value.as_u64().is_some_and(|integer| integer >= 1)
                        || value.as_i64().is_some_and(|integer| integer >= 1);
                    if positive_integer {
                        continue;
                    }
                    return Err(ConfigError::InvalidConfig(
                        "search minimum_productive_providers must be a positive integer",
                    ));
                }
                "allow_platform_search_supplement" if !value.is_boolean() => {
                    return Err(ConfigError::InvalidConfig(
                        "search allow_platform_search_supplement must be a boolean",
                    ));
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn canonicalize_known_aliases(file: &mut ProviderFile) -> Result<(), ConfigError> {
    let providers = std::mem::take(&mut file.providers);
    for (raw_provider, mut entry) in providers {
        let provider = normalize_provider(&raw_provider);
        if PROVIDERS.contains(&provider.as_str()) {
            canonicalize_known_provider_fields(&provider, &mut entry)?;
            file.providers.insert(provider, entry);
        } else {
            file.providers.insert(raw_provider, entry);
        }
    }

    if let Some(search) = file
        .extra
        .get_mut("search")
        .and_then(serde_json::Value::as_object_mut)
    {
        let fields = std::mem::take(search);
        for (raw_field, value) in fields {
            let field = normalize_key(&raw_field);
            let canonical = matches!(
                field.as_str(),
                "minimum_productive_providers" | "allow_platform_search_supplement"
            );
            search.insert(if canonical { field } else { raw_field }, value);
        }
    }

    Ok(())
}

fn canonicalize_known_provider_fields(
    provider: &str,
    entry: &mut serde_json::Value,
) -> Result<(), ConfigError> {
    let entry = entry.as_object_mut().ok_or(ConfigError::InvalidConfig(
        "known provider entry must be an object",
    ))?;
    let fields = std::mem::take(entry);
    for (raw_field, value) in fields {
        let field = normalize_key(&raw_field);
        let canonical = field == "enabled" || is_supported_field(provider, &field);
        entry.insert(if canonical { field } else { raw_field }, value);
    }
    Ok(())
}

fn write_provider_file(path: &Path, file: &ProviderFile) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = format!("{}\n", serde_json::to_string_pretty(file)?);
    write_owner_only(path, text.as_bytes()).map_err(ConfigError::Io)
}

fn write_owner_only(path: &Path, contents: &[u8]) -> Result<(), std::io::Error> {
    atomic_write_with_replacer(path, contents, replace_file)
}

fn atomic_write_with_replacer<F>(
    path: &Path,
    contents: &[u8],
    replace: F,
) -> Result<(), std::io::Error>
where
    F: FnOnce(&Path, &Path) -> Result<(), std::io::Error>,
{
    let (temporary, mut file) = create_temporary_file(path)?;
    let write_result = (|| {
        file.write_all(contents)?;
        file.flush()?;
        file.sync_all()
    })();
    drop(file);

    let result = write_result.and_then(|()| replace(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn create_temporary_file(path: &Path) -> Result<(PathBuf, File), std::io::Error> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "provider config path has no parent directory",
        )
    })?;
    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "provider config path has no file name",
        )
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    for _ in 0..TEMP_CREATE_ATTEMPTS {
        let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(".tmp-{}-{nonce}-{sequence}", std::process::id()));
        let temporary = parent.join(temporary_name);
        match create_owner_only_temporary_file(&temporary) {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate a unique provider config temporary file",
    ))
}

#[cfg(not(windows))]
fn create_owner_only_temporary_file(path: &Path) -> Result<File, std::io::Error> {
    use std::fs::OpenOptions;

    let mut options = OpenOptions::new();
    options.create_new(true).write(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }

    let file = options.open(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Err(error) = file.set_permissions(fs::Permissions::from_mode(0o600)) {
            drop(file);
            let _ = fs::remove_file(path);
            return Err(error);
        }
    }

    Ok(file)
}

#[cfg(windows)]
fn create_owner_only_temporary_file(path: &Path) -> Result<File, std::io::Error> {
    windows_security::create_owner_only_file(path)
}

#[cfg(windows)]
mod windows_security {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use std::path::Path;
    use std::ptr::null_mut;

    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_USER_CLASS: i32 = 1;
    const ACL_REVISION: u32 = 2;
    const FILE_ALL_ACCESS: u32 = 0x001f_01ff;
    const SE_DACL_PROTECTED: u16 = 0x1000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const READ_CONTROL: u32 = 0x0002_0000;
    const CREATE_NEW: u32 = 1;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
    const INVALID_HANDLE_VALUE: *mut c_void = -1_isize as *mut c_void;
    const OWNER_SECURITY_INFORMATION: u32 = 0x0000_0001;
    const DACL_SECURITY_INFORMATION: u32 = 0x0000_0004;
    const SE_FILE_OBJECT: i32 = 1;
    const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
    const INHERITED_ACE: u8 = 0x10;
    #[cfg(test)]
    const FILE_GENERIC_READ: u32 = 0x0012_0089;
    #[cfg(test)]
    const SECURITY_MAX_SID_SIZE: usize = 68;
    #[cfg(test)]
    const WIN_WORLD_SID: i32 = 1;

    #[repr(C)]
    struct SecurityAttributes {
        _length: u32,
        _security_descriptor: *mut c_void,
        _inherit_handle: i32,
    }

    #[repr(C)]
    struct SecurityDescriptor {
        _revision: u8,
        _reserved: u8,
        _control: u16,
        _owner: *mut c_void,
        _group: *mut c_void,
        _system_acl: *mut Acl,
        _discretionary_acl: *mut Acl,
    }

    #[repr(C)]
    struct Acl {
        _revision: u8,
        _reserved_1: u8,
        _size: u16,
        ace_count: u16,
        _reserved_2: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SidAndAttributes {
        sid: *mut c_void,
        _attributes: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct TokenUser {
        user: SidAndAttributes,
    }

    #[repr(C)]
    struct AceHeader {
        ace_type: u8,
        ace_flags: u8,
        _ace_size: u16,
    }

    #[repr(C)]
    struct AccessAllowedAce {
        header: AceHeader,
        mask: u32,
        sid_start: u32,
    }

    struct OwnedHandle(*mut c_void);

    struct OwnedLocal(*mut c_void);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // SAFETY: this wrapper is only constructed from a successful OpenProcessToken call.
            unsafe {
                close_handle(self.0);
            }
        }
    }

    impl Drop for OwnedLocal {
        fn drop(&mut self) {
            // SAFETY: this wrapper is only constructed from GetSecurityInfo-owned storage.
            unsafe {
                local_free(self.0);
            }
        }
    }

    #[link(name = "kernel32")]
    extern "system" {
        #[link_name = "CloseHandle"]
        fn close_handle(object: *mut c_void) -> i32;
        #[link_name = "CreateFileW"]
        fn create_file_w(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *mut SecurityAttributes,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: *mut c_void,
        ) -> *mut c_void;
        #[link_name = "GetCurrentProcess"]
        fn get_current_process() -> *mut c_void;
        #[link_name = "LocalFree"]
        fn local_free(memory: *mut c_void) -> *mut c_void;
    }

    #[link(name = "advapi32")]
    extern "system" {
        #[link_name = "AddAccessAllowedAceEx"]
        fn add_access_allowed_ace_ex(
            acl: *mut Acl,
            ace_revision: u32,
            ace_flags: u32,
            access_mask: u32,
            sid: *const c_void,
        ) -> i32;
        #[cfg(test)]
        #[link_name = "CreateWellKnownSid"]
        fn create_well_known_sid(
            sid_type: i32,
            domain_sid: *const c_void,
            sid: *mut c_void,
            sid_size: *mut u32,
        ) -> i32;
        #[link_name = "GetLengthSid"]
        fn get_length_sid(sid: *const c_void) -> u32;
        #[link_name = "GetSecurityDescriptorControl"]
        fn get_security_descriptor_control(
            security_descriptor: *const c_void,
            control: *mut u16,
            revision: *mut u32,
        ) -> i32;
        #[link_name = "GetSecurityInfo"]
        fn get_security_info(
            handle: *mut c_void,
            object_type: i32,
            requested_information: u32,
            owner: *mut *mut c_void,
            group: *mut *mut c_void,
            dacl: *mut *mut Acl,
            system_acl: *mut *mut Acl,
            security_descriptor: *mut *mut c_void,
        ) -> u32;
        #[link_name = "GetTokenInformation"]
        fn get_token_information(
            token_handle: *mut c_void,
            token_information_class: i32,
            token_information: *mut c_void,
            token_information_length: u32,
            return_length: *mut u32,
        ) -> i32;
        #[link_name = "InitializeAcl"]
        fn initialize_acl(acl: *mut Acl, acl_length: u32, acl_revision: u32) -> i32;
        #[link_name = "InitializeSecurityDescriptor"]
        fn initialize_security_descriptor(
            security_descriptor: *mut SecurityDescriptor,
            revision: u32,
        ) -> i32;
        #[link_name = "IsValidSid"]
        fn is_valid_sid(sid: *const c_void) -> i32;
        #[link_name = "OpenProcessToken"]
        fn open_process_token(
            process_handle: *mut c_void,
            desired_access: u32,
            token_handle: *mut *mut c_void,
        ) -> i32;
        #[link_name = "SetSecurityDescriptorControl"]
        fn set_security_descriptor_control(
            security_descriptor: *mut SecurityDescriptor,
            control_bits_of_interest: u16,
            control_bits_to_set: u16,
        ) -> i32;
        #[link_name = "SetSecurityDescriptorDacl"]
        fn set_security_descriptor_dacl(
            security_descriptor: *mut SecurityDescriptor,
            dacl_present: i32,
            dacl: *mut Acl,
            dacl_defaulted: i32,
        ) -> i32;
        #[link_name = "SetSecurityDescriptorOwner"]
        fn set_security_descriptor_owner(
            security_descriptor: *mut SecurityDescriptor,
            owner: *mut c_void,
            owner_defaulted: i32,
        ) -> i32;

        #[link_name = "EqualSid"]
        fn equal_sid(first_sid: *const c_void, second_sid: *const c_void) -> i32;
        #[link_name = "GetAce"]
        fn get_ace(acl: *const Acl, ace_index: u32, ace: *mut *mut c_void) -> i32;
    }

    pub(super) fn create_owner_only_file(path: &Path) -> io::Result<File> {
        with_current_user_sid(|sid| {
            let sid_length = unsafe { get_length_sid(sid) } as usize;
            if sid_length == 0 {
                return Err(io::Error::last_os_error());
            }

            let acl_length = size_of::<Acl>()
                .checked_add(size_of::<AccessAllowedAceLayout>() - size_of::<u32>())
                .and_then(|length| length.checked_add(sid_length))
                .ok_or_else(|| io::Error::other("provider config ACL is too large"))?;
            let acl_words = acl_length.div_ceil(size_of::<u32>());
            let mut acl_storage = vec![0_u32; acl_words];
            let acl = acl_storage.as_mut_ptr().cast::<Acl>();
            let acl_storage_length = u32::try_from(acl_words * size_of::<u32>())
                .map_err(|_| io::Error::other("provider config ACL is too large"))?;

            // SAFETY: acl_storage is writable, aligned, and retained until CreateFileW returns.
            if unsafe { initialize_acl(acl, acl_storage_length, ACL_REVISION) } == 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: sid is a validated token-user SID and acl has sufficient storage.
            if unsafe { add_access_allowed_ace_ex(acl, ACL_REVISION, 0, FILE_ALL_ACCESS, sid) } == 0
            {
                return Err(io::Error::last_os_error());
            }

            let mut descriptor = SecurityDescriptor {
                _revision: 0,
                _reserved: 0,
                _control: 0,
                _owner: null_mut(),
                _group: null_mut(),
                _system_acl: null_mut(),
                _discretionary_acl: null_mut(),
            };
            // SAFETY: descriptor is writable and has the Win32 SECURITY_DESCRIPTOR layout.
            if unsafe { initialize_security_descriptor(&mut descriptor, 1) } == 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: sid and acl remain alive through CreateFileW and contain valid structures.
            if unsafe { set_security_descriptor_owner(&mut descriptor, sid, 0) } == 0
                || unsafe { set_security_descriptor_dacl(&mut descriptor, 1, acl, 0) } == 0
                || unsafe {
                    set_security_descriptor_control(
                        &mut descriptor,
                        SE_DACL_PROTECTED,
                        SE_DACL_PROTECTED,
                    )
                } == 0
            {
                return Err(io::Error::last_os_error());
            }

            let mut security_attributes = SecurityAttributes {
                _length: size_of::<SecurityAttributes>() as u32,
                _security_descriptor: (&mut descriptor as *mut SecurityDescriptor).cast(),
                _inherit_handle: 0,
            };
            let wide_path = wide_path(path)?;
            // SAFETY: wide_path is null-terminated; all pointed-to security structures remain
            // alive for the duration of the call. CREATE_NEW preserves create_new semantics.
            let handle = unsafe {
                create_file_w(
                    wide_path.as_ptr(),
                    GENERIC_WRITE | READ_CONTROL,
                    0,
                    &mut security_attributes,
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                )
            };
            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            // SAFETY: CreateFileW returned a unique, owned file handle.
            let file = unsafe { File::from_raw_handle(handle) };
            if let Err(error) = verify_owner_only_handle(file.as_raw_handle()) {
                drop(file);
                let _ = std::fs::remove_file(path);
                return Err(error);
            }
            Ok(file)
        })
    }

    // This layout is used only for the documented ACL buffer-size formula.
    #[repr(C)]
    struct AccessAllowedAceLayout {
        _header: [u8; 4],
        _mask: u32,
        _sid_start: u32,
    }

    fn with_current_user_sid<T>(
        operation: impl FnOnce(*mut c_void) -> io::Result<T>,
    ) -> io::Result<T> {
        let mut token = null_mut();
        // SAFETY: token points to writable storage; GetCurrentProcess returns a pseudo-handle.
        if unsafe { open_process_token(get_current_process(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let _token = OwnedHandle(token);

        let mut required_length = 0_u32;
        // The first call is expected to fail with ERROR_INSUFFICIENT_BUFFER and report the size.
        unsafe {
            get_token_information(token, TOKEN_USER_CLASS, null_mut(), 0, &mut required_length);
        }
        if required_length < size_of::<TokenUser>() as u32 {
            return Err(io::Error::last_os_error());
        }

        let word_count = (required_length as usize).div_ceil(size_of::<usize>());
        let mut token_storage = vec![0_usize; word_count];
        // SAFETY: token_storage is writable, aligned, and sized from GetTokenInformation.
        if unsafe {
            get_token_information(
                token,
                TOKEN_USER_CLASS,
                token_storage.as_mut_ptr().cast(),
                required_length,
                &mut required_length,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: a successful TOKEN_USER query initialized the leading TokenUser structure.
        let token_user = unsafe { &*token_storage.as_ptr().cast::<TokenUser>() };
        if token_user.user.sid.is_null() || unsafe { is_valid_sid(token_user.user.sid) } == 0 {
            return Err(io::Error::last_os_error());
        }
        operation(token_user.user.sid)
    }

    fn wide_path(path: &Path) -> io::Result<Vec<u16>> {
        let mut encoded: Vec<u16> = path.as_os_str().encode_wide().collect();
        if encoded.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "provider config path contains a null character",
            ));
        }
        encoded.push(0);
        Ok(encoded)
    }

    fn verify_owner_only_handle(handle: *mut c_void) -> io::Result<()> {
        let requested = OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION;
        let mut owner = null_mut();
        let mut dacl = null_mut();
        let mut descriptor = null_mut();
        let status = unsafe {
            get_security_info(
                handle,
                SE_FILE_OBJECT,
                requested,
                &mut owner,
                null_mut(),
                &mut dacl,
                null_mut(),
                &mut descriptor,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        if descriptor.is_null() {
            return Err(io::Error::other(
                "provider config security descriptor is unavailable",
            ));
        }
        let _descriptor = OwnedLocal(descriptor);

        let mut control = 0_u16;
        let mut revision = 0_u32;
        if unsafe { get_security_descriptor_control(descriptor, &mut control, &mut revision) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if control & SE_DACL_PROTECTED == 0 {
            return Err(io::Error::other("provider config DACL is not protected"));
        }

        if owner.is_null() || dacl.is_null() {
            return Err(io::Error::other(
                "provider config owner or DACL is unavailable",
            ));
        }
        // SAFETY: GetSecurityInfo returned a non-null ACL in _descriptor-owned storage.
        let acl = unsafe { &*dacl };
        if acl.ace_count != 1 {
            return Err(io::Error::other(
                "provider config DACL must contain exactly one ACE",
            ));
        }

        let mut ace = null_mut();
        if unsafe { get_ace(dacl, 0, &mut ace) } == 0 || ace.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: GetAce returned the only ACE; the ACL was constructed as ACCESS_ALLOWED_ACE.
        let allowed_ace = unsafe { &*ace.cast::<AccessAllowedAce>() };
        if allowed_ace.header.ace_type != ACCESS_ALLOWED_ACE_TYPE
            || allowed_ace.header.ace_flags & INHERITED_ACE != 0
            || allowed_ace.mask != FILE_ALL_ACCESS
        {
            return Err(io::Error::other(
                "provider config DACL contains an unexpected ACE",
            ));
        }
        let ace_sid = (&allowed_ace.sid_start as *const u32).cast::<c_void>();

        with_current_user_sid(|current_user| {
            if unsafe { equal_sid(owner, current_user) } == 0
                || unsafe { equal_sid(ace_sid, current_user) } == 0
            {
                return Err(io::Error::other(
                    "provider config owner or DACL does not match the current user",
                ));
            }
            Ok(())
        })
    }

    #[cfg(test)]
    pub(super) fn verify_owner_only_file(path: &Path) -> io::Result<()> {
        let file = File::open(path)?;
        verify_owner_only_handle(file.as_raw_handle())
    }

    #[cfg(test)]
    pub(super) fn create_noncompliant_inheritable_acl_file(
        path: &Path,
        contents: &[u8],
    ) -> io::Result<()> {
        use std::io::Write as _;

        with_current_user_sid(|current_user| {
            let everyone_words = SECURITY_MAX_SID_SIZE.div_ceil(size_of::<usize>());
            let mut everyone_storage = vec![0_usize; everyone_words];
            let mut everyone_length = SECURITY_MAX_SID_SIZE as u32;
            // SAFETY: everyone_storage is aligned and sized to SECURITY_MAX_SID_SIZE.
            if unsafe {
                create_well_known_sid(
                    WIN_WORLD_SID,
                    null_mut(),
                    everyone_storage.as_mut_ptr().cast(),
                    &mut everyone_length,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            let everyone = everyone_storage.as_mut_ptr().cast::<c_void>();

            let current_user_length = unsafe { get_length_sid(current_user) } as usize;
            let everyone_length = unsafe { get_length_sid(everyone) } as usize;
            if current_user_length == 0 || everyone_length == 0 {
                return Err(io::Error::last_os_error());
            }
            let ace_prefix_length = size_of::<AccessAllowedAceLayout>() - size_of::<u32>();
            let acl_length = size_of::<Acl>()
                .checked_add(ace_prefix_length)
                .and_then(|length| length.checked_add(current_user_length))
                .and_then(|length| length.checked_add(ace_prefix_length))
                .and_then(|length| length.checked_add(everyone_length))
                .ok_or_else(|| io::Error::other("test provider config ACL is too large"))?;
            let acl_words = acl_length.div_ceil(size_of::<u32>());
            let mut acl_storage = vec![0_u32; acl_words];
            let acl = acl_storage.as_mut_ptr().cast::<Acl>();
            let acl_storage_length = u32::try_from(acl_words * size_of::<u32>())
                .map_err(|_| io::Error::other("test provider config ACL is too large"))?;

            // SAFETY: acl_storage is writable, aligned, and retained through CreateFileW.
            if unsafe { initialize_acl(acl, acl_storage_length, ACL_REVISION) } == 0
                || unsafe {
                    add_access_allowed_ace_ex(acl, ACL_REVISION, 0, FILE_ALL_ACCESS, current_user)
                } == 0
                || unsafe {
                    add_access_allowed_ace_ex(acl, ACL_REVISION, 0, FILE_GENERIC_READ, everyone)
                } == 0
            {
                return Err(io::Error::last_os_error());
            }

            let mut descriptor = SecurityDescriptor {
                _revision: 0,
                _reserved: 0,
                _control: 0,
                _owner: null_mut(),
                _group: null_mut(),
                _system_acl: null_mut(),
                _discretionary_acl: null_mut(),
            };
            // Deliberately leave SE_DACL_PROTECTED unset so this is a deterministic
            // noncompliant precondition even when the parent contributes no inherited ACEs.
            if unsafe { initialize_security_descriptor(&mut descriptor, 1) } == 0
                || unsafe { set_security_descriptor_owner(&mut descriptor, current_user, 0) } == 0
                || unsafe { set_security_descriptor_dacl(&mut descriptor, 1, acl, 0) } == 0
            {
                return Err(io::Error::last_os_error());
            }

            let mut security_attributes = SecurityAttributes {
                _length: size_of::<SecurityAttributes>() as u32,
                _security_descriptor: (&mut descriptor as *mut SecurityDescriptor).cast(),
                _inherit_handle: 0,
            };
            let wide_path = wide_path(path)?;
            // SAFETY: the path is null-terminated and the descriptor, ACL, and SIDs remain
            // alive for the duration of CreateFileW.
            let handle = unsafe {
                create_file_w(
                    wide_path.as_ptr(),
                    GENERIC_WRITE | READ_CONTROL,
                    0,
                    &mut security_attributes,
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                )
            };
            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            // SAFETY: CreateFileW returned a unique, owned file handle.
            let mut file = unsafe { File::from_raw_handle(handle) };
            let write_result = (|| {
                file.write_all(contents)?;
                file.flush()?;
                file.sync_all()
            })();
            drop(file);
            if write_result.is_err() {
                let _ = std::fs::remove_file(path);
            }
            write_result
        })
    }
}

#[cfg(unix)]
fn replace_file(temporary: &Path, path: &Path) -> Result<(), std::io::Error> {
    fs::rename(temporary, path)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, path: &Path) -> Result<(), std::io::Error> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "kernel32")]
    extern "system" {
        #[link_name = "MoveFileExW"]
        fn move_file_ex_w(
            existing_file_name: *const u16,
            new_file_name: *const u16,
            flags: u32,
        ) -> i32;
    }

    fn wide_path(path: &Path) -> Result<Vec<u16>, std::io::Error> {
        let mut encoded: Vec<u16> = path.as_os_str().encode_wide().collect();
        if encoded.contains(&0) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "provider config path contains a null character",
            ));
        }
        encoded.push(0);
        Ok(encoded)
    }

    let temporary = wide_path(temporary)?;
    let path = wide_path(path)?;
    // SAFETY: both path buffers are null-terminated and remain alive for the call.
    let result = unsafe {
        move_file_ex_w(
            temporary.as_ptr(),
            path.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(any(unix, windows)))]
fn replace_file(temporary: &Path, path: &Path) -> Result<(), std::io::Error> {
    fs::rename(temporary, path)
}

fn resolve_provider_file<F>(file: &ProviderFile, env_value: F) -> ResolvedProviderConfig
where
    F: Fn(&str) -> Option<String>,
{
    let mut resolved = ResolvedProviderConfig::default();
    for provider in PROVIDERS {
        let persisted = persisted_provider(file, provider);
        let mut values = BTreeMap::new();
        let mut environment_supplied = false;
        for spec in PROVIDER_FIELD_SPECS
            .iter()
            .filter(|spec| spec.provider == provider)
        {
            if let Some((value, from_environment)) =
                resolve_field_value(persisted, spec, &env_value)
            {
                environment_supplied |= from_environment;
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
            .and_then(|entry| persisted_value(entry, "enabled"))
            .and_then(serde_json::Value::as_bool);
        let enabled = if environment_supplied {
            true
        } else {
            explicitly_enabled.unwrap_or(configured)
        };
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

fn persisted_provider<'a>(
    file: &'a ProviderFile,
    provider: &str,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    file.providers
        .get(provider)
        .or_else(|| {
            file.providers.iter().find_map(|(raw_provider, entry)| {
                (normalize_provider(raw_provider) == provider).then_some(entry)
            })
        })
        .and_then(serde_json::Value::as_object)
}

fn persisted_value<'a>(
    persisted: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Option<&'a serde_json::Value> {
    persisted.get(field).or_else(|| {
        persisted
            .iter()
            .find_map(|(raw_field, value)| (normalize_key(raw_field) == field).then_some(value))
    })
}

fn resolve_field_value<F>(
    persisted: Option<&serde_json::Map<String, serde_json::Value>>,
    spec: &ProviderFieldSpec,
    env_value: &F,
) -> Option<(String, bool)>
where
    F: Fn(&str) -> Option<String>,
{
    let environment_value = spec
        .env_aliases
        .iter()
        .filter_map(|alias| env_value(alias))
        .find_map(non_empty);
    if let Some(value) = environment_value {
        return Some((value, true));
    }

    persisted
        .and_then(|entry| persisted_value(entry, spec.field))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| non_empty(value.to_string()))
        .map(|value| (value, false))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_suffix_rejects_cross_platform_absolute_forms_lexically() {
        for configured in ["~//abs", r"~/\abs", r"~/C:\abs", "~/C:relative"] {
            let suffix = configured.strip_prefix("~/").unwrap();
            assert!(!is_portable_relative_suffix(suffix), "{configured}");
        }
        for configured in ["~/", "~/nested/config", "~/colon:name"] {
            let suffix = configured.strip_prefix("~/").unwrap();
            assert!(is_portable_relative_suffix(suffix), "{configured}");
        }
    }

    #[test]
    fn replacement_failure_preserves_original_and_removes_temporary_file() {
        let directory = test_directory("replacement-failure");
        let path = directory.join("providers.json");
        fs::write(&path, b"original provider config\n").unwrap();

        let result = atomic_write_with_replacer(&path, b"replacement\n", |temporary, target| {
            assert_eq!(temporary.parent(), target.parent());
            assert_eq!(fs::read(temporary).unwrap(), b"replacement\n");
            Err(std::io::Error::other("injected replacement failure"))
        });

        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original provider config\n");
        let remaining: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(remaining, vec![OsString::from("providers.json")]);
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_new_and_replaced_files_have_a_protected_current_user_only_dacl() {
        let directory = test_directory("windows-owner-only-dacl");
        let path = directory.join("providers.json");

        save_provider_value_at(&path, "pubmed", "api_key", "first-secret").unwrap();
        windows_security::verify_owner_only_file(&path).unwrap();

        save_provider_value_at(&path, "pubmed", "api_key", "replacement-secret").unwrap();
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("replacement-secret"));
        windows_security::verify_owner_only_file(&path).unwrap();

        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_secure_replacement_overwrites_a_preexisting_inherited_acl_file() {
        let directory = test_directory("windows-preexisting-acl-replacement");
        let path = directory.join("providers.json");
        windows_security::create_noncompliant_inheritable_acl_file(
            &path,
            br#"{"version":1,"providers":{"arxiv":{"enabled":false}}}"#,
        )
        .unwrap();
        let precondition = windows_security::verify_owner_only_file(&path)
            .expect_err("the broad unprotected preexisting DACL must be rejected");
        assert_eq!(
            precondition.to_string(),
            "provider config DACL is not protected"
        );

        save_provider_value_at(&path, "crossref", "email", "person@example.com").unwrap();

        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(payload["providers"]["arxiv"]["enabled"], false);
        assert_eq!(
            payload["providers"]["crossref"]["email"],
            "person@example.com"
        );
        windows_security::verify_owner_only_file(&path).unwrap();
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_acl_setup_errors_are_redacted_and_leave_no_temporary_file() {
        let directory = test_directory("windows-redacted-acl-error");
        let path = directory.join("credential-canary\0providers.json");

        let error = write_owner_only(&path, b"credential-value-canary\n")
            .expect_err("a path containing a null character must fail closed");

        assert!(!error.to_string().contains("credential-canary"));
        assert!(!error.to_string().contains("credential-value-canary"));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        fs::remove_dir_all(directory).unwrap();
    }

    fn test_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "qiongli-lite-provider-config-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }
}
