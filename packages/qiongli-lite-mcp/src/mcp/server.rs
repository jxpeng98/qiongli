use std::sync::{Arc, Mutex};

use qiongli_runtime::evidence::{build_evidence_snapshot, EvidenceInput};
use qiongli_runtime::mcp::{prepare_zotero_upsert, validate_zotero_search};
use qiongli_runtime::orchestration::dispatch_lite_orchestration;
pub use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES as HANDLED_TOOL_NAMES;
use qiongli_runtime::{
    LiteConfigHandler, LiteDispatchTarget, LiteLiteratureHandler, LiteOrchestrationHandler,
    LiteToolId, LiteZoteroHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::provider_config::{
    normalize_key, normalize_provider, resolve_provider_config, save_provider_value, summary,
    ConfigError,
};
use crate::config::wizard::{start_config_wizard, ConfigWizardOptions, WizardError};
use crate::providers::runtime::ProviderRuntime;
use crate::providers::search::{execute_bounded_search, SearchRequest, PROVIDER_ORDER};
use crate::searchplan::{build_search_plan, SearchPlanInput, PLAN_PROVIDER_ORDER};
use crate::tools::definitions::lite_tool_definitions;
use crate::zotero::companion::{companion_from_env, probe_zotero_from_env};
use crate::zotero::export::{export_selected_import_files, ZoteroExportError, ZoteroExportRequest};

const MAX_CONTEXT_LENGTH: usize = 4096;
const REDACTED_CONFIG_ERROR: &str = "provider configuration is unavailable";
const REDACTED_CONFIG_SAVE_ERROR: &str = "provider configuration could not be saved";
const REDACTED_CONFIG_WIZARD_ERROR: &str = "provider configuration wizard could not start";
const REDACTED_PROVIDER_RUNTIME_ERROR: &str = "provider runtime is unavailable";
const STATUS_PROVIDER_ORDER: [&str; 5] = [
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
];

pub fn has_tool_handler(name: &str) -> bool {
    LiteToolId::from_public_name(name).is_some()
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
            _ => self.error(request.id, -32601, "Method not found"),
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
        let Some(tool_id) = LiteToolId::from_public_name(name) else {
            return self.error(id, -32601, "Tool not found");
        };
        if first_unknown_key(&arguments, allowed_arguments(tool_id)).is_some() {
            return self.error(id, -32602, "Unsupported argument");
        }

        match tool_id.dispatch_target() {
            LiteDispatchTarget::Config(LiteConfigHandler::Status) => {
                if arguments.get("cwd").is_some_and(|value| !value.is_string()) {
                    self.error(id, -32602, "cwd must be a string")
                } else {
                    match summary() {
                        Ok(status) => self.tool_result(id, json!(status)),
                        Err(_) => self.tool_error(id, REDACTED_CONFIG_ERROR.to_string()),
                    }
                }
            }
            LiteDispatchTarget::Config(LiteConfigHandler::SaveProvider) => {
                self.save_provider_config(id, &arguments)
            }
            LiteDispatchTarget::Config(LiteConfigHandler::ConfigureProvider) => {
                self.configure_provider(id, &arguments)
            }
            LiteDispatchTarget::Literature(LiteLiteratureHandler::Status) => {
                self.literature_status(id, &arguments)
            }
            LiteDispatchTarget::Literature(LiteLiteratureHandler::SearchPlan) => {
                self.search_plan(id, &arguments)
            }
            LiteDispatchTarget::Literature(LiteLiteratureHandler::Search) => {
                self.literature_search(id, &arguments)
            }
            LiteDispatchTarget::Literature(LiteLiteratureHandler::ExportEvidence) => {
                self.export_evidence(id, &arguments)
            }
            LiteDispatchTarget::Zotero(LiteZoteroHandler::Status) => self.zotero_status(id),
            LiteDispatchTarget::Zotero(LiteZoteroHandler::Search) => {
                self.zotero_search(id, &arguments)
            }
            LiteDispatchTarget::Zotero(LiteZoteroHandler::UpsertReferences) => {
                self.zotero_upsert_references(id, &arguments)
            }
            LiteDispatchTarget::Zotero(LiteZoteroHandler::ExportImportFiles) => {
                self.zotero_export_import_files(id, &arguments)
            }
            LiteDispatchTarget::Orchestration(handler) => {
                self.orchestration_preview(id, handler, &arguments)
            }
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
            Err(WizardError::NonLoopbackHost) => {
                self.error(id, -32602, "wizard host must be loopback")
            }
            Err(WizardError::UnsupportedProvider(_)) => {
                self.error(id, -32602, "unsupported provider")
            }
            Err(_) => self.tool_error(id, REDACTED_CONFIG_WIZARD_ERROR.to_string()),
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

        let normalized_provider = normalize_provider(provider);
        let normalized_field = normalize_key(field);
        match save_provider_value(provider, field, value.trim()) {
            Ok(path) => {
                let mut payload = json!({
                    "status": "saved",
                    "provider": normalized_provider,
                    "field": normalized_field,
                    "saved": true,
                    "config_path": path
                });
                if normalized_field == "api_key" {
                    payload["warning"] = json!(
                        "api_key was saved from chat input. Prefer qiongli_configure_provider \
                         so provider secrets do not enter chat history."
                    );
                }
                self.tool_result(id, payload)
            }
            Err(ConfigError::UnsupportedField(_, _)) => {
                self.error(id, -32602, "unsupported provider field")
            }
            Err(_) => self.tool_error(id, REDACTED_CONFIG_SAVE_ERROR.to_string()),
        }
    }

    fn literature_status(&self, id: Option<Value>, arguments: &Value) -> Value {
        if let Err(message) = validate_optional_context(arguments) {
            return self.error(id, -32602, message);
        }
        match summary() {
            Ok(status) => {
                let active_providers = active_providers_from_summary(&status);
                let mut payload = json!({
                    "status": &status.status,
                    "capability_mode": if active_providers.is_empty() {
                        "strategy_only"
                    } else {
                        "provider_connected"
                    },
                    "providers": &status.providers,
                    "active_providers": active_providers,
                    "missing": &status.missing,
                    "provider_capabilities": lite_provider_capabilities(),
                    "redacted_config": &status.redacted_config
                });
                if let Some(next_action) = status.next_action.as_ref() {
                    payload["next_action"] = json!(next_action);
                }
                self.tool_result(id, payload)
            }
            Err(_) => self.tool_error(id, REDACTED_CONFIG_ERROR.to_string()),
        }
    }

    fn search_plan(&self, id: Option<Value>, arguments: &Value) -> Value {
        let mut input = match SearchPlanInput::from_arguments(arguments, Vec::new()) {
            Ok(input) => input,
            Err(error) => return self.error(id, -32602, error.to_string()),
        };
        let active_providers = match self.active_provider_names() {
            Ok(providers) => providers,
            Err(_) => return self.tool_error(id, REDACTED_CONFIG_ERROR.to_string()),
        };
        input.active_providers = active_providers;
        let plan = build_search_plan(input);
        self.tool_result(id, json!(plan))
    }

    fn literature_search(&self, id: Option<Value>, arguments: &Value) -> Value {
        let request = match SearchRequest::from_arguments(arguments) {
            Ok(request) => request,
            Err(error) => return self.error(id, -32602, error.to_string()),
        };
        let runtime = match self.search_runtime() {
            Ok(runtime) => runtime,
            Err(error) => return self.tool_error(id, error),
        };
        let plan = build_search_plan(SearchPlanInput {
            query: request.query().to_string(),
            search_mode: arguments
                .get("search_mode")
                .and_then(Value::as_str)
                .unwrap_or("topic")
                .to_string(),
            platform: "unknown".to_string(),
            native_search_available: false,
            native_search_tools: Vec::new(),
            query_variants: Vec::new(),
            include_working_papers: None,
            from_year: None,
            to_year: None,
            venue_filter: None,
            document_types: Vec::new(),
            active_providers: PROVIDER_ORDER
                .iter()
                .filter(|provider| selected_providers_include(provider, request.providers()))
                .filter(|provider| runtime.config().is_active(provider))
                .map(|provider| (*provider).to_string())
                .collect(),
        });
        let output = match execute_bounded_search(&runtime, &request) {
            Ok(output) => output,
            Err(_) => return self.tool_error(id, "provider search was cancelled".to_string()),
        };
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

    fn active_provider_names(&self) -> Result<Vec<String>, ConfigError> {
        if let Some(runtime) = &self.provider_runtime {
            return Ok(PLAN_PROVIDER_ORDER
                .iter()
                .filter(|provider| runtime.config().is_active(provider))
                .map(|provider| (*provider).to_string())
                .collect());
        }
        summary().map(|status| active_providers_from_summary(&status))
    }

    fn search_runtime(&self) -> Result<ProviderRuntime, String> {
        if let Some(runtime) = &self.provider_runtime {
            return Ok(runtime.clone());
        }
        let config = resolve_provider_config().map_err(|_| REDACTED_CONFIG_ERROR.to_string())?;
        ProviderRuntime::production(config).map_err(|_| REDACTED_PROVIDER_RUNTIME_ERROR.to_string())
    }

    fn orchestration_preview(
        &self,
        id: Option<Value>,
        handler: LiteOrchestrationHandler,
        arguments: &Value,
    ) -> Value {
        match dispatch_lite_orchestration(handler, arguments) {
            Ok(preview) => self.tool_result(id, json!(preview)),
            Err(error) => self.error(id, -32602, error.to_string()),
        }
    }

    fn export_evidence(&self, id: Option<Value>, arguments: &Value) -> Value {
        match EvidenceInput::from_arguments(arguments) {
            Ok(input) => self.tool_result(id, json!(build_evidence_snapshot(input))),
            Err(error) => self.error(id, -32602, error.to_string()),
        }
    }

    fn zotero_status(&self, id: Option<Value>) -> Value {
        match probe_zotero_from_env() {
            Ok(status) => self.tool_result(id, json!(status)),
            Err(error) => self.tool_error(id, error.to_string()),
        }
    }

    fn zotero_search(&self, id: Option<Value>, arguments: &Value) -> Value {
        if let Err(message) = validate_zotero_search(arguments) {
            return self.error(id, -32602, message);
        }
        let client = match companion_from_env() {
            Ok(Some(client)) => client,
            Ok(None) => return self.tool_error(id, "local Zotero is disabled".to_owned()),
            Err(error) => return self.tool_error(id, error.to_string()),
        };
        match client.search(arguments) {
            Ok(result) => self.tool_result(id, result),
            Err(error) => self.tool_error(id, error.public_message().to_owned()),
        }
    }

    fn zotero_upsert_references(&self, id: Option<Value>, arguments: &Value) -> Value {
        let request = match prepare_zotero_upsert(arguments) {
            Ok(request) => request,
            Err(message) => return self.error(id, -32602, message),
        };
        let client = match companion_from_env() {
            Ok(Some(client)) => client,
            Ok(None) => return self.tool_error(id, "local Zotero is disabled".to_owned()),
            Err(error) => return self.tool_error(id, error.to_string()),
        };
        match client.upsert(&request) {
            Ok(result) => self.tool_result(id, result),
            Err(error) => self.tool_error(id, error.public_message().to_owned()),
        }
    }

    fn zotero_export_import_files(&self, id: Option<Value>, arguments: &Value) -> Value {
        let request = match ZoteroExportRequest::from_arguments(arguments) {
            Ok(request) => request,
            Err(error) => return self.error(id, -32602, error.to_string()),
        };
        let files = match export_selected_import_files(request) {
            Ok(files) => files,
            Err(ZoteroExportError::OutputTooLarge | ZoteroExportError::Serialization) => {
                return self.tool_error(id, "Zotero export failed".to_owned())
            }
            Err(error) => return self.error(id, -32602, error.to_string()),
        };
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
        let structured_content = redact_tool_output(structured_content);
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

fn redact_tool_output(value: Value) -> Value {
    match value {
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .filter_map(|(key, value)| {
                    (!credential_bearing_key(&key)).then(|| (key, redact_tool_output(value)))
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_tool_output).collect()),
        value => value,
    }
}

fn credential_bearing_key(key: &str) -> bool {
    let normalized = key.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    let compact = normalized.replace('_', "");
    let padded = format!("_{normalized}_");
    let has_sensitive_segment = normalized.split('_').any(|segment| {
        matches!(
            segment,
            "secret" | "password" | "passwd" | "credential" | "credentials" | "auth" | "bearer"
        )
    });
    let has_sensitive_marker = [
        "api_key",
        "access_key",
        "authorization",
        "cookie",
        "private_key",
        "client_secret",
        "access_token",
        "refresh_token",
        "auth_token",
        "id_token",
    ]
    .iter()
    .any(|marker| padded.contains(&format!("_{marker}_")));

    has_sensitive_segment
        || normalized == "token"
        || normalized == "authorization"
        || normalized.ends_with("_token")
        || compact.ends_with("token")
        || compact.ends_with("apikey")
        || compact.ends_with("accesstoken")
        || compact.ends_with("accesskey")
        || compact.ends_with("privatekey")
        || compact.ends_with("clientsecret")
        || has_sensitive_marker
}

fn allowed_arguments(tool_id: LiteToolId) -> &'static [&'static str] {
    match tool_id {
        LiteToolId::ConfigStatus | LiteToolId::LiteratureStatus => &["cwd"],
        LiteToolId::ZoteroStatus => &[],
        LiteToolId::SaveProviderConfig => &["provider", "field", "value"],
        LiteToolId::ConfigureProvider => &["provider", "host", "port"],
        LiteToolId::SearchPlan => &[
            "cwd",
            "query",
            "platform",
            "search_mode",
            "searchMode",
            "native_search_available",
            "native_search_usable",
            "nativeSearchAvailable",
            "native_search_tools",
            "nativeSearchTools",
            "query_variants",
            "queryVariants",
            "include_working_papers",
            "includeWorkingPapers",
            "from_year",
            "fromYear",
            "to_year",
            "toYear",
            "venue_filter",
            "venueFilter",
            "document_types",
            "documentTypes",
        ],
        LiteToolId::LiteratureSearch => &[
            "query",
            "search_mode",
            "providers",
            "limit",
            "per_provider_limit",
            "total_limit",
        ],
        LiteToolId::LiteratureExportEvidence => &[
            "cwd",
            "query",
            "provider_status",
            "search_plan",
            "results",
            "diagnostics",
            "query_plan",
            "search_results",
            "search_diagnostics",
        ],
        LiteToolId::ZoteroSearch => &[
            "doi",
            "title",
            "year",
            "citekey",
            "creator",
            "tag",
            "collection_path",
            "limit",
        ],
        LiteToolId::ZoteroUpsertReferences => &[
            "items",
            "collection_path",
            "update_policy",
            "dry_run",
            "write_intent",
            "dry_run_receipt",
        ],
        LiteToolId::ZoteroExportImportFiles => &["records", "formats"],
        LiteToolId::OrchestratorRoute => &["request", "platform"],
        LiteToolId::TaskPlan => &["task_id", "paper_type", "topic"],
    }
}

fn validate_optional_context(arguments: &Value) -> Result<(), String> {
    let Some(value) = arguments.get("cwd") else {
        return Ok(());
    };
    let cwd = value
        .as_str()
        .ok_or_else(|| "cwd must be a string".to_string())?;
    if cwd.trim().is_empty() {
        return Err("cwd must not be empty".to_string());
    }
    if cwd.chars().count() > MAX_CONTEXT_LENGTH {
        return Err(format!(
            "cwd must be at most {MAX_CONTEXT_LENGTH} characters"
        ));
    }
    Ok(())
}

fn active_providers_from_summary(
    status: &crate::config::provider_config::ProviderSummary,
) -> Vec<String> {
    STATUS_PROVIDER_ORDER
        .iter()
        .filter(|provider| {
            status
                .redacted_config
                .providers
                .get(**provider)
                .is_some_and(|entry| entry.enabled && entry.configured)
        })
        .map(|provider| (*provider).to_string())
        .collect()
}

fn lite_provider_capabilities() -> Value {
    json!({
        "openalex": {
            "status": "implemented",
            "max_per_provider_limit": 200,
            "capabilities": ["topic_search", "venue_metadata"]
        },
        "semantic_scholar": {
            "status": "implemented",
            "max_per_provider_limit": 200,
            "capabilities": ["topic_search", "venue_metadata"]
        },
        "crossref": {
            "status": "implemented",
            "max_per_provider_limit": 200,
            "capabilities": ["topic_search", "venue_metadata"]
        },
        "pubmed": {
            "status": "implemented",
            "max_per_provider_limit": 200,
            "capabilities": ["topic_search", "biomedical_topic_search"]
        },
        "arxiv": {
            "status": "implemented",
            "max_per_provider_limit": 200,
            "capabilities": ["topic_search", "preprint_search"]
        }
    })
}

fn selected_providers_include(
    provider: &str,
    selected: Option<&[qiongli_runtime::providers::ProviderId]>,
) -> bool {
    selected.is_none_or(|providers| {
        qiongli_runtime::providers::ProviderId::parse(provider)
            .is_ok_and(|candidate| providers.contains(&candidate))
    })
}
