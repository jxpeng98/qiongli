use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::csv::{MAX_FIELD_BYTES, MAX_TABLE_ROWS, parse_csv};
use crate::json::parse_unique_json;
use crate::model::valid_lower_hex;
use crate::paper_note::valid_note_path;
use crate::source_packet::valid_packet_path;
use crate::stage_summary::{revalidate_sources, valid_source_path};
use crate::storage::read_stage_handoff_file;
use crate::{ProjectError, StageSummarySourceV1};

pub(crate) const RETRIEVAL_MANIFEST_PATH: &str = "retrieval_manifest.csv";
const HEADER: &str = "record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes";

/// Reviewed additions to Stage B's retrieval history. No field grants source trust.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetrievalManifestDraftV1 {
    pub schema_version: u32,
    pub previous_sha256: Option<String>,
    pub attempts: Vec<RetrievalAttemptV1>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetrievalAttemptV1 {
    pub record_id: String,
    pub citekey: String,
    pub doi: String,
    pub retrieval_status: String,
    pub version_label: String,
    pub source_provider: String,
    pub retrieved_at: String,
    pub fulltext_path: String,
    pub access_url: String,
    pub license: String,
    pub notes: String,
    pub source_packet: Option<StageSummarySourceV1>,
    pub fulltext_sha256: Option<String>,
}

impl RetrievalManifestDraftV1 {
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
        if self.schema_version != 1
            || self
                .previous_sha256
                .as_deref()
                .is_some_and(|hash| !valid_lower_hex(hash, 64))
            || self.attempts.is_empty()
            || self.attempts.len() > 64
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        for attempt in &self.attempts {
            attempt.validate()?;
        }
        Ok(())
    }

    pub(crate) fn revalidate_sources(&self, root: &Path) -> Result<(), ProjectError> {
        self.validate()?;
        let mut sources = Vec::new();
        for attempt in &self.attempts {
            if let Some(packet) = &attempt.source_packet {
                sources.push(packet.clone());
            }
            if let Some(hash) = &attempt.fulltext_sha256 {
                sources.push(StageSummarySourceV1 {
                    relative_path: attempt.fulltext_path.clone(),
                    sha256: hash.clone(),
                });
            }
        }
        revalidate_sources(root, sources.iter())
    }

    pub(crate) fn render(&self, previous: Option<&str>) -> Result<String, ProjectError> {
        self.validate()?;
        let mut output = if let Some(previous) = previous {
            validate_csv(previous)?;
            previous.to_string()
        } else {
            HEADER.to_string()
        };
        // Preserve the entire previous byte prefix, including its line endings.
        if !output.ends_with('\n') {
            output.push('\n');
        }
        for attempt in &self.attempts {
            let notes = attempt.bound_notes();
            let fields = attempt.fields(&notes);
            for (index, field) in fields.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                if field.contains([',', '"', '\r', '\n']) {
                    output.push('"');
                    output.push_str(&field.replace('"', "\"\""));
                    output.push('"');
                } else {
                    output.push_str(field);
                }
            }
            output.push('\n');
        }
        if output.len() > 4 * 1024 * 1024 {
            return Err(ProjectError::DocumentTooLarge);
        }
        validate_csv(&output)?;
        Ok(output)
    }
}

impl RetrievalAttemptV1 {
    fn fields<'a>(&'a self, notes: &'a str) -> [&'a str; 11] {
        [
            &self.record_id,
            &self.citekey,
            &self.doi,
            &self.retrieval_status,
            &self.version_label,
            &self.source_provider,
            &self.retrieved_at,
            &self.fulltext_path,
            &self.access_url,
            &self.license,
            notes,
        ]
    }

    fn validate(&self) -> Result<(), ProjectError> {
        validate_fields(&self.fields(&self.notes))?;
        if let Some(packet) = &self.source_packet
            && (!valid_packet_path(&packet.relative_path)
                || !valid_lower_hex(&packet.sha256, 64)
                || packet.relative_path
                    != format!("sources/{}/{}.json", self.citekey, packet.sha256))
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        match (&self.fulltext_sha256, self.fulltext_path.is_empty()) {
            (None, true) => {}
            (Some(hash), false) if valid_lower_hex(hash, 64) => {}
            _ => return Err(ProjectError::InvalidProjectDocument),
        }
        if self.bound_notes().len() > MAX_FIELD_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(())
    }

    fn bound_notes(&self) -> String {
        let mut notes = self.notes.clone();
        if let Some(packet) = &self.source_packet {
            notes.push_str(&format!(
                "\nSaved source packet: `{}`; local packet SHA-256: `{}` (not a PDF digest).",
                packet.relative_path, packet.sha256,
            ));
        }
        if let Some(hash) = &self.fulltext_sha256 {
            notes.push_str(&format!("\nLocal fulltext file SHA-256: `{hash}`."));
        }
        notes
    }
}

fn validate_fields(fields: &[&str]) -> Result<(), ProjectError> {
    if fields.len() != 11
        || fields.iter().any(|field| {
            field.len() > MAX_FIELD_BYTES
                || field
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\t' | '\r' | '\n'))
        })
        || fields[0].trim().is_empty()
        || fields[0].len() > 256
        || fields[0].chars().any(char::is_control)
        || !valid_note_path(&format!("notes/{}.md", fields[1]))
        || fields[5].trim().is_empty()
        || !matches!(
            fields[4],
            "published" | "accepted" | "submitted" | "abstract_only" | "metadata_only" | "unknown"
        )
        || !matches!(
            fields[3],
            "retrieved_oa"
                | "retrieved_preprint"
                | "abstract_only"
                | "not_retrieved:paywall"
                | "not_retrieved:embargo"
                | "not_retrieved:broken_link"
                | "not_retrieved:not_found"
                | "not_retrieved:access_restricted"
                | "not_retrieved:needs_provider"
                | "not_retrieved:missing_locator"
                | "not_retrieved:oa_candidate"
        )
        || (!fields[7].is_empty()
            && (!valid_source_path(fields[7])
                || valid_packet_path(fields[7])
                || fields[7] == RETRIEVAL_MANIFEST_PATH))
    {
        return Err(ProjectError::InvalidProjectDocument);
    }
    if !fields[8].is_empty() {
        let url = url::Url::parse(fields[8]).map_err(|_| ProjectError::InvalidProjectDocument)?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
    }
    Ok(())
}

fn validate_csv(text: &str) -> Result<(), ProjectError> {
    let records = parse_csv(text).map_err(|()| ProjectError::InvalidProjectDocument)?;
    if text.ends_with('\r')
        || records.is_empty()
        || records.len() > MAX_TABLE_ROWS + 1
        || records[0]
            .fields
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != HEADER.split(',').collect::<Vec<_>>()
    {
        return Err(ProjectError::InvalidProjectDocument);
    }
    for record in &records[1..] {
        validate_fields(&record.fields.iter().map(String::as_str).collect::<Vec<_>>())?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_draft() -> RetrievalManifestDraftV1 {
    RetrievalManifestDraftV1 {
        schema_version: 1,
        previous_sha256: None,
        attempts: vec![RetrievalAttemptV1 {
            record_id: "R-001".into(),
            citekey: "Smith2024".into(),
            doi: String::new(),
            retrieval_status: "not_retrieved:oa_candidate".into(),
            version_label: "unknown".into(),
            source_provider: "native reader".into(),
            retrieved_at: String::new(),
            fulltext_path: String::new(),
            access_url: "https://example.org/article".into(),
            license: String::new(),
            notes: "DNS refusal before HTTP; identity and PDF digest unknown.".into(),
            source_packet: None,
            fulltext_sha256: None,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieval_manifest_preserves_csv_prefix_retries_and_unknowns() {
        let mut draft = test_draft();
        let prior = format!(
            "{HEADER}\r\nR-001,Smith2024,,abstract_only,unknown,Host,,,,,\"旧笔记,\"\"引用\"\"\n第二行\""
        );
        draft.attempts[0].notes = "Failed, \"quoted\"\r\nretained".into();
        let mut retry = draft.attempts[0].clone();
        retry.source_provider = "Host reader".into();
        retry.retrieval_status = "abstract_only".into();
        draft.attempts.push(retry);
        let output = draft.render(Some(&prior)).unwrap();
        assert!(output.starts_with(&prior));
        let rows = parse_csv(&output).unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[2].fields[10], draft.attempts[0].notes);
        assert_eq!(rows[2].fields[4], "unknown");
        for column in [2, 6, 7, 9] {
            assert!(rows[2].fields[column].is_empty());
        }
        assert_eq!(rows[1].fields[0], rows[2].fields[0]);
        assert_eq!(rows[3].fields[5], "Host reader");
    }

    #[test]
    fn retrieval_manifest_refuses_invalid_history_and_unbounded_rows() {
        let draft = test_draft();
        for prior in [
            "",
            "wrong,header\n",
            HEADER.trim_end_matches("notes"),
            &format!("{HEADER}\nR1,Smith2024,,verified,unknown,Host,,,,,unsupported"),
            &format!("{HEADER}\n\"unclosed"),
            &format!("{HEADER}\r"),
        ] {
            assert!(draft.render(Some(prior)).is_err(), "{prior}");
        }
        let row = "R1,Smith2024,,abstract_only,unknown,Host,,,,,\n";
        let prior = format!("{HEADER}\n{}", row.repeat(MAX_TABLE_ROWS));
        assert!(draft.render(Some(&prior)).is_err());
        let mut too_many = draft.clone();
        too_many.attempts = vec![draft.attempts[0].clone(); 65];
        assert!(too_many.validate().is_err());
    }

    #[test]
    fn retrieval_manifest_rejects_unsafe_or_misbound_sources() {
        for mutation in [
            "hash",
            "packet-citekey",
            "packet-hash",
            "fulltext-path",
            "fulltext-hash",
            "packet-as-fulltext",
            "url",
            "status",
            "version",
            "notes",
        ] {
            let mut draft = test_draft();
            let row = &mut draft.attempts[0];
            match mutation {
                "hash" => draft.previous_sha256 = Some("invalid".into()),
                "packet-citekey" => {
                    row.source_packet = Some(StageSummarySourceV1 {
                        relative_path: format!("sources/Other/{}.json", "a".repeat(64)),
                        sha256: "a".repeat(64),
                    })
                }
                "packet-hash" => {
                    row.source_packet = Some(StageSummarySourceV1 {
                        relative_path: format!("sources/Smith2024/{}.json", "a".repeat(64)),
                        sha256: "b".repeat(64),
                    })
                }
                "fulltext-path" => {
                    row.fulltext_path = "../paper.pdf".into();
                    row.fulltext_sha256 = Some("a".repeat(64));
                }
                "fulltext-hash" => row.fulltext_path = "sources/paper.pdf".into(),
                "packet-as-fulltext" => {
                    row.fulltext_path = format!("sources/Smith2024/{}.json", "a".repeat(64));
                    row.fulltext_sha256 = Some("a".repeat(64));
                }
                "url" => row.access_url = "https://user:secret@example.org/".into(),
                "status" => row.retrieval_status = "verified".into(),
                "version" => row.version_label.clear(),
                "notes" => row.notes = "x".repeat(MAX_FIELD_BYTES + 1),
                _ => unreachable!(),
            }
            assert!(draft.validate().is_err(), "{mutation}");
        }
    }
}
