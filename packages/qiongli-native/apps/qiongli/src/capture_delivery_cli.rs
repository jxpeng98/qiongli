use std::ffi::OsString;

use qiongli_project::{
    CaptureDeliveryRetryCause, CaptureDeliveryStatusV1, DeliveryEnvelopeId, ProjectError,
    ProjectStateService,
};
use serde::Serialize;

pub(crate) const USAGE: &str = "Qiongli Capture Delivery Ledger\n\nUsage:\n  qiongli project capture delivery list\n  qiongli project capture delivery inspect --envelope-id <env_id>\n  qiongli project capture delivery retry --envelope-id <env_id> --expected-generation <generation> --expected-record-sha256 <sha256> --retried-at-unix <timestamp> --cause <process-interrupted|transport-unavailable|destination-unavailable|recovery-required|conflict-resolved>\n  qiongli project capture delivery cancel --envelope-id <env_id> --expected-generation <generation> --expected-record-sha256 <sha256> --cancelled-at-unix <timestamp>\n  qiongli project capture delivery --help\n\nInspect output is path-redacted and contains only delivery identity, causal state, retry count, destination binding, digests, and acknowledgement summary. Retry and cancel require the exact current generation and record digest. They update only the private Qiongli delivery ledger and do not invoke a provider, model, Git, Python, Node, or host CLI.\n";

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Help,
    List,
    Inspect(DeliveryEnvelopeId),
    Retry {
        envelope_id: DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: String,
        retried_at_unix: u64,
        cause: CaptureDeliveryRetryCause,
    },
    Cancel {
        envelope_id: DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: String,
        cancelled_at_unix: u64,
    },
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    List(ListOutput),
    Inspect(InspectOutput),
    Mutation(MutationOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListOutput {
    schema_version: u32,
    command: &'static str,
    deliveries: Vec<CaptureDeliveryStatusV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InspectOutput {
    schema_version: u32,
    command: &'static str,
    delivery: Option<CaptureDeliveryStatusV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MutationOutput {
    schema_version: u32,
    command: &'static str,
    delivery: CaptureDeliveryStatusV1,
}

pub(crate) fn parse(args: &[OsString]) -> Result<Command, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a delivery subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::Help),
        "list" if args.len() == 1 => Ok(Command::List),
        "inspect" => parse_inspect(&args[1..]),
        "retry" => parse_mutation(&args[1..], true),
        "cancel" => parse_mutation(&args[1..], false),
        "--help" | "list" => Err("unexpected delivery argument"),
        _ => Err("unknown delivery subcommand"),
    }
}

pub(crate) fn execute(
    command: Command,
    service: &ProjectStateService,
) -> Result<Output, ProjectError> {
    match command {
        Command::Help => unreachable!("delivery help returns before service execution"),
        Command::List => service.list_capture_deliveries().map(|deliveries| {
            Output::List(ListOutput {
                schema_version: 1,
                command: "project-capture-delivery-list",
                deliveries,
            })
        }),
        Command::Inspect(envelope_id) => {
            service
                .inspect_capture_delivery(&envelope_id)
                .map(|delivery| {
                    Output::Inspect(InspectOutput {
                        schema_version: 1,
                        command: "project-capture-delivery-inspect",
                        delivery,
                    })
                })
        }
        Command::Retry {
            envelope_id,
            expected_generation,
            expected_record_sha256,
            retried_at_unix,
            cause,
        } => service
            .retry_capture_delivery(
                &envelope_id,
                expected_generation,
                &expected_record_sha256,
                retried_at_unix,
                cause,
            )
            .map(|delivery| {
                Output::Mutation(MutationOutput {
                    schema_version: 1,
                    command: "project-capture-delivery-retry",
                    delivery,
                })
            }),
        Command::Cancel {
            envelope_id,
            expected_generation,
            expected_record_sha256,
            cancelled_at_unix,
        } => service
            .cancel_capture_delivery(
                &envelope_id,
                expected_generation,
                &expected_record_sha256,
                cancelled_at_unix,
            )
            .map(|delivery| {
                Output::Mutation(MutationOutput {
                    schema_version: 1,
                    command: "project-capture-delivery-cancel",
                    delivery,
                })
            }),
    }
}

fn parse_inspect(args: &[OsString]) -> Result<Command, &'static str> {
    if args.len() != 2 || args[0] != "--envelope-id" {
        return Err("delivery inspect requires one envelope ID");
    }
    parse_envelope_id(&args[1]).map(Command::Inspect)
}

fn parse_mutation(args: &[OsString], retry: bool) -> Result<Command, &'static str> {
    let mut envelope_id = None;
    let mut expected_generation = None;
    let mut expected_record_sha256 = None;
    let mut transitioned_at_unix = None;
    let mut cause = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("delivery option is not valid UTF-8")?;
        let value = args
            .get(index + 1)
            .ok_or("delivery option value is required")?;
        match option {
            "--envelope-id" if envelope_id.is_none() => {
                envelope_id = Some(parse_envelope_id(value)?);
            }
            "--expected-generation" if expected_generation.is_none() => {
                expected_generation = Some(parse_positive_safe_integer(
                    value,
                    "delivery generation is invalid",
                )?);
            }
            "--expected-record-sha256" if expected_record_sha256.is_none() => {
                expected_record_sha256 = Some(parse_sha256(value)?);
            }
            "--retried-at-unix" if retry && transitioned_at_unix.is_none() => {
                transitioned_at_unix = Some(parse_safe_integer(
                    value,
                    "delivery retry timestamp is invalid",
                )?);
            }
            "--cancelled-at-unix" if !retry && transitioned_at_unix.is_none() => {
                transitioned_at_unix = Some(parse_safe_integer(
                    value,
                    "delivery cancellation timestamp is invalid",
                )?);
            }
            "--cause" if retry && cause.is_none() => {
                cause = Some(parse_retry_cause(value)?);
            }
            "--envelope-id"
            | "--expected-generation"
            | "--expected-record-sha256"
            | "--retried-at-unix"
            | "--cancelled-at-unix"
            | "--cause" => return Err("delivery option is unexpected or duplicate"),
            _ => return Err("unknown delivery option"),
        }
        index += 2;
    }

    let envelope_id = envelope_id.ok_or("delivery envelope ID is required")?;
    let expected_generation =
        expected_generation.ok_or("delivery expected generation is required")?;
    let expected_record_sha256 =
        expected_record_sha256.ok_or("delivery expected record digest is required")?;
    let transitioned_at_unix =
        transitioned_at_unix.ok_or("delivery transition timestamp is required")?;
    if retry {
        Ok(Command::Retry {
            envelope_id,
            expected_generation,
            expected_record_sha256,
            retried_at_unix: transitioned_at_unix,
            cause: cause.ok_or("delivery retry cause is required")?,
        })
    } else {
        Ok(Command::Cancel {
            envelope_id,
            expected_generation,
            expected_record_sha256,
            cancelled_at_unix: transitioned_at_unix,
        })
    }
}

fn parse_envelope_id(value: &OsString) -> Result<DeliveryEnvelopeId, &'static str> {
    value
        .to_str()
        .ok_or("delivery envelope ID is not valid UTF-8")
        .and_then(|value| {
            DeliveryEnvelopeId::parse(value.to_owned())
                .map_err(|_| "delivery envelope ID is invalid")
        })
}

fn parse_positive_safe_integer(value: &OsString, error: &'static str) -> Result<u64, &'static str> {
    parse_safe_integer(value, error).and_then(|value| (value > 0).then_some(value).ok_or(error))
}

fn parse_safe_integer(value: &OsString, error: &'static str) -> Result<u64, &'static str> {
    value
        .to_str()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value <= MAX_SAFE_INTEGER)
        .ok_or(error)
}

fn parse_sha256(value: &OsString) -> Result<String, &'static str> {
    value
        .to_str()
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .map(str::to_owned)
        .ok_or("delivery record digest is invalid")
}

fn parse_retry_cause(value: &OsString) -> Result<CaptureDeliveryRetryCause, &'static str> {
    match value.to_str() {
        Some("process-interrupted") => Ok(CaptureDeliveryRetryCause::ProcessInterrupted),
        Some("transport-unavailable") => Ok(CaptureDeliveryRetryCause::TransportUnavailable),
        Some("destination-unavailable") => Ok(CaptureDeliveryRetryCause::DestinationUnavailable),
        Some("recovery-required") => Ok(CaptureDeliveryRetryCause::RecoveryRequired),
        Some("conflict-resolved") => Ok(CaptureDeliveryRetryCause::ConflictResolved),
        _ => Err("delivery retry cause is invalid"),
    }
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
    fn parser_closes_inspect_retry_and_cancel_shapes() {
        let envelope_id = envelope_id();
        let digest = "b".repeat(64);
        assert!(matches!(parse(&args(&["list"])), Ok(Command::List)));
        assert!(matches!(
            parse(&args(&["inspect", "--envelope-id", &envelope_id])),
            Ok(Command::Inspect(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "retry",
                "--envelope-id",
                &envelope_id,
                "--expected-generation",
                "2",
                "--expected-record-sha256",
                &digest,
                "--retried-at-unix",
                "1800000012",
                "--cause",
                "transport-unavailable",
            ])),
            Ok(Command::Retry { .. })
        ));
        assert!(matches!(
            parse(&args(&[
                "cancel",
                "--envelope-id",
                &envelope_id,
                "--expected-generation",
                "3",
                "--expected-record-sha256",
                &digest,
                "--cancelled-at-unix",
                "1800000013",
            ])),
            Ok(Command::Cancel { .. })
        ));
    }

    #[test]
    fn parser_rejects_paths_unknowns_duplicates_and_incomplete_mutations() {
        let envelope_id = envelope_id();
        let digest = "b".repeat(64);
        for invalid in [
            args(&["inspect", "--envelope-id", "/Users/example/private"]),
            args(&[
                "retry",
                "--envelope-id",
                &envelope_id,
                "--expected-generation",
                "0",
                "--expected-record-sha256",
                &digest,
                "--retried-at-unix",
                "1800000012",
                "--cause",
                "transport-unavailable",
            ]),
            args(&[
                "retry",
                "--envelope-id",
                &envelope_id,
                "--expected-generation",
                "2",
                "--expected-record-sha256",
                &digest,
                "--retried-at-unix",
                "1800000012",
                "--cause",
                "provider-call",
            ]),
            args(&[
                "cancel",
                "--envelope-id",
                &envelope_id,
                "--expected-generation",
                "2",
                "--expected-record-sha256",
                &digest,
            ]),
            args(&["list", "--extra"]),
        ] {
            assert!(parse(&invalid).is_err());
        }
    }
}
