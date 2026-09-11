//! Fixed, opt-in Plugin reminders. Host trust and event delivery remain Host-owned.
use crate::{ClientKind, OperatingSystem};
use serde_json::{Value, json};

/// Inline Plugin manifest configuration; invokes only the bundled native program.
#[must_use]
pub fn plugin_context_hooks(host: ClientKind, os: OperatingSystem) -> Value {
    let handler = match host {
        ClientKind::Codex => json!({
            "type": "command",
            "command": "\"${CLAUDE_PLUGIN_ROOT}/bin/qiongli\" hooks context",
            "commandWindows": "\"%CLAUDE_PLUGIN_ROOT%\\bin\\qiongli.exe\" hooks context",
            "timeout": 5
        }),
        // Exec form avoids shell quoting and works without Git Bash on Windows.
        ClientKind::ClaudeCode => json!({
            "type": "command",
            "command": if os == OperatingSystem::Windows {
                "${CLAUDE_PLUGIN_ROOT}/bin/qiongli.exe"
            } else {
                "${CLAUDE_PLUGIN_ROOT}/bin/qiongli"
            },
            "args": ["hooks", "context"],
            "timeout": 5
        }),
    };
    let events = json!({
        "SessionStart": [{"matcher": "resume|compact", "hooks": [handler]}],
        "SubagentStart": [{"hooks": [handler]}]
    });
    // Codex embeds a HooksFile; Claude embeds the event map directly.
    match host {
        ClientKind::Codex => json!({"hooks": events}),
        ClientKind::ClaudeCode => events,
    }
}
