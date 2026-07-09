use serde::Deserialize;

use crate::providers::runtime::{ProviderRuntime, ProviderRuntimeError};
use crate::providers::search::{
    limit_for, normalize_doi, LiteratureResult, ProviderError, SearchInput,
};

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize)]
struct CrossrefMessage {
    #[serde(default)]
    items: Vec<CrossrefWork>,
}

#[derive(Debug, Deserialize)]
struct CrossrefWork {
    #[serde(default)]
    title: Vec<String>,
    issued: Option<CrossrefIssued>,
    #[serde(rename = "DOI")]
    doi: Option<String>,
    #[serde(default, rename = "container-title")]
    container_title: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefIssued {
    #[serde(default, rename = "date-parts")]
    date_parts: Vec<Vec<i64>>,
}

pub fn normalize_crossref_response(payload: &str) -> Result<Vec<LiteratureResult>, ProviderError> {
    let response: CrossrefResponse = serde_json::from_str(payload)?;
    Ok(response
        .message
        .items
        .into_iter()
        .filter_map(|work| {
            let title = work.title.into_iter().next()?;
            Some(LiteratureResult {
                title,
                doi: work.doi.as_deref().and_then(normalize_doi),
                year: work.issued.and_then(|issued| {
                    issued
                        .date_parts
                        .first()
                        .and_then(|part| part.first())
                        .copied()
                }),
                venue: work.container_title.into_iter().next(),
                provider: "crossref".to_string(),
                providers: vec!["crossref".to_string()],
            })
        })
        .collect())
}

pub fn search_crossref(
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    let mut url = runtime
        .endpoints()
        .crossref()
        .join("works")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("query", &input.query);
        query.append_pair("rows", &limit_for(input).min(200).to_string());
        if let Some(email) = runtime.config().value("crossref", "email") {
            query.append_pair("mailto", email);
        }
    }
    let payload = runtime.get_text(
        runtime
            .client()
            .get(url)
            .header("Accept", "application/json"),
    )?;
    normalize_crossref_response(&payload).map_err(|_| ProviderRuntimeError::Decode)
}
