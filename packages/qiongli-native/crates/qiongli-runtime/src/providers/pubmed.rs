use serde::Deserialize;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, FulltextCandidate, LiteratureResult, ProviderError, SearchInput, limit_for,
    normalize_doi, year_from_text,
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
            .iter()
            .find(|id| id.idtype.as_deref() == Some("doi"))
            .and_then(|id| id.value.as_deref())
            .and_then(normalize_doi);
        let external_ids = article
            .articleids
            .iter()
            .filter_map(|id| {
                Some((
                    id.idtype.clone()?,
                    id.value.clone().filter(|s| !s.trim().is_empty())?,
                ))
            })
            .collect();
        let fulltext_candidates = article.articleids.iter()
            .filter(|id| id.idtype.as_deref() == Some("pmc"))
            .filter_map(|id| id.value.as_deref())
            .filter_map(|id| id.strip_prefix("PMC"))
            .filter(|digits| !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()))
            .filter_map(|digits| FulltextCandidate::new(
                &format!("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pmc&id={digits}&retmode=xml"),
                "pubmed", Some("jats_xml"), None, None))
            .collect();
        records.push(LiteratureResult {
            title,
            external_ids,
            fulltext_candidates,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn summary_keeps_reported_ids_and_resolves_only_valid_pmc_leads() {
        let records = normalize_pubmed_summary_response(&json!({"result":{
            "uids":["123", "124"],
            "123":{"title":"Paper", "articleids":[{"idtype":"doi","value":"10.1000/TEST"},
                {"idtype":"pmc","value":"PMC456"}, {"idtype":"pubmed","value":"123"}]},
            "124":{"title":"No fulltext", "articleids":[{"idtype":"pmc","value":"PMC456&api_key=secret"}]}
        }}).to_string()).unwrap();
        assert_eq!(records[0].doi.as_deref(), Some("10.1000/test"));
        assert_eq!(records[0].external_ids["pmc"], "PMC456");
        assert_eq!(
            records[0].fulltext_candidates[0].url,
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pmc&id=456&retmode=xml"
        );
        assert_eq!(
            records[0].fulltext_candidates[0].format.as_deref(),
            Some("jats_xml")
        );
        assert!(records[0].abstract_text.is_none());
        assert!(records[1].fulltext_candidates.is_empty());
    }
}
