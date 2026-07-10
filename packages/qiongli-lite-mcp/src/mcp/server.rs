use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::provider_config::{
    normalize_key, resolve_provider_config, save_provider_value, summary,
};
use crate::config::wizard::{start_config_wizard, ConfigWizardOptions};
use crate::orchestrator::preview::{build_task_plan, TaskPlanInput};
use crate::providers::runtime::ProviderRuntime;
use crate::providers::search::{execute_search, SearchInput, PROVIDER_ORDER};
use crate::searchplan::{build_search_plan, SearchPlanInput};
use crate::tools::definitions::lite_tool_definitions;
use crate::zotero::companion::probe_zotero_from_env;
use crate::zotero::export::export_import_files;

pub const HANDLED_TOOL_NAMES: [&str; 12] = [
    "qiongli_config_status",
    "qiongli_save_provider_config",
    "qiongli_configure_provider",
    "qiongli_open_config_wizard",
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
    "qiongli_zotero_status",
    "qiongli_zotero_export_import_files",
    "qiongli_orchestrator_route",
    "qiongli_task_plan",
];

pub fn has_tool_handler(name: &str) -> bool {
    HANDLED_TOOL_NAMES.contains(&name)
}

#[derive(Debug, Clone, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct McpError {
    code: i64,
    message: String,
}

#[derive(Clone)]
pub struct McpServer {
    name: String,
    version: String,
    provider_runtime: Option<ProviderRuntime>,
    wizard_session: Arc<Mutex<Option<crate::config::wizard::ConfigWizard>>>,
}

impl McpServer {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            provider_runtime: None,
            wizard_session: Arc::new(Mutex::new(None)),
        }
    }

    #[doc(hidden)]
    pub fn with_provider_runtime(
        name: impl Into<String>,
        version: impl Into<String>,
        provider_runtime: ProviderRuntime,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            provider_runtime: Some(provider_runtime),
            wizard_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn handle(&self, request: McpRequest) -> Value {
        if request.jsonrpc != "2.0" {
            return self.error(request.id, -32600, "Invalid JSON-RPC version");
        }

        match request.method.as_str() {
            "initialize" => self.result(
                request.id,
                json!({
                    "protocolVersion": request
                        .params
                        .as_ref()
                        .and_then(|params| params.get("protocolVersion"))
                        .and_then(Value::as_str)
                        .unwrap_or("2025-11-25"),
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": self.name,
                        "version": self.version
                    }
                }),
            ),
            "ping" => self.result(request.id, json!({})),
            "tools/list" => self.result(request.id, json!({"tools": lite_tool_definitions()})),
            "tools/call" => self.handle_tool_call(request.id, request.params),
            _ => self.error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    fn handle_tool_call(&self, id: Option<Value>, params: Option<Value>) -> Value {
        let Some(params) = params else {
            return self.error(id, -32602, "Missing tool call params");
        };
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing tool name");
        };
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !arguments.is_object() {
            return self.error(id, -32602, "Tool arguments must be an object");
        }
        if let Some(allowed) = allowed_arguments(name) {
            if let Some(unknown) = first_unknown_key(&arguments, allowed) {
                return self.error(id, -32602, format!("Unsupported argument: {unknown}"));
            }
        }

        match name {
            "qiongli_config_status" => match summary() {
                Ok(status) => self.tool_result(id, json!(status)),
                Err(error) => self.tool_error(id, error.to_string()),
            },
            "qiongli_save_provider_config" => self.save_provider_config(id, &arguments),
            "qiongli_configure_provider" | "qiongli_open_config_wizard" => {
                self.configure_provider(id, &arguments)
            }
            "qiongli_literature_status" => self.literature_status(id),
            "qiongli_search_plan" => self.search_plan(id, &arguments),
            "qiongli_literature_search" => self.literature_search(id, &arguments),
            "qiongli_literature_export_evidence" => self.export_evidence(id, &arguments),
            "qiongli_zotero_status" => self.zotero_status(id),
            "qiongli_zotero_export_import_files" => self.zotero_export_import_files(id, &arguments),
            "qiongli_orchestrator_route" => self.orchestrator_route(id, &arguments),
            "qiongli_task_plan" => self.task_plan(id, &arguments),
            _ => self.error(id, -32601, format!("Tool not found: {name}")),
        }
    }

    fn configure_provider(&self, id: Option<Value>, arguments: &Value) -> Value {
        let host = match arguments.get("host") {
            Some(value) => match value.as_str() {
                Some(value) => value.to_string(),
                None => return self.error(id, -32602, "host must be a string"),
            },
            None => "127.0.0.1".to_string(),
        };
        let port = match arguments.get("port") {
            Some(value) => match value.as_u64() {
                Some(port) if port <= u16::MAX as u64 => port as u16,
                _ => return self.error(id, -32602, "port must be between 0 and 65535"),
            },
            None => 0,
        };
        let provider = match arguments.get("provider") {
            Some(value) => match value.as_str() {
                Some(value) => Some(value.to_string()),
                None => return self.error(id, -32602, "provider must be a string"),
            },
            None => None,
        };
        let options = ConfigWizardOptions {
            host,
            port,
            provider,
            ..ConfigWizardOptions::default()
        };

        let mut active_session = match self.wizard_session.lock() {
            Ok(session) => session,
            Err(_) => {
                return self.tool_error(id, "configuration wizard state is unavailable".to_string())
            }
        };
        if let Some(wizard) = active_session.as_ref() {
            if wizard.is_running() {
                return self.tool_result(id, wizard_payload(wizard, "already_running"));
            }
        }
        *active_session = None;

        match start_config_wizard(options) {
            Ok(wizard) => {
                let payload = wizard_payload(&wizard, "ready");
                *active_session = Some(wizard);
                self.tool_result(id, payload)
            }
            Err(error) => self.tool_error(id, error.to_string()),
        }
    }

    fn save_provider_config(&self, id: Option<Value>, arguments: &Value) -> Value {
        let Some(provider) = arguments.get("provider").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing provider");
        };
        let Some(field) = arguments.get("field").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing field");
        };
        let Some(value) = arguments.get("value").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing value");
        };
        if value.trim().is_empty() {
            return self.error(id, -32602, "value must not be empty");
        }

        let normalized_provider = normalize_key(provider);
        let normalized_field = normalize_key(field);
        match save_provider_value(provider, field, value) {
            Ok(path) => self.tool_result(
                id,
                json!({
                    "status": "saved",
                    "provider": normalized_provider,
                    "field": normalized_field,
                    "saved": true,
                    "config_path": path,
                    "warning": "Prefer qiongli_configure_provider for interactive API key setup."
                }),
            ),
            Err(error) => self.tool_error(id, error.to_string()),
        }
    }

    fn literature_status(&self, id: Option<Value>) -> Value {
        match summary() {
            Ok(status) => self.tool_result(
                id,
                json!({
                    "status": status.status,
                    "capability_mode": status.capability_mode,
                    "providers": status.providers,
                    "missing": status.missing,
                    "provider_capabilities": {
                        "openalex": ["search"],
                        "semantic_scholar": ["search"],
                        "crossref": ["search"],
                        "pubmed": ["search"],
                        "arxiv": ["search"]
                    }
                }),
            ),
            Err(error) => self.tool_error(id, error.to_string()),
        }
    }

    fn search_plan(&self, id: Option<Value>, arguments: &Value) -> Value {
        let Some(query) = arguments.get("query").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing query");
        };
        if query.trim().is_empty() {
            return self.error(id, -32602, "query must not be empty");
        }
        let search_mode = match parse_search_mode(arguments) {
            Ok(value) => value,
            Err(message) => return self.error(id, -32602, message),
        };
        let native_value = arguments
            .get("native_search_available")
            .or_else(|| arguments.get("native_search_usable"));
        let native_search_available = match native_value {
            Some(value) => match value.as_bool() {
                Some(value) => value,
                None => return self.error(id, -32602, "native_search_available must be a boolean"),
            },
            None => false,
        };
        let provider_connected = self.provider_connected();
        let plan = build_search_plan(SearchPlanInput {
            query: query.trim().to_string(),
            search_mode,
            provider_connected,
            native_search_available,
        });
        self.tool_result(id, json!(plan))
    }

    fn literature_search(&self, id: Option<Value>, arguments: &Value) -> Value {
        let Some(query) = arguments.get("query").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing query");
        };
        if query.trim().is_empty() {
            return self.error(id, -32602, "query must not be empty");
        }
        let search_mode = match parse_search_mode(arguments) {
            Ok(value) => value,
            Err(message) => return self.error(id, -32602, message),
        };
        let providers = match parse_providers(arguments) {
            Ok(value) => value,
            Err(message) => return self.error(id, -32602, message),
        };
        let limit = match parse_limit(arguments, "limit", 200) {
            Ok(value) => value,
            Err(message) => return self.error(id, -32602, message),
        };
        let per_provider_limit = match parse_limit(arguments, "per_provider_limit", 200) {
            Ok(value) => value,
            Err(message) => return self.error(id, -32602, message),
        };
        let total_limit = match parse_limit(arguments, "total_limit", 1000) {
            Ok(value) => value,
            Err(message) => return self.error(id, -32602, message),
        };
        let input = SearchInput {
            query: query.trim().to_string(),
            search_mode,
            limit,
            per_provider_limit,
            total_limit,
        };
        let runtime = match self.search_runtime() {
            Ok(runtime) => runtime,
            Err(error) => return self.tool_error(id, error),
        };
        let plan = build_search_plan(SearchPlanInput {
            query: input.query.clone(),
            search_mode: input.search_mode.clone(),
            provider_connected: PROVIDER_ORDER
                .iter()
                .filter(|provider| selected_providers_include(provider, providers.as_deref()))
                .any(|provider| runtime.config().is_active(provider)),
            native_search_available: false,
        });
        let output = execute_search(&runtime, &input, providers.as_deref());
        self.tool_result(
            id,
            json!({
                "status": output.status,
                "search_plan": plan,
                "diagnostics": output.diagnostics,
                "results": output.results
            }),
        )
    }

    fn provider_connected(&self) -> bool {
        if let Some(runtime) = &self.provider_runtime {
            return PROVIDER_ORDER
                .iter()
                .any(|provider| runtime.config().is_active(provider));
        }
        summary()
            .map(|status| status.capability_mode == "provider_connected")
            .unwrap_or(false)
    }

    fn search_runtime(&self) -> Result<ProviderRuntime, String> {
        if let Some(runtime) = &self.provider_runtime {
            return Ok(runtime.clone());
        }
        let config = resolve_provider_config().map_err(|error| error.to_string())?;
        ProviderRuntime::production(config).map_err(|error| error.to_string())
    }

    fn orchestrator_route(&self, id: Option<Value>, arguments: &Value) -> Value {
        let Some(request) = arguments.get("request").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing request");
        };
        if request.trim().is_empty() {
            return self.error(id, -32602, "request must not be empty");
        }
        if let Some(platform) = arguments.get("platform") {
            let Some(platform) = platform.as_str() else {
                return self.error(id, -32602, "platform must be a string");
            };
            if ![
                "codex",
                "claude_code",
                "claude",
                "antigravity",
                "cli",
                "unknown",
            ]
            .contains(&platform)
            {
                return self.error(id, -32602, "unsupported platform");
            }
        }
        self.tool_result(
            id,
            json!({
                "mode": "preview",
                "preview_only": true,
                "runtime_profile": "marketplace_lite",
                "run_agents_allowed": false,
                "shell_execution_allowed": false,
                "project_writes_allowed": false,
                "request": request,
                "platform": arguments
                    .get("platform")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                "recommended_runtime": "full_cli_for_execution",
                "upgrade": {
                    "required_for_execution": true,
                    "runtime_profile": "full_cli",
                    "command": "qiongli mcp serve --transport stdio"
                }
            }),
        )
    }

    fn task_plan(&self, id: Option<Value>, arguments: &Value) -> Value {
        let Some(task_id) = arguments.get("task_id").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing task_id");
        };
        let Some(paper_type) = arguments.get("paper_type").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing paper_type");
        };
        let Some(topic) = arguments.get("topic").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing topic");
        };
        if task_id.trim().is_empty() || paper_type.trim().is_empty() || topic.trim().is_empty() {
            return self.error(id, -32602, "task plan fields must not be empty");
        }
        let plan = build_task_plan(TaskPlanInput {
            task_id: task_id.trim().to_string(),
            paper_type: paper_type.trim().to_string(),
            topic: topic.trim().to_string(),
        });
        self.tool_result(id, json!(plan))
    }

    fn export_evidence(&self, id: Option<Value>, arguments: &Value) -> Value {
        if arguments.get("cwd").is_some_and(|value| !value.is_string()) {
            return self.error(id, -32602, "cwd must be a string");
        }
        if arguments
            .get("query")
            .is_some_and(|value| !value.is_string())
        {
            return self.error(id, -32602, "query must be a string");
        }
        for field in [
            "provider_status",
            "search_plan",
            "query_plan",
            "diagnostics",
            "search_diagnostics",
        ] {
            if arguments.get(field).is_some_and(|value| !value.is_object()) {
                return self.error(id, -32602, format!("{field} must be an object"));
            }
        }
        for field in ["results", "search_results"] {
            if let Some(value) = arguments.get(field) {
                let Some(values) = value.as_array() else {
                    return self.error(id, -32602, format!("{field} must be an array"));
                };
                if values.iter().any(|item| !item.is_object()) {
                    return self.error(id, -32602, format!("{field} must contain objects"));
                }
            }
        }
        let results = arguments
            .get("results")
            .or_else(|| arguments.get("search_results"))
            .cloned()
            .unwrap_or_else(|| json!([]));
        let result_count = results.as_array().map_or(0, Vec::len);
        self.tool_result(
            id,
            json!({
                "status": "ok",
                "artifact_type": "qiongli_literature_evidence_snapshot",
                "query": arguments
                    .get("query")
                    .cloned()
                    .unwrap_or_else(|| json!("")),
                "provider_status": arguments
                    .get("provider_status")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                "search_plan": arguments
                    .get("search_plan")
                    .or_else(|| arguments.get("query_plan"))
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                "result_count": result_count,
                "results": results,
                "diagnostics": arguments
                    .get("diagnostics")
                    .or_else(|| arguments.get("search_diagnostics"))
                    .cloned()
                    .unwrap_or_else(|| json!({}))
            }),
        )
    }

    fn zotero_status(&self, id: Option<Value>) -> Value {
        match probe_zotero_from_env() {
            Ok(status) => self.tool_result(id, json!(status)),
            Err(error) => self.tool_error(id, error.to_string()),
        }
    }

    fn zotero_export_import_files(&self, id: Option<Value>, arguments: &Value) -> Value {
        let records = arguments
            .get("records")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let records = match serde_json::from_value(records) {
            Ok(records) => records,
            Err(error) => return self.tool_error(id, error.to_string()),
        };
        let mut files = export_import_files(records);
        if let Some(formats) = arguments.get("formats") {
            let Some(formats) = formats.as_array() else {
                return self.error(id, -32602, "formats must be an array");
            };
            if !formats.is_empty() {
                let mut selected = Vec::new();
                for value in formats {
                    let Some(format) = value.as_str() else {
                        return self.error(id, -32602, "formats must contain strings");
                    };
                    if ![
                        "references.json",
                        "references.ris",
                        "bibliography.bib",
                        "zotero-import-report.md",
                    ]
                    .contains(&format)
                    {
                        return self.error(id, -32602, format!("Unsupported format: {format}"));
                    }
                    selected.push(format);
                }
                files.retain(|name, _| selected.contains(&name.as_str()));
            }
        }
        self.tool_result(
            id,
            json!({
                "status": "ok",
                "files": files
            }),
        )
    }

    fn result(&self, id: Option<Value>, result: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "result": result})
    }

    fn tool_result(&self, id: Option<Value>, structured_content: Value) -> Value {
        let text = serde_json::to_string_pretty(&structured_content)
            .unwrap_or_else(|_| "{\"status\":\"ok\"}".to_string());
        self.result(
            id,
            json!({
                "content": [{"type": "text", "text": text}],
                "structuredContent": structured_content
            }),
        )
    }

    fn tool_error(&self, id: Option<Value>, message: String) -> Value {
        let structured_message = message.clone();
        self.result(
            id,
            json!({
                "isError": true,
                "content": [{"type": "text", "text": message}],
                "structuredContent": {
                    "status": "error",
                    "error_kind": "tool_error",
                    "message": structured_message
                }
            }),
        )
    }

    fn error(&self, id: Option<Value>, code: i64, message: impl Into<String>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": McpError {
                code,
                message: message.into(),
            }
        })
    }
}

fn wizard_payload(wizard: &crate::config::wizard::ConfigWizard, status: &str) -> Value {
    let mut payload = json!({
        "status": status,
        "url": wizard.url(),
        "host": wizard.host(),
        "port": wizard.port(),
        "config_path": wizard.config_path()
    });
    if let Some(provider) = wizard.provider() {
        payload["provider"] = json!(provider);
    }
    payload
}

fn first_unknown_key<'a>(arguments: &'a Value, allowed: &[&str]) -> Option<&'a str> {
    arguments
        .as_object()?
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map(String::as_str)
}

fn allowed_arguments(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "qiongli_config_status" | "qiongli_literature_status" | "qiongli_zotero_status" => {
            Some(&[])
        }
        "qiongli_save_provider_config" => Some(&["provider", "field", "value"]),
        "qiongli_configure_provider" | "qiongli_open_config_wizard" => {
            Some(&["provider", "host", "port"])
        }
        "qiongli_search_plan" => Some(&[
            "query",
            "search_mode",
            "native_search_available",
            "native_search_usable",
        ]),
        "qiongli_literature_search" => Some(&[
            "query",
            "search_mode",
            "providers",
            "limit",
            "per_provider_limit",
            "total_limit",
        ]),
        "qiongli_literature_export_evidence" => Some(&[
            "cwd",
            "query",
            "provider_status",
            "search_plan",
            "results",
            "diagnostics",
            "query_plan",
            "search_results",
            "search_diagnostics",
        ]),
        "qiongli_zotero_export_import_files" => Some(&["records", "formats"]),
        "qiongli_orchestrator_route" => Some(&["request", "platform"]),
        "qiongli_task_plan" => Some(&["task_id", "paper_type", "topic"]),
        _ => None,
    }
}

fn parse_limit(arguments: &Value, name: &str, maximum: usize) -> Result<Option<usize>, String> {
    let Some(value) = arguments.get(name) else {
        return Ok(None);
    };
    let value = value
        .as_u64()
        .ok_or_else(|| format!("{name} must be an integer"))?;
    if value == 0 || value > maximum as u64 {
        return Err(format!("{name} must be between 1 and {maximum}"));
    }
    Ok(Some(value as usize))
}

fn parse_search_mode(arguments: &Value) -> Result<Option<String>, String> {
    let Some(value) = arguments.get("search_mode") else {
        return Ok(None);
    };
    let value = value
        .as_str()
        .ok_or_else(|| "search_mode must be a string".to_string())?;
    if !["auto", "topic", "review", "systematic_review"].contains(&value) {
        return Err("unsupported search_mode".to_string());
    }
    Ok(Some(value.to_string()))
}

fn parse_providers(arguments: &Value) -> Result<Option<Vec<String>>, String> {
    let Some(value) = arguments.get("providers") else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or_else(|| "providers must be an array".to_string())?;
    if values.is_empty() {
        return Err("providers must not be empty".to_string());
    }
    let mut providers = Vec::new();
    for value in values {
        let provider = value
            .as_str()
            .ok_or_else(|| "providers must contain strings".to_string())?;
        if !PROVIDER_ORDER.contains(&provider) {
            return Err(format!("unsupported provider: {provider}"));
        }
        if !providers.iter().any(|candidate| candidate == provider) {
            providers.push(provider.to_string());
        }
    }
    Ok(Some(providers))
}

fn selected_providers_include(provider: &str, selected: Option<&[String]>) -> bool {
    selected.is_none_or(|providers| providers.iter().any(|candidate| candidate == provider))
}
