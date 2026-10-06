use serde::Deserialize;
use serde_json::Value;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, FulltextCandidate, LiteratureResult, ProviderError, SearchInput, limit_for,
    normalize_doi, string_identifiers,
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
    external_ids: Option<Value>,
    #[serde(default, rename = "abstract")]
    abstract_text: Value,
    #[serde(default, rename = "openAccessPdf")]
    open_access_pdf: Value,
    #[serde(default, rename = "isOpenAccess")]
    is_open_access: Value,
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
            let fulltext_candidates: Vec<_> = paper.open_access_pdf["url"]
                .as_str()
                .and_then(|url| {
                    FulltextCandidate::new(
                        url,
                        "semantic_scholar",
                        Some("pdf"),
                        None,
                        paper.open_access_pdf["license"].as_str(),
                    )
                })
                .into_iter()
                .collect();
            let external_ids = string_identifiers(paper.external_ids.as_ref());
            Some(LiteratureResult {
                title,
                abstract_text: paper
                    .abstract_text
                    .as_str()
                    .filter(|s| !s.trim().is_empty())
                    .map(str::to_owned),
                metadata_conflicts: if paper.is_open_access.as_bool() == Some(false)
                    && !fulltext_candidates.is_empty()
                {
                    vec!["open_access".to_owned()]
                } else {
                    Vec::new()
                },
                fulltext_candidates,
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
                doi: external_ids.get("DOI").and_then(|doi| normalize_doi(doi)),
                external_ids,
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
            "title,year,venue,externalIds,authors,url,publicationTypes,publicationDate,journal,abstract,openAccessPdf,isOpenAccess",
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_pdf_lead_license_and_external_ids_separately_from_abstract() {
        let records = normalize_semantic_scholar_response(&json!({"data":[{
            "title":"Paper", "paperId":"abc", "abstract":"Summary only.", "isOpenAccess":true,
            "externalIds":{"DOI":"10.1000/TEST", "PubMedCentral":"PMC123", "ArXiv":"2601.12345", "CorpusId":123},
            "openAccessPdf":{"url":"https://repository.example/paper.pdf", "license":"CCBY", "status":"GREEN"}
        }]}).to_string()).unwrap();
        let record = &records[0];
        assert_eq!(record.abstract_text.as_deref(), Some("Summary only."));
        assert_eq!(record.doi.as_deref(), Some("10.1000/test"));
        assert_eq!(record.external_ids["PubMedCentral"], "PMC123");
        assert_eq!(record.external_ids["CorpusId"], "123");
        assert_eq!(
            record.fulltext_candidates[0].license.as_deref(),
            Some("CCBY")
        );
        assert!(record.fulltext_candidates[0].version.is_none());
        assert!(record.metadata_conflicts.is_empty());
    }

    #[test]
    fn bad_optional_fields_do_not_discard_papers_or_claim_access() {
        let records = normalize_semantic_scholar_response(&json!({"data":[
            {"title":"Missing"},
            {"title":"Malformed", "abstract":{}, "openAccessPdf":{"url":"https://user@paper.example/a"}, "externalIds":{"PubMed":null}},
            {"title":"Conflicting", "isOpenAccess":false, "openAccessPdf":{"url":"https://paper.example/a.pdf"}}
        ]}).to_string()).unwrap();
        assert_eq!(records.len(), 3);
        assert!(records[0].fulltext_candidates.is_empty());
        assert!(records[1].fulltext_candidates.is_empty());
        assert!(records[1].abstract_text.is_none());
        assert!(records[1].external_ids.is_empty());
        assert_eq!(records[2].metadata_conflicts, vec!["open_access"]);
    }
}
