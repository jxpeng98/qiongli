use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, FulltextCandidate, LiteratureResult, ProviderError, SearchInput, limit_for,
    normalize_doi, string_identifiers,
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
    #[serde(default)]
    best_oa_location: Value,
    #[serde(default)]
    locations: Value,
    #[serde(default)]
    abstract_inverted_index: Value,
    #[serde(default)]
    has_content: Value,
    #[serde(default)]
    content_urls: Value,
    #[serde(default)]
    ids: Value,
}

#[derive(Debug, Deserialize)]
struct OpenAlexLocation {
    source: Option<OpenAlexSource>,
    landing_page_url: Option<String>,
    version: Option<String>,
    #[serde(default)]
    pdf_url: Value,
    #[serde(default)]
    license: Value,
    #[serde(default)]
    is_oa: Value,
}

fn reconstruct_abstract(index: &Value) -> Option<String> {
    let mut words = BTreeMap::new();
    for (word, positions) in index.as_object()? {
        if word.trim().is_empty() {
            return None;
        }
        for position in positions.as_array()? {
            let position = position.as_u64()?;
            // Bound malformed sparse indexes; never allocate by an untrusted offset.
            if position >= 32_768 || words.insert(position, word.as_str()).is_some() {
                return None;
            }
        }
    }
    if words.is_empty() || words.keys().copied().ne(0..words.len() as u64) {
        return None;
    }
    Some(words.into_values().collect::<Vec<_>>().join(" "))
}

fn fulltext_candidates(work: &OpenAlexWork) -> Vec<FulltextCandidate> {
    let mut candidates = Vec::new();
    let mut add_location = |location: &OpenAlexLocation| {
        for (url, format) in [
            (location.pdf_url.as_str(), Some("pdf")),
            (
                location
                    .landing_page_url
                    .as_deref()
                    .filter(|_| location.is_oa.as_bool() == Some(true)),
                None,
            ),
        ] {
            if let Some(candidate) = url.and_then(|url| {
                FulltextCandidate::new(
                    url,
                    "openalex",
                    format,
                    location.version.as_deref(),
                    location.license.as_str(),
                )
            }) && !candidates.contains(&candidate)
            {
                candidates.push(candidate);
            }
        }
    };
    if let Ok(location) = serde_json::from_value::<OpenAlexLocation>(work.best_oa_location.clone())
    {
        add_location(&location);
    }
    if let Some(location) = &work.primary_location {
        add_location(location);
    }
    for location in work.locations.as_array().into_iter().flatten() {
        if let Ok(location) = serde_json::from_value::<OpenAlexLocation>(location.clone()) {
            add_location(&location);
        }
    }
    for (key, format) in [("pdf", "pdf"), ("grobid_xml", "tei_xml")] {
        if work.has_content[key].as_bool() == Some(true)
            && let Some(candidate) = work.content_urls[key]
                .as_str()
                .and_then(|url| FulltextCandidate::new(url, "openalex", Some(format), None, None))
            && !candidates.contains(&candidate)
        {
            candidates.push(candidate);
        }
    }
    candidates
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
            let fulltext_candidates = fulltext_candidates(&work);
            let abstract_text = reconstruct_abstract(&work.abstract_inverted_index);
            let external_ids = string_identifiers(Some(&work.ids));
            let title = work.display_name?;
            Some(LiteratureResult {
                title,
                abstract_text,
                fulltext_candidates,
                external_ids,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_abstract_locations_and_reported_content_without_borrowing_license() {
        let location = json!({"pdf_url":"https://repo.example/paper.pdf", "version":"acceptedVersion", "license":"cc-by"});
        let records = normalize_openalex_response(&json!({"results":[{
            "display_name":"Paper", "id":"https://openalex.org/W1",
            "ids":{"openalex":"https://openalex.org/W1", "pmcid":"https://pmc.ncbi.nlm.nih.gov/articles/PMC1/", "mag":123},
            "abstract_inverted_index":{"evidence":[1,3], "An":[0], "with":[2]},
            "best_oa_location":location, "locations":[location, {"pdf_url":"file:///tmp/private.pdf"}, {"pdf_url":42}],
            "has_content":{"pdf":true,"grobid_xml":true},
            "content_urls":{"pdf":"https://content.openalex.org/works/W1.pdf", "grobid_xml":"https://content.openalex.org/works/W1.grobid-xml"}
        }]}).to_string()).unwrap();
        let record = &records[0];
        assert_eq!(
            record.abstract_text.as_deref(),
            Some("An evidence with evidence")
        );
        assert_eq!(record.fulltext_candidates.len(), 3);
        assert_eq!(
            record.fulltext_candidates[0].license.as_deref(),
            Some("cc-by")
        );
        assert_eq!(
            record.fulltext_candidates[0].version.as_deref(),
            Some("acceptedVersion")
        );
        assert_eq!(
            record.fulltext_candidates[2].format.as_deref(),
            Some("tei_xml")
        );
        assert!(record.fulltext_candidates[1].license.is_none());
        assert!(record.fulltext_candidates[2].version.is_none());
        assert!(!record.external_ids.contains_key("mag"));
    }

    #[test]
    fn malformed_or_sparse_abstracts_are_not_invented_and_unavailable_content_is_not_a_lead() {
        for index in [
            json!(null),
            json!({}),
            json!({"A":[1]}),
            json!({"A":[0],"B":[0]}),
            json!({"A":[-1]}),
            json!({"A":[0, 1.5]}),
            json!({"A":[18446744073709551615_u64]}),
            json!({"A":"zero"}),
        ] {
            assert!(reconstruct_abstract(&index).is_none(), "{index}");
        }
        let records = normalize_openalex_response(&json!({"results":[{
            "display_name":"Paper", "abstract_inverted_index":{"A":[0]},
            "has_content":{"pdf":false}, "content_urls":{"pdf":"https://content.openalex.org/works/W1.pdf"},
            "best_oa_location":false, "locations":{}, "primary_location":{"pdf_url":42,"license":[]}
        }]}).to_string()).unwrap();
        assert!(records[0].fulltext_candidates.is_empty());
        assert_eq!(records[0].abstract_text.as_deref(), Some("A"));
    }
}
