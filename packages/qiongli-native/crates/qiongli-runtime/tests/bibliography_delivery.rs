use qiongli_runtime::zotero::export::{ZoteroExportRequest, export_selected_import_files};
use serde_json::json;

#[test]
fn exports_preserve_authors_type_links_and_existing_keys() {
    let request = ZoteroExportRequest::from_arguments(&json!({"records": [{
        "title": "Verified Conference Paper", "year": 2017,
        "authors": [{"family": "García", "given": "Ana"}, {"literal": "Research & Development Group"}],
        "record_type": "paper-conference", "venue": "Example Proceedings",
        "pages": "10-19", "volume": "30", "publisher": "Example Press",
        "url": "https://example.org/paper", "source_id": "source:paper",
        "citekey": "garcia2017paper", "provider": "user_corpus", "providers": ["user_corpus"]
    }]})).unwrap();
    let files = export_selected_import_files(request).unwrap();
    let bib = &files["bibliography.bib"];
    assert!(bib.contains("@inproceedings{garcia2017paper,"));
    assert!(bib.contains("García, Ana and {Research \\& Development Group}"));
    assert!(bib.contains("booktitle = {Example Proceedings}"));
    assert!(bib.contains("pages = {10-19}"));
    assert!(bib.contains("url = {https://example.org/paper}"));
    let csl: serde_json::Value = serde_json::from_str(&files["references.json"]).unwrap();
    assert_eq!(csl[0]["id"], "garcia2017paper");
    assert_eq!(csl[0]["author"][0]["family"], "García");
    assert_eq!(csl[0]["type"], "paper-conference");
    assert!(files["references.ris"].contains("TY  - CPAPER"));
    assert!(files["references.ris"].contains("AU  - García, Ana"));
}

fn files(records: serde_json::Value) -> std::collections::BTreeMap<String, String> {
    export_selected_import_files(
        ZoteroExportRequest::from_arguments(&json!({"records":records})).unwrap(),
    )
    .unwrap()
}

#[test]
fn keys_survive_order_duplicates_and_explicit_aliases() {
    let a = json!({"title":"First", "provider":"crossref", "doi":"https://doi.org/10.1234/A"});
    let b = json!({"title":"Second", "provider":"arxiv", "source_id":"arxiv:1234.56789v1"});
    let first: serde_json::Value =
        serde_json::from_str(&files(json!([a, b]))["references.json"]).unwrap();
    let reordered: serde_json::Value =
        serde_json::from_str(&files(json!([b, a, a]))["references.json"]).unwrap();
    assert_eq!(first[0]["id"], reordered[1]["id"]);
    assert_eq!(first[1]["id"], reordered[0]["id"]);
    assert_eq!(reordered.as_array().unwrap().len(), 2);
    let mut alias = a.clone();
    alias["citekey"] = json!("existingKey");
    let mut other = a.clone();
    other["citekey"] = json!("otherKey");
    let aliases = files(json!([alias, other, alias]));
    assert!(aliases["bibliography.bib"].contains("existingKey"));
    assert!(aliases["bibliography.bib"].contains("otherKey"));
    assert!(aliases["zotero-import-report.md"].contains("conflicts [citekey]"));
    for record in [
        json!({"title":"Bad","provider":"user_corpus","citekey":"x}, injected"}),
        json!("string record"),
        json!({"title":"Bad","provider":"user_corpus","authors":[{"literal":"Group","given":"A"}]}),
    ] {
        assert!(ZoteroExportRequest::from_arguments(&json!({"records":[record]})).is_err());
    }
}

#[test]
fn generic_and_conflicting_metadata_are_visible_without_invented_fields() {
    let exported = files(json!([
        {"title":"Same", "provider":"crossref", "doi":"10.1234/same", "year":2017},
        {"title":"Same", "provider":"openalex", "doi":"10.1234/same", "year":2025}
    ]));
    assert!(exported["bibliography.bib"].starts_with("@misc"));
    assert!(!exported["bibliography.bib"].contains("author ="));
    assert!(exported["zotero-import-report.md"].contains("conflicts [year]"));
    assert!(exported["zotero-import-report.md"].contains("missing [authors"));
}

#[test]
fn exact_identity_filters_reject_wrong_year_and_rank_before_truncation() {
    use qiongli_runtime::providers::search::{
        LiteratureResult, SearchInput, SearchRequest, select_results,
    };
    let records: Vec<LiteratureResult> = serde_json::from_value(json!([
        {"title":"Attention Is All You Need: Commentary", "year":2017,"provider":"openalex"},
        {"title":"Attention Is All You Need", "year":2025,"provider":"openalex"},
        {"title":"Attention Is All You Need", "year":2017,"venue":"NeurIPS","doi":"10.1234/target","provider":"crossref"},
        {"title":"Attention Is All You Need", "provider":"arxiv"}
    ])).unwrap();
    let input = SearchInput {
        query: "Attention Is All You Need".into(),
        search_mode: Some("title".into()),
        from_year: Some(2017),
        to_year: Some(2017),
        venue_filter: Some("neurips".into()),
        ..Default::default()
    };
    let result = select_results(records.clone(), &input);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].year, Some(2017));
    let topic = select_results(
        records.clone(),
        &SearchInput {
            query: input.query,
            ..Default::default()
        },
    );
    assert_eq!(topic[0].title, "Attention Is All You Need");
    let doi = select_results(
        records,
        &SearchInput {
            query: "https://doi.org/10.1234/TARGET".into(),
            search_mode: Some("doi".into()),
            ..Default::default()
        },
    );
    assert_eq!(doi.len(), 1);
    for args in [
        json!({"query":"x","search_mode":"doi"}),
        json!({"query":"x","from_year":2025,"to_year":2017}),
        json!({"query":"x","from_year":"2017"}),
        json!({"query":"x","venue_filter":""}),
    ] {
        assert!(SearchRequest::from_arguments(&args).is_err());
    }
}

#[test]
fn registry_metadata_survives_export_and_openalex_article_is_not_assumed_journal() {
    use qiongli_runtime::providers::{
        arxiv::normalize_arxiv_response, crossref::normalize_crossref_response,
        openalex::normalize_openalex_response,
    };
    let records = normalize_crossref_response(&json!({"message":{
        "title":["Conference"],"DOI":"10.1234/test","type":"proceedings-article",
        "author":[{"family":"García","given":"Ana"},{"name":"Example Consortium"}],
        "issued":{"date-parts":[[2017,12,4]]},"page":"10-19","volume":"30","publisher":"Example",
        "container-title":["Proceedings"],"URL":"https://example.org/conference"
    }}).to_string()).unwrap();
    assert_eq!(records[0].published_date.as_deref(), Some("2017-12-04"));
    let exported = files(serde_json::to_value(records).unwrap());
    assert!(exported["bibliography.bib"].contains("@inproceedings"));
    assert!(exported["bibliography.bib"].contains("date = {2017-12-04}"));
    let works = normalize_openalex_response(&json!({"results":[
        {"id":"https://openalex.org/W1","display_name":"Conference","type":"article","primary_location":{"source":{"type":"conference"}}},
        {"display_name":"Unknown","type":"article"}
    ]}).to_string()).unwrap();
    assert_eq!(works[0].record_type.as_deref(), Some("paper-conference"));
    assert!(
        files(serde_json::to_value(&works[1..]).unwrap())["bibliography.bib"].starts_with("@misc")
    );
    let preprints = normalize_arxiv_response(r#"<feed><entry><id>https://arxiv.org/abs/1706.03762v1</id><title>Attention Is All You Need</title><author><name>Ashish Vaswani</name></author><published>2017-06-12T17:57:34Z</published></entry></feed>"#).unwrap();
    assert_eq!(
        preprints[0].authors[0].literal.as_deref(),
        Some("Ashish Vaswani")
    );
    assert_eq!(
        preprints[0].source_id.as_deref(),
        Some("https://arxiv.org/abs/1706.03762v1")
    );
}

#[test]
fn preprint_and_published_versions_keep_distinct_entries_and_keys() {
    let exported = files(json!([
        {"title":"Same work","provider":"arxiv","year":2024,"doi":"10.1234/final","record_type":"preprint","source_id":"https://arxiv.org/abs/2401.12345v1"},
        {"title":"Same work","provider":"crossref","year":2024,"doi":"10.1234/final","record_type":"journal-article"},
        {"title":"Without DOI","provider":"user_corpus","year":2024,"record_type":"preprint"},
        {"title":"Without DOI","provider":"user_corpus","year":2024,"record_type":"journal-article"}
    ]));
    let csl: serde_json::Value = serde_json::from_str(&exported["references.json"]).unwrap();
    assert_eq!(csl.as_array().unwrap().len(), 4);
    assert_ne!(csl[0]["id"], csl[1]["id"]);
    assert_ne!(csl[2]["id"], csl[3]["id"]);
}
