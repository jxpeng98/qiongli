use serde::Deserialize;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, LiteratureResult, ProviderError, SearchInput, limit_for, normalize_doi, year_from_text,
};

#[derive(Debug, Deserialize)]
struct PubmedSearchResponse {
    esearchresult: Option<PubmedSearchResult>,
}

#[derive(Debug, Deserialize)]
struct PubmedSearchResult {
    #[serde(default)]
    idlist: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PubmedArticle {
    title: Option<String>,
    #[serde(default)]
    authors: Vec<serde_json::Value>,
    volume: Option<String>,
    issue: Option<String>,
    pages: Option<String>,
    #[serde(default)]
    pubtype: Vec<String>,
    pubdate: Option<String>,
    fulljournalname: Option<String>,
    #[serde(default)]
    articleids: Vec<PubmedArticleId>,
}

#[derive(Debug, Deserialize)]
struct PubmedArticleId {
    idtype: Option<String>,
    value: Option<String>,
}

pub fn normalize_pubmed_summary_response(
    payload: &str,
) -> Result<Vec<LiteratureResult>, ProviderError> {
    let root: serde_json::Value = serde_json::from_str(payload)?;
    let Some(result) = root.get("result") else {
        return Ok(Vec::new());
    };
    let uids = result
        .get("uids")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut records = Vec::new();
    for uid in uids {
        let Some(uid) = uid.as_str() else {
            continue;
        };
        let Some(article_value) = result.get(uid) else {
            continue;
        };
        let article: PubmedArticle = serde_json::from_value(article_value.clone())?;
        let Some(title) = article.title else {
            continue;
        };
        let doi = article
            .articleids
            .into_iter()
            .find(|id| id.idtype.as_deref() == Some("doi"))
            .and_then(|id| id.value)
            .as_deref()
            .and_then(normalize_doi);
        records.push(LiteratureResult {
            title,
            source_id: Some(format!("pmid:{uid}")),
            url: Some(format!("https://pubmed.ncbi.nlm.nih.gov/{uid}/")),
            authors: article
                .authors
                .iter()
                .filter_map(|a| a["name"].as_str())
                .map(|a| Author::literal(a.to_owned()))
                .collect(),
            record_type: article
                .pubtype
                .iter()
                .any(|p| p == "Journal Article")
                .then(|| "article-journal".to_owned()),
            published_date: article.pubdate.clone(),
            volume: article.volume,
            issue: article.issue,
            pages: article.pages,
            doi,
            year: article.pubdate.as_deref().and_then(year_from_text),
            venue: article.fulljournalname,
            provider: "pubmed".to_string(),
            providers: vec!["pubmed".to_string()],
            ..Default::default()
        });
    }
    Ok(records)
}

pub fn normalize_pubmed_search_response(payload: &str) -> Result<Vec<String>, ProviderError> {
    let response: PubmedSearchResponse = serde_json::from_str(payload)?;
    Ok(response
        .esearchresult
        .map(|result| result.idlist)
        .unwrap_or_default()
        .into_iter()
        .filter(|id| !id.trim().is_empty())
        .collect())
}

pub fn search_pubmed(
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    let mut search_url = runtime
        .endpoints()
        .pubmed()
        .join("esearch.fcgi")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    {
        let mut query = search_url.query_pairs_mut();
        query.append_pair("db", "pubmed");
        query.append_pair("term", &input.query);
        query.append_pair("retmode", "json");
        query.append_pair("sort", "relevance");
        query.append_pair("retmax", &limit_for(input).min(200).to_string());
        if let Some(api_key) = runtime
            .access()
            .value(super::ProviderId::PubMed, super::ProviderField::ApiKey)
        {
            query.append_pair("api_key", api_key);
        }
    }
    let search_payload = runtime.get_text(
        runtime
            .client()
            .get(search_url)
            .header("Accept", "application/json"),
    )?;
    let mut ids = normalize_pubmed_search_response(&search_payload)
        .map_err(|_| ProviderRuntimeError::Decode)?;
    ids.truncate(limit_for(input).min(200));
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    runtime.check_cancelled()?;

    let mut summary_url = runtime
        .endpoints()
        .pubmed()
        .join("esummary.fcgi")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    {
        let mut query = summary_url.query_pairs_mut();
        query.append_pair("db", "pubmed");
        query.append_pair("id", &ids.join(","));
        query.append_pair("retmode", "json");
        if let Some(api_key) = runtime
            .access()
            .value(super::ProviderId::PubMed, super::ProviderField::ApiKey)
        {
            query.append_pair("api_key", api_key);
        }
    }
    let summary_payload = runtime.get_text(
        runtime
            .client()
            .get(summary_url)
            .header("Accept", "application/json"),
    )?;
    normalize_pubmed_summary_response(&summary_payload).map_err(|_| ProviderRuntimeError::Decode)
}
