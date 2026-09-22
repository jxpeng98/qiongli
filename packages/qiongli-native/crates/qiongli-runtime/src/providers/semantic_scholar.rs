use serde::Deserialize;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, LiteratureResult, ProviderError, SearchInput, limit_for, normalize_doi,
};

#[derive(Debug, Deserialize)]
struct SemanticScholarResponse {
    #[serde(default)]
    data: Vec<SemanticScholarPaper>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarPaper {
    title: Option<String>,
    #[serde(rename = "paperId")]
    paper_id: Option<String>,
    url: Option<String>,
    #[serde(default)]
    authors: Vec<serde_json::Value>,
    #[serde(default, rename = "publicationTypes")]
    publication_types: Option<Vec<String>>,
    #[serde(rename = "publicationDate")]
    publication_date: Option<String>,
    journal: Option<serde_json::Value>,
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
                source_id: paper.paper_id.map(|id| format!("semantic_scholar:{id}")),
                url: paper.url,
                authors: paper
                    .authors
                    .iter()
                    .filter_map(|a| a["name"].as_str())
                    .map(|a| Author::literal(a.to_owned()))
                    .collect(),
                record_type: paper.publication_types.as_ref().and_then(|kinds| {
                    kinds.iter().find_map(|kind| match kind.as_str() {
                        "JournalArticle" => Some("article-journal".to_owned()),
                        "Conference" => Some("paper-conference".to_owned()),
                        "Book" => Some("book".to_owned()),
                        _ => None,
                    })
                }),
                published_date: paper.publication_date,
                volume: paper
                    .journal
                    .as_ref()
                    .and_then(|j| j["volume"].as_str())
                    .map(str::to_owned),
                pages: paper
                    .journal
                    .as_ref()
                    .and_then(|j| j["pages"].as_str())
                    .map(str::to_owned),
                doi: paper
                    .external_ids
                    .and_then(|ids| ids.doi)
                    .as_deref()
                    .and_then(normalize_doi),
                year: paper.year,
                venue: paper.venue,
                provider: "semantic_scholar".to_string(),
                providers: vec!["semantic_scholar".to_string()],
                ..Default::default()
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
        query.append_pair(
            "fields",
            "title,year,venue,externalIds,authors,url,publicationTypes,publicationDate,journal",
        );
    }
    let mut request = runtime
        .client()
        .get(url)
        .header("Accept", "application/json");
    if let Some(api_key) = runtime.access().value(
        super::ProviderId::SemanticScholar,
        super::ProviderField::ApiKey,
    ) {
        request = request.header("x-api-key", api_key);
    }
    let payload = runtime.get_text(request)?;
    normalize_semantic_scholar_response(&payload).map_err(|_| ProviderRuntimeError::Decode)
}
