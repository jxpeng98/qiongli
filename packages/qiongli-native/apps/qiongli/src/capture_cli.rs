use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    ApprovedCaptureIntake, ArtifactChangeSnapshotV1, CaptureCoverageSnapshotV1, CaptureId,
    CaptureInboxSnapshotV1, CaptureIntakeCommitV1, CaptureIntakePreviewV1, ProjectError, ProjectId,
    ProjectStateService, ResearchCaptureV1, read_portable_capture_packet,
};
use serde::Serialize;

pub(crate) const CAPTURE_USAGE: &str = "Qiongli Capture Inbox\n\nUsage:\n  qiongli project capture list --project-id <prj_id>\n  qiongli project capture coverage --project-id <prj_id>\n  qiongli project capture changes --project-id <prj_id>\n  qiongli project capture read --project-id <prj_id> --capture-id <cap_id>\n  qiongli project capture preview --file <absolute-capture.json>\n  qiongli project capture apply --file <absolute-capture.json> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project capture delivery <list|inspect|retry|cancel> [options]\n  qiongli project capture repository <list|read|preview|apply> --project-id <prj_id> [--capture-id <cap_id>]\n  qiongli project capture consolidate preview --project-id <prj_id> --capture-id <cap_id> [--reviewed-at-unix <timestamp>]\n  qiongli project capture consolidate apply --project-id <prj_id> --capture-id <cap_id> --reviewed-at-unix <timestamp> --expected-plan-digest <sha256> --approve-academic-review --approve-filesystem-write\n  qiongli project capture --help\n\nCoverage reports only observed normalized delivery evidence and labels unsupported or unobserved sources unknown. Changes reports revision-bound registered-artifact drift as unattributed unless accepted capture lineage proves otherwise; aggregate detection never guesses a file or client source. Portable capture files contain a strict, bounded qiongli-research-capture document. Repository intake reads only context/capture-inbox/<cap_id>.json inside an already registered project and never accepts an arbitrary repository path.\nDelivery inspect/retry/cancel operates only on the private versioned delivery ledger and returns path-redacted causal state. Intake preview/apply stores a capture in the review Inbox. Consolidation preview/apply converts one reviewed capture into explicit academic artifact deltas.\nApply must reuse the reviewedAtUnix and planDigest returned by its preview and requires every listed approval.\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CaptureCliCommand {
    Help,
    List(ProjectId),
    Coverage(ProjectId),
    Changes(ProjectId),
    Read(ProjectId, CaptureId),
    Preview(PathBuf),
    Apply(PathBuf, String),
    Delivery(crate::capture_delivery_cli::Command),
    Repository(crate::repository_capture_cli::Command),
    Consolidate(crate::capture_consolidation_cli::Command),
}

pub(crate) fn parse(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a capture subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(CaptureCliCommand::Help),
        "list" => parse_list(&args[1..]),
        "coverage" => parse_coverage(&args[1..]),
        "changes" => parse_changes(&args[1..]),
        "read" => parse_read(&args[1..]),
        "preview" => parse_intake(false, &args[1..]),
        "apply" => parse_intake(true, &args[1..]),
        "delivery" => parse_delivery_route(&args[1..]),
        "repository" => parse_repository_route(&args[1..]),
        "consolidate" => parse_consolidation_route(&args[1..]),
        _ => Err("unknown capture subcommand"),
    }
}

pub(crate) fn execute(
    command: CaptureCliCommand,
    service: &ProjectStateService,
) -> Result<CaptureCliOutput, ProjectError> {
    match command {
        CaptureCliCommand::Help => unreachable!("capture help returns before service execution"),
        CaptureCliCommand::List(project_id) => service.capture_inbox(&project_id).map(|inbox| {
            CaptureCliOutput::Inbox(CaptureInboxOutput {
                schema_version: 1,
                command: "project-capture-list",
                inbox,
            })
        }),
        CaptureCliCommand::Coverage(project_id) => {
            service.capture_coverage(&project_id).map(|coverage| {
                CaptureCliOutput::Coverage(CaptureCoverageOutput {
                    schema_version: 1,
                    command: "project-capture-coverage",
                    coverage,
                })
            })
        }
        CaptureCliCommand::Changes(project_id) => {
            service.artifact_changes(&project_id).map(|changes| {
                CaptureCliOutput::Changes(ArtifactChangesOutput {
                    schema_version: 1,
                    command: "project-capture-artifact-changes",
                    changes,
                })
            })
        }
        CaptureCliCommand::Read(project_id, capture_id) => service
            .read_capture(&project_id, &capture_id)
            .map(|capture| {
                CaptureCliOutput::Capture(CaptureReadOutput {
                    schema_version: 1,
                    command: "project-capture-read",
                    capture,
                })
            }),
        CaptureCliCommand::Preview(file) => {
            let capture = read_portable_capture_packet(file)?;
            service.preview_capture(capture).map(|plan| {
                CaptureCliOutput::Preview(CapturePreviewOutput {
                    schema_version: 1,
                    command: "project-capture-preview",
                    preview: plan.preview().clone(),
                })
            })
        }
        CaptureCliCommand::Apply(file, digest) => {
            let capture = read_portable_capture_packet(file)?;
            let plan = service.preview_capture(capture)?;
            let commit = service.apply_capture(
                &plan,
                &ApprovedCaptureIntake::new(digest, true),
                now_unix()?,
            )?;
            Ok(CaptureCliOutput::Commit(CaptureCommitOutput {
                schema_version: 1,
                command: "project-capture-apply",
                commit,
            }))
        }
        CaptureCliCommand::Delivery(command) => {
            crate::capture_delivery_cli::execute(command, service).map(CaptureCliOutput::Delivery)
        }
        CaptureCliCommand::Repository(command) => {
            crate::repository_capture_cli::execute(command, service)
                .map(CaptureCliOutput::Repository)
        }
        CaptureCliCommand::Consolidate(command) => {
            crate::capture_consolidation_cli::execute(command, service)
                .map(CaptureCliOutput::Consolidation)
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum CaptureCliOutput {
    Inbox(CaptureInboxOutput),
    Coverage(CaptureCoverageOutput),
    Changes(ArtifactChangesOutput),
    Capture(CaptureReadOutput),
    Preview(CapturePreviewOutput),
    Commit(CaptureCommitOutput),
    Delivery(crate::capture_delivery_cli::Output),
    Repository(crate::repository_capture_cli::Output),
    Consolidation(crate::capture_consolidation_cli::Output),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptureInboxOutput {
    schema_version: u32,
    command: &'static str,
    inbox: CaptureInboxSnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptureCoverageOutput {
    schema_version: u32,
    command: &'static str,
    coverage: CaptureCoverageSnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactChangesOutput {
    schema_version: u32,
    command: &'static str,
    changes: ArtifactChangeSnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptureReadOutput {
    schema_version: u32,
    command: &'static str,
    capture: Option<ResearchCaptureV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CapturePreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: CaptureIntakePreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptureCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: CaptureIntakeCommitV1,
}

fn parse_list(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    let mut project_id = None;
    parse_identity_options(args, &mut project_id, None)?;
    Ok(CaptureCliCommand::List(
        project_id.ok_or("project ID is required")?,
    ))
}

fn parse_coverage(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    let mut project_id = None;
    parse_identity_options(args, &mut project_id, None)?;
    Ok(CaptureCliCommand::Coverage(
        project_id.ok_or("project ID is required")?,
    ))
}

fn parse_changes(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    let mut project_id = None;
    parse_identity_options(args, &mut project_id, None)?;
    Ok(CaptureCliCommand::Changes(
        project_id.ok_or("project ID is required")?,
    ))
}

fn parse_read(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    let mut project_id = None;
    let mut capture_id = None;
    parse_identity_options(args, &mut project_id, Some(&mut capture_id))?;
    Ok(CaptureCliCommand::Read(
        project_id.ok_or("project ID is required")?,
        capture_id.ok_or("capture ID is required")?,
    ))
}

fn parse_identity_options(
    args: &[OsString],
    project_id: &mut Option<ProjectId>,
    mut capture_id: Option<&mut Option<CaptureId>>,
) -> Result<(), &'static str> {
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("capture option is not valid UTF-8")?;
        let value = args
            .get(index + 1)
            .ok_or("capture option value is required")?
            .to_str()
            .ok_or("capture option value is not valid UTF-8")?;
        match option {
            "--project-id" if project_id.is_none() => {
                *project_id =
                    Some(ProjectId::parse(value.to_string()).map_err(|_| "project ID is invalid")?);
            }
            "--capture-id"
                if capture_id
                    .as_ref()
                    .is_some_and(|capture_id| capture_id.is_none()) =>
            {
                **capture_id.as_mut().expect("guarded above") =
                    Some(CaptureId::parse(value.to_string()).map_err(|_| "capture ID is invalid")?);
            }
            "--project-id" | "--capture-id" => {
                return Err("capture option is unexpected or duplicate");
            }
            _ => return Err("unknown capture option"),
        }
        index += 2;
    }
    Ok(())
}

fn parse_intake(apply: bool, args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    let mut file = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("capture intake option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("capture approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("capture intake option value is required")?;
        match option {
            "--file" if file.is_none() => file = Some(PathBuf::from(value)),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--file" | "--expected-plan-digest" => {
                return Err("capture intake option is unexpected or duplicate");
            }
            _ => return Err("unknown capture intake option"),
        }
        index += 2;
    }
    let file = file.ok_or("capture file is required")?;
    if apply {
        if !approved || digest.is_none() {
            return Err("capture apply requires plan digest and filesystem approval");
        }
        Ok(CaptureCliCommand::Apply(
            file,
            digest.expect("validated above"),
        ))
    } else {
        Ok(CaptureCliCommand::Preview(file))
    }
}

fn parse_consolidation_route(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    match crate::capture_consolidation_cli::parse(args)? {
        crate::capture_consolidation_cli::Command::Help => Ok(CaptureCliCommand::Help),
        command => Ok(CaptureCliCommand::Consolidate(command)),
    }
}

fn parse_delivery_route(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    match crate::capture_delivery_cli::parse(args)? {
        crate::capture_delivery_cli::Command::Help => Ok(CaptureCliCommand::Help),
        command => Ok(CaptureCliCommand::Delivery(command)),
    }
}

fn parse_repository_route(args: &[OsString]) -> Result<CaptureCliCommand, &'static str> {
    match crate::repository_capture_cli::parse(args)? {
        crate::repository_capture_cli::Command::Help => Ok(CaptureCliCommand::Help),
        command => Ok(CaptureCliCommand::Repository(command)),
    }
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
    fn parser_closes_capture_read_and_mutation_shapes() {
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        let capture_id = format!("cap_{}", "a".repeat(64));
        assert!(matches!(
            parse(&args(&["list", "--project-id", project_id])),
            Ok(CaptureCliCommand::List(_))
        ));
        assert!(matches!(
            parse(&args(&["coverage", "--project-id", project_id])),
            Ok(CaptureCliCommand::Coverage(_))
        ));
        assert!(matches!(
            parse(&args(&["changes", "--project-id", project_id])),
            Ok(CaptureCliCommand::Changes(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "read",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
            ])),
            Ok(CaptureCliCommand::Read(_, _))
        ));
        assert!(matches!(
            parse(&args(&["preview", "--file", "/tmp/capture.json"])),
            Ok(CaptureCliCommand::Preview(_))
        ));
        assert!(
            parse(&args(&[
                "apply",
                "--file",
                "/tmp/capture.json",
                "--expected-plan-digest",
                &"a".repeat(64),
            ]))
            .is_err()
        );
        assert!(
            parse(&args(&[
                "preview",
                "--file",
                "/tmp/capture.json",
                "--approve-filesystem-write",
            ]))
            .is_err()
        );

        assert!(matches!(
            parse(&args(&[
                "consolidate",
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--reviewed-at-unix",
                "1721337601",
            ])),
            Ok(CaptureCliCommand::Consolidate(
                crate::capture_consolidation_cli::Command::Preview(_)
            ))
        ));
    }
}
