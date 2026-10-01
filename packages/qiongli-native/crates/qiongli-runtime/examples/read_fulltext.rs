//! Public-source smoke probe, without provider credentials or project writes.
//! cargo run -p qiongli-runtime --example read_fulltext -- https://…
use qiongli_runtime::fulltext::{FulltextReader, FulltextRequest};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .ok_or("supply a public HTTPS PDF, TEI or JATS URL")?;
    let reader = FulltextReader::default();
    let request = FulltextRequest::from_arguments(&json!({"url": url, "limit": 1}))?;
    let first = serde_json::to_value(reader.read(&request, None)?)?;
    println!("{}", serde_json::to_string(&first)?);
    if let Some(offset) = first.get("next_offset") {
        let next = FulltextRequest::from_arguments(&json!({"url": url, "offset": offset,
            "limit": 1, "expected_sha256": first["source_sha256"]}))?;
        println!("{}", serde_json::to_string(&reader.read(&next, None)?)?);
    }
    Ok(())
}
