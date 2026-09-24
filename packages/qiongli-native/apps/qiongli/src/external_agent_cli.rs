//! Prepare/collect an external Agent turn; the current Host owns process execution.
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use qiongli_execution::{
    CODEX_EXEC_MAX_INPUT_BYTES, CodexExecOutcomeV1, CodexExecPacketV1, ExternalAgentV1,
    OrchestrationHandoffV1, collect_external_exec, prepare_external_exec,
};

use crate::CliOutput;

pub(crate) const USAGE: &str = "Host-executed external Agent adapters\n\nUsage:\n  qiongli agent <codex|claude|deepseek|antigravity> prepare --handoff <handoff.json> --packet <packet.json> --json\n  qiongli agent <codex|claude|deepseek|antigravity> collect --handoff <handoff.json> --packet <packet.json> --events <events.jsonl> --status <completed|cancelled|timed-out|failed> --exit-code <0..255> --json\n\nPacket fields: scope and sourceText (authorized snapshot with identifiers and limits).\nprepare emits argv, stdin and optional per-process env overrides. Merge env into\nonly the child process and use the Host execution tool in an authorized directory.\nRetain exact stdout JSON/JSONL and the observed process status/exit code. The Host owns timeout, cancellation and process cleanup.\ncollect requires the original packet/handoff, a completed turn, successful process\nexit and a reply bound to both digests. It emits one external-agent delegation\nresult for coordinator review; source evidence and project approval stay separate.\nModel, effort, account and saved settings remain Host-owned. DeepSeek selects the\nheadless profile; its --help must expose --json (npm 0.1.5-rc.3 does not).\nTransport flags and proposal instructions are not full isolation from Host hooks\nor configured tools. Use an authorized, isolated working directory.\nThese commands do not launch a process.\nAfter failure inspect the original execution and current sources before an explicit\nfresh run; no automatic retry, resume --last, canonical writes or apply.\n";

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ExternalAgentCommand {
    Prepare {
        agent: ExternalAgentV1,
        handoff: PathBuf,
        packet: PathBuf,
    },
    Collect {
        agent: ExternalAgentV1,
        handoff: PathBuf,
        packet: PathBuf,
        events: PathBuf,
        status: CodexExecOutcomeV1,
        exit_code: u8,
    },
}

pub(crate) fn parse(args: &[OsString]) -> Result<ExternalAgentCommand, &'static str> {
    let agent = match args.first().and_then(|v| v.to_str()) {
        Some("codex") => ExternalAgentV1::Codex,
        Some("claude" | "claude-code") => ExternalAgentV1::ClaudeCode,
        Some("deepseek" | "dsh") => ExternalAgentV1::DeepSeek,
        Some("antigravity" | "agy") => ExternalAgentV1::Antigravity,
        _ => return Err("external agent must be codex, claude, deepseek or antigravity"),
    };
    let collect = match args.get(1).and_then(|v| v.to_str()) {
        Some("prepare") => false,
        Some("collect") => true,
        _ => return Err("external agent command must be prepare or collect"),
    };
    if args.len() < 2 || !(args.len() - 2).is_multiple_of(2) {
        return Err("external agent options require values");
    }
    let (mut handoff, mut packet, mut events, mut status, mut exit_code) =
        (None, None, None, None, None);
    for pair in args[2..].chunks_exact(2) {
        match pair[0].to_str() {
            Some("--handoff") if handoff.is_none() => handoff = Some(PathBuf::from(&pair[1])),
            Some("--packet") if packet.is_none() => packet = Some(PathBuf::from(&pair[1])),
            Some("--events") if collect && events.is_none() => {
                events = Some(PathBuf::from(&pair[1]))
            }
            Some("--status") if collect && status.is_none() => {
                status = Some(match pair[1].to_str() {
                    Some("completed") => CodexExecOutcomeV1::Completed,
                    Some("cancelled") => CodexExecOutcomeV1::Cancelled,
                    Some("timed-out") => CodexExecOutcomeV1::TimedOut,
                    Some("failed") => CodexExecOutcomeV1::Failed,
                    _ => return Err("external agent status is invalid"),
                })
            }
            Some("--exit-code") if collect && exit_code.is_none() => {
                exit_code = Some(
                    pair[1]
                        .to_str()
                        .and_then(|v| v.parse::<u8>().ok())
                        .ok_or("external agent exit code is invalid")?,
                );
            }
            _ => return Err("external agent option is unexpected or duplicated"),
        }
    }
    let handoff = handoff
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or("handoff path is required")?;
    let packet = packet
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or("packet path is required")?;
    if collect {
        Ok(ExternalAgentCommand::Collect {
            agent,
            handoff,
            packet,
            events: events
                .filter(|p| !p.as_os_str().is_empty())
                .ok_or("events path is required")?,
            status: status.ok_or("observed process status is required")?,
            exit_code: exit_code.ok_or("observed process exit code is required")?,
        })
    } else {
        Ok(ExternalAgentCommand::Prepare {
            agent,
            handoff,
            packet,
        })
    }
}

pub(crate) fn run(command: &ExternalAgentCommand) -> CliOutput {
    match execute(command) {
        Ok(value) => CliOutput::success_text(format!("{value}\n")),
        Err(reason) => CliOutput::operation_failure(reason),
    }
}

fn execute(command: &ExternalAgentCommand) -> Result<serde_json::Value, &'static str> {
    let (handoff, packet) = match command {
        ExternalAgentCommand::Prepare {
            handoff, packet, ..
        }
        | ExternalAgentCommand::Collect {
            handoff, packet, ..
        } => (handoff, packet),
    };
    let handoff: OrchestrationHandoffV1 =
        serde_json::from_slice(&read(handoff)?).map_err(|_| "codex-exec-handoff-invalid")?;
    let packet: CodexExecPacketV1 =
        serde_json::from_slice(&read(packet)?).map_err(|_| "codex-exec-packet-invalid")?;
    let value = match command {
        ExternalAgentCommand::Prepare { agent, .. } => {
            serde_json::to_value(prepare_external_exec(*agent, &handoff, &packet)?)
        }
        ExternalAgentCommand::Collect {
            agent,
            events,
            status,
            exit_code,
            ..
        } => {
            let events =
                String::from_utf8(read(events)?).map_err(|_| "codex-exec-event-invalid")?;
            serde_json::to_value(collect_external_exec(
                *agent, &handoff, &packet, &events, *status, *exit_code,
            )?)
        }
    };
    value.map_err(|_| "codex-exec-output-invalid")
}

fn read(path: &Path) -> Result<Vec<u8>, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "codex-exec-input-unreadable")?;
    if !metadata.is_file() || metadata.len() > CODEX_EXEC_MAX_INPUT_BYTES as u64 {
        return Err("codex-exec-input-not-bounded-file");
    }
    let file = fs::File::open(path).map_err(|_| "codex-exec-input-unreadable")?;
    if !file
        .metadata()
        .map_err(|_| "codex-exec-input-unreadable")?
        .is_file()
    {
        return Err("codex-exec-input-not-bounded-file");
    }
    let mut bytes = Vec::new();
    file.take(CODEX_EXEC_MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "codex-exec-input-unreadable")?;
    if bytes.len() > CODEX_EXEC_MAX_INPUT_BYTES {
        return Err("codex-exec-input-not-bounded-file");
    }
    Ok(bytes)
}
