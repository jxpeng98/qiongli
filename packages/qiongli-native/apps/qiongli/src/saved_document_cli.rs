use std::ffi::OsString;

use qiongli_project::SavedDocumentReadRequest;
use serde_json::{Map, Value};

pub(crate) const USAGE: &str = "Saved research document:\n  qiongli project document read --project-id <prj_id> --expected-project-revision <revision> --relative-path <path> --expected-sha256 <sha256> [--offset-bytes <offset>] [--max-bytes <4..65536>]\n\nReads only notes/<citekey>.md, sources/<citekey>/<sha256>.json, or retrieval_manifest.csv. The SHA-256 always binds the entire file; continue with nextOffsetBytes and the same revision/digest. This is a file snapshot, not academic verification or a Graph projection.";

pub(crate) fn parse(args: &[OsString]) -> Result<SavedDocumentReadRequest, &'static str> {
    if args.first().and_then(|arg| arg.to_str()) != Some("read") {
        return Err("expected project document read");
    }
    let mut fields = Map::new();
    let mut pairs = args[1..].chunks_exact(2);
    for pair in &mut pairs {
        let (field, numeric) = match pair[0].to_str() {
            Some("--project-id") => ("project_id", false),
            Some("--expected-project-revision") => ("expected_project_revision", true),
            Some("--relative-path") => ("relative_path", false),
            Some("--expected-sha256") => ("expected_sha256", false),
            Some("--offset-bytes") => ("offset_bytes", true),
            Some("--max-bytes") => ("max_bytes", true),
            _ => return Err("unknown saved-document read option"),
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
    let request: SavedDocumentReadRequest = serde_json::from_value(Value::Object(fields))
        .map_err(|_| "invalid saved-document read arguments")?;
    request
        .validate()
        .map_err(|_| "invalid saved-document read arguments")?;
    Ok(request)
}
