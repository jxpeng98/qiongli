use std::ffi::OsString;

use qiongli_project::{
    ApprovedCaptureResolution, CaptureAssignmentReceiptId, CaptureResolutionCommitV1,
    CaptureResolutionDisposition, CaptureResolutionItemId, CaptureResolutionPreviewV1,
    CaptureResolutionReceiptId, CaptureResolutionReceiptV1, CaptureResolutionSelectionSetV1,
    CaptureResolutionSelectionV1, ProjectError, ProjectId, ProjectStateService,
};
use serde::Serialize;

pub(crate) const USAGE: &str = "Qiongli Academic Capture Resolution\n\nUsage:\n  qiongli project capture resolution list --project-id <prj_id>\n  qiongli project capture resolution inspect --project-id <prj_id> --receipt-id <crr_id>\n  qiongli project capture resolution preview --assignment-receipt-id <car_id> --reviewed-at-unix <timestamp> [--select <cri_id>=<disposition> ...]\n  qiongli project capture resolution apply --assignment-receipt-id <car_id> --reviewed-at-unix <timestamp> --resolved-at-unix <timestamp> --select <cri_id>=<disposition> [...] --expected-plan-digest <sha256> --expected-selection-digest <sha256> --approve-academic-review --approve-filesystem-write\n\nDispositions are accept-current, accept-capture, retain-both, and reject-capture. Preview without selections returns the closed item set. Preview with a complete ordered selection set also returns its digest. Apply recomputes both digests and rejects incomplete, reordered, unsupported, stale, or unapproved decisions.\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Help,
    List(ProjectId),
    Inspect(ProjectId, CaptureResolutionReceiptId),
    Preview(ResolutionOptions, Vec<CaptureResolutionSelectionV1>),
    Apply(ApplyOptions),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolutionOptions {
    assignment_receipt_id: CaptureAssignmentReceiptId,
    reviewed_at_unix: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyOptions {
    resolution: ResolutionOptions,
    resolved_at_unix: u64,
    selections: Vec<CaptureResolutionSelectionV1>,
    expected_plan_digest: String,
    expected_selection_digest: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    List(ListOutput),
    Inspect(Box<InspectOutput>),
    Preview(PreviewOutput),
    Commit(CommitOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListOutput {
    schema_version: u32,
    command: &'static str,
    resolutions: Vec<CaptureResolutionReceiptV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InspectOutput {
    schema_version: u32,
    command: &'static str,
    resolution: Option<CaptureResolutionReceiptV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: CaptureResolutionPreviewV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_set: Option<CaptureResolutionSelectionSetV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: CaptureResolutionCommitV1,
}

pub(crate) fn parse(args: &[OsString]) -> Result<Command, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a capture resolution subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::Help),
        "list" => parse_list(&args[1..]).map(Command::List),
        "inspect" => parse_inspect(&args[1..]),
        "preview" => parse_resolution_options(false, &args[1..]),
        "apply" => parse_resolution_options(true, &args[1..]),
        _ => Err("unknown capture resolution subcommand"),
    }
}

pub(crate) fn execute(
    command: Command,
    service: &ProjectStateService,
) -> Result<Output, ProjectError> {
    match command {
        Command::Help => unreachable!("resolution help returns before service execution"),
        Command::List(project_id) => {
            service
                .list_capture_resolutions(&project_id)
                .map(|resolutions| {
                    Output::List(ListOutput {
                        schema_version: 1,
                        command: "project-capture-resolution-list",
                        resolutions,
                    })
                })
        }
        Command::Inspect(project_id, receipt_id) => service
            .inspect_capture_resolution(&project_id, &receipt_id)
            .map(|resolution| {
                Output::Inspect(Box::new(InspectOutput {
                    schema_version: 1,
                    command: "project-capture-resolution-inspect",
                    resolution,
                }))
            }),
        Command::Preview(options, selections) => {
            let plan = preview(service, &options)?;
            let selection_set = if selections.is_empty() {
                None
            } else {
                Some(CaptureResolutionSelectionSetV1::new(
                    plan.resolution_plan(),
                    selections,
                )?)
            };
            Ok(Output::Preview(PreviewOutput {
                schema_version: 1,
                command: "project-capture-resolution-preview",
                preview: plan.preview().clone(),
                selection_set,
            }))
        }
        Command::Apply(options) => {
            let plan = preview(service, &options.resolution)?;
            let selections =
                CaptureResolutionSelectionSetV1::new(plan.resolution_plan(), options.selections)?;
            let approval = ApprovedCaptureResolution::new(
                options.expected_plan_digest,
                options.expected_selection_digest,
                true,
                true,
            );
            let commit = service.apply_capture_resolution(
                &plan,
                &selections,
                &approval,
                options.resolved_at_unix,
            )?;
            Ok(Output::Commit(CommitOutput {
                schema_version: 1,
                command: "project-capture-resolution-apply",
                commit,
            }))
        }
    }
}

fn preview(
    service: &ProjectStateService,
    options: &ResolutionOptions,
) -> Result<qiongli_project::VerifiedCaptureResolution, ProjectError> {
    service.preview_capture_resolution(&options.assignment_receipt_id, options.reviewed_at_unix)
}

fn parse_list(args: &[OsString]) -> Result<ProjectId, &'static str> {
    if args.len() != 2 || args[0] != "--project-id" {
        return Err("capture resolution list requires one project ID");
    }
    parse_project_id(&args[1])
}

fn parse_inspect(args: &[OsString]) -> Result<Command, &'static str> {
    let mut project_id = None;
    let mut receipt_id = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("capture resolution inspect option is not valid UTF-8")?;
        let value = args
            .get(index + 1)
            .ok_or("capture resolution inspect option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--receipt-id" if receipt_id.is_none() => {
                receipt_id = Some(parse_resolution_receipt_id(value)?);
            }
            "--project-id" | "--receipt-id" => {
                return Err("capture resolution inspect option is duplicate");
            }
            _ => return Err("unknown capture resolution inspect option"),
        }
        index += 2;
    }
    Ok(Command::Inspect(
        project_id.ok_or("project ID is required")?,
        receipt_id.ok_or("resolution receipt ID is required")?,
    ))
}

fn parse_resolution_options(apply: bool, args: &[OsString]) -> Result<Command, &'static str> {
    let mut assignment_receipt_id = None;
    let mut reviewed_at_unix = None;
    let mut resolved_at_unix = None;
    let mut selections = Vec::new();
    let mut expected_plan_digest = None;
    let mut expected_selection_digest = None;
    let mut academic_review = false;
    let mut filesystem_write = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("capture resolution option is not valid UTF-8")?;
        match option {
            "--approve-academic-review" => {
                if !apply || academic_review {
                    return Err("academic approval is unexpected or duplicate");
                }
                academic_review = true;
                index += 1;
                continue;
            }
            "--approve-filesystem-write" => {
                if !apply || filesystem_write {
                    return Err("filesystem approval is unexpected or duplicate");
                }
                filesystem_write = true;
                index += 1;
                continue;
            }
            _ => {}
        }
        let value = args
            .get(index + 1)
            .ok_or("capture resolution option value is required")?;
        match option {
            "--assignment-receipt-id" if assignment_receipt_id.is_none() => {
                assignment_receipt_id = Some(parse_assignment_receipt_id(value)?);
            }
            "--reviewed-at-unix" if reviewed_at_unix.is_none() => {
                reviewed_at_unix = Some(parse_timestamp(value, "review timestamp is invalid")?);
            }
            "--resolved-at-unix" if apply && resolved_at_unix.is_none() => {
                resolved_at_unix = Some(parse_timestamp(value, "resolution timestamp is invalid")?);
            }
            "--select" => selections.push(parse_selection(value)?),
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest = Some(parse_sha256(value)?);
            }
            "--expected-selection-digest" if apply && expected_selection_digest.is_none() => {
                expected_selection_digest = Some(parse_sha256(value)?);
            }
            "--assignment-receipt-id"
            | "--reviewed-at-unix"
            | "--resolved-at-unix"
            | "--expected-plan-digest"
            | "--expected-selection-digest" => {
                return Err("capture resolution option is unexpected or duplicate");
            }
            _ => return Err("unknown capture resolution option"),
        }
        index += 2;
    }
    let resolution = ResolutionOptions {
        assignment_receipt_id: assignment_receipt_id.ok_or("assignment receipt ID is required")?,
        reviewed_at_unix: reviewed_at_unix.ok_or("review timestamp is required")?,
    };
    if !apply {
        return Ok(Command::Preview(resolution, selections));
    }
    if selections.is_empty()
        || resolved_at_unix.is_none()
        || expected_plan_digest.is_none()
        || expected_selection_digest.is_none()
        || !academic_review
        || !filesystem_write
    {
        return Err(
            "capture resolution apply requires timestamps, selections, both digests, academic approval, and filesystem approval",
        );
    }
    Ok(Command::Apply(ApplyOptions {
        resolution,
        resolved_at_unix: resolved_at_unix.expect("validated above"),
        selections,
        expected_plan_digest: expected_plan_digest.expect("validated above"),
        expected_selection_digest: expected_selection_digest.expect("validated above"),
    }))
}

fn parse_project_id(value: &OsString) -> Result<ProjectId, &'static str> {
    value
        .to_str()
        .ok_or("project ID is not valid UTF-8")
        .and_then(|value| ProjectId::parse(value.to_owned()).map_err(|_| "project ID is invalid"))
}

fn parse_assignment_receipt_id(
    value: &OsString,
) -> Result<CaptureAssignmentReceiptId, &'static str> {
    value
        .to_str()
        .ok_or("assignment receipt ID is not valid UTF-8")
        .and_then(|value| {
            CaptureAssignmentReceiptId::parse(value.to_owned())
                .map_err(|_| "assignment receipt ID is invalid")
        })
}

fn parse_resolution_receipt_id(
    value: &OsString,
) -> Result<CaptureResolutionReceiptId, &'static str> {
    value
        .to_str()
        .ok_or("resolution receipt ID is not valid UTF-8")
        .and_then(|value| {
            CaptureResolutionReceiptId::parse(value.to_owned())
                .map_err(|_| "resolution receipt ID is invalid")
        })
}

fn parse_selection(value: &OsString) -> Result<CaptureResolutionSelectionV1, &'static str> {
    let value = value
        .to_str()
        .ok_or("capture resolution selection is not valid UTF-8")?;
    let (item_id, disposition) = value
        .split_once('=')
        .ok_or("capture resolution selection must be item-id=disposition")?;
    if disposition.contains('=') {
        return Err("capture resolution selection must be item-id=disposition");
    }
    Ok(CaptureResolutionSelectionV1 {
        item_id: CaptureResolutionItemId::parse(item_id.to_owned())
            .map_err(|_| "resolution item ID is invalid")?,
        disposition: match disposition {
            "accept-current" => CaptureResolutionDisposition::AcceptCurrent,
            "accept-capture" => CaptureResolutionDisposition::AcceptCapture,
            "retain-both" => CaptureResolutionDisposition::RetainBoth,
            "reject-capture" => CaptureResolutionDisposition::RejectCapture,
            _ => return Err("capture resolution disposition is invalid"),
        },
    })
}

fn parse_timestamp(value: &OsString, error: &'static str) -> Result<u64, &'static str> {
    value
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .ok_or(error)
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
        .ok_or("digest must be 64 lowercase hexadecimal characters")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_accepts_closed_resolution_commands() {
        let project_id = "prj_0123456789abcdef0123456789abcdef";
        let assignment_id = format!("car_{}", "a".repeat(64));
        let receipt_id = format!("crr_{}", "b".repeat(64));
        let item_id = format!("cri_{}", "c".repeat(64));
        assert!(matches!(
            parse(&args(&["list", "--project-id", project_id])),
            Ok(Command::List(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "inspect",
                "--receipt-id",
                &receipt_id,
                "--project-id",
                project_id,
            ])),
            Ok(Command::Inspect(_, _))
        ));
        assert!(matches!(
            parse(&args(&[
                "preview",
                "--assignment-receipt-id",
                &assignment_id,
                "--reviewed-at-unix",
                "1800000020",
                "--select",
                &format!("{item_id}=accept-capture"),
            ])),
            Ok(Command::Preview(_, _))
        ));
        assert!(matches!(
            parse(&args(&[
                "apply",
                "--assignment-receipt-id",
                &assignment_id,
                "--reviewed-at-unix",
                "1800000020",
                "--resolved-at-unix",
                "1800000021",
                "--select",
                &format!("{item_id}=reject-capture"),
                "--expected-plan-digest",
                &"d".repeat(64),
                "--expected-selection-digest",
                &"e".repeat(64),
                "--approve-academic-review",
                "--approve-filesystem-write",
            ])),
            Ok(Command::Apply(_))
        ));
    }

    #[test]
    fn parser_rejects_paths_incomplete_apply_and_unknown_dispositions() {
        let assignment_id = format!("car_{}", "a".repeat(64));
        let item_id = format!("cri_{}", "c".repeat(64));
        for invalid in [
            args(&["list", "--project-id", "/private/project"]),
            args(&[
                "preview",
                "--assignment-receipt-id",
                &assignment_id,
                "--reviewed-at-unix",
                "1800000020",
                "--select",
                &format!("{item_id}=overwrite"),
            ]),
            args(&[
                "apply",
                "--assignment-receipt-id",
                &assignment_id,
                "--reviewed-at-unix",
                "1800000020",
                "--resolved-at-unix",
                "1800000021",
                "--select",
                &format!("{item_id}=accept-capture"),
                "--expected-plan-digest",
                &"d".repeat(64),
                "--expected-selection-digest",
                &"e".repeat(64),
                "--approve-academic-review",
            ]),
        ] {
            assert!(parse(&invalid).is_err());
        }
    }
}
