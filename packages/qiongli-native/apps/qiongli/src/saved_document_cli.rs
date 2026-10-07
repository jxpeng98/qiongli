use std::ffi::OsString;

use qiongli_project::{
    SavedDocumentListRequest, SavedDocumentReadRequest, SavedDocumentSearchRequest,
};
use serde_json::{Map, Value};

pub(crate) const USAGE: &str = "Saved research document:\n  qiongli project document read --project-id <prj_id> --expected-project-revision <revision> --relative-path <path> --expected-sha256 <sha256> [--offset-bytes <offset>] [--max-bytes <4..65536>] [--json-pointer <pointer>]\n\nReads only notes/<citekey>.md, sources/<citekey>/<sha256>.json, retrieval_manifest.csv, or context/stage_summaries/STG-*.md. The SHA-256 always binds the entire file; continue with nextOffsetBytes and the same revision/digest. With --json-pointer, select an observed string in a source packet; offsets, content and nextOffsetBytes address decoded UTF-8 text. Keep the same pointer on continuation; sourceSizeBytes and SHA-256 still identify the raw file. Missing or non-string targets refuse. This is a file snapshot, not academic verification or a Graph projection.";

pub(crate) fn parse(args: &[OsString]) -> Result<SavedDocumentReadRequest, &'static str> {
    if args.first().and_then(|arg| arg.to_str()) != Some("read") {
        return Err("expected project document read");
    }
    let fields = parse_fields(&args[1..], "read")?;
    let request: SavedDocumentReadRequest = serde_json::from_value(Value::Object(fields))
        .map_err(|_| "invalid saved-document read arguments")?;
    request
        .validate()
        .map_err(|_| "invalid saved-document read arguments")?;
    Ok(request)
}

pub(crate) const LIST_USAGE: &str = "Saved document bindings:\n  qiongli project document list --project-id <prj_id> --expected-project-revision <revision> [--offset <offset>] [--limit <1..64>] [--expected-bindings-sha256 <sha256>]\n\nLists only receipt-backed paper notes, source packets, retrieval history and stage summaries. Only current entries provide readArguments. Continuation requires bindingsSha256; it identifies saved history, not current file bytes. Missing, changed or unavailable entries never authorize rebinding.";

pub(crate) fn parse_list(args: &[OsString]) -> Result<SavedDocumentListRequest, &'static str> {
    if args.first().and_then(|arg| arg.to_str()) != Some("list") {
        return Err("expected project document list");
    }
    let fields = parse_fields(&args[1..], "list")?;
    let request: SavedDocumentListRequest = serde_json::from_value(Value::Object(fields))
        .map_err(|_| "invalid saved-document list arguments")?;
    request
        .validate()
        .map_err(|_| "invalid saved-document list arguments")?;
    Ok(request)
}

pub(crate) const SEARCH_USAGE: &str = "Saved passage search:\n  qiongli project document search --project-id <prj_id> --expected-project-revision <revision> --relative-path <path> --expected-sha256 <sha256> --search-text <literal> [--json-pointer <pointer>] [--match-offset <offset>] [--max-matches <1..16>] [--context-bytes <4..512>] [--expected-search-sha256 <sha256>]\n\nSearches one explicitly bound saved document, not a directory. Packet string values are decoded; other saved documents use raw UTF-8. Matching is case-sensitive, literal and non-overlapping within each string. context-bytes bounds each side of a match. Continue with nextMatchOffset, searchSha256 and unchanged query/scope/options. Matches provide exact context and readArguments; a match is not academic support and no match does not cover unread paper material.";

pub(crate) fn parse_search(args: &[OsString]) -> Result<SavedDocumentSearchRequest, &'static str> {
    if args.first().and_then(|arg| arg.to_str()) != Some("search") {
        return Err("expected project document search");
    }
    let fields = parse_fields(&args[1..], "search")?;
    let request: SavedDocumentSearchRequest = serde_json::from_value(Value::Object(fields))
        .map_err(|_| "invalid saved-document search arguments")?;
    request
        .validate()
        .map_err(|_| "invalid saved-document search arguments")?;
    Ok(request)
}

fn parse_fields(args: &[OsString], mode: &str) -> Result<Map<String, Value>, &'static str> {
    let mut fields = Map::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        let (field, numeric) = match pair[0].to_str() {
            Some("--project-id") => ("project_id", false),
            Some("--expected-project-revision") => ("expected_project_revision", true),
            Some("--relative-path") if mode != "list" => ("relative_path", false),
            Some("--expected-sha256") if mode != "list" => ("expected_sha256", false),
            Some("--json-pointer") if mode != "list" => ("json_pointer", false),
            Some("--offset-bytes") if mode == "read" => ("offset_bytes", true),
            Some("--max-bytes") if mode == "read" => ("max_bytes", true),
            Some("--offset") if mode == "list" => ("offset", true),
            Some("--limit") if mode == "list" => ("limit", true),
            Some("--expected-bindings-sha256") if mode == "list" => {
                ("expected_bindings_sha256", false)
            }
            Some("--search-text") if mode == "search" => ("search_text", false),
            Some("--match-offset") if mode == "search" => ("match_offset", true),
            Some("--max-matches") if mode == "search" => ("max_matches", true),
            Some("--context-bytes") if mode == "search" => ("context_bytes", true),
            Some("--expected-search-sha256") if mode == "search" => {
                ("expected_search_sha256", false)
            }
            _ => return Err("unknown saved-document option"),
        };
        let text = pair[1]
            .to_str()
            .ok_or("invalid saved-document read value")?;
        let value = if numeric {
            if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err("saved-document read number must be unsigned decimal");
            }
            Value::from(
                text.parse::<u64>()
                    .map_err(|_| "invalid saved-document read number")?,
            )
        } else {
            Value::from(text)
        };
        if fields.insert(field.into(), value).is_some() {
            return Err("duplicate saved-document read option");
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("saved-document read option requires a value");
    }
    Ok(fields)
}
