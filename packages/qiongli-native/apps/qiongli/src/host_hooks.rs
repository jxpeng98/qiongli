//! Opt-in Host context reminders; no project or conversation access.
use std::io::Read;

use serde_json::{Value, json};

use crate::CliOutput;

const MAX_INPUT_BYTES: u64 = 64 * 1024;
const CONTEXT: &str = "When Qiongli research is in scope: resume from authorized canonical research_state, stage_handoff and the latest stage summary, not this reminder. Check current source/candidate revisions before reusing prior decisions or agent results. Preserve claim/decision IDs, citekeys, source anchors and method limits across design, writing, Graph and summaries. A summary or reviewer opinion is not new source evidence. Use actual available Host subagents for requested bounded independent work; report execution identity, scope and unavailable evidence. The same conversation changing roles is self-review. Return source-bound findings or candidate edits to one coordinator; do not concurrently overwrite canonical research files. Keep existing preview, approval and revision checks. This hook establishes no completed work, review, approval or persistence.";

pub fn run_context_hook(reader: impl Read) -> CliOutput {
    let mut bytes = Vec::new();
    if reader
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() as u64 > MAX_INPUT_BYTES
    {
        return CliOutput::operation_failure("hook-input-unreadable-or-too-large");
    }
    let Ok(Value::Object(input)) = serde_json::from_slice::<Value>(&bytes) else {
        return CliOutput::operation_failure("hook-input-invalid");
    };
    let Some(event) = input.get("hook_event_name").and_then(Value::as_str) else {
        return CliOutput::operation_failure("hook-input-invalid");
    };
    let supported = match event {
        "SessionStart" => match input.get("source").and_then(Value::as_str) {
            Some("resume" | "compact") => true,
            Some(_) => false,
            None => return CliOutput::operation_failure("hook-input-invalid"),
        },
        "SubagentStart" => true,
        _ => false,
    };
    let output = if supported {
        json!({"hookSpecificOutput": {"hookEventName": event, "additionalContext": CONTEXT}})
    } else {
        json!({})
    };
    CliOutput::success_text(format!("{output}\n"))
}
