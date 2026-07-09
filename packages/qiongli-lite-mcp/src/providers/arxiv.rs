use quick_xml::events::Event;
use quick_xml::Reader;

use crate::providers::runtime::{ProviderRuntime, ProviderRuntimeError};
use crate::providers::search::{
    clean_text, limit_for, normalize_doi, year_from_text, LiteratureResult, ProviderError,
    SearchInput,
};

#[derive(Default)]
struct ArxivEntry {
    id: Option<String>,
    title: Option<String>,
    published: Option<String>,
    doi: Option<String>,
    journal_ref: Option<String>,
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
                } else if current.is_some() {
                    current_field = Some(name);
                }
            }
            Event::Text(event) => {
                if let (Some(entry), Some(field)) = (&mut current, &current_field) {
                    let text = event.unescape()?.into_owned();
                    match local_name(field) {
                        b"id" => entry.id = Some(clean_text(&text)),
                        b"title" => entry.title = Some(clean_text(&text)),
                        b"published" => entry.published = Some(clean_text(&text)),
                        b"doi" => entry.doi = Some(clean_text(&text)),
                        b"journal_ref" => entry.journal_ref = Some(clean_text(&text)),
                        _ => {}
                    }
                }
            }
            Event::End(event) => {
                let name = event.name().as_ref().to_vec();
                if local_name(&name) == b"entry" {
                    if let Some(entry) = current.take() {
                        if let Some(title) = entry.title {
                            results.push(LiteratureResult {
                                title,
                                doi: entry.doi.as_deref().and_then(normalize_doi),
                                year: entry.published.as_deref().and_then(year_from_text),
                                venue: entry.journal_ref,
                                provider: "arxiv".to_string(),
                                providers: vec!["arxiv".to_string()],
                            });
                        }
                    }
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
        query.append_pair("search_query", &format!("all:{}", input.query));
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
