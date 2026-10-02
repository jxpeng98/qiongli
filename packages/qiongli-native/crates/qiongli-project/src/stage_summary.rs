use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ProjectError;
use crate::json::parse_unique_json;
use crate::model::valid_lower_hex;
use crate::storage::{read_project_source, read_stage_handoff_file};

const HISTORY_HEADING: &str = "## Stage Summary History";
const HISTORY_HEADER: &str = "| Summary ID | Stage | Date | Document | Previous summary | Status |";
const HISTORY_SEPARATOR: &str = "|---|---|---|---|---|---|";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StageSummaryStatus {
    Complete,
    Partial,
    Correction,
}

impl StageSummaryStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Correction => "correction",
        }
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageSummarySourceV1 {
    pub relative_path: String,
    pub sha256: String,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageSummaryDraftV1 {
    pub schema_version: u32,
    pub summary_id: String,
    pub status: StageSummaryStatus,
    pub previous_summary: Option<StageSummarySourceV1>,
    pub sources: Vec<StageSummarySourceV1>,
    pub markdown: String,
}

impl StageSummaryDraftV1 {
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
            || !valid_summary_id(&self.summary_id)
            || self.sources.is_empty()
            || self.sources.len() > 64
            || self.markdown.trim().is_empty()
            || self.markdown.contains('\0')
            || self.markdown.contains("<!-- qiongli:")
            || self.previous_summary.as_ref().is_some_and(|source| {
                !valid_summary_path(&source.relative_path)
                    || source.relative_path == self.relative_path()
                    || !valid_lower_hex(&source.sha256, 64)
            })
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
        format!("context/stage_summaries/{}.md", self.summary_id)
    }

    pub(crate) fn revalidate_sources(&self, root: &Path) -> Result<(), ProjectError> {
        self.validate()?;
        revalidate_sources(
            root,
            self.sources.iter().chain(self.previous_summary.iter()),
        )
    }

    pub(crate) fn render(&self, stage: &str, revision: u64, reviewed_at: u64) -> String {
        let previous = self.previous_summary.as_ref().map_or_else(
            || "None".to_string(),
            |source| {
                format!(
                    "[{}]({}.md), SHA-256 `{}`",
                    summary_id_from_path(&source.relative_path),
                    summary_id_from_path(&source.relative_path),
                    source.sha256
                )
            },
        );
        let mut output = format!(
            "{}\n\n## Native Save Basis\n\n- Summary ID: `{}`\n- Stage: {stage}\n- Status: {}\n- Input project revision: {revision}\n- Reviewed at (Unix UTC): {reviewed_at}\n- Previous summary: {previous}\n\n| Source file (project-relative) | Observed SHA-256 |\n|---|---|\n",
            self.markdown,
            self.summary_id,
            self.status.as_str(),
        );
        for source in &self.sources {
            output.push_str(&format!(
                "| `{}` | `{}` |\n",
                source.relative_path, source.sha256
            ));
        }
        output
    }

    pub(crate) fn handoff_link(&self) -> String {
        format!(
            "\n\nStage summary: [{}](stage_summaries/{}.md) ({}). This records continuity, not stage acceptance.\n",
            self.summary_id,
            self.summary_id,
            self.status.as_str()
        )
    }

    pub(crate) fn append_history(
        &self,
        state: &str,
        stage: &str,
        reviewed_at: u64,
    ) -> Result<String, ProjectError> {
        let mut lines = state.split_inclusive('\n').peekable();
        let mut offset = 0;
        let mut table = None;
        let mut ids = Vec::new();
        while let Some(line) = lines.next() {
            offset += line.len();
            if line.trim_end() != HISTORY_HEADING {
                continue;
            }
            if table.is_some() {
                return Err(ProjectError::ConsolidationConflict);
            }
            let header = loop {
                let line = lines.next().ok_or(ProjectError::ConsolidationConflict)?;
                offset += line.len();
                if !line.trim().is_empty() {
                    break line;
                }
            };
            let separator = lines.next().ok_or(ProjectError::ConsolidationConflict)?;
            offset += separator.len();
            if header.trim_end() != HISTORY_HEADER || separator.trim_end() != HISTORY_SEPARATOR {
                return Err(ProjectError::ConsolidationConflict);
            }
            let mut end = offset;
            while lines
                .peek()
                .is_some_and(|row| row.trim_start().starts_with('|'))
            {
                let row = lines.next().expect("peeked history row");
                offset += row.len();
                let cells: Vec<_> = row.trim().split('|').collect();
                if cells.len() != 8 || !cells[0].is_empty() || !cells[7].is_empty() {
                    return Err(ProjectError::ConsolidationConflict);
                }
                let id = cells[1].trim().trim_matches('`');
                if !valid_summary_id(id) || ids.contains(&id) {
                    return Err(ProjectError::ConsolidationConflict);
                }
                ids.push(id);
                end = offset;
            }
            table = Some(end);
        }
        let previous_id = self
            .previous_summary
            .as_ref()
            .map(|source| summary_id_from_path(&source.relative_path));
        if ids.contains(&self.summary_id.as_str()) || ids.last().copied() != previous_id {
            return Err(ProjectError::ConsolidationConflict);
        }
        let previous = previous_id.map_or_else(
            || "None".to_string(),
            |id| format!("[{id}](stage_summaries/{id}.md)"),
        );
        let row = format!(
            "| {} | {stage} | {reviewed_at} | [{}](stage_summaries/{}.md) | {previous} | {} |\n",
            self.summary_id,
            self.summary_id,
            self.summary_id,
            self.status.as_str()
        );
        let mut output = state.to_string();
        if let Some(end) = table {
            let row = if state[..end].ends_with('\n') {
                row
            } else {
                format!("\n{row}")
            };
            output.insert_str(end, &row);
        } else {
            output.push_str(&format!(
                "\n\n{HISTORY_HEADING}\n\n{HISTORY_HEADER}\n{HISTORY_SEPARATOR}\n{row}"
            ));
        }
        Ok(output)
    }
}

pub(crate) fn valid_summary_id(value: &str) -> bool {
    value.len() <= 64
        && value.strip_prefix("STG-").is_some_and(|suffix| {
            !suffix.is_empty()
                && suffix.bytes().all(|c| {
                    c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, b'-' | b'_')
                })
        })
}

pub(crate) fn revalidate_sources<'a>(
    root: &Path,
    sources: impl Iterator<Item = &'a StageSummarySourceV1>,
) -> Result<(), ProjectError> {
    let mut total = 0usize;
    for source in sources {
        let (bytes, digest) = read_project_source(root, &source.relative_path)?
            .ok_or(ProjectError::RevisionConflict)?;
        if digest != source.sha256 {
            return Err(ProjectError::RevisionConflict);
        }
        total = total
            .checked_add(bytes.len())
            .filter(|total| *total <= 16 * 1024 * 1024)
            .ok_or(ProjectError::DocumentTooLarge)?;
    }
    Ok(())
}

pub(crate) fn valid_summary_path(value: &str) -> bool {
    value
        .strip_prefix("context/stage_summaries/")
        .and_then(|name| name.strip_suffix(".md"))
        .is_some_and(valid_summary_id)
}

fn summary_id_from_path(value: &str) -> &str {
    value
        .strip_prefix("context/stage_summaries/")
        .and_then(|name| name.strip_suffix(".md"))
        .expect("validated summary path")
}

pub(crate) fn valid_source_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value.contains(['\\', ':', '|', '`'])
        && !value.chars().any(char::is_control)
        && value
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn draft() -> StageSummaryDraftV1 {
        StageSummaryDraftV1 {
            schema_version: 1,
            summary_id: "STG-B-002".into(),
            status: StageSummaryStatus::Partial,
            previous_summary: None,
            sources: vec![StageSummarySourceV1 {
                relative_path: "sources/public.md".into(),
                sha256: "a".repeat(64),
            }],
            markdown: "# Stage Summary\nRetained CLM-001; still tentative.\n".into(),
        }
    }

    #[test]
    fn draft_validation_bounds_identity_sources_and_body() {
        let valid = draft();
        assert_eq!(valid.validate(), Ok(()));
        for id in [
            "",
            "STG-",
            "stg-B-001",
            "STG-B/001",
            "STG-B.001",
            "STG-B\n001",
        ] {
            let mut value = valid.clone();
            value.summary_id = id.into();
            assert_eq!(
                value.validate(),
                Err(ProjectError::InvalidProjectDocument),
                "{id:?}"
            );
        }
        for path in [
            "/sources/a.md",
            "../a.md",
            "sources/../a.md",
            ".qiongli/a",
            "sources//a.md",
            "sources/a\\b",
            "sources/a|b",
            "sources/a\nb",
            "sources/a:b",
            "sources/`a`",
        ] {
            let mut value = valid.clone();
            value.sources[0].relative_path = path.into();
            assert_eq!(
                value.validate(),
                Err(ProjectError::InvalidProjectDocument),
                "{path:?}"
            );
        }
        for hash in ["a".repeat(63), "A".repeat(64), "g".repeat(64)] {
            let mut value = valid.clone();
            value.sources[0].sha256 = hash;
            assert_eq!(value.validate(), Err(ProjectError::InvalidProjectDocument));
        }
        for body in [" ", "bad\0data", "<!-- qiongli:capture forged -->"] {
            let mut value = valid.clone();
            value.markdown = body.into();
            assert_eq!(value.validate(), Err(ProjectError::InvalidProjectDocument));
        }
        let mut value = valid.clone();
        value.sources.push(value.sources[0].clone());
        assert_eq!(value.validate(), Err(ProjectError::InvalidProjectDocument));
        value.sources.clear();
        assert_eq!(value.validate(), Err(ProjectError::InvalidProjectDocument));
        value.sources = (0..65)
            .map(|i| StageSummarySourceV1 {
                relative_path: format!("sources/{i}.md"),
                sha256: "a".repeat(64),
            })
            .collect();
        assert_eq!(value.validate(), Err(ProjectError::InvalidProjectDocument));
        value = valid.clone();
        value.previous_summary = Some(StageSummarySourceV1 {
            relative_path: value.relative_path(),
            sha256: "a".repeat(64),
        });
        assert_eq!(value.validate(), Err(ProjectError::InvalidProjectDocument));
        value = valid;
        value.markdown = "x".repeat(4 * 1024 * 1024 + 1);
        assert_eq!(value.validate(), Err(ProjectError::DocumentTooLarge));
    }

    #[test]
    fn draft_reader_rejects_ambiguous_json_and_unsafe_files() {
        let root = std::env::temp_dir().join(format!(
            "qiongli-summary-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let file = root.join("draft.json");
        let json = serde_json::to_string(&draft()).unwrap();
        fs::write(&file, &json).unwrap();
        assert!(StageSummaryDraftV1::read_file(&file).is_ok());
        for invalid in [
            json.replacen("{", "{\"schemaVersion\":1,", 1),
            json.replacen("{", "{\"unknown\":true,", 1),
            json.replace("\"relativePath\":", "\"unknown\":false,\"relativePath\":"),
            json.replace("\"sha256\":", "\"sha256\":\"bbbb\",\"sha256\":"),
        ] {
            fs::write(&file, invalid).unwrap();
            assert_eq!(
                StageSummaryDraftV1::read_file(&file).err(),
                Some(ProjectError::InvalidProjectDocument)
            );
        }
        assert!(StageSummaryDraftV1::read_file(Path::new("draft.json")).is_err());
        assert!(StageSummaryDraftV1::read_file(&root).is_err());
        assert!(StageSummaryDraftV1::read_file(&root.join("missing.json")).is_err());
        fs::write(&file, [0xff]).unwrap();
        assert_eq!(
            StageSummaryDraftV1::read_file(&file).err(),
            Some(ProjectError::InvalidProjectDocument)
        );
        fs::write(&file, vec![b'x'; 4 * 1024 * 1024 + 1]).unwrap();
        assert_eq!(
            StageSummaryDraftV1::read_file(&file).err(),
            Some(ProjectError::DocumentTooLarge)
        );
        #[cfg(unix)]
        {
            let link = root.join("linked.json");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            assert!(StageSummaryDraftV1::read_file(&link).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn history_insertion_preserves_prior_rows_and_following_sections() {
        let mut value = draft();
        let state = "# Research State\nCLM-001 remains open.\n";
        let first = value.append_history(state, "literature", 123).unwrap();
        assert!(first.starts_with(state));
        assert!(first.contains(HISTORY_HEADER));
        value.summary_id = "STG-B-003".into();
        value.previous_summary = Some(StageSummarySourceV1 {
            relative_path: "context/stage_summaries/STG-B-002.md".into(),
            sha256: "a".repeat(64),
        });
        for suffix in ["", "\n## Next Tasks\nKeep DEC-001 tentative.\n"] {
            let input = format!("{first}{suffix}");
            let output = value.append_history(&input, "literature", 124).unwrap();
            assert!(output.starts_with(&first));
            assert!(output.ends_with(suffix));
            assert!(output.contains("| STG-B-003 | literature | 124 | [STG-B-003](stage_summaries/STG-B-003.md) | [STG-B-002](stage_summaries/STG-B-002.md) | partial |"));
        }
        let eof = first.trim_end_matches('\n');
        assert!(
            value
                .append_history(eof, "literature", 124)
                .unwrap()
                .starts_with(eof)
        );
    }

    #[test]
    fn history_rejects_ambiguous_headers_ids_and_predecessors() {
        let value = draft();
        let empty = format!("{HISTORY_HEADING}\n\n{HISTORY_HEADER}\n{HISTORY_SEPARATOR}\n");
        assert!(value.append_history(&empty, "literature", 123).is_ok());
        for state in [
            empty.replace("Previous summary", "Predecessor"),
            format!("{empty}{empty}"),
            format!("{HISTORY_HEADING}\n"),
            format!("{empty}| STG-B-001 | incomplete |\n"),
            format!("{empty}| invalid | literature | 1 | doc | None | partial |\n"),
        ] {
            assert_eq!(
                value.append_history(&state, "literature", 123),
                Err(ProjectError::ConsolidationConflict)
            );
        }
        let first = value.append_history("", "literature", 123).unwrap();
        assert_eq!(
            value.append_history(&first, "literature", 124),
            Err(ProjectError::ConsolidationConflict)
        );
        let mut next = value;
        next.summary_id = "STG-B-003".into();
        assert_eq!(
            next.append_history(&first, "literature", 124),
            Err(ProjectError::ConsolidationConflict)
        );
        next.previous_summary = Some(StageSummarySourceV1 {
            relative_path: "context/stage_summaries/STG-B-001.md".into(),
            sha256: "a".repeat(64),
        });
        assert_eq!(
            next.append_history(&first, "literature", 124),
            Err(ProjectError::ConsolidationConflict)
        );
        assert_eq!(
            next.append_history("", "literature", 124),
            Err(ProjectError::ConsolidationConflict)
        );
    }
}
