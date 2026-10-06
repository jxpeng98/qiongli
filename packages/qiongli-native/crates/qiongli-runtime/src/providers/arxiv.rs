use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::runtime::{ProviderRuntime, ProviderRuntimeError};
use super::search::{
    Author, FulltextCandidate, LiteratureResult, ProviderError, SearchInput, clean_text, limit_for,
    normalize_doi, year_from_text,
};

#[derive(Default)]
struct ArxivEntry {
    id: Option<String>,
    title: Option<String>,
    published: Option<String>,
    doi: Option<String>,
    journal_ref: Option<String>,
    authors: Vec<Author>,
    summary: Option<String>,
    pdf_urls: Vec<String>,
}

fn pdf_link(event: &BytesStart<'_>, entry: &mut ArxivEntry) -> Result<(), ProviderError> {
    if event.local_name().as_ref() != b"link" {
        return Ok(());
    }
    let mut url = None;
    let mut is_pdf = false;
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|_| ProviderError::Xml)?;
        let value = attribute.unescape_value().map_err(|_| ProviderError::Xml)?;
        match attribute.key.as_ref() {
            b"href" => url = Some(value.into_owned()),
            b"type" if value == "application/pdf" => is_pdf = true,
            b"title" if value == "pdf" => is_pdf = true,
            _ => {}
        }
    }
    if is_pdf && let Some(url) = url {
        entry.pdf_urls.push(url);
    }
    Ok(())
}

pub fn normalize_arxiv_response(payload: &str) -> Result<Vec<LiteratureResult>, ProviderError> {
    let mut reader = Reader::from_str(payload);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current = None::<ArxivEntry>;
    let mut current_field = None::<Vec<u8>>;
    let mut results = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                let name = event.name().as_ref().to_vec();
                if local_name(&name) == b"entry" {
                    current = Some(ArxivEntry::default());
                } else if let Some(entry) = &mut current {
                    pdf_link(&event, entry)?;
                    current_field = Some(name);
                }
            }
            Event::Empty(event) => {
                if let Some(entry) = &mut current {
                    pdf_link(&event, entry)?;
                }
            }
            Event::Text(event) => {
                if let (Some(entry), Some(field)) = (&mut current, &current_field) {
                    let text = event.unescape()?.into_owned();
                    match local_name(field) {
                        b"name" => entry.authors.push(Author::literal(clean_text(&text))),
                        b"id" => entry.id = Some(clean_text(&text)),
                        b"title" => entry.title = Some(clean_text(&text)),
                        b"published" => entry.published = Some(clean_text(&text)),
                        b"doi" => entry.doi = Some(clean_text(&text)),
                        b"journal_ref" => entry.journal_ref = Some(clean_text(&text)),
                        b"summary" => entry.summary = Some(clean_text(&text)),
                        _ => {}
                    }
                }
            }
            Event::End(event) => {
                let name = event.name().as_ref().to_vec();
                if local_name(&name) == b"entry"
                    && let Some(entry) = current.take()
                    && let Some(title) = entry.title
                {
                    results.push(LiteratureResult {
                        title,
                        abstract_text: entry.summary.filter(|s| !s.is_empty()),
                        fulltext_candidates: entry
                            .pdf_urls
                            .iter()
                            .filter_map(|url| {
                                FulltextCandidate::new(
                                    url,
                                    "arxiv",
                                    Some("pdf"),
                                    Some("submittedVersion"),
                                    None,
                                )
                            })
                            .collect(),
                        source_id: entry.id.clone(),
                        url: entry.id,
                        authors: entry.authors,
                        record_type: Some("preprint".to_owned()),
                        published_date: entry.published.clone(),
                        doi: entry.doi.as_deref().and_then(normalize_doi),
                        year: entry.published.as_deref().and_then(year_from_text),
                        venue: entry.journal_ref,
                        provider: "arxiv".to_string(),
                        providers: vec!["arxiv".to_string()],
                        ..Default::default()
                    });
                }
                current_field = None;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(results)
}

pub fn search_arxiv(
    runtime: &ProviderRuntime,
    input: &SearchInput,
) -> Result<Vec<LiteratureResult>, ProviderRuntimeError> {
    let mut url = runtime
        .endpoints()
        .arxiv()
        .join("api/query")
        .map_err(|_| ProviderRuntimeError::InvalidEndpoint)?;
    {
        let mut query = url.query_pairs_mut();
        let expression = if input.search_mode.as_deref() == Some("title") {
            format!("ti:\"{}\"", input.query.replace('"', " "))
        } else {
            format!("all:{}", input.query)
        };
        query.append_pair("search_query", &expression);
        query.append_pair("start", "0");
        query.append_pair("max_results", &limit_for(input).min(200).to_string());
        query.append_pair("sortBy", "relevance");
        query.append_pair("sortOrder", "descending");
    }
    let payload = runtime.get_text(
        runtime
            .client()
            .get(url)
            .header("Accept", "application/atom+xml"),
    )?;
    normalize_arxiv_response(&payload).map_err(|_| ProviderRuntimeError::Decode)
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_and_actual_pdf_links_retain_preprint_version_identity() {
        let records = normalize_arxiv_response(r#"<feed xmlns="http://www.w3.org/2005/Atom"><entry>
            <id>https://arxiv.org/abs/2601.12345v2</id><title>Paper</title>
            <summary>Summary &amp; evidence only.</summary>
            <link rel="alternate" href="https://arxiv.org/abs/2601.12345v2"/>
            <link title="pdf" type="application/pdf" href="https://arxiv.org/pdf/2601.12345v2"/>
            <link title="pdf" href="javascript:alert(1)"/>
            </entry><entry><id>https://arxiv.org/abs/2601.45678</id><title>No PDF link</title></entry></feed>"#).unwrap();
        assert_eq!(
            records[0].abstract_text.as_deref(),
            Some("Summary & evidence only.")
        );
        assert_eq!(records[0].fulltext_candidates.len(), 1);
        assert_eq!(
            records[0].fulltext_candidates[0].url,
            "https://arxiv.org/pdf/2601.12345v2"
        );
        assert_eq!(
            records[0].fulltext_candidates[0].version.as_deref(),
            Some("submittedVersion")
        );
        assert!(records[1].fulltext_candidates.is_empty());
    }
}
