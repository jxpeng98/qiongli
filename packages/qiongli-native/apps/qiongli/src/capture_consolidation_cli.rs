use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    ApprovedCaptureConsolidation, CaptureConsolidationCommitV1, CaptureConsolidationDrafts,
    CaptureConsolidationPreviewV1, CaptureId, PaperNoteDraftV1, ProjectError, ProjectId,
    ProjectStateService, RetrievalManifestDraftV1, SourcePacketDraftV1, StageSummaryDraftV1,
    read_stage_handoff_file,
};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Help,
    Preview(Options),
    Apply(Options, String),
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Options {
    project_id: ProjectId,
    capture_id: CaptureId,
    reviewed_at_unix: Option<u64>,
    stage_handoff_file: Option<PathBuf>,
    stage_summary_file: Option<PathBuf>,
    paper_note_file: Option<PathBuf>,
    source_packet_file: Option<PathBuf>,
    retrieval_manifest_file: Option<PathBuf>,
}

impl std::fmt::Debug for Options {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Options")
            .field("project_id", &self.project_id)
            .field("capture_id", &self.capture_id)
            .field("reviewed_at_unix", &self.reviewed_at_unix)
            .field(
                "stage_handoff_file",
                &self.stage_handoff_file.as_ref().map(|_| "<handoff-draft>"),
            )
            .field(
                "stage_summary_file",
                &self.stage_summary_file.as_ref().map(|_| "<summary-draft>"),
            )
            .field(
                "paper_note_file",
                &self.paper_note_file.as_ref().map(|_| "<paper-note-draft>"),
            )
            .field(
                "source_packet_file",
                &self
                    .source_packet_file
                    .as_ref()
                    .map(|_| "<source-packet-draft>"),
            )
            .field(
                "retrieval_manifest_file",
                &self
                    .retrieval_manifest_file
                    .as_ref()
                    .map(|_| "<retrieval-manifest-draft>"),
            )
            .finish()
    }
}

pub(crate) fn parse(args: &[OsString]) -> Result<Command, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a capture consolidation subcommand is required");
    };
    if subcommand == "--help" && args.len() == 1 {
        return Ok(Command::Help);
    }
    let apply = match subcommand {
        "preview" => false,
        "apply" => true,
        _ => return Err("unknown capture consolidation subcommand"),
    };
    parse_options(apply, &args[1..])
}

pub(crate) fn execute(
    command: Command,
    service: &ProjectStateService,
) -> Result<Output, ProjectError> {
    match command {
        Command::Help => unreachable!("consolidation help returns before service execution"),
        Command::Preview(options) => {
            let reviewed_at_unix = options.reviewed_at_unix.map_or_else(now_unix, Ok)?;
            let handoff = options
                .stage_handoff_file
                .as_deref()
                .map(read_stage_handoff_file)
                .transpose()?;
            let summary = options
                .stage_summary_file
                .as_deref()
                .map(StageSummaryDraftV1::read_file)
                .transpose()?;
            let note = options
                .paper_note_file
                .as_deref()
                .map(PaperNoteDraftV1::read_file)
                .transpose()?;
            let packet = options
                .source_packet_file
                .as_deref()
                .map(SourcePacketDraftV1::read_file)
                .transpose()?;
            let manifest = options
                .retrieval_manifest_file
                .as_deref()
                .map(RetrievalManifestDraftV1::read_file)
                .transpose()?;
            service
                .preview_capture_consolidation_with_drafts(
                    &options.project_id,
                    &options.capture_id,
                    reviewed_at_unix,
                    CaptureConsolidationDrafts {
                        stage_handoff: handoff.as_deref(),
                        stage_summary: summary.as_ref(),
                        paper_note: note.as_ref(),
                        source_packet: packet.as_ref(),
                        retrieval_manifest: manifest.as_ref(),
                    },
                )
                .map(|plan| {
                    Output::Preview(Box::new(PreviewOutput {
                        schema_version: 1,
                        command: "project-capture-consolidate-preview",
                        preview: plan.preview().clone(),
                        stage_handoff_content: plan.stage_handoff_content().map(str::to_owned),
                        stage_summary_content: plan.stage_summary_content().map(str::to_owned),
                        paper_note_content: plan.paper_note_content().map(str::to_owned),
                        source_packet_content: plan.source_packet_content().map(str::to_owned),
                        retrieval_manifest_content: plan
                            .retrieval_manifest_content()
                            .map(str::to_owned),
                        research_state_content: plan
                            .summary_research_state_content()
                            .map(str::to_owned),
                    }))
                })
        }
        Command::Apply(options, digest) => {
            let reviewed_at_unix = options
                .reviewed_at_unix
                .expect("consolidation apply parser requires a review timestamp");
            let handoff = options
                .stage_handoff_file
                .as_deref()
                .map(read_stage_handoff_file)
                .transpose()?;
            let summary = options
                .stage_summary_file
                .as_deref()
                .map(StageSummaryDraftV1::read_file)
                .transpose()?;
            let note = options
                .paper_note_file
                .as_deref()
                .map(PaperNoteDraftV1::read_file)
                .transpose()?;
            let packet = options
                .source_packet_file
                .as_deref()
                .map(SourcePacketDraftV1::read_file)
                .transpose()?;
            let manifest = options
                .retrieval_manifest_file
                .as_deref()
                .map(RetrievalManifestDraftV1::read_file)
                .transpose()?;
            let plan = service.preview_capture_consolidation_with_drafts(
                &options.project_id,
                &options.capture_id,
                reviewed_at_unix,
                CaptureConsolidationDrafts {
                    stage_handoff: handoff.as_deref(),
                    stage_summary: summary.as_ref(),
                    paper_note: note.as_ref(),
                    source_packet: packet.as_ref(),
                    retrieval_manifest: manifest.as_ref(),
                },
            )?;
            let commit = service.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(digest, true, true),
            )?;
            Ok(Output::Commit(CommitOutput {
                schema_version: 1,
                command: "project-capture-consolidate-apply",
                commit,
            }))
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    Preview(Box<PreviewOutput>),
    Commit(CommitOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: CaptureConsolidationPreviewV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_handoff_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_summary_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    research_state_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paper_note_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_packet_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retrieval_manifest_content: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: CaptureConsolidationCommitV1,
}

fn parse_options(apply: bool, args: &[OsString]) -> Result<Command, &'static str> {
    let mut project_id = None;
    let mut capture_id = None;
    let mut reviewed_at_unix = None;
    let mut stage_handoff_file = None;
    let mut stage_summary_file = None;
    let mut paper_note_file = None;
    let mut source_packet_file = None;
    let mut retrieval_manifest_file = None;
    let mut digest = None;
    let mut filesystem_write = false;
    let mut academic_review = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("capture consolidation option is not valid UTF-8")?;
        match option {
            "--approve-filesystem-write" => {
                if !apply || filesystem_write {
                    return Err("filesystem approval is unexpected or duplicate");
                }
                filesystem_write = true;
                index += 1;
                continue;
            }
            "--approve-academic-review" => {
                if !apply || academic_review {
                    return Err("academic approval is unexpected or duplicate");
                }
                academic_review = true;
                index += 1;
                continue;
            }
            _ => {}
        }
        let value = args
            .get(index + 1)
            .ok_or("capture consolidation option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => {
                let value = value.to_str().ok_or("project ID is not valid UTF-8")?;
                project_id =
                    Some(ProjectId::parse(value.to_string()).map_err(|_| "project ID is invalid")?);
            }
            "--capture-id" if capture_id.is_none() => {
                let value = value.to_str().ok_or("capture ID is not valid UTF-8")?;
                capture_id =
                    Some(CaptureId::parse(value.to_string()).map_err(|_| "capture ID is invalid")?);
            }
            "--stage-handoff-file" if stage_handoff_file.is_none() => {
                stage_handoff_file = Some(PathBuf::from(value));
            }
            "--stage-summary-file" if stage_summary_file.is_none() => {
                stage_summary_file = Some(PathBuf::from(value));
            }
            "--paper-note-file" if paper_note_file.is_none() => {
                paper_note_file = Some(PathBuf::from(value));
            }
            "--source-packet-file" if source_packet_file.is_none() => {
                source_packet_file = Some(PathBuf::from(value));
            }
            "--retrieval-manifest-file" if retrieval_manifest_file.is_none() => {
                retrieval_manifest_file = Some(PathBuf::from(value));
            }
            "--reviewed-at-unix" if reviewed_at_unix.is_none() => {
                reviewed_at_unix = Some(parse_unix_timestamp(value)?);
            }
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--project-id"
            | "--capture-id"
            | "--reviewed-at-unix"
            | "--expected-plan-digest"
            | "--stage-handoff-file"
            | "--stage-summary-file"
            | "--paper-note-file"
            | "--source-packet-file"
            | "--retrieval-manifest-file" => {
                return Err("capture consolidation option is unexpected or duplicate");
            }
            _ => return Err("unknown capture consolidation option"),
        }
        index += 2;
    }
    let options = Options {
        project_id: project_id.ok_or("project ID is required")?,
        capture_id: capture_id.ok_or("capture ID is required")?,
        reviewed_at_unix,
        stage_handoff_file,
        stage_summary_file,
        paper_note_file,
        source_packet_file,
        retrieval_manifest_file,
    };
    if !apply {
        return Ok(Command::Preview(options));
    }
    if options.reviewed_at_unix.is_none()
        || digest.is_none()
        || !filesystem_write
        || !academic_review
    {
        return Err(
            "capture consolidation apply requires review timestamp, plan digest, academic approval, and filesystem approval",
        );
    }
    Ok(Command::Apply(options, digest.expect("validated above")))
}

fn parse_unix_timestamp(value: &OsString) -> Result<u64, &'static str> {
    value
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .ok_or("review timestamp must be an unsigned decimal integer")
}

fn parse_sha256(value: &OsString) -> Result<String, &'static str> {
    value
        .to_str()
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        .map(str::to_owned)
        .ok_or("plan digest must be 64 lowercase hexadecimal characters")
}

fn now_unix() -> Result<u64, ProjectError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| ProjectError::HomeUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_accepts_preview_and_apply_contracts() {
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        let capture_id = format!("cap_{}", "a".repeat(64));
        assert!(matches!(
            parse(&args(&[
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--reviewed-at-unix",
                "1721337601",
            ])),
            Ok(Command::Preview(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--reviewed-at-unix",
                "1721337601",
                "--expected-plan-digest",
                &"b".repeat(64),
                "--approve-academic-review",
                "--approve-filesystem-write",
            ])),
            Ok(Command::Apply(_, _))
        ));
    }

    #[test]
    fn parser_accepts_optional_continuity_files_without_granting_approval() {
        for option in [
            "--stage-handoff-file",
            "--stage-summary-file",
            "--paper-note-file",
            "--source-packet-file",
            "--retrieval-manifest-file",
        ] {
            let mut values = args(&[
                "preview",
                "--project-id",
                "prj_0123456789abcdef0123456789abcdef",
                "--capture-id",
                &format!("cap_{}", "a".repeat(64)),
                option,
                "/tmp/draft",
            ]);
            assert!(matches!(parse(&values), Ok(Command::Preview(_))));
            assert!(!format!("{:?}", parse(&values).unwrap()).contains("/tmp/draft"));
            values.extend(args(&[option, "/tmp/other"]));
            assert!(parse(&values).is_err());
            values.truncate(values.len() - 2);
            values[0] = OsString::from("apply");
            assert!(parse(&values).is_err());
        }
    }

    #[test]
    fn parser_requires_reproducible_review_and_dual_approval() {
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        let capture_id = format!("cap_{}", "a".repeat(64));
        assert!(matches!(
            parse(&args(&[
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
            ])),
            Ok(Command::Preview(Options {
                reviewed_at_unix: None,
                ..
            }))
        ));
        assert_eq!(
            parse(&args(&[
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--expected-plan-digest",
                &"b".repeat(64),
                "--approve-academic-review",
                "--approve-filesystem-write",
            ])),
            Err(
                "capture consolidation apply requires review timestamp, plan digest, academic approval, and filesystem approval"
            )
        );
        assert_eq!(
            parse(&args(&[
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--reviewed-at-unix",
                "1721337601",
                "--expected-plan-digest",
                &"b".repeat(64),
                "--approve-filesystem-write",
            ])),
            Err(
                "capture consolidation apply requires review timestamp, plan digest, academic approval, and filesystem approval"
            )
        );
        assert_eq!(
            parse(&args(&[
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--reviewed-at-unix",
                "+1721337601",
            ])),
            Err("review timestamp must be an unsigned decimal integer")
        );
        assert_eq!(
            parse(&args(&[
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--approve-academic-review",
            ])),
            Err("academic approval is unexpected or duplicate")
        );
    }
}
