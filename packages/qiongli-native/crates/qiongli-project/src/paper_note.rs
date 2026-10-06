use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::json::parse_unique_json;
use crate::model::valid_lower_hex;
use crate::stage_summary::{revalidate_sources, valid_source_path};
use crate::storage::read_stage_handoff_file;
use crate::{CaptureId, ProjectError, StageSummarySourceV1};

/// One reviewed addition; an existing note's bytes are never rewritten.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperNoteDraftV1 {
    pub schema_version: u32,
    pub citekey: String,
    pub previous_sha256: Option<String>,
    pub sources: Vec<StageSummarySourceV1>,
    pub markdown: String,
}

impl PaperNoteDraftV1 {
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
        if self.markdown.len() > 4 * 1024 * 1024 {
            return Err(ProjectError::DocumentTooLarge);
        }
        if self.schema_version != 1
            || !valid_note_path(&self.relative_path())
            || self
                .previous_sha256
                .as_deref()
                .is_some_and(|hash| !valid_lower_hex(hash, 64))
            || self.sources.is_empty()
            || self.sources.len() > 64
            || self.markdown.trim().is_empty()
            || self.markdown.contains('\0')
            || self.markdown.contains("<!-- qiongli:")
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let mut paths = Vec::new();
        for source in &self.sources {
            if !valid_source_path(&source.relative_path)
                || !valid_lower_hex(&source.sha256, 64)
                || source.relative_path == self.relative_path()
                || paths.contains(&source.relative_path.as_str())
            {
                return Err(ProjectError::InvalidProjectDocument);
            }
            paths.push(source.relative_path.as_str());
        }
        Ok(())
    }

    pub(crate) fn relative_path(&self) -> String {
        format!("notes/{}.md", self.citekey)
    }

    pub(crate) fn revalidate_sources(&self, root: &Path) -> Result<(), ProjectError> {
        self.validate()?;
        revalidate_sources(root, self.sources.iter())
    }

    pub(crate) fn render(&self, previous: &str, capture: &CaptureId, reviewed_at: u64) -> String {
        let capture = capture.as_str();
        let mut output = previous.to_string();
        if !output.is_empty() {
            output.push_str("\n\n");
        }
        output.push_str(&format!(
            "<!-- qiongli:capture {capture} begin -->\n{}\n\n### Native Note Save Basis\n\n- Citekey: `{}`\n- Capture: `{capture}`\n- Reviewed at (Unix UTC): {reviewed_at}\n- Previous note SHA-256: `{}`\n\n| Source file (project-relative) | Observed SHA-256 |\n|---|---|\n",
            self.markdown, self.citekey, self.previous_sha256.as_deref().unwrap_or("new-note"),
        ));
        for source in &self.sources {
            output.push_str(&format!(
                "| `{}` | `{}` |\n",
                source.relative_path, source.sha256
            ));
        }
        output.push_str(&format!("\n<!-- qiongli:capture {capture} end -->\n"));
        output
    }
}

pub(crate) fn valid_note_path(path: &str) -> bool {
    let Some(key) = path
        .strip_prefix("notes/")
        .and_then(|name| name.strip_suffix(".md"))
    else {
        return false;
    };
    // Keep the same citekey across supported filesystems; never silently rename it.
    let upper = key.to_ascii_uppercase();
    !key.is_empty()
        && key.len() <= 128
        && key.as_bytes()[0].is_ascii_alphanumeric()
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && !matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !((upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.len() == 4
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}
