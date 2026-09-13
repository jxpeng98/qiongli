use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::redirect::Policy;
use thiserror::Error;
use url::Url;

use super::access::ProviderAccess;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_PROVIDER_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

const OPENALEX_ENDPOINT: &str = "https://api.openalex.org/";
const SEMANTIC_SCHOLAR_ENDPOINT: &str = "https://api.semanticscholar.org/graph/v1/";
const CROSSREF_ENDPOINT: &str = "https://api.crossref.org/";
const PUBMED_ENDPOINT: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/";
const ARXIV_ENDPOINT: &str = "https://export.arxiv.org/";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProviderRuntimeError {
    #[error("provider request timed out")]
    Timeout,
    #[error("provider returned HTTP status {status}")]
    HttpStatus { status: u16 },
    #[error("provider request failed")]
    Transport,
    #[error("provider response could not be decoded")]
    Decode,
    #[error("provider endpoint is invalid")]
    InvalidEndpoint,
    #[error("provider request was cancelled")]
    Cancelled,
}

impl ProviderRuntimeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::HttpStatus { .. } => "http_error",
            Self::Transport => "transport_error",
            Self::Decode => "decode_error",
            Self::InvalidEndpoint => "invalid_endpoint",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn http_status(&self) -> Option<u16> {
        match self {
            Self::HttpStatus { status } => Some(*status),
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
pub struct ProviderEndpoints {
    openalex: Url,
    semantic_scholar: Url,
    crossref: Url,
    pubmed: Url,
    arxiv: Url,
}

impl ProviderEndpoints {
    fn production() -> Result<Self, ProviderRuntimeError> {
        Self::from_urls(
            OPENALEX_ENDPOINT,
            SEMANTIC_SCHOLAR_ENDPOINT,
            CROSSREF_ENDPOINT,
            PUBMED_ENDPOINT,
            ARXIV_ENDPOINT,
        )
    }

    /// Inject provider endpoint roots for deterministic loopback tests. The
    /// production MCP runtime never reads endpoint overrides from arguments,
    /// provider config, or process environment.
    #[doc(hidden)]
    pub fn from_urls(
        openalex: &str,
        semantic_scholar: &str,
        crossref: &str,
        pubmed: &str,
        arxiv: &str,
    ) -> Result<Self, ProviderRuntimeError> {
        Ok(Self {
            openalex: parse_endpoint(openalex)?,
            semantic_scholar: parse_endpoint(semantic_scholar)?,
            crossref: parse_endpoint(crossref)?,
            pubmed: parse_endpoint(pubmed)?,
            arxiv: parse_endpoint(arxiv)?,
        })
    }

    pub(crate) fn openalex(&self) -> &Url {
        &self.openalex
    }

    pub(crate) fn semantic_scholar(&self) -> &Url {
        &self.semantic_scholar
    }

    pub(crate) fn crossref(&self) -> &Url {
        &self.crossref
    }

    pub(crate) fn pubmed(&self) -> &Url {
        &self.pubmed
    }

    pub(crate) fn arxiv(&self) -> &Url {
        &self.arxiv
    }
}

#[derive(Clone)]
pub struct ProviderRuntime {
    client: Client,
    endpoints: ProviderEndpoints,
    access: ProviderAccess,
    cancellation: CancellationToken,
}

impl ProviderRuntime {
    pub fn production(access: ProviderAccess) -> Result<ProviderRuntime, ProviderRuntimeError> {
        let client = bounded_client()?;
        let endpoints = ProviderEndpoints::production()?;
        Ok(Self {
            client,
            endpoints,
            access,
            cancellation: CancellationToken::new(),
        })
    }

    pub fn production_with_cancellation(
        access: ProviderAccess,
        cancellation: CancellationToken,
    ) -> Result<ProviderRuntime, ProviderRuntimeError> {
        let mut runtime = Self::production(access)?;
        runtime.cancellation = cancellation;
        Ok(runtime)
    }

    /// Inject a bounded client and endpoint set for integration tests. This is
    /// intentionally not wired to any MCP input or environment variable.
    #[doc(hidden)]
    pub fn with_client(
        client: Client,
        endpoints: ProviderEndpoints,
        access: ProviderAccess,
    ) -> Self {
        Self {
            client,
            endpoints,
            access,
            cancellation: CancellationToken::new(),
        }
    }

    /// Inject every network dependency plus cancellation for deterministic
    /// integration tests. This is never wired to peer-controlled input.
    #[doc(hidden)]
    pub fn with_client_and_cancellation(
        client: Client,
        endpoints: ProviderEndpoints,
        access: ProviderAccess,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            client,
            endpoints,
            access,
            cancellation,
        }
    }

    /// Build a production-equivalent bounded client with injected endpoints.
    #[doc(hidden)]
    pub fn with_endpoints(
        endpoints: ProviderEndpoints,
        access: ProviderAccess,
    ) -> Result<Self, ProviderRuntimeError> {
        Ok(Self::with_client(bounded_client()?, endpoints, access))
    }

    #[must_use]
    pub fn access(&self) -> &ProviderAccess {
        &self.access
    }

    #[must_use]
    pub fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    pub fn check_cancelled(&self) -> Result<(), ProviderRuntimeError> {
        if self.cancellation.is_cancelled() {
            Err(ProviderRuntimeError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn endpoints(&self) -> &ProviderEndpoints {
        &self.endpoints
    }

    pub(crate) fn get_text(&self, request: RequestBuilder) -> Result<String, ProviderRuntimeError> {
        self.check_cancelled()?;
        let response = request.send().map_err(map_reqwest_error)?;
        self.check_cancelled()?;
        let status = response.status();
        if !status.is_success() {
            return Err(ProviderRuntimeError::HttpStatus {
                status: status.as_u16(),
            });
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_PROVIDER_RESPONSE_BYTES)
        {
            return Err(ProviderRuntimeError::Decode);
        }
        let mut body = Vec::new();
        response
            .take(MAX_PROVIDER_RESPONSE_BYTES + 1)
            .read_to_end(&mut body)
            .map_err(|_| ProviderRuntimeError::Transport)?;
        self.check_cancelled()?;
        if body.len() as u64 > MAX_PROVIDER_RESPONSE_BYTES {
            return Err(ProviderRuntimeError::Decode);
        }
        String::from_utf8(body).map_err(|_| ProviderRuntimeError::Decode)
    }
}

fn bounded_client() -> Result<Client, ProviderRuntimeError> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::none())
        .no_proxy()
        .user_agent(concat!("qiongli-runtime/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| ProviderRuntimeError::Transport)
}

fn parse_endpoint(raw: &str) -> Result<Url, ProviderRuntimeError> {
    let mut endpoint = Url::parse(raw).map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    if !matches!(endpoint.scheme(), "http" | "https") || endpoint.cannot_be_a_base() {
        return Err(ProviderRuntimeError::InvalidEndpoint);
    }
    if !endpoint.username().is_empty() || endpoint.password().is_some() {
        return Err(ProviderRuntimeError::InvalidEndpoint);
    }
    if !endpoint.path().ends_with('/') {
        endpoint.set_path(&format!("{}/", endpoint.path()));
    }
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    Ok(endpoint)
}

fn map_reqwest_error(error: reqwest::Error) -> ProviderRuntimeError {
    if error.is_timeout() {
        ProviderRuntimeError::Timeout
    } else if let Some(status) = error.status() {
        ProviderRuntimeError::HttpStatus {
            status: status.as_u16(),
        }
    } else {
        ProviderRuntimeError::Transport
    }
}
