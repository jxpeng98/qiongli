use qiongli_lite_mcp::zotero::companion::CompanionClient;

#[test]
fn rejects_non_loopback_connector_url() {
    let result = CompanionClient::new("http://example.com:23119");
    assert!(result.is_err());
}

#[test]
fn accepts_loopback_connector_url() {
    let result = CompanionClient::new("http://127.0.0.1:23119");
    assert!(result.is_ok());
}

