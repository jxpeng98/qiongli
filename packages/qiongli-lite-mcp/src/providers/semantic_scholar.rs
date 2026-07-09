use serde::Deserialize;

use crate::providers::runtime::{ProviderRuntime, ProviderRuntimeError};
use crate::providers::search::{
    limit_for, normalize_doi, LiteratureResult, ProviderError, SearchInput,
};

#[derive(Debug, Deserialize)]
struct SemanticScholarResponse {
    #[serde(default)]
    data: Vec<SemanticScholarPaper>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarPaper {
    title: Option<String>,
    year: Option<i64>,
    venue: Option<String>,
    #[serde(rename = "externalIds")]
    external_ids: Option<SemanticScholarExternalIds>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarExternalIds {
    #[serde(rename = "DOI")]
    doi: Option<String>,
}

pub fn normalize_semantic_scholar_response(
    payload: &str,
) -> Result<Vec<LiteratureResult>, ProviderError> {
    let response: SemanticScholarResponse = serde_json::from_str(payload)?;
    Ok(response
        .data
        .into_iter()
        .filter_map(|paper| {
            let title = paper.title?;
            Some(LiteratureResult {
                title,
                doi: paper
                    .external_ids
                    .and_then(|ids| ids.doi)
                    .as_deref()
                    .and_then(normalize_doi),
                year: paper.year,
                venue: paper.venue,
                provider: "semantic_scholar".to_string(),
                providers: vec!["semantic_scholar".to_string()],
            })
        })
        .collect())
}

pub fn search_semantic_scholar(
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    let mut url = runtime
        .endpoints()
        .semantic_scholar()
        .join("paper/search")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("query", &input.query);
        query.append_pair("limit", &limit_for(input).min(200).to_string());
        query.append_pair("fields", "title,year,venue,externalIds");
    }
    let mut request = runtime
        .client()
        .get(url)
        .header("Accept", "application/json");
    if let Some(api_key) = runtime.config().value("semantic_scholar", "api_key") {
        request = request.header("x-api-key", api_key);
    }
    let payload = runtime.get_text(request)?;
    normalize_semantic_scholar_response(&payload).map_err(|_| ProviderRuntimeError::Decode)
}
