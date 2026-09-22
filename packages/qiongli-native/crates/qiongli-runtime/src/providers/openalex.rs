use serde::Deserialize;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, LiteratureResult, ProviderError, SearchInput, limit_for, normalize_doi,
};

#[derive(Debug, Deserialize)]
struct OpenAlexResponse {
    #[serde(default)]
    results: Vec<OpenAlexWork>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexWork {
    display_name: Option<String>,
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    publication_date: Option<String>,
    #[serde(default)]
    authorships: Vec<serde_json::Value>,
    biblio: Option<serde_json::Value>,
    publication_year: Option<i64>,
    doi: Option<String>,
    primary_location: Option<OpenAlexLocation>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexLocation {
    source: Option<OpenAlexSource>,
    landing_page_url: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexSource {
    display_name: Option<String>,
    #[serde(rename = "type")]
    source_type: Option<String>,
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
                source_id: work.id.clone(),
                url: work
                    .primary_location
                    .as_ref()
                    .and_then(|l| l.landing_page_url.clone())
                    .or(work.id),
                authors: work
                    .authorships
                    .iter()
                    .filter_map(|a| a["author"]["display_name"].as_str())
                    .map(|a| Author::literal(a.to_owned()))
                    .collect(),
                record_type: match work.record_type.as_deref() {
                    Some("article") => match work.primary_location.as_ref() {
                        Some(location)
                            if location.version.as_deref() == Some("submittedVersion") =>
                        {
                            Some("preprint".into())
                        }
                        Some(location) => match location
                            .source
                            .as_ref()
                            .and_then(|s| s.source_type.as_deref())
                        {
                            Some("journal") => Some("article-journal".into()),
                            Some("conference") => Some("paper-conference".into()),
                            _ => work.record_type,
                        },
                        _ => work.record_type,
                    },
                    _ => work.record_type,
                },
                published_date: work.publication_date,
                volume: work
                    .biblio
                    .as_ref()
                    .and_then(|b| b["volume"].as_str())
                    .map(str::to_owned),
                issue: work
                    .biblio
                    .as_ref()
                    .and_then(|b| b["issue"].as_str())
                    .map(str::to_owned),
                pages: work.biblio.as_ref().and_then(|b| {
                    b["first_page"]
                        .as_str()
                        .map(|first| match b["last_page"].as_str() {
                            Some(last) if last != first => format!("{first}-{last}"),
                            _ => first.to_owned(),
                        })
                }),
                doi: work.doi.as_deref().and_then(normalize_doi),
                year: work.publication_year,
                venue: work
                    .primary_location
                    .and_then(|location| location.source)
                    .and_then(|source| source.display_name),
                provider: "openalex".to_string(),
                providers: vec!["openalex".to_string()],
                ..Default::default()
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
        if input.search_mode.as_deref() == Some("doi") {
            let doi = normalize_doi(&input.query).ok_or(ProviderRuntimeError::InvalidEndpoint)?;
            query.append_pair("filter", &format!("doi:https://doi.org/{doi}"));
        } else {
            query.append_pair("search", &input.query);
        }
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
