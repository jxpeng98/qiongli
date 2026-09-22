use serde::Deserialize;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, LiteratureResult, ProviderError, SearchInput, limit_for, normalize_doi,
};

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CrossrefMessage {
    Many { items: Vec<CrossrefWork> },
    One(CrossrefWork),
}

#[derive(Debug, Deserialize)]
struct CrossrefWork {
    #[serde(default)]
    title: Vec<String>,
    issued: Option<CrossrefIssued>,
    #[serde(default)]
    author: Vec<serde_json::Value>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    #[serde(rename = "URL")]
    url: Option<String>,
    volume: Option<String>,
    issue: Option<String>,
    page: Option<String>,
    #[serde(rename = "article-number")]
    article_number: Option<String>,
    publisher: Option<String>,
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
    let works = match response.message {
        CrossrefMessage::Many { items } => items,
        CrossrefMessage::One(work) => vec![work],
    };
    Ok(works
        .into_iter()
        .filter_map(|work| {
            let title = work.title.into_iter().next()?;
            Some(LiteratureResult {
                title,
                authors: work
                    .author
                    .iter()
                    .map(|author| Author {
                        family: author["family"].as_str().map(str::to_owned),
                        given: author["given"].as_str().map(str::to_owned),
                        literal: author["name"].as_str().map(str::to_owned),
                    })
                    .filter(|a| !a.display_name().is_empty())
                    .collect(),
                source_id: work.doi.clone(),
                url: work.url,
                record_type: work.record_type,
                volume: work.volume,
                issue: work.issue,
                pages: work.page.or(work.article_number),
                publisher: work.publisher,
                doi: work.doi.as_deref().and_then(normalize_doi),
                published_date: work
                    .issued
                    .as_ref()
                    .and_then(|issued| issued.date_parts.first())
                    .map(|parts| {
                        parts
                            .iter()
                            .map(|part| format!("{part:02}"))
                            .collect::<Vec<_>>()
                            .join("-")
                    }),
                year: work.issued.and_then(|issued| {
                    issued
                        .date_parts
                        .first()
                        .and_then(|part| part.first())
                        .copied()
                }),
                venue: work.container_title.into_iter().next(),
                provider: "crossref".to_string(),
                providers: vec!["crossref".to_string()],
                ..Default::default()
            })
        })
        .collect())
}

pub fn search_crossref(
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    let mut url = runtime
        .endpoints()
        .crossref()
        .join("works")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    let lookup = input.search_mode.as_deref() == Some("doi");
    if lookup {
        let doi = normalize_doi(&input.query).ok_or(ProviderRuntimeError::InvalidEndpoint)?;
        url.path_segments_mut()
            .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?
            .push(&doi);
    }
    {
        let mut query = url.query_pairs_mut();
        if !lookup {
            query.append_pair(
                if input.search_mode.as_deref() == Some("title") {
                    "query.title"
                } else {
                    "query"
                },
                &input.query,
            );
            query.append_pair("rows", &limit_for(input).min(200).to_string());
        }
        if let Some(email) = runtime
            .access()
            .value(super::ProviderId::Crossref, super::ProviderField::Email)
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
    normalize_crossref_response(&payload).map_err(|_| ProviderRuntimeError::Decode)
}
