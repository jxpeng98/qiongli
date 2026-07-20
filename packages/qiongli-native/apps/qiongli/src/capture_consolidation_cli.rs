use std::ffi::OsString;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    ApprovedCaptureConsolidation, CaptureConsolidationCommitV1, CaptureConsolidationPreviewV1,
    CaptureId, ProjectError, ProjectId, ProjectStateService,
};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Help,
    Preview(Options),
    Apply(Options, String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    project_id: ProjectId,
    capture_id: CaptureId,
    reviewed_at_unix: Option<u64>,
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
            service
                .preview_capture_consolidation(
                    &options.project_id,
                    &options.capture_id,
                    reviewed_at_unix,
                )
                .map(|plan| {
                    Output::Preview(PreviewOutput {
                        schema_version: 1,
                        command: "project-capture-consolidate-preview",
                        preview: plan.preview().clone(),
                    })
                })
        }
        Command::Apply(options, digest) => {
            let reviewed_at_unix = options
                .reviewed_at_unix
                .expect("consolidation apply parser requires a review timestamp");
            let plan = service.preview_capture_consolidation(
                &options.project_id,
                &options.capture_id,
                reviewed_at_unix,
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
    Preview(PreviewOutput),
    Commit(CommitOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: CaptureConsolidationPreviewV1,
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
            "--reviewed-at-unix" if reviewed_at_unix.is_none() => {
                reviewed_at_unix = Some(parse_unix_timestamp(value)?);
            }
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--capture-id" | "--reviewed-at-unix" | "--expected-plan-digest" => {
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
