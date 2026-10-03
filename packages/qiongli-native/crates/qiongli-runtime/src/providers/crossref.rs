use serde::Deserialize;
use serde_json::Value;

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, FulltextCandidate, LiteratureResult, ProviderError, SearchInput, clean_text, limit_for,
    normalize_doi,
};

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CrossrefMessage {
    Many { items: Vec<CrossrefWork> },
    One(Box<CrossrefWork>),
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
    #[serde(default, rename = "abstract")]
    abstract_text: Value,
    #[serde(default)]
    link: Value,
    #[serde(default)]
    license: Value,
}

fn abstract_text(value: &Value) -> Option<String> {
    let raw = value.as_str()?;
    let mut reader = quick_xml::Reader::from_str(raw);
    let mut text = String::new();
    loop {
        match reader.read_event().ok()? {
            quick_xml::events::Event::Text(event) => text.push_str(&event.unescape().ok()?),
            quick_xml::events::Event::CData(event) => {
                text.push_str(std::str::from_utf8(&event).ok()?)
            }
            quick_xml::events::Event::End(event)
                if matches!(event.local_name().as_ref(), b"p" | b"title") =>
            {
                text.push(' ')
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    let text = clean_text(&text);
    (!text.is_empty()).then_some(text)
}

fn fulltext_candidates(links: &Value, licenses: &Value) -> Vec<FulltextCandidate> {
    links
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|link| {
            let format = match link["content-type"].as_str() {
                Some("application/pdf") => Some("pdf"),
                Some("application/jats+xml" | "application/vnd.jats+xml") => Some("jats_xml"),
                Some("text/html" | "application/xhtml+xml") => Some("html"),
                _ => None,
            };
            let version = link["content-version"].as_str();
            let matching: Vec<_> = licenses
                .as_array()
                .into_iter()
                .flatten()
                .filter(|license| {
                    version.is_some() && license["content-version"].as_str() == version
                })
                .filter_map(|license| license["URL"].as_str())
                .map(Some)
                .collect();
            let matching = if matching.is_empty() {
                vec![None]
            } else {
                matching
            };
            matching.into_iter().filter_map(move |license| {
                FulltextCandidate::new(link["URL"].as_str()?, "crossref", format, version, license)
            })
        })
        .collect()
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
        CrossrefMessage::One(work) => vec![*work],
    };
    Ok(works
        .into_iter()
        .filter_map(|work| {
            let title = work.title.into_iter().next()?;
            Some(LiteratureResult {
                title,
                abstract_text: abstract_text(&work.abstract_text),
                fulltext_candidates: fulltext_candidates(&work.link, &work.license),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_reported_tdm_links_and_jats_abstract_without_guessing_pdf_from_doi() {
        let records = normalize_crossref_response(&json!({"message":{"items":[
            {"title":["Paper"], "DOI":"10.1000/test", "abstract":"<jats:p>An <jats:italic>abstract</jats:italic> &amp; evidence.</jats:p>",
             "license":[{"URL":"https://license.example/vor", "content-version":"vor"},
                 {"URL":"https://license.example/am", "content-version":"am"}],
             "link":[{"URL":"https://publisher.example/full.pdf","content-type":"application/pdf","content-version":"vor"},
                 {"URL":"ftp://publisher.example/full.pdf","content-type":"application/pdf"},
                 {"URL":42}]},
            {"title":["Metadata only"], "DOI":"10.1000/other", "abstract":false, "link":null}
        ]}}).to_string()).unwrap();
        assert_eq!(
            records[0].abstract_text.as_deref(),
            Some("An abstract & evidence.")
        );
        assert_eq!(records[0].fulltext_candidates.len(), 1);
        assert_eq!(
            records[0].fulltext_candidates[0].version.as_deref(),
            Some("vor")
        );
        assert_eq!(
            records[0].fulltext_candidates[0].license.as_deref(),
            Some("https://license.example/vor")
        );
        assert!(records[1].fulltext_candidates.is_empty());
        assert!(records[1].abstract_text.is_none());
    }
}
