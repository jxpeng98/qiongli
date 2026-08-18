use std::io::Read;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use url::{Host, Url};

use super::export::ALL_FORMAT_NAMES;

pub const DEFAULT_CONNECTOR_URL: &str = "http://127.0.0.1:23119";
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
pub const SUPPORTED_COMPANION_ENDPOINT_VERSION: &str = "2";

const MAX_PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_PROBE_TIMEOUT: Duration = Duration::from_millis(1);
const MAX_CONNECTOR_URL_BYTES: usize = 2 * 1024;
const MAX_COMPANION_RESPONSE_BYTES: u64 = 32 * 1024;
const MAX_COMPANION_OPERATION_BYTES: u64 = 1024 * 1024;
const MAX_VERSION_CHARS: usize = 80;
const MAX_SEARCH_RESULTS: usize = 200;
const MAX_UPSERT_RESULTS: usize = 100;

#[derive(Debug, Error)]
pub enum CompanionError {
    #[error("Zotero connector URL exceeds the byte limit")]
    UrlTooLong,
    #[error("Zotero connector URL must use http or https")]
    InvalidScheme,
    #[error("Zotero connector URL must point to a loopback host")]
    NonLoopback,
    #[error("Zotero connector URL must not contain credentials")]
    CredentialsNotAllowed,
    #[error("invalid Zotero connector URL")]
    InvalidUrl(#[source] url::ParseError),
    #[error("Zotero probe timeout is outside the supported range")]
    InvalidTimeout,
    #[error("failed to construct the Zotero loopback client")]
    ClientBuild(#[source] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum CompanionOperationError {
    #[error("Zotero Desktop is unavailable; use the import-file fallback")]
    ZoteroUnavailable,
    #[error("Qiongli Zotero Companion is unavailable; use the import-file fallback")]
    CompanionUnavailable,
    #[error("Qiongli Zotero Companion endpoint contract is incompatible")]
    CompanionIncompatible,
    #[error("Zotero Companion request exceeds the byte limit")]
    RequestTooLarge,
    #[error("Zotero Companion request could not be serialized")]
    RequestInvalid,
    #[error("Zotero Companion request failed")]
    RequestFailed(#[source] reqwest::Error),
    #[error("Zotero Companion returned HTTP {0}")]
    HttpStatus(StatusCode),
    #[error("Zotero Companion response is invalid or exceeds the byte limit")]
    InvalidResponse,
}

impl CompanionOperationError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::ZoteroUnavailable => "zotero-not-running",
            Self::CompanionUnavailable => "zotero-companion-unavailable",
            Self::CompanionIncompatible => "zotero-companion-incompatible",
            Self::RequestTooLarge => "zotero-request-too-large",
            Self::RequestInvalid => "zotero-request-invalid",
            Self::RequestFailed(_) => "zotero-request-failed",
            Self::HttpStatus(_) => "zotero-companion-http-error",
            Self::InvalidResponse => "zotero-companion-response-invalid",
        }
    }

    #[must_use]
    pub const fn public_message(&self) -> &'static str {
        match self {
            Self::ZoteroUnavailable => {
                "Zotero Desktop is unavailable; use the import-file fallback"
            }
            Self::CompanionUnavailable => {
                "Qiongli Zotero Companion is unavailable; use the import-file fallback"
            }
            Self::CompanionIncompatible => {
                "Qiongli Zotero Companion endpoint contract is incompatible"
            }
            Self::RequestTooLarge => "Zotero Companion request exceeds the byte limit",
            Self::RequestInvalid => "Zotero Companion request is invalid",
            Self::RequestFailed(_) | Self::HttpStatus(_) => "Zotero Companion request failed",
            Self::InvalidResponse => "Zotero Companion response is invalid",
        }
    }
}

#[derive(Clone)]
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ZoteroIntegrationState {
    Disabled,
    ZoteroNotRunning,
    CompanionMissing,
    CompanionIncompatible,
    Ready,
}

impl ZoteroStatus {
    #[must_use]
    pub fn disabled() -> Self {
        disabled_status()
    }

    #[must_use]
    pub fn integration_state(&self) -> ZoteroIntegrationState {
        if self.status == "disabled" {
            ZoteroIntegrationState::Disabled
        } else if !self.connector.available {
            ZoteroIntegrationState::ZoteroNotRunning
        } else if !self.companion.available {
            ZoteroIntegrationState::CompanionMissing
        } else if self.companion.endpoint_version.as_deref()
            != Some(SUPPORTED_COMPANION_ENDPOINT_VERSION)
        {
            ZoteroIntegrationState::CompanionIncompatible
        } else {
            ZoteroIntegrationState::Ready
        }
    }
}

impl CompanionClient {
    pub fn new(raw: &str) -> Result<Self, CompanionError> {
        Self::with_timeout(raw, DEFAULT_PROBE_TIMEOUT)
    }

    pub fn with_timeout(raw: &str, timeout: Duration) -> Result<Self, CompanionError> {
        if !(MIN_PROBE_TIMEOUT..=MAX_PROBE_TIMEOUT).contains(&timeout) {
            return Err(CompanionError::InvalidTimeout);
        }
        let base_url = validate_connector_url(raw)?;
        let client = Client::builder()
            .connect_timeout(timeout)
            .timeout(timeout)
            .redirect(Policy::none())
            .no_proxy()
            .user_agent(concat!("qiongli-runtime/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(CompanionError::ClientBuild)?;
        Ok(Self { base_url, client })
    }

    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    #[must_use]
    pub fn probe(&self, local_enabled: bool) -> ZoteroStatus {
        if !local_enabled {
            return disabled_status();
        }

        let connector = self.probe_connector();
        if !connector.available {
            return ZoteroStatus {
                status: "fallback_only".to_owned(),
                error_code: Some("zotero_not_running".to_owned()),
                connector,
                companion: unavailable_companion(None),
                fallback_import_files: import_file_fallback(),
            };
        }

        let companion = self.probe_companion();
        if !companion.available {
            return ZoteroStatus {
                status: "companion_missing".to_owned(),
                error_code: Some("companion_missing".to_owned()),
                connector,
                companion,
                fallback_import_files: import_file_fallback(),
            };
        }

        let compatible =
            companion.endpoint_version.as_deref() == Some(SUPPORTED_COMPANION_ENDPOINT_VERSION);
        ZoteroStatus {
            status: if compatible {
                "ok"
            } else {
                "companion_incompatible"
            }
            .to_owned(),
            error_code: (!compatible).then(|| "companion_endpoint_incompatible".to_owned()),
            connector,
            companion,
            fallback_import_files: import_file_fallback(),
        }
    }

    pub fn search(&self, query: &Value) -> Result<Value, CompanionOperationError> {
        let payload = self.post_operation("qiongli/search", query, MAX_SEARCH_RESULTS)?;
        if payload
            .get("limit")
            .and_then(Value::as_u64)
            .is_none_or(|limit| !(1..=MAX_SEARCH_RESULTS as u64).contains(&limit))
        {
            return Err(CompanionOperationError::InvalidResponse);
        }
        Ok(payload)
    }

    pub fn upsert(&self, request: &Value) -> Result<Value, CompanionOperationError> {
        let payload = self.post_operation("qiongli/upsertItems", request, MAX_UPSERT_RESULTS)?;
        if payload.get("dry_run").and_then(Value::as_bool).is_none() {
            return Err(CompanionOperationError::InvalidResponse);
        }
        Ok(payload)
    }

    fn post_operation(
        &self,
        path: &str,
        payload: &Value,
        max_results: usize,
    ) -> Result<Value, CompanionOperationError> {
        let body =
            serde_json::to_vec(payload).map_err(|_| CompanionOperationError::RequestInvalid)?;
        if body.len() as u64 > MAX_COMPANION_OPERATION_BYTES {
            return Err(CompanionOperationError::RequestTooLarge);
        }
        match self.probe(true).integration_state() {
            ZoteroIntegrationState::ZoteroNotRunning => {
                return Err(CompanionOperationError::ZoteroUnavailable);
            }
            ZoteroIntegrationState::CompanionMissing => {
                return Err(CompanionOperationError::CompanionUnavailable);
            }
            ZoteroIntegrationState::CompanionIncompatible => {
                return Err(CompanionOperationError::CompanionIncompatible);
            }
            ZoteroIntegrationState::Ready => {}
            ZoteroIntegrationState::Disabled => unreachable!("operation probe is always enabled"),
        }

        let endpoint = self
            .base_url
            .join(path)
            .expect("static companion endpoint must be valid");
        let response = self
            .client
            .post(endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .map_err(CompanionOperationError::RequestFailed)?;
        let status = response.status();
        if !status.is_success() {
            return Err(CompanionOperationError::HttpStatus(status));
        }
        let payload = read_limited_json(response, MAX_COMPANION_OPERATION_BYTES)
            .ok_or(CompanionOperationError::InvalidResponse)?;
        let Some(object) = payload.as_object() else {
            return Err(CompanionOperationError::InvalidResponse);
        };
        if object.get("status").and_then(Value::as_str).is_none()
            || object
                .get("results")
                .and_then(Value::as_array)
                .is_none_or(|results| results.len() > max_results)
        {
            return Err(CompanionOperationError::InvalidResponse);
        }
        Ok(payload)
    }

    fn probe_connector(&self) -> ProbeStatus {
        let endpoint = self
            .base_url
            .join("connector/ping")
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
            .join("qiongli/ping")
            .expect("static companion endpoint must be valid");
        let response = match self.client.get(endpoint).send() {
            Ok(response) => response,
            Err(_) => return unavailable_companion(None),
        };
        let status = response.status().as_u16();
        if !response.status().is_success() {
            return unavailable_companion(Some(status));
        }
        let Some(payload) = read_limited_json(response, MAX_COMPANION_RESPONSE_BYTES) else {
            return unavailable_companion(Some(status));
        };
        if !payload.is_object() {
            return unavailable_companion(Some(status));
        }

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

fn validate_connector_url(raw: &str) -> Result<Url, CompanionError> {
    if raw.len() > MAX_CONNECTOR_URL_BYTES {
        return Err(CompanionError::UrlTooLong);
    }
    let mut base_url = Url::parse(raw).map_err(CompanionError::InvalidUrl)?;
    if base_url.scheme() != "http" && base_url.scheme() != "https" {
        return Err(CompanionError::InvalidScheme);
    }
    if base_url.cannot_be_a_base() {
        return Err(CompanionError::InvalidUrl(
            url::ParseError::RelativeUrlWithoutBase,
        ));
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

fn read_limited_json(response: Response, limit: u64) -> Option<Value> {
    if response
        .content_length()
        .is_some_and(|length| length > limit)
    {
        return None;
    }
    let mut body = Vec::new();
    response.take(limit + 1).read_to_end(&mut body).ok()?;
    if body.len() as u64 > limit {
        return None;
    }
    serde_json::from_slice(&body).ok()
}

fn filtered_version(value: Option<&Value>) -> Option<String> {
    let value = value?.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    let filtered = value
        .chars()
        .take(MAX_VERSION_CHARS)
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    let filtered = filtered.trim();
    (!filtered.is_empty()).then(|| filtered.to_owned())
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
        status: "disabled".to_owned(),
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
        formats: ALL_FORMAT_NAMES.map(str::to_owned).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Instant;

    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_only_bounded_loopback_urls_without_credentials() {
        let normalized = CompanionClient::new("http://127.0.0.1:23119/path?ignored=true").unwrap();
        assert_eq!(normalized.base_url().as_str(), "http://127.0.0.1:23119/");
        assert!(CompanionClient::new("https://localhost:23119").is_ok());
        assert!(CompanionClient::new("http://[::1]:23119").is_ok());
        assert!(matches!(
            CompanionClient::new("http://example.com:23119"),
            Err(CompanionError::NonLoopback)
        ));
        assert!(matches!(
            CompanionClient::new("ftp://127.0.0.1:23119"),
            Err(CompanionError::InvalidScheme)
        ));
        assert!(matches!(
            CompanionClient::new("http://user:password@127.0.0.1:23119"),
            Err(CompanionError::CredentialsNotAllowed)
        ));
        assert!(matches!(
            CompanionClient::new(&format!(
                "http://127.0.0.1:23119/{}",
                "x".repeat(MAX_CONNECTOR_URL_BYTES)
            )),
            Err(CompanionError::UrlTooLong)
        ));
        assert!(matches!(
            CompanionClient::with_timeout("http://127.0.0.1:23119", Duration::ZERO),
            Err(CompanionError::InvalidTimeout)
        ));
    }

    #[test]
    fn disabled_status_performs_no_probe_and_keeps_fallback() {
        let client = CompanionClient::new("http://127.0.0.1:9").unwrap();
        let status = client.probe(false);
        assert_eq!(status.status, "disabled");
        assert_eq!(status.integration_state(), ZoteroIntegrationState::Disabled);
        assert!(!status.connector.available);
        assert!(!status.companion.available);
        assert_eq!(
            status.fallback_import_files.formats,
            ALL_FORMAT_NAMES.map(str::to_owned)
        );
    }

    #[test]
    fn reports_success_without_returning_response_content() {
        let server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(
                r#"{"version":"1.2.3\nfiltered","endpoint_version":"2","library":"private"}"#,
            ),
        ]);
        let client = CompanionClient::new(&server.base_url).unwrap();
        let status = client.probe(true);
        assert_eq!(status.status, "ok");
        assert_eq!(status.integration_state(), ZoteroIntegrationState::Ready);
        assert_eq!(status.companion.version.as_deref(), Some("1.2.3 filtered"));
        let rendered = serde_json::to_string(&status).unwrap();
        assert!(!rendered.contains("private"));
        assert_eq!(server.finish(), vec!["/connector/ping", "/qiongli/ping"]);
    }

    #[test]
    fn classifies_missing_and_incompatible_companion_observations() {
        let missing = ZoteroStatus {
            status: "companion_missing".to_owned(),
            error_code: Some("companion_missing".to_owned()),
            connector: ProbeStatus {
                available: true,
                status: Some(200),
            },
            companion: unavailable_companion(Some(404)),
            fallback_import_files: import_file_fallback(),
        };
        assert_eq!(
            missing.integration_state(),
            ZoteroIntegrationState::CompanionMissing
        );

        let incompatible = ZoteroStatus {
            status: "ok".to_owned(),
            error_code: None,
            connector: ProbeStatus {
                available: true,
                status: Some(200),
            },
            companion: CompanionProbeStatus {
                available: true,
                status: Some(200),
                version: Some("0.3.0".to_owned()),
                endpoint_version: Some("1".to_owned()),
            },
            fallback_import_files: import_file_fallback(),
        };
        assert_eq!(
            incompatible.integration_state(),
            ZoteroIntegrationState::CompanionIncompatible
        );
    }

    #[test]
    fn reports_and_rejects_an_incompatible_endpoint_contract() {
        let server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"1"}"#),
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"1"}"#),
        ]);
        let client = CompanionClient::new(&server.base_url).unwrap();
        let status = client.probe(true);
        assert_eq!(status.status, "companion_incompatible");
        assert_eq!(
            status.error_code.as_deref(),
            Some("companion_endpoint_incompatible")
        );
        assert_eq!(
            client
                .search(&json!({"title": "Paper"}))
                .unwrap_err()
                .reason_code(),
            "zotero-companion-incompatible"
        );
        assert_eq!(
            server.finish(),
            vec![
                "/connector/ping",
                "/qiongli/ping",
                "/connector/ping",
                "/qiongli/ping"
            ]
        );
    }

    #[test]
    fn relays_bounded_search_upsert_and_approval_rejections() {
        let search_server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json(
                r#"{"status":"ok","limit":1,"results":[{"item_key":"A","title":"Paper"}]}"#,
            ),
        ]);
        let client = CompanionClient::new(&search_server.base_url).unwrap();
        let search = client
            .search(&json!({"title": "Paper", "limit": 1}))
            .unwrap();
        assert_eq!(search["results"][0]["item_key"], "A");
        assert_eq!(
            search_server.finish(),
            vec!["/connector/ping", "/qiongli/ping", "/qiongli/search"]
        );

        let upsert_server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json(
                r#"{"status":"ok","dry_run":true,"results":[{"status":"created"}],"write_approval":{"receipt":"zwr1_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","expires_in_seconds":300,"required_write_intent":"apply"}}"#,
            ),
        ]);
        let client = CompanionClient::new(&upsert_server.base_url).unwrap();
        let preview = client
            .upsert(&json!({"dry_run": true, "items": [{"title": "Paper"}]}))
            .unwrap();
        assert_eq!(preview["dry_run"], true);
        assert_eq!(preview["write_approval"]["required_write_intent"], "apply");
        assert_eq!(
            upsert_server.finish(),
            vec!["/connector/ping", "/qiongli/ping", "/qiongli/upsertItems"]
        );

        let rejection_server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json(
                r#"{"status":"approval_required","error_code":"zotero_dry_run_plan_changed","dry_run":true,"results":[]}"#,
            ),
        ]);
        let client = CompanionClient::new(&rejection_server.base_url).unwrap();
        let rejected = client
            .upsert(&json!({
                "dry_run": false,
                "write_intent": "apply",
                "dry_run_receipt": "zwr1_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "items": [{"title": "Changed"}]
            }))
            .unwrap();
        assert_eq!(rejected["status"], "approval_required");
        assert_eq!(rejected["error_code"], "zotero_dry_run_plan_changed");
        assert_eq!(
            rejection_server.finish(),
            vec!["/connector/ping", "/qiongli/ping", "/qiongli/upsertItems"]
        );
    }

    #[test]
    fn rejects_oversized_requests_and_malformed_or_slow_responses() {
        let client = CompanionClient::new("http://127.0.0.1:9").unwrap();
        let oversized = json!({"title": "x".repeat(MAX_COMPANION_OPERATION_BYTES as usize)});
        assert_eq!(
            client.search(&oversized).unwrap_err().reason_code(),
            "zotero-request-too-large"
        );

        let malformed_server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json("[]"),
        ]);
        let client = CompanionClient::new(&malformed_server.base_url).unwrap();
        assert_eq!(
            client
                .search(&json!({"title": "Paper"}))
                .unwrap_err()
                .reason_code(),
            "zotero-companion-response-invalid"
        );
        assert_eq!(
            malformed_server.finish(),
            vec!["/connector/ping", "/qiongli/ping", "/qiongli/search"]
        );

        let missing_fields_server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json(r#"{"status":"ok","results":[]}"#),
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json(r#"{"status":"ok","results":[]}"#),
        ]);
        let client = CompanionClient::new(&missing_fields_server.base_url).unwrap();
        assert!(matches!(
            client.search(&json!({"title": "Paper"})),
            Err(CompanionOperationError::InvalidResponse)
        ));
        assert!(matches!(
            client.upsert(&json!({"dry_run": true, "items": [{"title": "Paper"}]})),
            Err(CompanionOperationError::InvalidResponse)
        ));
        assert_eq!(
            missing_fields_server.finish(),
            vec![
                "/connector/ping",
                "/qiongli/ping",
                "/qiongli/search",
                "/connector/ping",
                "/qiongli/ping",
                "/qiongli/upsertItems"
            ]
        );

        let slow_server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(r#"{"version":"0.3.0","endpoint_version":"2"}"#),
            ResponseFixture::json(r#"{"status":"ok","results":[]}"#)
                .with_delay(Duration::from_secs(2)),
        ]);
        let client =
            CompanionClient::with_timeout(&slow_server.base_url, Duration::from_secs(1)).unwrap();
        let started = Instant::now();
        assert_eq!(
            client
                .search(&json!({"title": "Paper"}))
                .unwrap_err()
                .reason_code(),
            "zotero-request-failed"
        );
        assert!(started.elapsed() < Duration::from_secs(3));
        assert_eq!(
            slow_server.finish(),
            vec!["/connector/ping", "/qiongli/ping", "/qiongli/search"]
        );
    }

    #[test]
    fn refuses_redirects_and_oversized_companion_responses() {
        let redirect =
            FixtureServer::start(vec![ResponseFixture::redirect("http://example.com/steal")]);
        let client = CompanionClient::new(&redirect.base_url).unwrap();
        let status = client.probe(true);
        assert_eq!(status.status, "fallback_only");
        assert_eq!(status.connector.status, Some(302));
        assert_eq!(redirect.finish(), vec!["/connector/ping"]);

        let oversized = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json(&format!(
                r#"{{"version":"{}"}}"#,
                "x".repeat(MAX_COMPANION_RESPONSE_BYTES as usize)
            )),
        ]);
        let client = CompanionClient::new(&oversized.base_url).unwrap();
        let status = client.probe(true);
        assert_eq!(status.status, "companion_missing");
        assert_eq!(status.companion.status, Some(200));
        assert_eq!(oversized.finish(), vec!["/connector/ping", "/qiongli/ping"]);

        let non_object = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running"),
            ResponseFixture::json("[]"),
        ]);
        let client = CompanionClient::new(&non_object.base_url).unwrap();
        let status = client.probe(true);
        assert_eq!(status.status, "companion_missing");
        assert_eq!(status.companion.status, Some(200));
        assert_eq!(
            non_object.finish(),
            vec!["/connector/ping", "/qiongli/ping"]
        );
    }

    #[test]
    fn bounds_connector_timeout_and_retains_fallback() {
        let server = FixtureServer::start(vec![
            ResponseFixture::ok("Zotero is running").with_delay(Duration::from_millis(200)),
        ]);
        let client =
            CompanionClient::with_timeout(&server.base_url, Duration::from_millis(50)).unwrap();
        let started = Instant::now();
        let status = client.probe(true);
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(status.status, "fallback_only");
        assert!(status.fallback_import_files.available);
        assert_eq!(server.finish(), vec!["/connector/ping"]);
    }

    struct FixtureServer {
        base_url: String,
        paths: Arc<Mutex<Vec<String>>>,
        worker: thread::JoinHandle<()>,
    }

    impl FixtureServer {
        fn start(responses: Vec<ResponseFixture>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let paths = Arc::new(Mutex::new(Vec::new()));
            let worker_paths = Arc::clone(&paths);
            let worker = thread::spawn(move || {
                for response in responses {
                    let deadline = Instant::now() + Duration::from_secs(2);
                    let (mut stream, _) = loop {
                        match listener.accept() {
                            Ok(connection) => break connection,
                            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                                assert!(Instant::now() < deadline, "timed out waiting for probe");
                                thread::sleep(Duration::from_millis(5));
                            }
                            Err(error) => panic!("fixture accept failed: {error}"),
                        }
                    };
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut request = Vec::new();
                    let mut chunk = [0_u8; 1024];
                    loop {
                        let count = stream.read(&mut chunk).unwrap();
                        if count == 0 {
                            break;
                        }
                        request.extend_from_slice(&chunk[..count]);
                        if request.windows(4).any(|window| window == b"\r\n\r\n") {
                            break;
                        }
                    }
                    let request = String::from_utf8(request).unwrap();
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap()
                        .to_owned();
                    worker_paths.lock().unwrap().push(path);
                    thread::sleep(response.delay);
                    let _ = stream.write_all(response.render().as_bytes());
                    let _ = stream.flush();
                }
            });
            Self {
                base_url: format!("http://{address}"),
                paths,
                worker,
            }
        }

        fn finish(self) -> Vec<String> {
            self.worker.join().unwrap();
            Arc::try_unwrap(self.paths).unwrap().into_inner().unwrap()
        }
    }

    struct ResponseFixture {
        status: u16,
        reason: &'static str,
        content_type: &'static str,
        body: String,
        location: Option<String>,
        delay: Duration,
    }

    impl ResponseFixture {
        fn ok(body: &str) -> Self {
            Self {
                status: 200,
                reason: "OK",
                content_type: "text/plain",
                body: body.to_owned(),
                location: None,
                delay: Duration::ZERO,
            }
        }

        fn json(body: &str) -> Self {
            Self {
                status: 200,
                reason: "OK",
                content_type: "application/json",
                body: body.to_owned(),
                location: None,
                delay: Duration::ZERO,
            }
        }

        fn redirect(location: &str) -> Self {
            Self {
                status: 302,
                reason: "Found",
                content_type: "text/plain",
                body: String::new(),
                location: Some(location.to_owned()),
                delay: Duration::ZERO,
            }
        }

        const fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = delay;
            self
        }

        fn render(&self) -> String {
            let location = self
                .location
                .as_ref()
                .map(|value| format!("Location: {value}\r\n"))
                .unwrap_or_default();
            format!(
                "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                self.status,
                self.reason,
                self.content_type,
                location,
                self.body.len(),
                self.body
            )
        }
    }
}
