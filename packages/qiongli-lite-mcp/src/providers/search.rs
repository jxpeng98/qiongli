use std::collections::{BTreeMap, HashMap};
use std::thread;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::providers::arxiv::search_arxiv;
use crate::providers::crossref::search_crossref;
use crate::providers::openalex::search_openalex;
use crate::providers::pubmed::search_pubmed;
use crate::providers::runtime::{ProviderRuntime, ProviderRuntimeError};
use crate::providers::semantic_scholar::search_semantic_scholar;

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

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("xml error: {0}")]
    Xml(#[from] quick_xml::Error),
}

#[derive(Debug, Clone)]
pub struct SearchInput {
    pub query: String,
    pub search_mode: Option<String>,
    pub limit: Option<usize>,
    pub per_provider_limit: Option<usize>,
    pub total_limit: Option<usize>,
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
    let attempted: Vec<&'static str> = PROVIDER_ORDER
        .into_iter()
        .filter(|provider| runtime.config().is_active(provider))
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
    if value.is_empty() {
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
        providers
            .iter()
            .any(|candidate| normalize_provider(candidate) == provider)
    })
}

fn normalize_provider(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "s2" | "semanticscholar" => "semantic_scholar".to_string(),
        "ncbi" => "pubmed".to_string(),
        _ => normalized,
    }
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
        _ => "transport_error",
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
