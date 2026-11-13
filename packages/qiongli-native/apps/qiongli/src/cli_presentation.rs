//! Terminal presentation over the existing command results; never a write owner.
use std::ffi::{OsStr, OsString};

use qiongli_content::EmbeddedContent;
use serde_json::Value;

use crate::{CliOutput, CommandEnvironment, ProductAction};

pub fn prepare_cli_action(
    mut args: Vec<OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    terminal: bool,
) -> ProductAction {
    let mut text_mode = None;
    while let Some(mode) = args.first().and_then(|arg| output_mode(arg)) {
        if text_mode.replace(mode).is_some() {
            return output_option_error();
        }
        args.remove(0);
    }
    // A valid trailing value such as --name "--text" belongs to its command.
    // The existing `paths --json` option must also retain its original meaning.
    while let Some(mode) = args.last().and_then(|arg| output_mode(arg)) {
        let paths_json = args == [OsString::from("paths"), OsString::from("--json")];
        if !paths_json && crate::command::accepts_arguments(&args) {
            break;
        }
        if text_mode.replace(mode).is_some() {
            return output_option_error();
        }
        args.pop();
    }
    if text_mode.is_some() && args.is_empty() {
        args.push("--help".into());
    }
    if text_mode.is_some() && args.starts_with(&["project".into(), "graph".into(), "view".into()]) {
        return ProductAction::Output(CliOutput::usage_text(
            "graph view uses HTML or a saved-file notice; use graph snapshot for --json or --text",
        ));
    }
    let readable = text_mode.unwrap_or(terminal);
    if args == [OsString::from("paths")] && (readable || text_mode == Some(false)) {
        args.push("--json".into());
    }
    // Validate output options before executing any command. Streaming and
    // interactive commands have their own protocols and cannot be reformatted.
    if text_mode.is_some()
        && (args
            .first()
            .is_some_and(|arg| arg == "setup" || arg == "ui")
            || args.starts_with(&["mcp".into(), "serve".into()])
            || args.starts_with(&["provider".into(), "setup".into()])
            || args.starts_with(&["hooks".into(), "context".into()])
            || args.starts_with(&["install".into(), "review".into()])
            || args.starts_with(&["install".into(), "migrate".into()]))
    {
        return ProductAction::Output(CliOutput::usage_text(
            "output options apply to queries, not interactive or streaming commands",
        ));
    }
    if terminal
        && text_mode.is_none()
        && (args == [OsString::from("install")] || args == [OsString::from("upgrade")])
    {
        return ProductAction::GuideInstallation(Default::default());
    }
    if terminal
        && text_mode.is_none()
        && matches!(
            args.iter()
                .filter_map(|arg| arg.to_str())
                .collect::<Vec<_>>()
                .as_slice(),
            ["config", "backend"]
                | ["project", "graph" | "capture" | "portfolio"]
                | ["app", "plan"]
        )
        && let Some(help) = crate::cli_help::topic(&args)
    {
        return ProductAction::Output(CliOutput::success_text(help));
    }
    match crate::prepare_action(args, environment, content) {
        ProductAction::ReviewBundledContent(_) | ProductAction::GuideInstallation(_)
            if text_mode.is_some() =>
        {
            ProductAction::Output(CliOutput::usage_text(
                "interactive installation does not accept output options; use --dry-run --json for a file plan",
            ))
        }
        ProductAction::Output(output) if readable => ProductAction::Output(readable_output(output)),
        action => action,
    }
}

fn output_mode(arg: &OsStr) -> Option<bool> {
    match arg.to_str() {
        Some("--text") => Some(true),
        Some("--json") => Some(false),
        _ => None,
    }
}

fn output_option_error() -> ProductAction {
    ProductAction::Output(CliOutput::usage_text("choose --json or --text once"))
}

fn readable_output(output: CliOutput) -> CliOutput {
    if output.stdout().is_empty()
        && let Some(code) = output.stderr().trim().strip_prefix("error: ")
        && !code.is_empty()
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        let message = match code {
            "source-build-read-only" | "native-release-authority-unavailable" =>
                "This operation requires a verified managed installation. For registry packages, use the original package manager to upgrade or remove the CLI.".to_owned(),
            "managed-skills-plan-required" =>
                "Preview managed Skills changes with `qiongli app plan --help`, then approve the exact plan.".to_owned(),
            _ => label(code),
        };
        let message = format!("error: {message}\n  Code: {code}\n");
        return output.with_stderr(message);
    }
    let Ok(value) = serde_json::from_str::<Value>(output.stdout()) else {
        return output;
    };
    if value["artifact"]["documentKind"] == "qiongli-project-artifact-view" {
        let artifact = &value["artifact"];
        let mut text = format!(
            "{} — revision {}\n",
            scalar(&artifact["artifactPath"]),
            scalar(&artifact["projectRevision"])
        );
        row(&mut text, "Anchor matched", &artifact["anchorMatched"]);
        row(&mut text, "Source digest", &artifact["contentDigest"]);
        if artifact["truncatedBefore"] == true || artifact["truncatedAfter"] == true {
            text.push_str("Bounded excerpt; some source content is omitted.\n");
        }
        let start = artifact["startLine"].as_u64().unwrap_or(1);
        for (offset, line) in artifact["content"]
            .as_str()
            .unwrap_or("")
            .lines()
            .enumerate()
        {
            text.push_str(&format!("{:>5}  {}\n", start + offset as u64, safe(line)));
        }
        return output.with_stdout(text);
    }
    let command = value["command"].as_str().unwrap_or("result");
    let mut text = String::new();
    match command {
        "project-graph-snapshot" => {
            let graph = &value["snapshot"];
            let readiness = &value["readiness"];
            text.push_str(&format!(
                "Research Graph — revision {}\n\n",
                scalar(&graph["projectRevision"])
            ));
            row(&mut text, "Projection state", &readiness["state"]);
            row(
                &mut text,
                "Semantic records",
                &readiness["semanticNodeCount"],
            );
            row(
                &mut text,
                "Sources present",
                &readiness["presentSourceCount"],
            );
            row(
                &mut text,
                "Sources missing",
                &readiness["missingSourceCount"],
            );
            let edges = array(&graph["edges"]);
            let semantic = edges
                .iter()
                .filter(|edge| edge["relation"] != "contains")
                .count();
            text.push_str(&format!(
                "  Scholarly relations: {semantic}\n  Diagnostics: {}\n",
                array(&graph["diagnostics"]).len()
            ));
            text.push_str("\nClaims (up to 5; counts are recorded relationships, not scientific validation):\n");
            for node in array(&graph["nodes"])
                .iter()
                .filter(|node| node["nodeType"] == "claim")
                .take(5)
            {
                let supports = edges
                    .iter()
                    .filter(|edge| {
                        edge["targetNodeId"] == node["nodeId"]
                            && edge["relation"] == "supports"
                            && edge["status"] == "reviewed"
                    })
                    .count();
                text.push_str(&format!(
                    "  {}: {}\n    Reviewed support records: {supports}\n",
                    scalar(&node["canonicalId"]),
                    scalar(&node["label"])
                ));
            }
            row(&mut text, "Projection", &graph["projectionId"]);
            text.push_str(&format!("\nView: qiongli project graph view --project-id {} --open\nUse --save to keep the page without opening it. --json retains the complete snapshot; source commands are available in the view.\n", scalar(&graph["projectId"])));
        }
        "status" => {
            text.push_str(&format!(
                "Qiongli {}\n\n",
                scalar(&value["product_version"])
            ));
            row(&mut text, "Content", &value["content"]["state"]);
            row(
                &mut text,
                "Content version",
                &value["content"]["content_version"],
            );
            row(&mut text, "Configuration", &value["config"]["state"]);
            row(
                &mut text,
                "Default profile",
                &value["config"]["default_profile"],
            );
            text.push_str("\nNext: qiongli doctor  |  qiongli project  |  qiongli mcp --help\n");
        }
        "doctor" => {
            text.push_str(&format!(
                "Local installation checks: {}\n\n",
                scalar(&value["overall"])
            ));
            pending_research_checks(&mut text);
            if let Some(store) = array(&value["checks"])
                .iter()
                .find(|c| c["id"] == "secure-store")
            {
                credential_storage(&mut text, store["state"].as_str());
            }
            text.push_str("\nLocal diagnostics\n");
            for check in array(&value["checks"]) {
                // The legacy wire check concerns in-process execution, not Full MCP.
                if check["id"] == "full-runtime"
                    && check["remediation"] == "upgrade-to-r4-full-runtime"
                {
                    text.push_str("  [deferred    ] Legacy in-process runtime\n      Models run in your Host. Install the Plugin with Full MCP: qiongli install plugin\n");
                    continue;
                }
                text.push_str(&format!(
                    "  [{:<12}] {}{}\n",
                    scalar(&check["state"]),
                    label(check["id"].as_str().unwrap_or("check")),
                    if check["blocking"] == true {
                        " (blocking)"
                    } else {
                        ""
                    }
                ));
                let next = check["remediation"]
                    .as_str()
                    .filter(|v| !v.is_empty() && *v != "none");
                if check["state"] != "ready" || next.is_some() {
                    row(&mut text, "    Reason", &check["code"]);
                    if let Some(next) = next {
                        text.push_str(&format!(
                            "      Next: {}\n",
                            match next {
                                "restart-host-to-check-live-tools" => "Start a new Host session; list Qiongli tools and call qiongli_config_status.".into(),
                                "upgrade-local-plugin" => "Compare CLI and Plugin versions with qiongli install inventory before refreshing the Plugin.".into(),
                                _ => label(next),
                            }
                        ));
                    }
                }
            }
            if !value["cli"].is_null() {
                installations(&mut text, &value["cli"]);
            }
            if let Some(paths) = value.get("paths") {
                paths_text(&mut text, paths);
            }
        }
        "install-inventory" => {
            text.push_str("Research in your Host (local installation only)\n\n");
            pending_research_checks(&mut text);
            for client in array(&value["inventory"]["clients"]) {
                text.push_str(&format!(
                    "\n  {}: {}\n",
                    scalar(&client["client"]),
                    scalar(&client["readiness"])
                ));
                host_tasks(&mut text, client);
                row(&mut text, "    Host", &client["host_presence"]);
                row(
                    &mut text,
                    "    Plugin version",
                    &client["installed_plugin_version"],
                );
                row(&mut text, "    Reason", &client["reason_code"]);
            }
            installations(&mut text, &value["cli"]);
            text.push_str("\nInstall or update content: qiongli install\nReview CLI versions: qiongli setup  |  qiongli install list --paths exact\n");
        }
        "mcp-check" => {
            text.push_str("Local MCP protocol check\n\n");
            text.push_str("  Online retrieval: not checked; inspect qiongli provider status.\n");
            match value["profile"].as_str() {
                Some("full") => text.push_str("  Saving: Full MCP includes project tools; saving was not exercised. A save still requires preview and approval.\n"),
                Some("lite") => text.push_str("  Saving: unavailable in Lite; use Full MCP for project tools.\n"),
                _ => text.push_str("  Saving: profile not known; project tools are not confirmed.\n"),
            }
            text.push_str("  Host session: not checked by this local test.\n\nProtocol details\n");
            fields(&mut text, &value, 0);
        }
        "provider-status" => {
            text.push_str("Literature retrieval (offline configuration check)\n\n");
            for provider in array(&value["providers"]) {
                let name = provider["provider"].as_str().unwrap_or("unknown");
                let needs_key = matches!(name, "openalex" | "semantic_scholar" | "pubmed");
                let state = if provider["enabled"] == false {
                    "disabled"
                } else {
                    match provider["readiness"].as_str() {
                        Some("ready") if needs_key && value["secure_store"] == "unavailable" => {
                            "configured, but credential storage is unavailable on this platform"
                        }
                        Some("ready") => "configured; online retrieval not checked",
                        Some("needs_secret") => "API key missing; configure this provider",
                        Some("needs_public_setting") => {
                            "public setting missing; configure this provider"
                        }
                        Some("secret_store_unavailable") => "credential storage unavailable",
                        Some("disabled") => "disabled",
                        _ => "configuration not known; inspect the structured diagnostics",
                    }
                };
                text.push_str(&format!("  {}: {state}\n", safe(name)));
            }
            credential_storage(&mut text, value["secure_store"].as_str());
            text.push_str("\nNext: qiongli provider setup <provider>\nOptional online check: qiongli provider test <provider> --confirm-network-request\nThis sends one request only when you explicitly run it.\n\nDiagnostics\n");
            fields(&mut text, &value, 0);
        }
        "project-list" => {
            let library = &value["library"];
            let projects = array(&library["projects"]);
            text.push_str(&format!(
                "Research projects: {} ({})\n",
                projects.len(),
                scalar(&library["health"])
            ));
            for project in projects {
                text.push_str(&format!(
                    "\n  {}\n    {} | {} | {}\n    ID: {}\n",
                    scalar(&project["displayName"]),
                    scalar(&project["stage"]),
                    scalar(&project["lifecycle"]),
                    scalar(&project["health"]),
                    scalar(&project["projectId"])
                ));
            }
            if projects.is_empty() {
                text.push_str("\nNo registered projects. Start with:\n  qiongli project create --help\n  qiongli project register --help\n");
            } else {
                text.push_str("\nNext: qiongli project show <id>\n");
            }
        }
        "content-list" => {
            text.push_str(&format!(
                "Embedded research content {}\n\n",
                scalar(&value["content_version"])
            ));
            for profile in array(&value["profiles"]) {
                let id = scalar(&profile["id"]);
                let purpose = match id.as_str() {
                    "skill-only" => "Workflow, Skills and research references",
                    "marketplace-lite" => "Research content and Lite MCP (alias: lite)",
                    "full" => "Research content and Full MCP",
                    _ => "Embedded content profile",
                };
                text.push_str(&format!("  {id:<18} {purpose}\n"));
            }
            text.push_str("\nInstall Plugin with Skills and MCP: qiongli install plugin\nExport standalone Skills: qiongli install skills\n");
        }
        "paths" => paths_text(&mut text, &value["paths"]),
        _ => {
            text.push_str(&format!("{}\n\n", label(command)));
            fields(&mut text, &value, 0);
            if command == "update-status" {
                text.push_str("\nThis is the managed CLI updater status.\nFor package or archive upgrades: qiongli upgrade cli\nRefresh Plugin/Skills from this CLI: qiongli install\n");
            }
        }
    }
    text.push_str("\nUse --json for the complete structured result.\n");
    output.with_stdout(text)
}

fn pending_research_checks(text: &mut String) {
    text.push_str("  Online retrieval: not checked; inspect qiongli provider status.\n  Saving in the Host: not checked; requires Full MCP, preview and approval.\n  After installation or refresh, start a new Host session; list Qiongli tools and call qiongli_config_status.\n");
}

fn credential_storage(text: &mut String, state: Option<&str>) {
    text.push_str(match state {
        Some("unavailable") => "  API key storage: cannot save or access keys on this platform. Public settings remain configurable.\n",
        Some("ready" | "available") => "  API key storage: supported; actual credential access has not been tested.\n",
        _ => "  API key storage: availability not confirmed.\n",
    });
}

fn host_tasks(text: &mut String, client: &Value) {
    let readiness = client["readiness"].as_str();
    let reason = client["reason_code"].as_str();
    let summary = match (readiness, reason) {
        (Some("unavailable"), _) | (_, Some("local-host-observation-unavailable")) => {
            "Observation unavailable; installed capabilities cannot be confirmed. Inspect Host inventory before changing anything."
        }
        (Some("resolve-conflict"), _) => {
            "Resolve the reported local conflict before using this integration."
        }
        (Some("repair-ready"), _) | (_, Some("local-host-refresh-required")) => {
            "Refresh required; compare CLI and Plugin versions before updating, then start a new session."
        }
        (Some("current"), _) if client["components"]["full_mcp"] == "ready" => {
            "Full MCP is installed locally; research and saving in the active session are not verified."
        }
        (Some("current"), _) => {
            "Full MCP is not confirmed; Lite and standalone Skills do not provide project saving tools."
        }
        (Some("install-ready" | "inspect-only"), _) => {
            "Integration not confirmed; review installation with qiongli install."
        }
        _ => "Capabilities not confirmed; inspect the structured diagnostics.",
    };
    text.push_str(&format!("    {summary}\n"));
}

fn installations(text: &mut String, inventory: &Value) {
    let entries = array(&inventory["installations"]);
    text.push_str(&format!("\nCLI installations: {}\n", entries.len()));
    for (index, item) in entries.iter().enumerate() {
        text.push_str(&format!(
            "\n  {}. {}  {}{}\n",
            index + 1,
            scalar(&item["channel"]),
            scalar(&item["version"]),
            if item["running"] == true {
                "  (running now)"
            } else {
                ""
            }
        ));
        row(text, "    Version source", &item["version_source"]);
        if !array(&item["active_commands"]).is_empty() {
            row(text, "    First on PATH", &item["active_commands"]);
        }
        if array(&item["entries"])
            .first()
            .and_then(Value::as_str)
            .is_some_and(|path| path.starts_with("cli-entry-"))
        {
            text.push_str("    Location hidden; use --paths exact to show it.\n");
        } else {
            for path in array(&item["entries"]) {
                text.push_str(&format!("    {}\n", scalar(path)));
            }
        }
    }
    text.push_str("\nDetection covers PATH and known locations; aliases and other environments may need review.\nArchive and removal are user-operated.\n");
}

fn paths_text(text: &mut String, paths: &Value) {
    text.push_str("\nResolved paths (exact locations)\n");
    for path in array(paths) {
        text.push_str(&format!(
            "\n  {}\n    {}\n    State: {} | Safety: {}\n",
            scalar(&path["label"]),
            scalar(&path["exact_path"]),
            scalar(&path["file_type"]),
            scalar(&path["safety"])
        ));
    }
}

pub(crate) fn plugin_install_summary(
    source: &crate::plugin_source::PluginSourcePlan,
    plugin: &str,
    cache: &std::path::Path,
) -> String {
    use crate::managed_operation::ManagedIntegrationTargetV1;
    let (host, target, hook_step) = match source.target {
        ManagedIntegrationTargetV1::Codex => (
            "Codex",
            "codex",
            "  Open /hooks (Hook settings) in Codex; review/trust Qiongli commands.\n",
        ),
        ManagedIntegrationTargetV1::ClaudeCode => (
            "Claude Code",
            "claude",
            "  Open /hooks in Claude Code; review the Qiongli Plugin entries.\n",
        ),
    };
    let mut text = format!(
        "\nQiongli Plugin installed — {host}\n\n  Research  Skills and Full MCP are installed locally; open a new Host session.\n  Retrieve  Provider connectivity has not been checked.\n  Save      Full MCP includes project tools; saving is untested and requires preview and approval.\n\n  Version   {}\n  Plugin    {}\n  Source    {}\n  Cache     {}\n\n  Verified  registration, enabled state and cached files\n  Includes  Skills + Full MCP (35 tools)\n  Pending   session tools{}\n\nNext steps\n",
        env!("CARGO_PKG_VERSION"),
        safe(plugin),
        safe(&source.destination.to_string_lossy()),
        safe(&cache.to_string_lossy()),
        if source.context_hooks {
            "; Hook trust and execution"
        } else {
            " (Hooks off in this Plugin)"
        },
    );
    if source.context_hooks {
        text.push_str(hook_step);
    }
    text.push_str("  Start a new session; list Qiongli tools and call qiongli_config_status.\n");
    if source.context_hooks {
        text.push_str("  Check for a Hook reminder on resume/compact or child start.\n");
    }
    text.push_str(&format!(
        "\n  Check     qiongli doctor\n  Update    qiongli install plugin --target {target}\n  Providers qiongli_literature_status (ask the Host)\n\nSkills load from the Plugin. MCP starts with the Host; no separate install.\n"
    ));
    text
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

fn row(text: &mut String, name: &str, value: &Value) {
    text.push_str(&format!("  {name}: {}\n", scalar(value)));
}

fn safe(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if ch.is_control() || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
                ch.escape_default().collect::<Vec<_>>()
            } else {
                vec![ch]
            }
        })
        .collect()
}

fn scalar(value: &Value) -> String {
    match value {
        Value::Null => "not available".into(),
        Value::Bool(value) => if *value { "yes" } else { "no" }.into(),
        Value::String(value) => safe(value),
        Value::Array(values) if values.is_empty() => "none".into(),
        Value::Array(values) => values.iter().map(scalar).collect::<Vec<_>>().join(", "),
        other => safe(&other.to_string()),
    }
}

fn label(value: &str) -> String {
    let mut text = String::new();
    for ch in safe(value).chars() {
        if ch == '_' || ch == '-' {
            text.push(' ');
        } else if ch.is_ascii_uppercase() && !text.is_empty() {
            text.push(' ');
            text.push(ch.to_ascii_lowercase());
        } else {
            text.push(ch);
        }
    }
    if let Some(first) = text.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    text
}

// Preserve every field of plans, receipts and less common results. Only known
// read-only overviews above are summarized; approval IDs/digests are never cut.
pub(crate) fn fields(text: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                if indent == 0
                    && matches!(key.as_str(), "schema_version" | "schemaVersion" | "command")
                {
                    continue;
                }
                text.push_str(&format!("{}{}:", " ".repeat(indent), label(key)));
                if value.is_object()
                    || (value.is_array() && array(value).iter().any(Value::is_object))
                {
                    text.push('\n');
                    fields(text, value, indent + 2);
                } else {
                    text.push_str(&format!(" {}\n", scalar(value)));
                }
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                text.push_str(&format!("{}{}.\n", " ".repeat(indent), index + 1));
                fields(text, value, indent + 2);
            }
        }
        _ => text.push_str(&format!("{}{}\n", " ".repeat(indent), scalar(value))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locally_ready_doctor_keeps_the_session_check_visible() {
        let output = readable_output(CliOutput::success_text(
            serde_json::json!({
                "command":"doctor", "overall":"ready", "checks":[{
                    "id":"codex-local", "state":"ready", "blocking":false,
                    "code":"local-host-registered-session-unchecked",
                    "remediation":"restart-host-to-check-live-tools"
                }]
            })
            .to_string(),
        ));
        assert!(
            output
                .stdout()
                .contains("local-host-registered-session-unchecked")
        );
        assert!(output.stdout().contains("qiongli_config_status"));
        assert!(output.stdout().contains("Online retrieval: not checked"));
        assert!(output.stdout().contains("Saving in the Host: not checked"));
        assert_eq!(output.exit_code(), 0);
    }

    #[test]
    fn inventory_distinguishes_current_refresh_conflict_and_unknown_capabilities() {
        for (state, reason, full, expected) in [
            (
                "current",
                "local-host-registered-session-unchecked",
                "ready",
                "Full MCP is installed locally",
            ),
            (
                "repair-ready",
                "local-host-refresh-required",
                "ready",
                "compare CLI and Plugin versions",
            ),
            (
                "unavailable",
                "local-host-observation-unavailable",
                "ready",
                "Observation unavailable",
            ),
            (
                "resolve-conflict",
                "conflict",
                "ready",
                "Resolve the reported local conflict",
            ),
            (
                "current",
                "current",
                "unavailable",
                "Full MCP is not confirmed",
            ),
            (
                "install-ready",
                "missing",
                "missing",
                "Integration not confirmed",
            ),
            ("unknown", "unknown", "ready", "Capabilities not confirmed"),
        ] {
            let output = readable_output(CliOutput::success_text(
                serde_json::json!({
                    "command":"install-inventory", "inventory":{"clients":[{
                        "client":"codex", "readiness":state,"reason_code":reason,
                        "installed_plugin_version":"2.5.1", "components":{"full_mcp":full}
                    }]}, "cli":{"installations":[]}
                })
                .to_string(),
            ));
            let text = output.stdout();
            assert!(text.contains(expected), "{text}");
            assert!(text.contains("2.5.1") && text.contains(reason));
            assert!(text.contains("Saving in the Host: not checked"));
            assert_eq!(
                text.contains("Full MCP is installed locally"),
                state == "current" && full == "ready"
            );
            assert!(text.find(expected).unwrap() < text.find("CLI installations").unwrap());
        }
    }

    #[test]
    fn provider_summary_preserves_missing_settings_and_unchecked_credentials() {
        for store in ["available", "unavailable", "unknown"] {
            let output = readable_output(CliOutput::success_text(
                serde_json::json!({
                    "command":"provider-status", "secure_store":store,
                    "credential_access":"not-tested", "online_connection":"not-tested",
                    "providers":[
                        {"provider":"openalex","enabled":true,"readiness":"ready"},
                        {"provider":"semantic_scholar","enabled":true,"readiness":"needs_secret"},
                        {"provider":"crossref","enabled":true,"readiness":"needs_public_setting"},
                        {"provider":"pubmed","enabled":false,"readiness":"disabled"},
                        {"provider":"arxiv","enabled":true,"readiness":"ready"},
                        {"provider":"unknown\u{001b}[2J","enabled":true,"readiness":"unknown"}
                    ]
                })
                .to_string(),
            ));
            let text = output.stdout();
            assert!(text.contains("semantic_scholar: API key missing"));
            assert!(text.contains("crossref: public setting missing"));
            assert!(text.contains("pubmed: disabled"));
            assert!(text.contains("arxiv: configured; online retrieval not checked"));
            assert!(text.contains("configuration not known"));
            assert!(text.contains("not-tested") && text.contains("--confirm-network-request"));
            assert!(!text.contains('\u{001b}'));
            assert_eq!(
                text.contains("cannot save or access keys"),
                store == "unavailable"
            );
            assert_eq!(
                text.contains("openalex: configured, but credential storage is unavailable"),
                store == "unavailable"
            );
            assert_eq!(
                text.contains("availability not confirmed"),
                store == "unknown"
            );
        }
    }

    #[test]
    fn mcp_summary_distinguishes_lite_full_and_unknown_without_a_save_claim() {
        for (profile, expected) in [
            ("lite", "Saving: unavailable in Lite"),
            ("full", "saving was not exercised"),
            ("unknown", "profile not known"),
        ] {
            let output = readable_output(CliOutput::success_text(
                serde_json::json!({
                    "command":"mcp-check", "profile":profile,
                    "scope":"local-in-process-protocol", "initialize":"passed",
                    "host_session":"not-checked", "provider_connectivity":"not-checked"
                })
                .to_string(),
            ));
            let text = output.stdout();
            assert!(text.contains(expected));
            assert!(text.contains("Online retrieval: not checked"));
            assert!(text.contains("Host session: not checked"));
            assert!(text.contains("local-in-process-protocol"));
            assert_eq!(
                text.contains("Full MCP includes project tools"),
                profile == "full"
            );
        }
    }

    #[test]
    fn failed_online_provider_result_is_not_replaced_by_configuration_readiness() {
        let output = readable_output(
            CliOutput::success_text(
                serde_json::json!({
                    "command":"provider-test", "provider":"arxiv", "status":"failed",
                    "online_connection":"failed", "reason_code":"transport_error"
                })
                .to_string(),
            )
            .with_exit_code(1),
        );
        assert_eq!(output.exit_code(), 1);
        assert!(output.stdout().contains("failed") && output.stdout().contains("transport_error"));
        assert!(!output.stdout().contains("configured"));
    }

    #[test]
    fn plugin_summary_keeps_session_and_hook_checks_pending_and_escapes_paths() {
        use crate::managed_operation::ManagedIntegrationTargetV1;
        let mut source = crate::plugin_source::contract_source();
        source.destination = "/example/\u{001b}[2J\nqiongli-next".into();
        for (host, target) in [
            (ManagedIntegrationTargetV1::Codex, "codex"),
            (ManagedIntegrationTargetV1::ClaudeCode, "claude"),
        ] {
            source.target = host;
            for hooks in [false, true] {
                source.context_hooks = hooks;
                let text = plugin_install_summary(
                    &source,
                    "qiongli-next@qiongli-cli-local",
                    std::path::Path::new("/example/\u{202e}cache"),
                );
                assert!(text.contains("Pending   session tools"));
                assert!(text.contains("Provider connectivity has not been checked"));
                assert!(text.contains("saving is untested and requires preview and approval"));
                assert!(text.find("Research  ").unwrap() < text.find("Version   ").unwrap());
                assert!(text.contains(&format!("qiongli install plugin --target {target}")));
                assert!(text.contains("qiongli_config_status"));
                assert_eq!(text.contains("/hooks"), hooks);
                assert_eq!(text.contains("Hook trust and execution"), hooks);
                assert_eq!(text.contains("Hooks off in this Plugin"), !hooks);
                assert!(!text.contains(['\u{001b}', '\u{202e}']));
                assert!(text.contains("\\nqiongli-next"));
            }
        }
    }

    #[test]
    fn empty_command_groups_offer_terminal_help_and_preserve_script_errors() {
        let environment = CommandEnvironment::with_paths(None, None, None);
        let content = crate::embedded_content().unwrap();
        for words in [
            ["config", "backend"],
            ["project", "graph"],
            ["project", "capture"],
            ["project", "portfolio"],
            ["app", "plan"],
        ] {
            for (terminal, explicit_json, expected) in
                [(true, false, 0), (false, false, 2), (true, true, 2)]
            {
                let mut args = words.map(OsString::from).to_vec();
                if explicit_json {
                    args.insert(0, "--json".into());
                }
                let ProductAction::Output(output) =
                    prepare_cli_action(args, &environment, &content, terminal)
                else {
                    panic!("command group must not execute an operation")
                };
                assert_eq!(output.exit_code(), expected, "{words:?}");
                if expected == 0 {
                    assert!(
                        output
                            .stdout()
                            .contains(&format!("qiongli {}", words.join(" ")))
                    );
                }
            }
        }
        for words in [["project", "create"], ["mcp", "serve"], ["app", "apply"]] {
            let ProductAction::Output(output) = prepare_cli_action(
                words.map(OsString::from).to_vec(),
                &environment,
                &content,
                true,
            ) else {
                panic!("missing mutation or server arguments must refuse")
            };
            assert_eq!(output.exit_code(), 2);
        }
    }

    #[test]
    fn installation_shortcuts_select_guides_without_changing_script_queries() {
        let environment = CommandEnvironment::with_paths(None, None, None);
        let content = crate::embedded_content().unwrap();
        let action = |args: &[&str], terminal| {
            prepare_cli_action(
                args.iter().map(OsString::from).collect(),
                &environment,
                &content,
                terminal,
            )
        };
        for command in ["install", "upgrade"] {
            assert!(matches!(
                action(&[command], true),
                ProductAction::GuideInstallation(_)
            ));
            assert!(matches!(
                action(&[command], false),
                ProductAction::Output(_)
            ));
            assert!(matches!(
                action(&[command, "--json"], true),
                ProductAction::Output(_)
            ));
        }
        for command in ["install", "upgrade", "update"] {
            let ProductAction::GuideInstallation(guide) = action(&[command, "plugin"], true) else {
                panic!("plugin guide required")
            };
            assert!(guide.plugin && guide.targets.is_empty() && guide.destination.is_none());
            let ProductAction::GuideInstallation(guide) =
                action(&[command, "plugin", "--target", "all"], true)
            else {
                panic!("multiple Hosts required")
            };
            use crate::cli_content::PluginInstallHost;
            use crate::managed_operation::ManagedIntegrationTargetV1;
            assert_eq!(
                guide.targets,
                vec![
                    PluginInstallHost::Managed(ManagedIntegrationTargetV1::Codex),
                    PluginInstallHost::Managed(ManagedIntegrationTargetV1::ClaudeCode),
                    PluginInstallHost::DeepSeek,
                    PluginInstallHost::Antigravity,
                    PluginInstallHost::Pi,
                ]
            );
        }
        for args in [
            vec!["install", "plugin", "--target", "unknown"],
            vec![
                "install", "plugin", "--target", "codex", "--target", "claude",
            ],
            vec![
                "install",
                "plugin",
                "--target",
                "all",
                "--destination",
                "/shared/qiongli-next",
            ],
            vec![
                "install",
                "plugin",
                "--destination",
                "relative/qiongli-next",
            ],
            vec!["install", "plugin", "--dry-run"],
            vec!["install", "plugin", "--target", "all", "--dry-run"],
        ] {
            let ProductAction::Output(output) = action(&args, true) else {
                panic!("invalid invocation must refuse")
            };
            assert_eq!(output.exit_code(), 2);
        }
    }

    #[test]
    fn readable_results_preserve_exit_status_redaction_and_approval_values() {
        let json = serde_json::json!({"command":"project-create-preview", "preview": {
            "displayName":"A\u{001b}[2J\nB", "planDigest":"a".repeat(64),
            "projectId":"prj_00000000000000000000000000000001", "warnings":["review first"]
        }});
        let output = readable_output(CliOutput::success_text(json.to_string()));
        assert!(!output.stdout().contains('\u{001b}'));
        assert!(output.stdout().contains("\\nB"));
        assert!(output.stdout().contains(&"a".repeat(64)));
        assert!(output.stdout().contains("review first"));
        let output = CliOutput::operation_failure("failure").with_stdout(serde_json::json!({
            "command":"doctor", "overall":"attention", "checks":[{"id":"global-config", "state":"insecure", "blocking":true,"code":"global-config-insecure","remediation":"repair-permissions"},{"id":"full-runtime","state":"deferred","remediation":"upgrade-to-r4-full-runtime"}]
        }).to_string());
        let output = readable_output(output);
        assert_eq!(output.exit_code(), 1);
        assert!(output.stdout().contains("(blocking)"));
        assert!(output.stdout().contains("Repair permissions"));
        assert!(output.stdout().contains("Models run in your Host"));
        assert!(!output.stdout().contains("r4"));
        assert_eq!(output.stderr(), "error: failure\n");
    }
}
