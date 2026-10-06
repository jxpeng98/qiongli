//! Literal passage discovery over one explicitly bound saved document.
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::json::parse_unique_json;
use crate::model::valid_lower_hex;
use crate::saved_document::valid_json_pointer;
use crate::source_packet::valid_packet_path;
use crate::storage::sha256_bytes;
use crate::{ProjectError, ProjectId, ProjectStateService, SavedDocumentReadRequest};

const fn default_max_matches() -> usize {
    8
}

const fn default_context_bytes() -> usize {
    128
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SavedDocumentSearchRequest {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub relative_path: String,
    pub expected_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<String>,
    pub search_text: String,
    #[serde(default)]
    pub match_offset: u64,
    #[serde(default = "default_max_matches")]
    pub max_matches: usize,
    #[serde(default = "default_context_bytes")]
    pub context_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_search_sha256: Option<String>,
}

impl fmt::Debug for SavedDocumentSearchRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SavedDocumentSearchRequest")
            .field("project_id", &self.project_id)
            .field("expected_project_revision", &self.expected_project_revision)
            .finish_non_exhaustive()
    }
}

impl SavedDocumentSearchRequest {
    pub fn validate(&self) -> Result<(), ProjectError> {
        self.read_request().validate()?;
        if self.search_text.trim().is_empty()
            || self.search_text.len() > 512
            || self.search_text.contains('\0')
            || !(1..=16).contains(&self.max_matches)
            || !(4..=512).contains(&self.context_bytes)
            || (self.match_offset > 0 && self.expected_search_sha256.is_none())
            || self
                .expected_search_sha256
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        Ok(())
    }

    fn read_request(&self) -> SavedDocumentReadRequest {
        SavedDocumentReadRequest {
            project_id: self.project_id.clone(),
            expected_project_revision: self.expected_project_revision,
            relative_path: self.relative_path.clone(),
            expected_sha256: self.expected_sha256.clone(),
            json_pointer: self.json_pointer.clone(),
            offset_bytes: 0,
            max_bytes: 16384,
        }
    }

    fn search_digest(&self) -> Result<String, ProjectError> {
        // Bind the complete query/scope/page policy, not its current cursor.
        let mut basis = self.clone();
        basis.match_offset = 0;
        basis.expected_search_sha256 = None;
        serde_json_canonicalizer::to_vec(&basis)
            .map(|bytes| sha256_bytes(&bytes))
            .map_err(|_| ProjectError::InvalidProjectDocument)
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDocumentMatchV1 {
    pub match_offset_bytes: u64,
    pub match_end_offset_bytes: u64,
    pub context_offset_bytes: u64,
    pub content: String,
    pub truncated_before: bool,
    pub truncated_after: bool,
    pub read_arguments: SavedDocumentReadRequest,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDocumentSearchViewV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub relative_path: String,
    pub sha256: String,
    pub source_size_bytes: u64,
    pub search_sha256: String,
    pub search_scope: String,
    pub total_matches: u64,
    pub scanned_text_fields: u64,
    pub next_match_offset: Option<u64>,
    pub matches: Vec<SavedDocumentMatchV1>,
}

impl fmt::Debug for SavedDocumentSearchViewV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SavedDocumentSearchViewV1")
            .field("project_id", &self.project_id)
            .field("project_revision", &self.project_revision)
            .field("sha256", &self.sha256)
            .field("total_matches", &self.total_matches)
            .finish_non_exhaustive()
    }
}

impl ProjectStateService {
    pub fn search_saved_document(
        &self,
        request: &SavedDocumentSearchRequest,
    ) -> Result<SavedDocumentSearchViewV1, ProjectError> {
        request.validate()?;
        let search_sha256 = request.search_digest()?;
        if request
            .expected_search_sha256
            .as_ref()
            .is_some_and(|expected| expected != &search_sha256)
        {
            return Err(ProjectError::RevisionConflict);
        }
        self.with_saved_document(&request.read_request(), |bytes, digest| {
            let mut scan = PassageScan {
                request,
                total: 0,
                fields: 0,
                matches: Vec::new(),
            };
            let scope;
            if valid_packet_path(&request.relative_path) {
                let packet = parse_unique_json(bytes)
                    .map_err(|_| ProjectError::ProjectArtifactContentInvalid)?;
                if let Some(pointer) = &request.json_pointer {
                    let text = packet
                        .pointer(pointer)
                        .and_then(Value::as_str)
                        .ok_or(ProjectError::InvalidProjectDocument)?;
                    scan.text(text, Some(pointer));
                    scope = "selected_saved_string";
                } else {
                    if !matches!(packet, Value::Object(_) | Value::Array(_)) {
                        return Err(ProjectError::ProjectArtifactContentInvalid);
                    }
                    scan.json(&packet, "")?;
                    scope = "saved_json_string_values";
                }
            } else {
                let text = std::str::from_utf8(bytes)
                    .map_err(|_| ProjectError::ProjectArtifactContentInvalid)?;
                scan.text(text, None);
                scope = "saved_plain_text";
            }
            if request.match_offset > scan.total {
                return Err(ProjectError::InvalidProjectDocument);
            }
            let next = request.match_offset + scan.matches.len() as u64;
            Ok(SavedDocumentSearchViewV1 {
                schema_version: 1,
                document_kind: "qiongli-saved-document-search".into(),
                project_id: request.project_id.clone(),
                project_revision: request.expected_project_revision,
                relative_path: request.relative_path.clone(),
                sha256: digest.to_owned(),
                source_size_bytes: bytes.len() as u64,
                search_sha256,
                search_scope: scope.into(),
                total_matches: scan.total,
                scanned_text_fields: scan.fields,
                next_match_offset: (next < scan.total).then_some(next),
                matches: scan.matches,
            })
        })
    }
}

struct PassageScan<'a> {
    request: &'a SavedDocumentSearchRequest,
    total: u64,
    fields: u64,
    matches: Vec<SavedDocumentMatchV1>,
}

impl PassageScan<'_> {
    fn json(&mut self, value: &Value, pointer: &str) -> Result<(), ProjectError> {
        if !pointer.is_empty() && !valid_json_pointer(pointer) {
            return Err(ProjectError::InvalidProjectDocument);
        }
        match value {
            Value::String(text) => self.text(text, Some(pointer)),
            Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    self.json(item, &format!("{pointer}/{index}"))?;
                }
            }
            Value::Object(fields) => {
                // Explicit sorting also holds if another crate enables preserve_order.
                let mut keys: Vec<_> = fields.keys().collect();
                keys.sort_unstable();
                for key in keys {
                    let escaped = key.replace('~', "~0").replace('/', "~1");
                    self.json(&fields[key], &format!("{pointer}/{escaped}"))?;
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
        Ok(())
    }

    fn text(&mut self, text: &str, pointer: Option<&str>) {
        self.fields += 1;
        // match_indices supplies literal, case-sensitive, non-overlapping hits.
        // Count every match, but allocate excerpts only for the bounded result page.
        for (start, found) in text.match_indices(self.request.search_text.as_str()) {
            if self.total >= self.request.match_offset
                && self.matches.len() < self.request.max_matches
            {
                self.matches
                    .push(self.hit(text, pointer, start, start + found.len()));
            }
            self.total += 1;
        }
    }

    fn hit(
        &self,
        text: &str,
        pointer: Option<&str>,
        start: usize,
        end: usize,
    ) -> SavedDocumentMatchV1 {
        let mut left = start.saturating_sub(self.request.context_bytes);
        while !text.is_char_boundary(left) {
            left += 1;
        }
        let mut right = end
            .saturating_add(self.request.context_bytes)
            .min(text.len());
        while !text.is_char_boundary(right) {
            right -= 1;
        }
        let mut read_arguments = self.request.read_request();
        read_arguments.json_pointer = pointer.map(str::to_owned);
        read_arguments.offset_bytes = left as u64;
        read_arguments.max_bytes = (right - left).max(4);
        SavedDocumentMatchV1 {
            match_offset_bytes: start as u64,
            match_end_offset_bytes: end as u64,
            context_offset_bytes: left as u64,
            content: text[left..right].to_owned(),
            truncated_before: left > 0,
            truncated_after: right < text.len(),
            read_arguments,
        }
    }
}
