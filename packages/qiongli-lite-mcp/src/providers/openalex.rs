use serde::Deserialize;

use crate::providers::search::{
    limit_for, normalize_doi, LiteratureResult, ProviderError, SearchInput,
};

#[derive(Debug, Deserialize)]
struct OpenAlexResponse {
    #[serde(default)]
    results: Vec<OpenAlexWork>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexWork {
    display_name: Option<String>,
    publication_year: Option<i64>,
    doi: Option<String>,
    primary_location: Option<OpenAlexLocation>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexLocation {
    source: Option<OpenAlexSource>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexSource {
    display_name: Option<String>,
}

pub fn normalize_openalex_response(payload: &str) -> Result<Vec<LiteratureResult>, ProviderError> {
    let response: OpenAlexResponse = serde_json::from_str(payload)?;
    Ok(response
        .results
        .into_iter()
        .filter_map(|work| {
            let title = work.display_name?;
            Some(LiteratureResult {
                title,
                doi: work.doi.as_deref().and_then(normalize_doi),
                year: work.publication_year,
                venue: work
                    .primary_location
                    .and_then(|location| location.source)
                    .and_then(|source| source.display_name),
                provider: "openalex".to_string(),
                providers: vec!["openalex".to_string()],
            })
        })
        .collect())
}

pub fn search_openalex(
    client: &reqwest::blocking::Client,
    base_url: &str,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderError> {
    let payload = client
        .get(format!("{}/works", base_url.trim_end_matches('/')))
        .query(&[
            ("search", input.query.as_str()),
            ("per-page", &limit_for(input).to_string()),
        ])
        .send()?
        .error_for_status()?
        .text()?;
    normalize_openalex_response(&payload)
}
