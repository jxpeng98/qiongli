use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Clone, Serialize)]
pub struct SearchDiagnostics {
    pub status: String,
    pub provider_counts: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchOutput {
    pub status: String,
    pub results: Vec<LiteratureResult>,
    pub diagnostics: SearchDiagnostics,
}

pub fn empty_search_output() -> SearchOutput {
    SearchOutput {
        status: "ok".to_string(),
        results: Vec::new(),
        diagnostics: SearchDiagnostics {
            status: "not_run".to_string(),
            provider_counts: BTreeMap::new(),
            warnings: Vec::new(),
        },
    }
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

pub fn limit_for(input: &SearchInput) -> usize {
    input
        .per_provider_limit
        .or(input.limit)
        .or(input.total_limit)
        .unwrap_or(10)
        .max(1)
}
