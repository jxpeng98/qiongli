//! Opt-in CLI transports. The calling Host owns execution and process outcomes.
use serde_json::Value;

use crate::{
    CODEX_EXEC_MAX_INPUT_BYTES, CodexExecDispatchV1, CodexExecOutcomeV1, CodexExecPacketV1,
    HostDelegationResultV1, OrchestrationHandoffV1, collect_codex_exec, prepare_codex_exec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalAgentV1 {
    Codex,
    ClaudeCode,
    DeepSeek,
    Antigravity,
}

pub fn prepare_external_exec(
    agent: ExternalAgentV1,
    handoff: &OrchestrationHandoffV1,
    packet: &CodexExecPacketV1,
) -> Result<CodexExecDispatchV1, &'static str> {
    let mut dispatch = prepare_codex_exec(handoff, packet)?;
    match agent {
        ExternalAgentV1::Codex => {}
        ExternalAgentV1::ClaudeCode => {
            dispatch.argv = vec![
                "claude",
                "--print",
                "--output-format",
                "json",
                "--no-session-persistence",
                "--tools",
                "",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--disable-slash-commands",
            ];
        }
        ExternalAgentV1::DeepSeek => {
            dispatch.argv = vec!["dsh", "--profile", "headless", "--json", "-"];
            dispatch.env.insert("DSH_PERMISSION_MODE", "read-only");
        }
        ExternalAgentV1::Antigravity => {
            dispatch.argv = vec![
                "agy",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--mode",
                "plan",
                "--disable-slash-commands",
            ];
            dispatch.stdin =
                serde_json::json!({"event":"user","message":{"content":dispatch.stdin}})
                    .to_string()
                    + "\n";
        }
    }
    Ok(dispatch)
}

/// Never infer process success from the model's reply or a final-looking event.
pub fn collect_external_exec(
    agent: ExternalAgentV1,
    handoff: &OrchestrationHandoffV1,
    packet: &CodexExecPacketV1,
    events: &str,
    outcome: CodexExecOutcomeV1,
    exit_code: u8,
) -> Result<HostDelegationResultV1, &'static str> {
    if agent == ExternalAgentV1::Codex {
        return collect_codex_exec(handoff, packet, events, outcome, exit_code);
    }
    if outcome != CodexExecOutcomeV1::Completed || exit_code != 0 {
        return Err("external-exec-not-completed");
    }
    let prepared = prepare_external_exec(agent, handoff, packet)?;
    if events.len() > CODEX_EXEC_MAX_INPUT_BYTES {
        return Err("external-exec-events-too-large");
    }
    let (id, text, tool) = match agent {
        ExternalAgentV1::ClaudeCode => {
            let result: Value =
                serde_json::from_str(events).map_err(|_| "external-exec-event-invalid")?;
            if result["type"] != "result"
                || result["subtype"] != "success"
                || result["is_error"] != false
            {
                return Err("external-exec-transport-failed");
            }
            (
                identity(&result["session_id"])?,
                reply(&result["result"])?,
                "claude.print",
            )
        }
        ExternalAgentV1::DeepSeek | ExternalAgentV1::Antigravity => {
            let mut id = None;
            let mut text = None;
            let mut turn = None;
            let mut ended = false;
            for (index, line) in events.lines().enumerate() {
                if index >= 4096 || line.len() > 262_144 {
                    return Err("external-exec-events-too-large");
                }
                if text.is_some() {
                    return Err("external-exec-event-order-invalid");
                }
                let event: Value =
                    serde_json::from_str(line).map_err(|_| "external-exec-event-invalid")?;
                if agent == ExternalAgentV1::DeepSeek {
                    if let Some(observed) = event.get("sessionId")
                        && event["type"] != "session"
                        && Some(identity(observed)?) != id
                    {
                        return Err("external-exec-identity-mismatch");
                    }
                    match event["type"].as_str() {
                        Some("session")
                            if id.is_none() && index == 0 && event["truncated"] != true =>
                        {
                            id = Some(identity(&event["sessionId"])?)
                        }
                        Some("status") if id.is_some() && !ended && event["truncated"] != true => {
                            match event["phase"].as_str() {
                                Some("turn_start") if turn.is_none() => {
                                    turn = Some(
                                        event["turn"]
                                            .as_u64()
                                            .ok_or("external-exec-event-invalid")?,
                                    );
                                }
                                Some("step_start" | "step_end" | "turn_end")
                                    if turn.is_some() && event["turn"].as_u64() == turn =>
                                {
                                    if event["phase"] == "turn_end" {
                                        if event["reason"]["kind"] != "completed" {
                                            return Err("external-exec-transport-failed");
                                        }
                                        ended = true;
                                    }
                                }
                                _ => return Err("external-exec-event-order-invalid"),
                            }
                        }
                        Some("text" | "thinking" | "tool_call" | "tool_result")
                            if turn.is_some() && !ended => {}
                        Some("final") if ended && event["truncated"] != true => {
                            text = Some(reply(&event["text"])?)
                        }
                        Some("error") => return Err("external-exec-transport-failed"),
                        _ => return Err("external-exec-event-order-invalid"),
                    }
                } else {
                    match event["event"].as_str() {
                        Some("init") if index == 0 => {
                            id = Some(identity(&event["conversation_id"])?)
                        }
                        Some("step_update" | "result") if id.is_some() => {
                            let payload = &event[event["event"].as_str().unwrap()];
                            if Some(identity(&payload["conversation_id"])?) != id {
                                return Err("external-exec-identity-mismatch");
                            }
                            if event["event"] == "result" {
                                if payload["status"] != "SUCCESS"
                                    || payload.get("error").is_some()
                                    || payload["num_turns"] != 1
                                {
                                    return Err("external-exec-transport-failed");
                                }
                                text = Some(reply(&payload["response"])?);
                            }
                        }
                        _ => return Err("external-exec-event-order-invalid"),
                    }
                }
            }
            (
                id.ok_or("external-exec-stream-incomplete")?,
                text.ok_or("external-exec-stream-incomplete")?,
                if agent == ExternalAgentV1::DeepSeek {
                    "dsh.headless"
                } else {
                    "antigravity.print"
                },
            )
        }
        ExternalAgentV1::Codex => unreachable!(),
    };
    crate::codex_exec::collect_bound_reply(handoff, packet, &prepared, id, tool, text)
}

fn identity(value: &Value) -> Result<String, &'static str> {
    value
        .as_str()
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 256
                && id.chars().all(|c| !c.is_control() && !c.is_whitespace())
        })
        .map(str::to_owned)
        .ok_or("external-exec-identity-invalid")
}

fn reply(value: &Value) -> Result<String, &'static str> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or("external-exec-reply-invalid")
}
