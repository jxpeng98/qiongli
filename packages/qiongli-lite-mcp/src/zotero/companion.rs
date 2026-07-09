use std::io::Read;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use url::{Host, Url};

pub const DEFAULT_CONNECTOR_URL: &str = "http://127.0.0.1:23119";
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

const MAX_COMPANION_RESPONSE_BYTES: u64 = 32 * 1024;

#[derive(Debug, Error)]
pub enum CompanionError {
    #[error("Zotero connector URL must use http or https")]
    InvalidScheme,
    #[error("Zotero connector URL must point to a loopback host")]
    NonLoopback,
    #[error("Zotero connector URL must not contain credentials")]
    CredentialsNotAllowed,
    #[error("invalid Zotero connector URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("failed to construct the Zotero loopback client")]
    ClientBuild(#[source] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct CompanionClient {
    base_url: Url,
    client: Client,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ZoteroStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub connector: ProbeStatus,
    pub companion: CompanionProbeStatus,
    pub fallback_import_files: ImportFileFallback,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProbeStatus {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CompanionProbeStatus {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImportFileFallback {
    pub available: bool,
    pub formats: Vec<String>,
}

impl CompanionClient {
    pub fn new(raw: &str) -> Result<Self, CompanionError> {
        Self::with_timeout(raw, DEFAULT_PROBE_TIMEOUT)
    }

    pub fn with_timeout(raw: &str, timeout: Duration) -> Result<Self, CompanionError> {
        let base_url = validate_connector_url(raw)?;
        let client = Client::builder()
            .connect_timeout(timeout)
            .timeout(timeout)
            .redirect(Policy::none())
            .build()
            .map_err(CompanionError::ClientBuild)?;
        Ok(Self { base_url, client })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn probe(&self, local_enabled: bool) -> ZoteroStatus {
        if !local_enabled {
            return disabled_status();
        }

        let connector = self.probe_connector();
        if !connector.available {
            return ZoteroStatus {
                status: "fallback_only".to_string(),
                error_code: Some("zotero_not_running".to_string()),
                connector,
                companion: unavailable_companion(None),
                fallback_import_files: import_file_fallback(),
            };
        }

        let companion = self.probe_companion();
        if !companion.available {
            return ZoteroStatus {
                status: "companion_missing".to_string(),
                error_code: Some("companion_missing".to_string()),
                connector,
                companion,
                fallback_import_files: import_file_fallback(),
            };
        }

        ZoteroStatus {
            status: "ok".to_string(),
            error_code: None,
            connector,
            companion,
            fallback_import_files: import_file_fallback(),
        }
    }

    fn probe_connector(&self) -> ProbeStatus {
        let endpoint = self
            .base_url
            .join("/connector/ping")
            .expect("static connector endpoint must be valid");
        match self.client.get(endpoint).send() {
            Ok(response) => ProbeStatus {
                available: response.status().is_success(),
                status: Some(response.status().as_u16()),
            },
            Err(_) => ProbeStatus {
                available: false,
                status: None,
            },
        }
    }

    fn probe_companion(&self) -> CompanionProbeStatus {
        let endpoint = self
            .base_url
            .join("/qiongli/ping")
            .expect("static companion endpoint must be valid");
        let response = match self.client.get(endpoint).send() {
            Ok(response) => response,
            Err(_) => return unavailable_companion(None),
        };
        let status = response.status().as_u16();
        if !response.status().is_success() {
            return unavailable_companion(Some(status));
        }
        let Some(payload) = read_limited_json(response) else {
            return unavailable_companion(Some(status));
        };

        CompanionProbeStatus {
            available: true,
            status: Some(status),
            version: filtered_version(
                payload
                    .get("version")
                    .or_else(|| payload.get("companion_version")),
            ),
            endpoint_version: filtered_version(payload.get("endpoint_version")),
        }
    }
}

pub fn probe_zotero_from_env() -> Result<ZoteroStatus, CompanionError> {
    let enabled = read_env_boolean("QIONGLI_ZOTERO_LOCAL_ENABLED", true);
    if !enabled {
        return Ok(disabled_status());
    }
    let connector_url = std::env::var("QIONGLI_ZOTERO_CONNECTOR_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_CONNECTOR_URL.to_string());
    Ok(CompanionClient::new(&connector_url)?.probe(true))
}

fn validate_connector_url(raw: &str) -> Result<Url, CompanionError> {
    let mut base_url = Url::parse(raw)?;
    if base_url.scheme() != "http" && base_url.scheme() != "https" {
        return Err(CompanionError::InvalidScheme);
    }
    if !base_url.username().is_empty() || base_url.password().is_some() {
        return Err(CompanionError::CredentialsNotAllowed);
    }
    let loopback = match base_url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    if !loopback {
        return Err(CompanionError::NonLoopback);
    }

    base_url.set_path("/");
    base_url.set_query(None);
    base_url.set_fragment(None);
    Ok(base_url)
}

fn read_limited_json(response: Response) -> Option<Value> {
    let mut body = Vec::new();
    response
        .take(MAX_COMPANION_RESPONSE_BYTES + 1)
        .read_to_end(&mut body)
        .ok()?;
    if body.len() as u64 > MAX_COMPANION_RESPONSE_BYTES {
        return None;
    }
    serde_json::from_slice(&body).ok()
}

fn filtered_version(value: Option<&Value>) -> Option<String> {
    let value = value?.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.chars().take(80).collect())
}

fn unavailable_companion(status: Option<u16>) -> CompanionProbeStatus {
    CompanionProbeStatus {
        available: false,
        status,
        version: None,
        endpoint_version: None,
    }
}

fn disabled_status() -> ZoteroStatus {
    ZoteroStatus {
        status: "disabled".to_string(),
        error_code: None,
        connector: ProbeStatus {
            available: false,
            status: None,
        },
        companion: unavailable_companion(None),
        fallback_import_files: import_file_fallback(),
    }
}

fn import_file_fallback() -> ImportFileFallback {
    ImportFileFallback {
        available: true,
        formats: [
            "references.json",
            "references.ris",
            "bibliography.bib",
            "zotero-import-report.md",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect(),
    }
}

fn read_env_boolean(name: &str, fallback: bool) -> bool {
    let Ok(value) = std::env::var(name) else {
        return fallback;
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => fallback,
    }
}
