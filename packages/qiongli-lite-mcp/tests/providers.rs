use qiongli_lite_mcp::providers::arxiv::normalize_arxiv_response;
use qiongli_lite_mcp::providers::crossref::normalize_crossref_response;
use qiongli_lite_mcp::providers::openalex::normalize_openalex_response;
use qiongli_lite_mcp::providers::pubmed::normalize_pubmed_summary_response;
use qiongli_lite_mcp::providers::semantic_scholar::normalize_semantic_scholar_response;

#[test]
fn openalex_response_normalizes_title_year_and_doi() {
    let fixture =
        include_str!("../../../content/mcp-contracts/fixtures/openalex-search-response.json");
    let results = normalize_openalex_response(fixture).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "A Test Paper");
    assert_eq!(results[0].doi.as_deref(), Some("10.1234/example"));
    assert_eq!(results[0].year, Some(2025));
    assert_eq!(results[0].provider, "openalex");
}

#[test]
fn semantic_scholar_response_normalizes_title_year_and_doi() {
    let fixture = include_str!(
        "../../../content/mcp-contracts/fixtures/semantic-scholar-search-response.json"
    );
    let results = normalize_semantic_scholar_response(fixture).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "A Test Paper");
    assert_eq!(results[0].doi.as_deref(), Some("10.1234/example"));
    assert_eq!(results[0].year, Some(2025));
    assert_eq!(results[0].provider, "semantic_scholar");
}

#[test]
fn crossref_response_normalizes_title_year_and_doi() {
    let fixture =
        include_str!("../../../content/mcp-contracts/fixtures/crossref-search-response.json");
    let results = normalize_crossref_response(fixture).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "A Test Paper");
    assert_eq!(results[0].doi.as_deref(), Some("10.1234/example"));
    assert_eq!(results[0].year, Some(2025));
    assert_eq!(results[0].provider, "crossref");
}

#[test]
fn pubmed_summary_response_normalizes_title_year_and_doi() {
    let fixture =
        include_str!("../../../content/mcp-contracts/fixtures/pubmed-summary-response.json");
    let results = normalize_pubmed_summary_response(fixture).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "A Test Paper");
    assert_eq!(results[0].doi.as_deref(), Some("10.1234/example"));
    assert_eq!(results[0].year, Some(2025));
    assert_eq!(results[0].provider, "pubmed");
}

#[test]
fn arxiv_atom_response_normalizes_title_year_and_url() {
    let fixture =
        include_str!("../../../content/mcp-contracts/fixtures/arxiv-search-response.xml");
    let results = normalize_arxiv_response(fixture).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "A Test Paper");
    assert_eq!(results[0].doi.as_deref(), Some("10.1234/example"));
    assert_eq!(results[0].year, Some(2025));
    assert_eq!(results[0].provider, "arxiv");
}

