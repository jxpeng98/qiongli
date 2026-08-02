use std::ffi::OsString;

use qiongli_project::{
    ApprovedCaptureAssignment, CaptureAssignmentCommitV1, CaptureAssignmentDecision,
    CaptureAssignmentIntentId, CaptureAssignmentPreviewV1, CaptureAssignmentStatusV1,
    DeliveryEnvelopeId, ProjectError, ProjectId, ProjectStateService,
};
use serde::Serialize;

pub(crate) const USAGE: &str = "Qiongli Capture Assignment\n\nUsage:\n  qiongli project capture assignment list\n  qiongli project capture assignment inspect --intent-id <cai_id>\n  qiongli project capture assignment preview --source-envelope-id <env_id> --target-project-id <prj_id> --decision <assign|reject> --decided-at-unix <timestamp>\n  qiongli project capture assignment apply --source-envelope-id <env_id> --target-project-id <prj_id> --decision <assign|reject> --decided-at-unix <timestamp> --expected-plan-digest <sha256> --approve-assignment-write\n\nAssignment never edits the source capture or academic project content. Preview and apply expose only content-addressed lineage identities and revision evidence. Apply recomputes the exact preview and requires its plan digest plus explicit assignment-write approval.\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Help,
    List,
    Inspect(CaptureAssignmentIntentId),
    Preview(Options),
    Apply(Options, String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    source_envelope_id: DeliveryEnvelopeId,
    target_project_id: ProjectId,
    decision: CaptureAssignmentDecision,
    decided_at_unix: u64,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    List(ListOutput),
    Inspect(InspectOutput),
    Preview(PreviewOutput),
    Commit(CommitOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListOutput {
    schema_version: u32,
    command: &'static str,
    assignments: Vec<CaptureAssignmentStatusV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InspectOutput {
    schema_version: u32,
    command: &'static str,
    assignment: Option<CaptureAssignmentStatusV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: CaptureAssignmentPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: CaptureAssignmentCommitV1,
}

pub(crate) fn parse(args: &[OsString]) -> Result<Command, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a capture assignment subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::Help),
        "list" if args.len() == 1 => Ok(Command::List),
        "inspect" => parse_inspect(&args[1..]).map(Command::Inspect),
        "preview" => parse_options(false, &args[1..]),
        "apply" => parse_options(true, &args[1..]),
        "--help" | "list" => Err("unexpected capture assignment argument"),
        _ => Err("unknown capture assignment subcommand"),
    }
}

pub(crate) fn execute(
    command: Command,
    service: &ProjectStateService,
) -> Result<Output, ProjectError> {
    match command {
        Command::Help => unreachable!("assignment help returns before service execution"),
        Command::List => service.list_capture_assignments().map(|assignments| {
            Output::List(ListOutput {
                schema_version: 1,
                command: "project-capture-assignment-list",
                assignments,
            })
        }),
        Command::Inspect(intent_id) => {
            service
                .inspect_capture_assignment(&intent_id)
                .map(|assignment| {
                    Output::Inspect(InspectOutput {
                        schema_version: 1,
                        command: "project-capture-assignment-inspect",
                        assignment,
                    })
                })
        }
        Command::Preview(options) => preview(service, &options).map(|plan| {
            Output::Preview(PreviewOutput {
                schema_version: 1,
                command: "project-capture-assignment-preview",
                preview: plan.preview().clone(),
            })
        }),
        Command::Apply(options, digest) => {
            let plan = preview(service, &options)?;
            let commit = service
                .apply_capture_assignment(&plan, &ApprovedCaptureAssignment::new(digest, true))?;
            Ok(Output::Commit(CommitOutput {
                schema_version: 1,
                command: "project-capture-assignment-apply",
                commit,
            }))
        }
    }
}

fn preview(
    service: &ProjectStateService,
    options: &Options,
) -> Result<qiongli_project::VerifiedCaptureAssignment, ProjectError> {
    service.preview_capture_assignment(
        &options.source_envelope_id,
        &options.target_project_id,
        options.decision,
        options.decided_at_unix,
    )
}

fn parse_inspect(args: &[OsString]) -> Result<CaptureAssignmentIntentId, &'static str> {
    if args.len() != 2 || args[0] != "--intent-id" {
        return Err("capture assignment inspect requires one intent ID");
    }
    parse_intent_id(&args[1])
}

fn parse_options(apply: bool, args: &[OsString]) -> Result<Command, &'static str> {
    let mut source_envelope_id = None;
    let mut target_project_id = None;
    let mut decision = None;
    let mut decided_at_unix = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("capture assignment option is not valid UTF-8")?;
        if option == "--approve-assignment-write" {
            if !apply || approved {
                return Err("capture assignment approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("capture assignment option value is required")?;
        match option {
            "--source-envelope-id" if source_envelope_id.is_none() => {
                source_envelope_id = Some(parse_envelope_id(value)?);
            }
            "--target-project-id" if target_project_id.is_none() => {
                target_project_id = Some(parse_project_id(value)?);
            }
            "--decision" if decision.is_none() => decision = Some(parse_decision(value)?),
            "--decided-at-unix" if decided_at_unix.is_none() => {
                decided_at_unix = Some(parse_timestamp(value)?);
            }
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source-envelope-id"
            | "--target-project-id"
            | "--decision"
            | "--decided-at-unix"
            | "--expected-plan-digest" => {
                return Err("capture assignment option is unexpected or duplicate");
            }
            _ => return Err("unknown capture assignment option"),
        }
        index += 2;
    }
    let options = Options {
        source_envelope_id: source_envelope_id.ok_or("source envelope ID is required")?,
        target_project_id: target_project_id.ok_or("target project ID is required")?,
        decision: decision.ok_or("capture assignment decision is required")?,
        decided_at_unix: decided_at_unix.ok_or("assignment decision timestamp is required")?,
    };
    if !apply {
        return Ok(Command::Preview(options));
    }
    if digest.is_none() || !approved {
        return Err("capture assignment apply requires plan digest and assignment-write approval");
    }
    Ok(Command::Apply(options, digest.expect("validated above")))
}

fn parse_envelope_id(value: &OsString) -> Result<DeliveryEnvelopeId, &'static str> {
    value
        .to_str()
        .ok_or("source envelope ID is not valid UTF-8")
        .and_then(|value| {
            DeliveryEnvelopeId::parse(value.to_owned()).map_err(|_| "source envelope ID is invalid")
        })
}

fn parse_intent_id(value: &OsString) -> Result<CaptureAssignmentIntentId, &'static str> {
    value
        .to_str()
        .ok_or("assignment intent ID is not valid UTF-8")
        .and_then(|value| {
            CaptureAssignmentIntentId::parse(value.to_owned())
                .map_err(|_| "assignment intent ID is invalid")
        })
}

fn parse_project_id(value: &OsString) -> Result<ProjectId, &'static str> {
    value
        .to_str()
        .ok_or("target project ID is not valid UTF-8")
        .and_then(|value| {
            ProjectId::parse(value.to_owned()).map_err(|_| "target project ID is invalid")
        })
}

fn parse_decision(value: &OsString) -> Result<CaptureAssignmentDecision, &'static str> {
    match value.to_str() {
        Some("assign") => Ok(CaptureAssignmentDecision::Assign),
        Some("reject") => Ok(CaptureAssignmentDecision::Reject),
        _ => Err("capture assignment decision is invalid"),
    }
}

fn parse_timestamp(value: &OsString) -> Result<u64, &'static str> {
    value
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .ok_or("assignment decision timestamp is invalid")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    fn envelope_id() -> String {
        format!("env_{}", "a".repeat(64))
    }

    #[test]
    fn parser_accepts_closed_assignment_commands() {
        let envelope_id = envelope_id();
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        let intent_id = format!("cai_{}", "b".repeat(64));
        assert!(matches!(parse(&args(&["list"])), Ok(Command::List)));
        assert!(matches!(
            parse(&args(&["inspect", "--intent-id", &intent_id])),
            Ok(Command::Inspect(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "preview",
                "--source-envelope-id",
                &envelope_id,
                "--target-project-id",
                project_id,
                "--decision",
                "assign",
                "--decided-at-unix",
                "1800000010",
            ])),
            Ok(Command::Preview(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "apply",
                "--source-envelope-id",
                &envelope_id,
                "--target-project-id",
                project_id,
                "--decision",
                "reject",
                "--decided-at-unix",
                "1800000010",
                "--expected-plan-digest",
                &"c".repeat(64),
                "--approve-assignment-write",
            ])),
            Ok(Command::Apply(_, _))
        ));
    }

    #[test]
    fn parser_rejects_paths_unknowns_and_missing_approval() {
        let envelope_id = envelope_id();
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        for invalid in [
            args(&["inspect", "--intent-id", "/private/assignment"]),
            args(&[
                "preview",
                "--source-envelope-id",
                &envelope_id,
                "--target-project-id",
                project_id,
                "--decision",
                "merge",
                "--decided-at-unix",
                "1800000010",
            ]),
            args(&[
                "apply",
                "--source-envelope-id",
                &envelope_id,
                "--target-project-id",
                project_id,
                "--decision",
                "assign",
                "--decided-at-unix",
                "1800000010",
                "--expected-plan-digest",
                &"c".repeat(64),
            ]),
        ] {
            assert!(parse(&invalid).is_err());
        }
    }
}
