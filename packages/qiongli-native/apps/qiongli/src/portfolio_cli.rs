use std::ffi::{OsStr, OsString};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    ApprovedPortfolioMaintenance, IncrementalPortfolioService, PortfolioCancellationToken,
    PortfolioMaintenancePreviewV1, PortfolioQueryResultV1, PortfolioQueryService, PortfolioQueryV1,
    ProjectStateService, SemanticTimelineQueryV1, SemanticTimelineResultV1,
    SemanticTimelineService,
};
use serde::Serialize;

use crate::command::CliOutput;

pub(crate) const USAGE: &str = "Qiongli incremental portfolio\n\nUsage:\n  qiongli project portfolio status\n  qiongli project portfolio reconcile <preview|apply> [--expected-plan-digest <sha256> --approve-derived-state-write]\n  qiongli project portfolio rebuild <preview|apply> [--expected-plan-digest <sha256> --approve-derived-state-write]\n  qiongli project portfolio delete-derived-state <preview|apply> [--expected-plan-digest <sha256> --approve-derived-state-write]\n  qiongli project portfolio query --request-json <canonical-query-json>\n  qiongli project portfolio timeline --request-json <canonical-timeline-query-json>\n  qiongli project portfolio doctor\n  qiongli project portfolio --help\n\nMutation rules:\n  Preview is read-only and returns a digest bound to the current Library and catalog generation.\n  Apply requires the exact preview digest and explicit derived-state approval.\n  Delete removes only the private rebuildable portfolio catalog; project artifacts and receipts remain.\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PortfolioMutationKind {
    Reconcile,
    FullRebuild,
    DeleteDerivedState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PortfolioCliCommand {
    Help,
    Status,
    Preview(PortfolioMutationKind),
    Apply(PortfolioMutationKind, String),
    Query(PortfolioQueryV1),
    Timeline(SemanticTimelineQueryV1),
    Doctor,
}

pub(crate) fn parse(args: &[OsString]) -> Result<PortfolioCliCommand, &'static str> {
    let Some(command) = args.first().and_then(|value| value.to_str()) else {
        return Err("portfolio subcommand is required");
    };
    match command {
        "--help" if args.len() == 1 => Ok(PortfolioCliCommand::Help),
        "status" if args.len() == 1 => Ok(PortfolioCliCommand::Status),
        "doctor" if args.len() == 1 => Ok(PortfolioCliCommand::Doctor),
        "reconcile" => parse_mutation(&args[1..], PortfolioMutationKind::Reconcile),
        "rebuild" => parse_mutation(&args[1..], PortfolioMutationKind::FullRebuild),
        "delete-derived-state" => {
            parse_mutation(&args[1..], PortfolioMutationKind::DeleteDerivedState)
        }
        "query" => parse_query(&args[1..]),
        "timeline" => parse_timeline(&args[1..]),
        "--help" | "status" | "doctor" => Err("unexpected portfolio argument"),
        _ => Err("unknown portfolio subcommand"),
    }
}

pub(crate) fn execute(command: PortfolioCliCommand, projects: &ProjectStateService) -> CliOutput {
    if command == PortfolioCliCommand::Help {
        return CliOutput::success_text(USAGE);
    }
    let portfolio = IncrementalPortfolioService::new(projects.clone());
    let output = match command {
        PortfolioCliCommand::Help => unreachable!("help returns before portfolio dispatch"),
        PortfolioCliCommand::Status => {
            projects
                .snapshot()
                .and_then(|library| match portfolio.current() {
                    Ok(current) => Ok(PortfolioCliOutput::Status(PortfolioStatusOutput {
                        schema_version: 1,
                        command: "project-portfolio-status",
                        state: "current",
                        library_revision: library.revision,
                        catalog: Some(current.catalog),
                    })),
                    Err(qiongli_project::ProjectError::RecoveryRequired) => {
                        let catalog = projects.portfolio_catalog_snapshot()?;
                        Ok(PortfolioCliOutput::Status(PortfolioStatusOutput {
                            schema_version: 1,
                            command: "project-portfolio-status",
                            state: if catalog.is_some() {
                                "recovery-required"
                            } else {
                                "missing"
                            },
                            library_revision: library.revision,
                            catalog,
                        }))
                    }
                    Err(qiongli_project::ProjectError::RevisionConflict) => {
                        projects.portfolio_catalog_snapshot().map(|catalog| {
                            PortfolioCliOutput::Status(PortfolioStatusOutput {
                                schema_version: 1,
                                command: "project-portfolio-status",
                                state: "stale",
                                library_revision: library.revision,
                                catalog,
                            })
                        })
                    }
                    Err(error) => Err(error),
                })
        }
        PortfolioCliCommand::Preview(kind) => preview(&portfolio, kind).map(|plan| {
            PortfolioCliOutput::Preview(PortfolioPreviewOutput {
                schema_version: 1,
                command: preview_command(kind),
                preview: plan.preview().clone(),
            })
        }),
        PortfolioCliCommand::Apply(kind, digest) => {
            let plan = match preview(&portfolio, kind) {
                Ok(plan) => plan,
                Err(error) => return CliOutput::operation_failure(error.reason_code()),
            };
            let approval = ApprovedPortfolioMaintenance::new(digest, true);
            match kind {
                PortfolioMutationKind::Reconcile => {
                    let now = match now_unix() {
                        Ok(now) => now,
                        Err(error) => return CliOutput::operation_failure(error),
                    };
                    portfolio
                        .apply_reconcile(&plan, &approval, now, &PortfolioCancellationToken::new())
                        .map(|reconciliation| {
                            PortfolioCliOutput::Reconciliation(PortfolioReconciliationOutput {
                                schema_version: 1,
                                command: "project-portfolio-reconcile-apply",
                                reconciliation,
                            })
                        })
                }
                PortfolioMutationKind::FullRebuild => {
                    let now = match now_unix() {
                        Ok(now) => now,
                        Err(error) => return CliOutput::operation_failure(error),
                    };
                    portfolio
                        .apply_full_rebuild(
                            &plan,
                            &approval,
                            now,
                            &PortfolioCancellationToken::new(),
                        )
                        .map(|reconciliation| {
                            PortfolioCliOutput::Reconciliation(PortfolioReconciliationOutput {
                                schema_version: 1,
                                command: "project-portfolio-rebuild-apply",
                                reconciliation,
                            })
                        })
                }
                PortfolioMutationKind::DeleteDerivedState => portfolio
                    .apply_delete_derived_state(&plan, &approval)
                    .map(|deletion| {
                        PortfolioCliOutput::Deletion(PortfolioDeletionOutput {
                            schema_version: 1,
                            command: "project-portfolio-delete-derived-state-apply",
                            deletion,
                        })
                    }),
            }
        }
        PortfolioCliCommand::Query(query) => PortfolioQueryService::new(projects.clone())
            .query(&query)
            .map(|result| {
                PortfolioCliOutput::Query(PortfolioQueryOutput {
                    schema_version: 1,
                    command: "project-portfolio-query",
                    result,
                })
            }),
        PortfolioCliCommand::Timeline(query) => SemanticTimelineService::new(projects.clone())
            .query(&query)
            .map(|result| {
                PortfolioCliOutput::Timeline(PortfolioTimelineOutput {
                    schema_version: 1,
                    command: "project-portfolio-timeline",
                    result,
                })
            }),
        PortfolioCliCommand::Doctor => portfolio.doctor_compare().map(|doctor| {
            PortfolioCliOutput::Doctor(PortfolioDoctorOutput {
                schema_version: 1,
                command: "project-portfolio-doctor",
                doctor,
            })
        }),
    };
    match output {
        Ok(output) => json_output(&output),
        Err(error) => CliOutput::operation_failure(error.reason_code()),
    }
}

fn preview(
    portfolio: &IncrementalPortfolioService,
    kind: PortfolioMutationKind,
) -> Result<qiongli_project::VerifiedPortfolioMaintenance, qiongli_project::ProjectError> {
    match kind {
        PortfolioMutationKind::Reconcile => portfolio.preview_reconcile(),
        PortfolioMutationKind::FullRebuild => portfolio.preview_full_rebuild(),
        PortfolioMutationKind::DeleteDerivedState => portfolio.preview_delete_derived_state(),
    }
}

fn parse_mutation(
    args: &[OsString],
    kind: PortfolioMutationKind,
) -> Result<PortfolioCliCommand, &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("portfolio mutation mode is required");
    };
    if mode == "preview" {
        return if args.len() == 1 {
            Ok(PortfolioCliCommand::Preview(kind))
        } else {
            Err("portfolio preview does not accept apply options")
        };
    }
    if mode != "apply" {
        return Err("portfolio mutation mode must be preview or apply");
    }
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("portfolio option is not valid UTF-8")?;
        if option == "--approve-derived-state-write" {
            if approved {
                return Err("portfolio approval is duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("portfolio option value is required")?;
        match option {
            "--expected-plan-digest" if expected_plan_digest.is_none() => {
                expected_plan_digest = Some(parse_sha256(value)?);
            }
            "--expected-plan-digest" => return Err("portfolio plan digest is duplicate"),
            _ => return Err("unknown portfolio option"),
        }
        index += 2;
    }
    if !approved {
        return Err("portfolio apply requires derived-state approval");
    }
    Ok(PortfolioCliCommand::Apply(
        kind,
        expected_plan_digest.ok_or("portfolio apply plan digest is required")?,
    ))
}

fn parse_query(args: &[OsString]) -> Result<PortfolioCliCommand, &'static str> {
    parse_request_json(args, "portfolio query")
        .and_then(|bytes| {
            PortfolioQueryV1::from_json_slice(bytes).map_err(|_| "portfolio query JSON is invalid")
        })
        .map(PortfolioCliCommand::Query)
}

fn parse_timeline(args: &[OsString]) -> Result<PortfolioCliCommand, &'static str> {
    parse_request_json(args, "portfolio timeline")
        .and_then(|bytes| {
            SemanticTimelineQueryV1::from_json_slice(bytes)
                .map_err(|_| "portfolio timeline JSON is invalid")
        })
        .map(PortfolioCliCommand::Timeline)
}

fn parse_request_json<'a>(
    args: &'a [OsString],
    label: &'static str,
) -> Result<&'a [u8], &'static str> {
    if args.len() != 2 || args[0] != OsStr::new("--request-json") {
        return Err(match label {
            "portfolio query" => "portfolio query requires exactly one --request-json",
            _ => "portfolio timeline requires exactly one --request-json",
        });
    }
    args[1].to_str().map(str::as_bytes).ok_or(match label {
        "portfolio query" => "portfolio query JSON is invalid",
        _ => "portfolio timeline JSON is invalid",
    })
}

fn parse_sha256(value: &OsStr) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("portfolio plan digest is invalid")?;
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(value.to_string())
    } else {
        Err("portfolio plan digest is invalid")
    }
}

fn preview_command(kind: PortfolioMutationKind) -> &'static str {
    match kind {
        PortfolioMutationKind::Reconcile => "project-portfolio-reconcile-preview",
        PortfolioMutationKind::FullRebuild => "project-portfolio-rebuild-preview",
        PortfolioMutationKind::DeleteDerivedState => {
            "project-portfolio-delete-derived-state-preview"
        }
    }
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "portfolio-clock-unavailable")
}

fn json_output<T: Serialize>(value: &T) -> CliOutput {
    match serde_json::to_string_pretty(value) {
        Ok(rendered) => CliOutput::success_text(format!("{rendered}\n")),
        Err(_) => CliOutput::operation_failure("output-serialization-failed"),
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum PortfolioCliOutput {
    Status(PortfolioStatusOutput),
    Preview(PortfolioPreviewOutput),
    Reconciliation(PortfolioReconciliationOutput),
    Deletion(PortfolioDeletionOutput),
    Query(PortfolioQueryOutput),
    Timeline(PortfolioTimelineOutput),
    Doctor(PortfolioDoctorOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioStatusOutput {
    schema_version: u32,
    command: &'static str,
    state: &'static str,
    library_revision: u64,
    catalog: Option<qiongli_project::PortfolioCatalogSnapshotV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: PortfolioMaintenancePreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioReconciliationOutput {
    schema_version: u32,
    command: &'static str,
    reconciliation: qiongli_project::PortfolioReconciliationV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioDeletionOutput {
    schema_version: u32,
    command: &'static str,
    deletion: qiongli_project::PortfolioDerivedStateDeletionV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioQueryOutput {
    schema_version: u32,
    command: &'static str,
    result: PortfolioQueryResultV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioTimelineOutput {
    schema_version: u32,
    command: &'static str,
    result: SemanticTimelineResultV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioDoctorOutput {
    schema_version: u32,
    command: &'static str,
    doctor: qiongli_project::PortfolioDoctorV1,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_accepts_closed_status_maintenance_and_request_shapes() {
        assert_eq!(parse(&args(&["status"])), Ok(PortfolioCliCommand::Status));
        assert_eq!(
            parse(&args(&["reconcile", "preview"])),
            Ok(PortfolioCliCommand::Preview(
                PortfolioMutationKind::Reconcile
            ))
        );
        assert_eq!(
            parse(&args(&[
                "delete-derived-state",
                "apply",
                "--expected-plan-digest",
                &"a".repeat(64),
                "--approve-derived-state-write",
            ])),
            Ok(PortfolioCliCommand::Apply(
                PortfolioMutationKind::DeleteDerivedState,
                "a".repeat(64),
            ))
        );
        let portfolio_query =
            PortfolioQueryV1::new(format!("pca_{}", "b".repeat(64))).expect("query is valid");
        let portfolio_json = String::from_utf8(
            portfolio_query
                .to_canonical_json()
                .expect("query serializes"),
        )
        .expect("query is UTF-8");
        assert_eq!(
            parse(&args(&["query", "--request-json", &portfolio_json])),
            Ok(PortfolioCliCommand::Query(portfolio_query))
        );
        let timeline_query = SemanticTimelineQueryV1::new(format!("pca_{}", "c".repeat(64)))
            .expect("timeline is valid");
        let timeline_json = String::from_utf8(
            timeline_query
                .to_canonical_json()
                .expect("timeline serializes"),
        )
        .expect("timeline is UTF-8");
        assert_eq!(
            parse(&args(&["timeline", "--request-json", &timeline_json])),
            Ok(PortfolioCliCommand::Timeline(timeline_query))
        );
    }

    #[test]
    fn parser_rejects_paths_unknowns_and_unapproved_mutations() {
        assert_eq!(
            parse(&args(&["query", "--request-file", "/private/query.json"])),
            Err("portfolio query requires exactly one --request-json")
        );
        assert_eq!(
            parse(&args(&[
                "rebuild",
                "apply",
                "--expected-plan-digest",
                &"a".repeat(64),
            ])),
            Err("portfolio apply requires derived-state approval")
        );
        assert_eq!(
            parse(&args(&[
                "delete-derived-state",
                "preview",
                "--root",
                "/tmp"
            ])),
            Err("portfolio preview does not accept apply options")
        );
        assert_eq!(
            parse(&args(&["unknown"])),
            Err("unknown portfolio subcommand")
        );
    }
}
