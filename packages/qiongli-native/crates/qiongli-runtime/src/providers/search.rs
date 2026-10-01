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
const SEARCH_ARGUMENTS: [&str; 9] = [
    "query",
    "search_mode",
    "providers",
    "limit",
    "per_provider_limit",
    "total_limit",
    "from_year",
    "to_year",
    "venue_filter",
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

#[derive(Debug, Clone, Default)]
pub struct SearchInput {
    pub query: String,
    pub search_mode: Option<String>,
    pub limit: Option<usize>,
    pub per_provider_limit: Option<usize>,
    pub total_limit: Option<usize>,
    pub from_year: Option<i64>,
    pub to_year: Option<i64>,
    pub venue_filter: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SearchMode {
    Auto,
    Topic,
    Title,
    Doi,
    Review,
    SystematicReview,
}

impl SearchMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Topic => "topic",
            Self::Title => "title",
            Self::Doi => "doi",
            Self::Review => "review",
            Self::SystematicReview => "systematic_review",
        }
    }

    pub fn parse(value: Option<&str>) -> Result<Self, SearchRequestError> {
        match value.unwrap_or("auto") {
            "auto" => Ok(Self::Auto),
            "topic" => Ok(Self::Topic),
            "title" => Ok(Self::Title),
            "doi" => Ok(Self::Doi),
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
    #[error("DOI mode requires a valid DOI")]
    InvalidDoi,
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
    pub from_year: Option<i64>,
    pub to_year: Option<i64>,
    pub venue_filter: Option<String>,
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

        let mut request = Self::from_raw(
            query,
            mode,
            providers.as_deref(),
            limit,
            per_provider_limit,
            total_limit,
        )
        .map_err(SearchArgumentsError::from_request_error)?;
        fn year(arguments: &Value, key: &str) -> Result<Option<i64>, SearchArgumentsError> {
            arguments
                .get(key)
                .map(|v| {
                    v.as_i64().filter(|v| (1000..=9999).contains(v)).ok_or(
                        SearchArgumentsError::new(
                            "year filter must be an integer between 1000 and 9999",
                        ),
                    )
                })
                .transpose()
        }
        request.from_year = year(arguments, "from_year")?;
        request.to_year = year(arguments, "to_year")?;
        if request
            .from_year
            .zip(request.to_year)
            .is_some_and(|(from, to)| from > to)
        {
            return Err(SearchArgumentsError::new(
                "from_year must not exceed to_year",
            ));
        }
        request.venue_filter = arguments
            .get("venue_filter")
            .map(|v| {
                v.as_str()
                    .filter(|v| !v.trim().is_empty() && v.len() <= 256)
                    .map(|v| v.trim().to_owned())
                    .ok_or(SearchArgumentsError::new(
                        "venue_filter must contain 1 to 256 bytes",
                    ))
            })
            .transpose()?;
        Ok(request)
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
        if mode == SearchMode::Doi && normalize_doi(query).is_none() {
            return Err(SearchRequestError::InvalidDoi);
        }
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
            from_year: None,
            to_year: None,
            venue_filter: None,
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
            SearchRequestError::InvalidDoi => "DOI mode requires a valid DOI",
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,
}

impl Author {
    pub fn literal(name: String) -> Self {
        Self {
            literal: Some(name),
            ..Self::default()
        }
    }

    pub fn display_name(&self) -> String {
        self.literal
            .clone()
            .unwrap_or_else(|| match (&self.family, &self.given) {
                (Some(family), Some(given)) => format!("{family}, {given}"),
                (Some(family), None) => family.clone(),
                (None, Some(given)) => given.clone(),
                _ => String::new(),
            })
    }
}

/// A provider-reported access lead, not proof of retrieval or readable full text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FulltextCandidate {
    pub url: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl FulltextCandidate {
    pub fn new(
        url: &str,
        source: &str,
        format: Option<&str>,
        version: Option<&str>,
        license: Option<&str>,
    ) -> Option<Self> {
        let url = url.trim();
        let (_, authority) = url.split_once("://")?;
        let parsed = url::Url::parse(url).ok()?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || authority.split(['/', '?', '#']).next()?.contains('@')
            || url.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return None;
        }
        Some(Self {
            url: url.to_owned(),
            source: source.to_owned(),
            format: format.map(str::to_owned),
            version: version.map(str::to_owned),
            license: license.map(str::to_owned),
        })
    }
}

pub(crate) fn string_identifiers(value: Option<&Value>) -> BTreeMap<String, String> {
    value
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| {
            let identifier = value
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned)
                .or_else(|| value.as_u64().map(|id| id.to_string()));
            identifier.map(|id| (key.clone(), id))
        })
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiteratureResult {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    pub provider: String,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citekey: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata_conflicts: Vec<String>,
    #[serde(default, rename = "abstract", skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fulltext_candidates: Vec<FulltextCandidate>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub external_ids: BTreeMap<String, String>,
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
    let mut results = select_results(deduplicate_results(records), input);
    if input.from_year.is_some() || input.to_year.is_some() || input.venue_filter.is_some() {
        warnings.push("Filters checked against returned metadata before the total limit; missing fields excluded. Provider retrieval is bounded, not exhaustive.".to_owned());
    }
    if input.search_mode.as_deref() == Some("title") && results.len() > 1 {
        warnings.push("Multiple exact-title candidates; verify authors, year and publication version before citation.".to_owned());
    }
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

/// Filter and rank bounded provider candidates, never treat ranking as metadata verification.
pub fn select_results(
    mut records: Vec<LiteratureResult>,
    input: &SearchInput,
) -> Vec<LiteratureResult> {
    records.retain(|r| {
        input
            .from_year
            .is_none_or(|from| r.year.is_some_and(|y| y >= from))
            && input
                .to_year
                .is_none_or(|to| r.year.is_some_and(|y| y <= to))
            && input.venue_filter.as_ref().is_none_or(|v| {
                r.venue
                    .as_ref()
                    .is_some_and(|venue| normalize_title(venue) == normalize_title(v))
            })
            && match input.search_mode.as_deref() {
                Some("title") => normalize_title(&r.title) == normalize_title(&input.query),
                Some("doi") => normalize_doi(&input.query).is_some_and(|doi| {
                    r.doi.as_deref().and_then(normalize_doi).as_ref() == Some(&doi)
                }),
                _ => true,
            }
    });
    let query = normalize_title(&input.query);
    let tokens: Vec<_> = input
        .query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .collect();
    records.sort_by_cached_key(|r| {
        let title = r.title.to_lowercase();
        let words: Vec<_> = title.split(|c: char| !c.is_alphanumeric()).collect();
        std::cmp::Reverse((
            normalize_title(&r.title) == query,
            tokens
                .iter()
                .filter(|t| words.contains(&t.as_str()))
                .count(),
        ))
    });
    records
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
        from_year: request.from_year,
        to_year: request.to_year,
        venue_filter: request.venue_filter.clone(),
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
            value = stripped.trim().to_string();
        }
    }
    let (prefix, suffix) = value.split_once('/')?;
    let digits = prefix.strip_prefix("10.")?;
    if !(4..=9).contains(&digits.len())
        || !digits.bytes().all(|c| c.is_ascii_digit())
        || suffix.is_empty()
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        None
    } else {
        Some(value)
    }
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
    let mut positions: HashMap<String, Vec<usize>> = HashMap::new();

    for mut record in records {
        record.title = clean_text(&record.title);
        record.doi = record.doi.as_deref().and_then(normalize_doi);
        record.providers = ordered_providers(&record.provider, &record.providers);
        if let Some(source_id) = &record.source_id {
            record
                .external_ids
                .entry(record.provider.clone())
                .or_insert_with(|| source_id.clone());
        }
        let mut candidates = Vec::new();
        for candidate in record.fulltext_candidates {
            if FulltextCandidate::new(&candidate.url, &candidate.source, None, None, None).is_some()
                && !candidates.contains(&candidate)
            {
                candidates.push(candidate);
            }
        }
        record.fulltext_candidates = candidates;
        let key = dedupe_key(&record);
        if let Some(group) = key.as_ref().and_then(|key| positions.get(key)) {
            let matching = group
                .iter()
                .copied()
                .find(|&i| output[i].citekey == record.citekey)
                .or_else(|| {
                    group
                        .iter()
                        .copied()
                        .find(|&i| output[i].citekey.is_none() || record.citekey.is_none())
                });
            if let Some(position) = matching {
                merge_record(&mut output[position], record);
                continue;
            }
            for &position in group {
                if !output[position]
                    .metadata_conflicts
                    .iter()
                    .any(|v| v == "citekey")
                {
                    output[position]
                        .metadata_conflicts
                        .push("citekey".to_owned());
                }
            }
            record.metadata_conflicts.push("citekey".to_owned());
        }
        if let Some(key) = key {
            positions.entry(key).or_default().push(output.len());
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
        SearchMode::Auto | SearchMode::Topic | SearchMode::Title | SearchMode::Doi => {
            GENERAL_DEFAULT_LIMIT
        }
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

pub(crate) fn dedupe_key(record: &LiteratureResult) -> Option<String> {
    // A preprint can carry the later journal DOI; keep its own version identity.
    if matches!(
        record.record_type.as_deref(),
        Some("preprint" | "posted-content")
    ) {
        if let Some(id) = record.source_id.as_ref().or(record.url.as_ref()) {
            return Some(format!("preprint:{id}"));
        }
        return Some(format!(
            "preprint:{}:{:?}:{:?}",
            normalize_title(&record.title),
            record.year,
            record.authors
        ));
    }
    if let Some(doi) = record.doi.as_deref().and_then(normalize_doi) {
        return Some(format!("doi:{doi}"));
    }
    if let Some(id) = record.source_id.as_ref().filter(|id| !id.trim().is_empty()) {
        return Some(format!("source:{}:{id}", record.provider));
    }
    let year = record.year?;
    let title = normalize_title(&record.title);
    let authors: Vec<_> = record
        .authors
        .iter()
        .map(|a| normalize_title(&a.display_name()))
        .collect();
    (!title.is_empty())
        .then(|| format!("title:{title}:{year}:{authors:?}:{:?}", record.record_type))
}

fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn merge_record(existing: &mut LiteratureResult, incoming: LiteratureResult) {
    if normalize_title(&existing.title) != normalize_title(&incoming.title) {
        existing.metadata_conflicts.push("title".to_owned());
    }
    macro_rules! fill {
        ($($field:ident),+ $(,)?) => {$(
            if existing.$field.is_none() { existing.$field = incoming.$field.clone(); }
            else if incoming.$field.is_some() && existing.$field != incoming.$field {
                existing.metadata_conflicts.push(stringify!($field).to_owned());
            }
        )+};
    }
    fill!(
        doi,
        year,
        venue,
        record_type,
        published_date,
        volume,
        issue,
        pages,
        publisher,
        citekey
    );
    if existing.abstract_text.is_none() {
        existing.abstract_text = incoming.abstract_text;
    } else if incoming.abstract_text.is_some() && existing.abstract_text != incoming.abstract_text {
        existing.metadata_conflicts.push("abstract".to_owned());
    }
    for candidate in incoming.fulltext_candidates {
        if !existing.fulltext_candidates.contains(&candidate) {
            existing.fulltext_candidates.push(candidate);
        }
    }
    for (key, value) in incoming.external_ids {
        if let Some(previous) = existing.external_ids.get(&key) {
            if previous != &value {
                existing
                    .metadata_conflicts
                    .push(format!("external_ids.{key}"));
                existing
                    .external_ids
                    .entry(format!("{}:{key}", incoming.provider))
                    .or_insert(value);
            }
        } else {
            existing.external_ids.insert(key, value);
        }
    }
    if existing.source_id.is_none() {
        existing.source_id = incoming.source_id;
    }
    if existing.url.is_none() {
        existing.url = incoming.url;
    }
    if existing.authors.is_empty() {
        existing.authors = incoming.authors;
    } else if !incoming.authors.is_empty() && existing.authors != incoming.authors {
        existing.metadata_conflicts.push("authors".to_owned());
    }
    existing
        .metadata_conflicts
        .extend(incoming.metadata_conflicts);
    existing.metadata_conflicts.sort();
    existing.metadata_conflicts.dedup();
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
    let mut ordered: Vec<String> = PROVIDER_ORDER
        .into_iter()
        .filter(|provider| *provider == primary || providers.iter().any(|item| item == provider))
        .map(ToString::to_string)
        .collect();
    for provider in std::iter::once(primary).chain(providers.iter().map(String::as_str)) {
        if !ordered.iter().any(|p| p == provider) {
            ordered.push(provider.to_owned());
        }
    }
    ordered
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
                ..Default::default()
            },
            LiteratureResult {
                title: "A paper".to_owned(),
                doi: Some("10.1000/TEST".to_owned()),
                year: Some(2026),
                venue: Some("Venue".to_owned()),
                provider: "semantic_scholar".to_owned(),
                providers: vec!["semantic_scholar".to_owned()],
                ..Default::default()
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

    #[test]
    fn access_leads_are_optional_and_legacy_records_stay_readable() {
        let record: LiteratureResult = serde_json::from_value(json!({
            "title": "Legacy paper", "provider": "crossref"
        }))
        .unwrap();
        let value = serde_json::to_value(record).unwrap();
        for absent in [
            "abstract",
            "fulltext_candidates",
            "external_ids",
            "fulltext",
        ] {
            assert!(value.get(absent).is_none());
        }
        for url in [
            "file:///tmp/paper.pdf",
            "javascript:alert(1)",
            "https:paper.example/a",
            "https://user:secret@paper.example/a",
            "https://@paper.example/a",
            "/paper.pdf",
            "https://paper.example/a\nb",
            "https://paper.example/a b",
        ] {
            assert!(
                FulltextCandidate::new(url, "fixture", Some("pdf"), None, None).is_none(),
                "{url}"
            );
        }
        assert!(
            FulltextCandidate::new(
                "http://paper.example/a.pdf",
                "fixture",
                Some("pdf"),
                None,
                None
            )
            .is_some()
        );
    }

    #[test]
    fn dedup_preserves_access_provenance_versions_licenses_and_identity_conflicts() {
        let lead = |source: &str, version: &str, license: &str| {
            FulltextCandidate::new(
                "https://paper.example/full.pdf",
                source,
                Some("pdf"),
                Some(version),
                Some(license),
            )
            .unwrap()
        };
        let first = lead("openalex", "acceptedVersion", "cc-by");
        let second = lead("semantic_scholar", "publishedVersion", "cc-by-nc");
        let third = lead("openalex", "publishedVersion", "cc-by");
        let records = deduplicate_results(vec![
            LiteratureResult {
                title: "Paper".into(),
                doi: Some("10.1000/test".into()),
                provider: "openalex".into(),
                source_id: Some("https://openalex.org/W1".into()),
                abstract_text: Some("An abstract, not the body.".into()),
                fulltext_candidates: vec![first.clone(), first.clone()],
                external_ids: BTreeMap::from([("DOI".into(), "10.1000/test".into())]),
                ..Default::default()
            },
            LiteratureResult {
                title: "Paper".into(),
                doi: Some("10.1000/test".into()),
                provider: "semantic_scholar".into(),
                source_id: Some("semantic_scholar:123".into()),
                abstract_text: Some("Different abstract.".into()),
                fulltext_candidates: vec![second.clone(), third.clone()],
                external_ids: BTreeMap::from([
                    ("DOI".into(), "10.1000/TEST".into()),
                    ("PubMedCentral".into(), "PMC1".into()),
                ]),
                ..Default::default()
            },
        ]);
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.fulltext_candidates, vec![first, second, third]);
        assert_eq!(record.external_ids["openalex"], "https://openalex.org/W1");
        assert_eq!(
            record.external_ids["semantic_scholar"],
            "semantic_scholar:123"
        );
        assert_eq!(record.external_ids["PubMedCentral"], "PMC1");
        assert_eq!(record.external_ids["semantic_scholar:DOI"], "10.1000/TEST");
        assert_eq!(
            record.metadata_conflicts,
            vec!["abstract", "external_ids.DOI"]
        );
        let value = serde_json::to_value(record).unwrap();
        assert_eq!(value["abstract"], "An abstract, not the body.");
        assert!(value.get("fulltext").is_none());
        assert!(value["fulltext_candidates"][0].get("retrieved").is_none());
    }
}
