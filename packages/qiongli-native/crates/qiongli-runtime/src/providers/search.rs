use std::collections::{BTreeMap, HashMap};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use super::ProviderId;
use super::arxiv::search_arxiv;
use super::crossref::search_crossref;
use super::openalex::search_openalex;
use super::pubmed::search_pubmed;
use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::semantic_scholar::search_semantic_scholar;

pub const PROVIDER_ORDER: [&str; 5] = [
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
];

const GENERAL_DEFAULT_LIMIT: usize = 25;
const REVIEW_DEFAULT_LIMIT: usize = 50;
const MAX_PER_PROVIDER_LIMIT: usize = 200;
const MAX_TOTAL_LIMIT: usize = 1_000;
const MAX_QUERY_BYTES: usize = 4_096;
const SEARCH_ARGUMENTS: [&str; 6] = [
    "query",
    "search_mode",
    "providers",
    "limit",
    "per_provider_limit",
    "total_limit",
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
pub enum ProviderError {
    #[error("provider JSON response could not be decoded")]
    Json,
    #[error("provider XML response could not be decoded")]
    Xml,
}

impl From<serde_json::Error> for ProviderError {
    fn from(_error: serde_json::Error) -> Self {
        Self::Json
    }
}

impl From<quick_xml::Error> for ProviderError {
    fn from(_error: quick_xml::Error) -> Self {
        Self::Xml
    }
}

#[derive(Debug, Clone)]
pub struct SearchInput {
    pub query: String,
    pub search_mode: Option<String>,
    pub limit: Option<usize>,
    pub per_provider_limit: Option<usize>,
    pub total_limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SearchMode {
    Auto,
    Topic,
    Review,
    SystematicReview,
}

impl SearchMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Topic => "topic",
            Self::Review => "review",
            Self::SystematicReview => "systematic_review",
        }
    }

    pub fn parse(value: Option<&str>) -> Result<Self, SearchRequestError> {
        match value.unwrap_or("auto") {
            "auto" => Ok(Self::Auto),
            "topic" => Ok(Self::Topic),
            "review" => Ok(Self::Review),
            "systematic_review" => Ok(Self::SystematicReview),
            _ => Err(SearchRequestError::UnsupportedMode),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
pub enum SearchRequestError {
    #[error("search query is empty")]
    EmptyQuery,
    #[error("search query exceeds the byte limit")]
    QueryTooLarge,
    #[error("search mode is unsupported")]
    UnsupportedMode,
    #[error("search provider is unsupported")]
    UnsupportedProvider,
    #[error("search provider selection is empty")]
    EmptyProviders,
    #[error("search provider selection contains duplicates")]
    DuplicateProvider,
    #[error("per-provider limit is outside the supported range")]
    InvalidPerProviderLimit,
    #[error("total limit is outside the supported range")]
    InvalidTotalLimit,
}

#[derive(Clone)]
pub struct SearchRequest {
    query: String,
    mode: SearchMode,
    providers: Option<Vec<ProviderId>>,
    per_provider_limit: usize,
    total_limit: usize,
}

impl SearchRequest {
    pub fn from_arguments(arguments: &Value) -> Result<Self, SearchArgumentsError> {
        let entries = arguments.as_object().ok_or(SearchArgumentsError::new(
            "search arguments must be an object",
        ))?;
        if entries
            .keys()
            .any(|key| !SEARCH_ARGUMENTS.contains(&key.as_str()))
        {
            return Err(SearchArgumentsError::new("Unsupported argument"));
        }
        let query = entries
            .get("query")
            .and_then(Value::as_str)
            .ok_or(SearchArgumentsError::new("Missing query"))?;
        if query.trim().is_empty() {
            return Err(SearchArgumentsError::new("query must not be empty"));
        }
        let mode = entries
            .get("search_mode")
            .map(|value| {
                value
                    .as_str()
                    .ok_or(SearchArgumentsError::new("search_mode must be a string"))
            })
            .transpose()?;
        let providers = parse_provider_arguments(entries.get("providers"))?;
        let limit = parse_argument_limit(entries.get("limit"), "limit", 200)?;
        let per_provider_limit = parse_argument_limit(
            entries.get("per_provider_limit"),
            "per_provider_limit",
            MAX_PER_PROVIDER_LIMIT,
        )?;
        let total_limit =
            parse_argument_limit(entries.get("total_limit"), "total_limit", MAX_TOTAL_LIMIT)?;

        Self::from_raw(
            query,
            mode,
            providers.as_deref(),
            limit,
            per_provider_limit,
            total_limit,
        )
        .map_err(SearchArgumentsError::from_request_error)
    }

    pub fn from_raw(
        query: &str,
        mode: Option<&str>,
        providers: Option<&[String]>,
        limit: Option<usize>,
        per_provider_limit: Option<usize>,
        total_limit: Option<usize>,
    ) -> Result<Self, SearchRequestError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(SearchRequestError::EmptyQuery);
        }
        if query.len() > MAX_QUERY_BYTES {
            return Err(SearchRequestError::QueryTooLarge);
        }
        let mode = SearchMode::parse(mode)?;
        let providers = providers
            .map(|values| {
                if values.is_empty() {
                    return Err(SearchRequestError::EmptyProviders);
                }
                let mut parsed = Vec::with_capacity(values.len());
                for value in values {
                    let provider = ProviderId::parse(value)
                        .map_err(|_| SearchRequestError::UnsupportedProvider)?;
                    if parsed.contains(&provider) {
                        return Err(SearchRequestError::DuplicateProvider);
                    }
                    parsed.push(provider);
                }
                Ok(parsed)
            })
            .transpose()?;
        let per_provider_limit = per_provider_limit
            .or(limit)
            .unwrap_or_else(|| default_limit_for_mode(mode));
        if !(1..=MAX_PER_PROVIDER_LIMIT).contains(&per_provider_limit) {
            return Err(SearchRequestError::InvalidPerProviderLimit);
        }
        let total_limit = total_limit.unwrap_or(per_provider_limit.saturating_mul(5).min(1_000));
        if !(1..=MAX_TOTAL_LIMIT).contains(&total_limit) {
            return Err(SearchRequestError::InvalidTotalLimit);
        }
        Ok(Self {
            query: query.to_owned(),
            mode,
            providers,
            per_provider_limit,
            total_limit,
        })
    }

    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    #[must_use]
    pub const fn mode(&self) -> SearchMode {
        self.mode
    }

    #[must_use]
    pub fn providers(&self) -> Option<&[ProviderId]> {
        self.providers.as_deref()
    }

    #[must_use]
    pub const fn per_provider_limit(&self) -> usize {
        self.per_provider_limit
    }

    #[must_use]
    pub const fn total_limit(&self) -> usize {
        self.total_limit
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchArgumentsError {
    message: &'static str,
}

impl SearchArgumentsError {
    const fn new(message: &'static str) -> Self {
        Self { message }
    }

    const fn from_request_error(error: SearchRequestError) -> Self {
        let message = match error {
            SearchRequestError::EmptyQuery => "query must not be empty",
            SearchRequestError::QueryTooLarge => "search query exceeds the byte limit",
            SearchRequestError::UnsupportedMode => "unsupported search_mode",
            SearchRequestError::UnsupportedProvider => "unsupported provider",
            SearchRequestError::EmptyProviders => "providers must not be empty",
            SearchRequestError::DuplicateProvider => "providers must contain unique values",
            SearchRequestError::InvalidPerProviderLimit => {
                "per_provider_limit must be between 1 and 200"
            }
            SearchRequestError::InvalidTotalLimit => "total_limit must be between 1 and 1000",
        };
        Self { message }
    }
}

impl std::fmt::Display for SearchArgumentsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for SearchArgumentsError {}

fn parse_provider_arguments(
    value: Option<&Value>,
) -> Result<Option<Vec<String>>, SearchArgumentsError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or(SearchArgumentsError::new("providers must be an array"))?;
    if values.is_empty() {
        return Err(SearchArgumentsError::new("providers must not be empty"));
    }
    let mut providers = Vec::with_capacity(values.len());
    for value in values {
        let provider = value
            .as_str()
            .ok_or(SearchArgumentsError::new("providers must contain strings"))?;
        if !PROVIDER_ORDER.contains(&provider) {
            return Err(SearchArgumentsError::new("unsupported provider"));
        }
        if providers.iter().any(|candidate| candidate == provider) {
            return Err(SearchArgumentsError::new(
                "providers must contain unique values",
            ));
        }
        providers.push(provider.to_string());
    }
    Ok(Some(providers))
}

fn parse_argument_limit(
    value: Option<&Value>,
    name: &'static str,
    maximum: usize,
) -> Result<Option<usize>, SearchArgumentsError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.as_u64().ok_or(match name {
        "limit" => SearchArgumentsError::new("limit must be an integer"),
        "per_provider_limit" => SearchArgumentsError::new("per_provider_limit must be an integer"),
        _ => SearchArgumentsError::new("total_limit must be an integer"),
    })?;
    if value == 0 || value > maximum as u64 {
        return Err(match name {
            "limit" => SearchArgumentsError::new("limit must be between 1 and 200"),
            "per_provider_limit" => {
                SearchArgumentsError::new("per_provider_limit must be between 1 and 200")
            }
            _ => SearchArgumentsError::new("total_limit must be between 1 and 1000"),
        });
    }
    Ok(Some(value as usize))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiteratureResult {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    pub provider: String,
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProviderDiagnostic {
    pub status: String,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchDiagnostics {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    pub providers: BTreeMap<String, ProviderDiagnostic>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchOutput {
    pub status: String,
    pub results: Vec<LiteratureResult>,
    pub diagnostics: SearchDiagnostics,
}

pub fn execute_search(
    runtime: &ProviderRuntime,
    input: &SearchInput,
    selected_providers: Option<&[String]>,
) -> SearchOutput {
    if runtime.check_cancelled().is_err() {
        return cancelled_output();
    }
    let attempted: Vec<&'static str> = PROVIDER_ORDER
        .into_iter()
        .filter(|provider| {
            ProviderId::parse(provider).is_ok_and(|provider| runtime.access().is_active(provider))
        })
        .filter(|provider| provider_selected(provider, selected_providers))
        .collect();

    if attempted.is_empty() {
        return SearchOutput {
            status: "warning".to_string(),
            results: Vec::new(),
            diagnostics: SearchDiagnostics {
                status: "not_run".to_string(),
                status_reason: Some("no_active_providers".to_string()),
                providers: BTreeMap::new(),
                warnings: vec![
                    "no active literature providers; no network search was performed".to_string(),
                ],
            },
        };
    }

    let executions = thread::scope(|scope| {
        let handles: Vec<_> = attempted
            .iter()
            .map(|provider| {
                let runtime = runtime.clone();
                let input = input.clone();
                (
                    *provider,
                    scope.spawn(move || run_provider(provider, &runtime, &input)),
                )
            })
            .collect();

        handles
            .into_iter()
            .map(|(provider, handle)| {
                let result = handle
                    .join()
                    .unwrap_or(Err(ProviderRuntimeError::Transport));
                (provider, result)
            })
            .collect::<Vec<_>>()
    });

    let mut diagnostics = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut records = Vec::new();
    let mut succeeded = 0_usize;

    for (provider, execution) in executions {
        match execution {
            Ok(mut provider_records) => {
                provider_records.truncate(limit_for(input));
                succeeded += 1;
                diagnostics.insert(
                    provider.to_string(),
                    ProviderDiagnostic {
                        status: "ok".to_string(),
                        count: provider_records.len(),
                        error_kind: None,
                        warning: None,
                    },
                );
                records.append(&mut provider_records);
            }
            Err(error) => {
                let error_kind = public_error_kind(&error).to_string();
                let warning = format!("{provider}: {error_kind}");
                warnings.push(warning.clone());
                diagnostics.insert(
                    provider.to_string(),
                    ProviderDiagnostic {
                        status: "error".to_string(),
                        count: 0,
                        error_kind: Some(error_kind),
                        warning: Some(warning),
                    },
                );
            }
        }
    }

    let attempted_count = attempted.len();
    let (mut status, diagnostic_status) = if succeeded == attempted_count {
        ("ok", "complete")
    } else if succeeded > 0 {
        ("warning", "partial")
    } else {
        ("error", "failed")
    };
    let mut results = deduplicate_results(records);
    if let Some(total_limit) = input.total_limit {
        results.truncate(total_limit.max(1));
    }
    if status == "ok" && results.is_empty() {
        status = "warning";
        warnings.push("configured providers returned no results".to_string());
    }

    SearchOutput {
        status: status.to_string(),
        results,
        diagnostics: SearchDiagnostics {
            status: diagnostic_status.to_string(),
            status_reason: None,
            providers: diagnostics,
            warnings,
        },
    }
}

pub fn execute_bounded_search(
    runtime: &ProviderRuntime,
    request: &SearchRequest,
) -> Result<SearchOutput, ProviderRuntimeError> {
    runtime.check_cancelled()?;
    let selected = request.providers.as_ref().map(|providers| {
        providers
            .iter()
            .map(|provider| provider.as_str().to_owned())
            .collect::<Vec<_>>()
    });
    let input = SearchInput {
        query: request.query.clone(),
        search_mode: Some(request.mode.as_str().to_owned()),
        limit: None,
        per_provider_limit: Some(request.per_provider_limit),
        total_limit: Some(request.total_limit),
    };
    let output = execute_search(runtime, &input, selected.as_deref());
    if runtime.cancellation().is_cancelled()
        && output.results.is_empty()
        && output.diagnostics.status == "failed"
    {
        Err(ProviderRuntimeError::Cancelled)
    } else {
        Ok(output)
    }
}

pub fn limit_for(input: &SearchInput) -> usize {
    if let Some(explicit) = input.per_provider_limit.or(input.limit) {
        return explicit.clamp(1, MAX_PER_PROVIDER_LIMIT);
    }
    if matches!(
        input.search_mode.as_deref(),
        Some("review" | "systematic_review")
    ) {
        return REVIEW_DEFAULT_LIMIT;
    }
    configured_default_limit()
}

pub fn normalize_doi(raw: &str) -> Option<String> {
    let mut value = raw.trim().to_ascii_lowercase();
    for prefix in ["https://doi.org/", "http://doi.org/", "doi:"] {
        if let Some(stripped) = value.strip_prefix(prefix) {
            value = stripped.to_string();
        }
    }
    if value.is_empty() { None } else { Some(value) }
}

pub fn clean_text(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn year_from_text(raw: &str) -> Option<i64> {
    raw.trim().get(0..4)?.parse::<i64>().ok()
}

#[doc(hidden)]
pub fn deduplicate_results(records: Vec<LiteratureResult>) -> Vec<LiteratureResult> {
    let mut output: Vec<LiteratureResult> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();

    for mut record in records {
        record.title = clean_text(&record.title);
        record.doi = record.doi.as_deref().and_then(normalize_doi);
        record.providers = ordered_providers(&record.provider, &record.providers);
        let key = dedupe_key(&record);
        if let Some(position) = key.as_ref().and_then(|key| positions.get(key)).copied() {
            merge_record(&mut output[position], record);
            continue;
        }
        if let Some(key) = key {
            positions.insert(key, output.len());
        }
        output.push(record);
    }
    output
}

fn run_provider(
    provider: &str,
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    match provider {
        "openalex" => search_openalex(runtime, input),
        "semantic_scholar" => search_semantic_scholar(runtime, input),
        "crossref" => search_crossref(runtime, input),
        "pubmed" => search_pubmed(runtime, input),
        "arxiv" => search_arxiv(runtime, input),
        _ => Err(ProviderRuntimeError::Transport),
    }
}

fn provider_selected(provider: &str, selected: Option<&[String]>) -> bool {
    selected.is_none_or(|providers| {
        providers.iter().any(|candidate| {
            ProviderId::parse(candidate).is_ok_and(|candidate| candidate.as_str() == provider)
        })
    })
}

fn configured_default_limit() -> usize {
    let value = std::env::var("QIONGLI_MCPB_DEFAULT_LIMIT").ok();
    default_limit_from_value(value.as_deref())
}

#[doc(hidden)]
pub fn default_limit_from_value(value: Option<&str>) -> usize {
    value
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (1..=MAX_PER_PROVIDER_LIMIT).contains(value))
        .unwrap_or(GENERAL_DEFAULT_LIMIT)
}

fn public_error_kind(error: &ProviderRuntimeError) -> &'static str {
    match error.code() {
        "timeout" => "timeout",
        "http_error" => "http_error",
        "decode_error" => "decode_error",
        "cancelled" => "cancelled",
        _ => "transport_error",
    }
}

const fn default_limit_for_mode(mode: SearchMode) -> usize {
    match mode {
        SearchMode::Review | SearchMode::SystematicReview => REVIEW_DEFAULT_LIMIT,
        SearchMode::Auto | SearchMode::Topic => GENERAL_DEFAULT_LIMIT,
    }
}

fn cancelled_output() -> SearchOutput {
    SearchOutput {
        status: "error".to_owned(),
        results: Vec::new(),
        diagnostics: SearchDiagnostics {
            status: "failed".to_owned(),
            status_reason: Some("cancelled".to_owned()),
            providers: BTreeMap::new(),
            warnings: vec!["literature search was cancelled".to_owned()],
        },
    }
}

fn dedupe_key(record: &LiteratureResult) -> Option<String> {
    if let Some(doi) = record.doi.as_deref().and_then(normalize_doi) {
        return Some(format!("doi:{doi}"));
    }
    let year = record.year?;
    let title = normalize_title(&record.title);
    (!title.is_empty()).then(|| format!("title:{title}:{year}"))
}

fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn merge_record(existing: &mut LiteratureResult, incoming: LiteratureResult) {
    if existing.doi.is_none() {
        existing.doi = incoming.doi;
    }
    if existing.year.is_none() {
        existing.year = incoming.year;
    }
    if existing.venue.as_deref().is_none_or(str::is_empty) {
        existing.venue = incoming.venue;
    }
    existing.providers = ordered_providers(
        &existing.provider,
        &existing
            .providers
            .iter()
            .chain(incoming.providers.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );
}

fn ordered_providers(primary: &str, providers: &[String]) -> Vec<String> {
    PROVIDER_ORDER
        .into_iter()
        .filter(|provider| *provider == primary || providers.iter().any(|item| item == provider))
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::providers::{
        CancellationToken, ProviderAccess, ProviderAvailability, ProviderEndpoints,
    };

    #[test]
    fn canonical_request_rejects_invalid_inputs_before_execution() {
        let unsupported = vec!["unknown".to_owned()];
        let duplicates = vec!["s2".to_owned(), "semantic-scholar".to_owned()];

        assert!(matches!(
            SearchRequest::from_raw(" ", None, None, None, None, None),
            Err(SearchRequestError::EmptyQuery)
        ));
        assert!(matches!(
            SearchRequest::from_raw(
                &"x".repeat(MAX_QUERY_BYTES + 1),
                None,
                None,
                None,
                None,
                None
            ),
            Err(SearchRequestError::QueryTooLarge)
        ));
        assert!(matches!(
            SearchRequest::from_raw("topic", Some("deep"), None, None, None, None),
            Err(SearchRequestError::UnsupportedMode)
        ));
        assert!(matches!(
            SearchRequest::from_raw("topic", None, Some(&unsupported), None, None, None),
            Err(SearchRequestError::UnsupportedProvider)
        ));
        assert!(matches!(
            SearchRequest::from_raw("topic", None, Some(&duplicates), None, None, None),
            Err(SearchRequestError::DuplicateProvider)
        ));
        assert!(matches!(
            SearchRequest::from_raw("topic", None, None, None, Some(201), None),
            Err(SearchRequestError::InvalidPerProviderLimit)
        ));
        assert!(matches!(
            SearchRequest::from_raw("topic", None, None, None, None, Some(1_001)),
            Err(SearchRequestError::InvalidTotalLimit)
        ));
    }

    #[test]
    fn canonical_request_normalizes_aliases_and_review_defaults() {
        let providers = vec!["s2".to_owned(), "ncbi".to_owned()];
        let request = SearchRequest::from_raw(
            "  governance  ",
            Some("systematic_review"),
            Some(&providers),
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(request.query(), "governance");
        assert_eq!(request.mode(), SearchMode::SystematicReview);
        assert_eq!(
            request.providers(),
            Some([ProviderId::SemanticScholar, ProviderId::PubMed].as_slice())
        );
        assert_eq!(request.per_provider_limit(), 50);
        assert_eq!(request.total_limit(), 250);
    }

    #[test]
    fn lite_arguments_are_strict_bounded_and_do_not_accept_provider_aliases() {
        let request = SearchRequest::from_arguments(&json!({
            "query": " governance ",
            "search_mode": "review",
            "providers": ["openalex", "arxiv"],
            "limit": 20,
            "per_provider_limit": 30,
            "total_limit": 40
        }))
        .unwrap();
        assert_eq!(request.query(), "governance");
        assert_eq!(request.mode(), SearchMode::Review);
        assert_eq!(request.per_provider_limit(), 30);
        assert_eq!(request.total_limit(), 40);

        for (arguments, message) in [
            (json!([]), "search arguments must be an object"),
            (json!({}), "Missing query"),
            (
                json!({"query": "topic", "private-canary": true}),
                "Unsupported argument",
            ),
            (
                json!({"query": "topic", "providers": ["s2"]}),
                "unsupported provider",
            ),
            (
                json!({"query": "topic", "providers": ["arxiv", "arxiv"]}),
                "providers must contain unique values",
            ),
            (
                json!({"query": "topic", "per_provider_limit": 201}),
                "per_provider_limit must be between 1 and 200",
            ),
        ] {
            assert_eq!(
                SearchRequest::from_arguments(&arguments)
                    .err()
                    .unwrap()
                    .to_string(),
                message
            );
        }
    }

    #[test]
    fn pre_cancelled_search_returns_typed_error_without_networking() {
        let mut builder = ProviderAccess::builder();
        builder.set_availability(ProviderId::Arxiv, ProviderAvailability::Ready);
        let endpoints = ProviderEndpoints::from_urls(
            "http://127.0.0.1:9",
            "http://127.0.0.1:9",
            "http://127.0.0.1:9",
            "http://127.0.0.1:9",
            "http://127.0.0.1:9",
        )
        .unwrap();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let runtime = ProviderRuntime::with_client_and_cancellation(
            reqwest::blocking::Client::new(),
            endpoints,
            builder.build(),
            cancellation,
        );
        let request = SearchRequest::from_raw("topic", None, None, None, None, None).unwrap();

        assert_eq!(
            execute_bounded_search(&runtime, &request),
            Err(ProviderRuntimeError::Cancelled)
        );
    }

    #[test]
    fn deduplication_keeps_canonical_provider_order() {
        let records = vec![
            LiteratureResult {
                title: "A paper".to_owned(),
                doi: Some("https://doi.org/10.1000/test".to_owned()),
                year: Some(2026),
                venue: None,
                provider: "openalex".to_owned(),
                providers: vec!["openalex".to_owned()],
            },
            LiteratureResult {
                title: "A paper".to_owned(),
                doi: Some("10.1000/TEST".to_owned()),
                year: Some(2026),
                venue: Some("Venue".to_owned()),
                provider: "semantic_scholar".to_owned(),
                providers: vec!["semantic_scholar".to_owned()],
            },
        ];

        let deduplicated = deduplicate_results(records);
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(
            deduplicated[0].providers,
            vec!["openalex", "semantic_scholar"]
        );
        assert_eq!(deduplicated[0].venue.as_deref(), Some("Venue"));
    }
}
