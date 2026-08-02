use std::ffi::OsString;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    ApprovedCaptureIntake, ApprovedRepositoryCaptureDelivery, CaptureId, CaptureIntakeCommitV1,
    ProjectError, ProjectId, ProjectStateService, RepositoryCaptureDeliveryCommitV1,
    RepositoryCaptureDeliveryPreviewV1, RepositoryCaptureInboxSnapshotV1,
    RepositoryCaptureIntakePreviewV1, ResearchCaptureV1,
};
use serde::Serialize;

pub(crate) const USAGE: &str = "Repository Capture Inbox\n\nUsage:\n  qiongli project capture repository list --project-id <prj_id>\n  qiongli project capture repository read --project-id <prj_id> --capture-id <cap_id>\n  qiongli project capture repository preview --project-id <prj_id> --capture-id <cap_id>\n  qiongli project capture repository apply --project-id <prj_id> --capture-id <cap_id> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project capture repository delivery preview --project-id <prj_id> --capture-id <cap_id> --queued-at-unix <timestamp>\n  qiongli project capture repository delivery apply --project-id <prj_id> --capture-id <cap_id> --queued-at-unix <timestamp> --expected-plan-digest <sha256> --approve-delivery-write\n\nRepository agents place strict repository-backed packets at context/capture-inbox/<cap_id>.json inside an already registered project. Qiongli never accepts an arbitrary repository path through this adapter.\nThe delivery adapter binds the raw repository digest and any stable unattributed artifact-change identities into a content-addressed delivery preview. It does not invoke Git or infer an author or client. Delivery apply feeds the same assignment and academic-resolution services used by other surfaces.\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Help,
    List(ProjectId),
    Read(ProjectId, CaptureId),
    Preview(ProjectId, CaptureId),
    Apply(ProjectId, CaptureId, String),
    DeliveryPreview(ProjectId, CaptureId, u64),
    DeliveryApply(ProjectId, CaptureId, u64, String),
}

pub(crate) fn parse(args: &[OsString]) -> Result<Command, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a repository capture subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::Help),
        "list" => parse_options(&args[1..], false, false).map(|options| {
            Command::List(
                options
                    .project_id
                    .expect("repository list requires a project ID"),
            )
        }),
        "read" => parse_options(&args[1..], true, false).map(|options| {
            Command::Read(
                options
                    .project_id
                    .expect("repository read requires a project ID"),
                options
                    .capture_id
                    .expect("repository read requires a capture ID"),
            )
        }),
        "preview" => parse_options(&args[1..], true, false).map(|options| {
            Command::Preview(
                options
                    .project_id
                    .expect("repository preview requires a project ID"),
                options
                    .capture_id
                    .expect("repository preview requires a capture ID"),
            )
        }),
        "apply" => parse_options(&args[1..], true, true).map(|options| {
            Command::Apply(
                options
                    .project_id
                    .expect("repository apply requires a project ID"),
                options
                    .capture_id
                    .expect("repository apply requires a capture ID"),
                options
                    .plan_digest
                    .expect("repository apply requires a plan digest"),
            )
        }),
        "delivery" => parse_delivery(&args[1..]),
        _ => Err("unknown repository capture subcommand"),
    }
}

pub(crate) fn execute(
    command: Command,
    service: &ProjectStateService,
) -> Result<Output, ProjectError> {
    match command {
        Command::Help => unreachable!("repository capture help returns before service execution"),
        Command::List(project_id) => service.repository_capture_inbox(&project_id).map(|inbox| {
            Output::Inbox(InboxOutput {
                schema_version: 1,
                command: "project-capture-repository-list",
                inbox,
            })
        }),
        Command::Read(project_id, capture_id) => service
            .read_repository_capture(&project_id, &capture_id)
            .map(|capture| {
                Output::Capture(CaptureOutput {
                    schema_version: 1,
                    command: "project-capture-repository-read",
                    capture,
                })
            }),
        Command::Preview(project_id, capture_id) => service
            .preview_repository_capture(&project_id, &capture_id)
            .map(|plan| {
                Output::Preview(PreviewOutput {
                    schema_version: 1,
                    command: "project-capture-repository-preview",
                    preview: plan.preview().clone(),
                })
            }),
        Command::Apply(project_id, capture_id, digest) => {
            let plan = service.preview_repository_capture(&project_id, &capture_id)?;
            let commit = service.apply_repository_capture(
                &plan,
                &ApprovedCaptureIntake::new(digest, true),
                now_unix()?,
            )?;
            Ok(Output::Commit(CommitOutput {
                schema_version: 1,
                command: "project-capture-repository-apply",
                commit,
            }))
        }
        Command::DeliveryPreview(project_id, capture_id, queued_at_unix) => service
            .preview_repository_capture_delivery(&project_id, &capture_id, queued_at_unix)
            .map(|plan| {
                Output::DeliveryPreview(DeliveryPreviewOutput {
                    schema_version: 1,
                    command: "project-capture-repository-delivery-preview",
                    preview: plan.preview().clone(),
                })
            }),
        Command::DeliveryApply(project_id, capture_id, queued_at_unix, digest) => {
            let plan = service.preview_repository_capture_delivery(
                &project_id,
                &capture_id,
                queued_at_unix,
            )?;
            let commit = service.apply_repository_capture_delivery(
                &plan,
                &ApprovedRepositoryCaptureDelivery::new(digest, true),
            )?;
            Ok(Output::DeliveryCommit(DeliveryCommitOutput {
                schema_version: 1,
                command: "project-capture-repository-delivery-apply",
                commit,
            }))
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    Inbox(InboxOutput),
    Capture(CaptureOutput),
    Preview(PreviewOutput),
    Commit(CommitOutput),
    DeliveryPreview(DeliveryPreviewOutput),
    DeliveryCommit(DeliveryCommitOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InboxOutput {
    schema_version: u32,
    command: &'static str,
    inbox: RepositoryCaptureInboxSnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptureOutput {
    schema_version: u32,
    command: &'static str,
    capture: Option<ResearchCaptureV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: RepositoryCaptureIntakePreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: CaptureIntakeCommitV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeliveryPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: RepositoryCaptureDeliveryPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeliveryCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: RepositoryCaptureDeliveryCommitV1,
}

#[derive(Default)]
struct Options {
    project_id: Option<ProjectId>,
    capture_id: Option<CaptureId>,
    plan_digest: Option<String>,
}

fn parse_options(
    args: &[OsString],
    capture_required: bool,
    apply: bool,
) -> Result<Options, &'static str> {
    let mut options = Options::default();
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("repository capture option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("repository capture approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("repository capture option value is required")?
            .to_str()
            .ok_or("repository capture option value is not valid UTF-8")?;
        match option {
            "--project-id" if options.project_id.is_none() => {
                options.project_id =
                    Some(ProjectId::parse(value.to_string()).map_err(|_| "project ID is invalid")?);
            }
            "--capture-id" if capture_required && options.capture_id.is_none() => {
                options.capture_id =
                    Some(CaptureId::parse(value.to_string()).map_err(|_| "capture ID is invalid")?);
            }
            "--expected-plan-digest" if apply && options.plan_digest.is_none() => {
                options.plan_digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--capture-id" | "--expected-plan-digest" => {
                return Err("repository capture option is unexpected or duplicate");
            }
            _ => return Err("unknown repository capture option"),
        }
        index += 2;
    }
    if options.project_id.is_none() {
        return Err("project ID is required");
    }
    if capture_required && options.capture_id.is_none() {
        return Err("capture ID is required");
    }
    if apply && (!approved || options.plan_digest.is_none()) {
        return Err("repository capture apply requires plan digest and filesystem approval");
    }
    Ok(options)
}

fn parse_delivery(args: &[OsString]) -> Result<Command, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a repository delivery subcommand is required");
    };
    let apply = match subcommand {
        "preview" => false,
        "apply" => true,
        _ => return Err("unknown repository delivery subcommand"),
    };
    let mut project_id = None;
    let mut capture_id = None;
    let mut queued_at_unix = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("repository delivery option is not valid UTF-8")?;
        if option == "--approve-delivery-write" {
            if !apply || approved {
                return Err("repository delivery approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("repository delivery option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => {
                let value = value.to_str().ok_or("project ID is not valid UTF-8")?;
                project_id =
                    Some(ProjectId::parse(value.to_owned()).map_err(|_| "project ID is invalid")?);
            }
            "--capture-id" if capture_id.is_none() => {
                let value = value.to_str().ok_or("capture ID is not valid UTF-8")?;
                capture_id =
                    Some(CaptureId::parse(value.to_owned()).map_err(|_| "capture ID is invalid")?);
            }
            "--queued-at-unix" if queued_at_unix.is_none() => {
                queued_at_unix = Some(parse_timestamp(value)?);
            }
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256_os(value)?);
            }
            "--project-id" | "--capture-id" | "--queued-at-unix" | "--expected-plan-digest" => {
                return Err("repository delivery option is unexpected or duplicate");
            }
            _ => return Err("unknown repository delivery option"),
        }
        index += 2;
    }
    let project_id = project_id.ok_or("project ID is required")?;
    let capture_id = capture_id.ok_or("capture ID is required")?;
    let queued_at_unix = queued_at_unix.ok_or("repository delivery timestamp is required")?;
    if !apply {
        return Ok(Command::DeliveryPreview(
            project_id,
            capture_id,
            queued_at_unix,
        ));
    }
    if digest.is_none() || !approved {
        return Err("repository delivery apply requires plan digest and delivery-write approval");
    }
    Ok(Command::DeliveryApply(
        project_id,
        capture_id,
        queued_at_unix,
        digest.expect("validated above"),
    ))
}

fn parse_timestamp(value: &OsString) -> Result<u64, &'static str> {
    value
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .ok_or("repository delivery timestamp is invalid")
}

fn parse_sha256_os(value: &OsString) -> Result<String, &'static str> {
    value
        .to_str()
        .ok_or("plan digest is not valid UTF-8")
        .and_then(parse_sha256)
}

fn parse_sha256(value: &str) -> Result<String, &'static str> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(value.to_string())
    } else {
        Err("plan digest must be 64 lowercase hexadecimal characters")
    }
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
    fn parser_closes_repository_identity_and_approval_shapes() {
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        let capture_id = format!("cap_{}", "a".repeat(64));
        assert!(matches!(
            parse(&args(&["list", "--project-id", project_id])),
            Ok(Command::List(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "preview",
                "--capture-id",
                &capture_id,
                "--project-id",
                project_id,
            ])),
            Ok(Command::Preview(_, _))
        ));
        assert!(matches!(
            parse(&args(&[
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--expected-plan-digest",
                &"b".repeat(64),
                "--approve-filesystem-write",
            ])),
            Ok(Command::Apply(_, _, _))
        ));
        assert!(
            parse(&args(&[
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--expected-plan-digest",
                &"b".repeat(64),
            ]))
            .is_err()
        );
        assert!(
            parse(&args(&[
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--repository-path",
                "/private/repository",
            ]))
            .is_err()
        );
        assert!(matches!(
            parse(&args(&[
                "delivery",
                "preview",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--queued-at-unix",
                "1800000010",
            ])),
            Ok(Command::DeliveryPreview(_, _, 1_800_000_010))
        ));
        assert!(matches!(
            parse(&args(&[
                "delivery",
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--queued-at-unix",
                "1800000010",
                "--expected-plan-digest",
                &"c".repeat(64),
                "--approve-delivery-write",
            ])),
            Ok(Command::DeliveryApply(_, _, 1_800_000_010, _))
        ));
        assert!(
            parse(&args(&[
                "delivery",
                "apply",
                "--project-id",
                project_id,
                "--capture-id",
                &capture_id,
                "--queued-at-unix",
                "1800000010",
                "--expected-plan-digest",
                &"c".repeat(64),
            ]))
            .is_err()
        );
    }
}
