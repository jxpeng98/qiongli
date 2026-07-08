use serde::Deserialize;

use crate::providers::search::{
    limit_for, normalize_doi, year_from_text, LiteratureResult, ProviderError, SearchInput,
};

#[derive(Debug, Deserialize)]
struct PubmedArticle {
    title: Option<String>,
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
            doi,
            year: article.pubdate.as_deref().and_then(year_from_text),
            venue: article.fulljournalname,
            provider: "pubmed".to_string(),
            providers: vec!["pubmed".to_string()],
        });
    }
    Ok(records)
}

pub fn search_pubmed(
    client: &reqwest::blocking::Client,
    base_url: &str,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderError> {
    let payload = client
        .get(base_url)
        .query(&[
            ("db", "pubmed"),
            ("term", input.query.as_str()),
            ("retmode", "json"),
            ("retmax", &limit_for(input).to_string()),
        ])
        .send()?
        .error_for_status()?
        .text()?;
    normalize_pubmed_summary_response(&payload)
}
