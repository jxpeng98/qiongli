use qiongli_lite_mcp::providers::search::LiteratureResult;
use qiongli_lite_mcp::zotero::export::export_import_files;

#[test]
fn export_import_files_includes_ris_bibtex_csl_and_report() {
    let files = export_import_files(vec![LiteratureResult {
        title: "A Test Paper".to_string(),
        doi: Some("10.1234/example".to_string()),
        year: Some(2025),
        venue: Some("Journal of Tests".to_string()),
        provider: "openalex".to_string(),
        providers: vec!["openalex".to_string()],
    }]);

    assert!(files.contains_key("references.json"));
    assert!(files.contains_key("references.ris"));
    assert!(files.contains_key("bibliography.bib"));
    assert!(files.contains_key("zotero-import-report.md"));
    assert!(files["references.ris"].contains("TY  - JOUR"));
    assert!(files["bibliography.bib"].contains("@article"));
}

