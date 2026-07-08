use serde::Deserialize;

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
                year: work
                    .issued
                    .and_then(|issued| issued.date_parts.first().and_then(|part| part.first()).copied()),
                venue: work.container_title.into_iter().next(),
                provider: "crossref".to_string(),
                providers: vec!["crossref".to_string()],
            })
        })
        .collect())
}

pub fn search_crossref(
    client: &reqwest::blocking::Client,
    base_url: &str,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderError> {
    let payload = client
        .get(format!("{}/works", base_url.trim_end_matches('/')))
        .query(&[
            ("query", input.query.as_str()),
            ("rows", &limit_for(input).to_string()),
        ])
        .send()?
        .error_for_status()?
        .text()?;
    normalize_crossref_response(&payload)
}
