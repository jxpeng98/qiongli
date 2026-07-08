use serde::Deserialize;

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
    client: &reqwest::blocking::Client,
    base_url: &str,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderError> {
    let payload = client
        .get(format!("{}/paper/search", base_url.trim_end_matches('/')))
        .query(&[
            ("query", input.query.as_str()),
            ("limit", &limit_for(input).to_string()),
            ("fields", "title,year,venue,externalIds"),
        ])
        .send()?
        .error_for_status()?
        .text()?;
    normalize_semantic_scholar_response(&payload)
}
