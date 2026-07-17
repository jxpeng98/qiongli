use serde::Deserialize;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{LiteratureResult, ProviderError, SearchInput, limit_for, normalize_doi};

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
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    let mut url = runtime
        .endpoints()
        .openalex()
        .join("works")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("search", &input.query);
        query.append_pair("per-page", &limit_for(input).min(200).to_string());
        if let Some(api_key) = runtime
            .access()
            .value(super::ProviderId::OpenAlex, super::ProviderField::ApiKey)
        {
            query.append_pair("api_key", api_key);
        }
        if let Some(email) = runtime
            .access()
            .value(super::ProviderId::OpenAlex, super::ProviderField::Email)
        {
            query.append_pair("mailto", email);
        }
    }
    let payload = runtime.get_text(
        runtime
            .client()
            .get(url)
            .header("Accept", "application/json"),
    )?;
    normalize_openalex_response(&payload).map_err(|_| ProviderRuntimeError::Decode)
}
