use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ProjectError;
use crate::json::parse_unique_json;
use crate::model::valid_lower_hex;
use crate::paper_note::valid_note_path;
use crate::storage::{read_stage_handoff_file, sha256_bytes};

/// Reviewed raw retrieval results. Storage validates bytes, not scholarly provenance.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcePacketDraftV1 {
    pub schema_version: u32,
    pub citekey: String,
    pub content: String,
}

impl SourcePacketDraftV1 {
    pub fn read_file(path: &Path) -> Result<Self, ProjectError> {
        let text = read_stage_handoff_file(path)?;
        let value =
            parse_unique_json(text.as_bytes()).map_err(|_| ProjectError::InvalidProjectDocument)?;
        let draft: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidProjectDocument)?;
        draft.validate()?;
        Ok(draft)
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self.content.len() > 4 * 1024 * 1024 {
            return Err(ProjectError::DocumentTooLarge);
        }
        if self.schema_version != 1 || !valid_note_path(&format!("notes/{}.md", self.citekey)) {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let value = parse_unique_json(self.content.as_bytes())
            .map_err(|_| ProjectError::InvalidProjectDocument)?;
        match value {
            serde_json::Value::Object(fields) if !fields.is_empty() => Ok(()),
            serde_json::Value::Array(items) if !items.is_empty() => Ok(()),
            _ => Err(ProjectError::InvalidProjectDocument),
        }
    }

    /// Content-addressed, create-only; later retrievals never rewrite previous packets.
    #[must_use]
    pub fn relative_path(&self) -> String {
        format!(
            "sources/{}/{}.json",
            self.citekey,
            sha256_bytes(self.content.as_bytes())
        )
    }
}

pub(crate) fn valid_packet_path(path: &str) -> bool {
    let Some((citekey, file)) = path
        .strip_prefix("sources/")
        .and_then(|p| p.split_once('/'))
    else {
        return false;
    };
    valid_note_path(&format!("notes/{citekey}.md"))
        && file
            .strip_suffix(".json")
            .is_some_and(|hash| valid_lower_hex(hash, 64))
}
