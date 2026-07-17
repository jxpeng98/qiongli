//! Marketplace Lite JSON-RPC/MCP composition for native Qiongli entrypoints.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::evidence::{EvidenceInput, build_evidence_snapshot};
use crate::orchestration::dispatch_lite_orchestration;
use crate::protocol::{read_message, write_message};
use crate::providers::search::{PROVIDER_ORDER as SEARCH_PROVIDER_ORDER, SearchRequest};
use crate::providers::{ProviderAccess, ProviderField, ProviderId, ProviderRuntime};
use crate::searchplan::{PLAN_PROVIDER_ORDER, SearchPlanInput, build_search_plan};
use crate::zotero::companion::ZoteroStatus;
use crate::zotero::export::{ZoteroExportError, ZoteroExportRequest, export_selected_import_files};
use crate::{
    LiteConfigHandler, LiteDispatchTarget, LiteLiteratureHandler, LiteOrchestrationHandler,
    LiteToolId, LiteToolRegistry, LiteZoteroHandler, RuntimeError,
};

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

const MAX_TOOL_NAME_BYTES: usize = 256;
const MAX_REQUEST_ID_BYTES: usize = 256;
const MAX_CONTEXT_CHARS: usize = 4_096;
const MAX_SECRET_INPUT_BYTES: usize = 64 * 1_024;
const MANAGED_CONFIG_IDENTIFIER: &str = "<managed-native-config>";
const CONFIG_UNAVAILABLE: &str = "native provider configuration is unavailable";
const PROVIDER_RUNTIME_UNAVAILABLE: &str = "native provider runtime is unavailable";
const CONFIG_MUTATION_UNAVAILABLE: &str =
    "provider configuration writes are unavailable in this native preview";
const CONFIG_WIZARD_UNAVAILABLE: &str =
    "provider configuration wizard is unavailable in this native preview";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct McpRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Clone)]
enum ProviderState {
    Available(Box<ProviderServices>),
    ConfigUnavailable,
}

#[derive(Clone)]
struct ProviderServices {
    access: ProviderAccess,
    runtime: Option<ProviderRuntime>,
}

/// Shared Marketplace Lite MCP server. This type intentionally has no Debug
/// implementation because its provider runtime can contain resolved secrets.
#[derive(Clone)]
pub struct LiteMcpServer {
    name: String,
    version: String,
    registry: LiteToolRegistry,
    providers: ProviderState,
}

impl LiteMcpServer {
    #[must_use]
    pub fn production(
        name: impl Into<String>,
        version: impl Into<String>,
        registry: LiteToolRegistry,
        access: ProviderAccess,
    ) -> Self {
        let runtime = ProviderRuntime::production(access.clone()).ok();
        Self {
            name: name.into(),
            version: version.into(),
            registry,
            providers: ProviderState::Available(Box::new(ProviderServices { access, runtime })),
        }
    }

    #[must_use]
    pub fn config_unavailable(
        name: impl Into<String>,
        version: impl Into<String>,
        registry: LiteToolRegistry,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            registry,
            providers: ProviderState::ConfigUnavailable,
        }
    }

    /// Inject a complete bounded provider runtime for deterministic tests.
    #[doc(hidden)]
    #[must_use]
    pub fn with_provider_runtime(
        name: impl Into<String>,
        version: impl Into<String>,
        registry: LiteToolRegistry,
        runtime: ProviderRuntime,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            registry,
            providers: ProviderState::Available(Box::new(ProviderServices {
                access: runtime.access().clone(),
                runtime: Some(runtime),
            })),
        }
    }

    pub fn serve<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<(), RuntimeError> {
        while let Some(message) = read_message(reader)? {
            let response = match serde_json::from_str::<Value>(&message.payload) {
                Ok(request) => self.handle(request),
                Err(_) => Some(json_rpc_error(None, -32700, "Parse error")),
            };
            if let Some(response) = response {
                write_message(writer, &response, message.framing)?;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn handle(&self, request: Value) -> Option<Value> {
        let has_id = request
            .as_object()
            .is_some_and(|entries| entries.contains_key("id"));
        let request = match serde_json::from_value::<McpRequest>(request) {
            Ok(request) => request,
            Err(_) => return Some(json_rpc_error(None, -32600, "Invalid Request")),
        };
        if request.jsonrpc != "2.0" || request.method.is_empty() {
            return Some(json_rpc_error(request.id, -32600, "Invalid Request"));
        }
        if request.id.as_ref().is_some_and(|id| !valid_request_id(id)) {
            return Some(json_rpc_error(None, -32600, "Invalid Request"));
        }
        if request.method == "notifications/initialized" {
            return if has_id {
                Some(json_rpc_error(request.id, -32600, "Invalid notification"))
            } else {
                None
            };
        }
        if !has_id {
            return None;
        }
        let id = request.id.unwrap_or(Value::Null);

        Some(match request.method.as_str() {
            "initialize" => self.initialize(id, request.params),
            "ping" => self.empty_result(id, request.params),
            "tools/list" => self.list_tools(id, request.params),
            "tools/call" => self.handle_tool_call(id, request.params),
            _ => json_rpc_error(Some(id), -32601, "Method not found"),
        })
    }

    fn initialize(&self, id: Value, params: Option<Value>) -> Value {
        if params.as_ref().is_some_and(|params| !params.is_object()) {
            return json_rpc_error(Some(id), -32602, "Invalid initialize params");
        }
        json_rpc_result(
            id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": self.name,
                    "version": self.version
                }
            }),
        )
    }

    fn empty_result(&self, id: Value, params: Option<Value>) -> Value {
        if params.as_ref().is_some_and(|params| !params.is_object()) {
            return json_rpc_error(Some(id), -32602, "Invalid params");
        }
        json_rpc_result(id, json!({}))
    }

    fn list_tools(&self, id: Value, params: Option<Value>) -> Value {
        if params.as_ref().is_some_and(|params| !params.is_object()) {
            return json_rpc_error(Some(id), -32602, "Invalid tools/list params");
        }
        json_rpc_result(id, json!({"tools": self.registry.tools()}))
    }

    fn handle_tool_call(&self, id: Value, params: Option<Value>) -> Value {
        let Some(params) = params.and_then(|params| params.as_object().cloned()) else {
            return json_rpc_error(Some(id), -32602, "Missing tool call params");
        };
        if params
            .keys()
            .any(|key| !["name", "arguments"].contains(&key.as_str()))
        {
            return json_rpc_error(Some(id), -32602, "Unsupported tool call param");
        }
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return json_rpc_error(Some(id), -32602, "Missing tool name");
        };
        if name.is_empty() || name.len() > MAX_TOOL_NAME_BYTES {
            return json_rpc_error(Some(id), -32602, "Invalid tool name");
        }
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !arguments.is_object() {
            return json_rpc_error(Some(id), -32602, "Tool arguments must be an object");
        }
        let Some(tool_id) = self.registry.resolve(name) else {
            return json_rpc_error(Some(id), -32601, "Tool not found");
        };
        if first_unknown_key(&arguments, allowed_arguments(tool_id)).is_some() {
            return json_rpc_error(Some(id), -32602, "Unsupported argument");
        }

        match tool_id.dispatch_target() {
            LiteDispatchTarget::Config(handler) => self.dispatch_config(id, handler, &arguments),
            LiteDispatchTarget::Literature(handler) => {
                self.dispatch_literature(id, handler, &arguments)
            }
            LiteDispatchTarget::Zotero(handler) => self.dispatch_zotero(id, handler, &arguments),
            LiteDispatchTarget::Orchestration(handler) => {
                self.dispatch_orchestration(id, handler, &arguments)
            }
        }
    }

    fn dispatch_config(&self, id: Value, handler: LiteConfigHandler, arguments: &Value) -> Value {
        match handler {
            LiteConfigHandler::Status => {
                if arguments.get("cwd").is_some_and(|value| !value.is_string()) {
                    return json_rpc_error(Some(id), -32602, "cwd must be a string");
                }
                let Some(access) = self.provider_access() else {
                    return tool_error(id, "native-config-unavailable", CONFIG_UNAVAILABLE);
                };
                tool_result(id, config_status(access))
            }
            LiteConfigHandler::SaveProvider => {
                if let Err(message) = validate_save_provider(arguments) {
                    return json_rpc_error(Some(id), -32602, message);
                }
                tool_error(id, "capability-unavailable", CONFIG_MUTATION_UNAVAILABLE)
            }
            LiteConfigHandler::ConfigureProvider => {
                if let Err(message) = validate_configure_provider(arguments) {
                    return json_rpc_error(Some(id), -32602, message);
                }
                tool_error(id, "capability-unavailable", CONFIG_WIZARD_UNAVAILABLE)
            }
        }
    }

    fn dispatch_literature(
        &self,
        id: Value,
        handler: LiteLiteratureHandler,
        arguments: &Value,
    ) -> Value {
        match handler {
            LiteLiteratureHandler::Status => {
                if let Err(message) = validate_optional_context(arguments) {
                    return json_rpc_error(Some(id), -32602, message);
                }
                let Some(access) = self.provider_access() else {
                    return tool_error(id, "native-config-unavailable", CONFIG_UNAVAILABLE);
                };
                tool_result(id, literature_status(access))
            }
            LiteLiteratureHandler::SearchPlan => {
                let mut input = match SearchPlanInput::from_arguments(arguments, Vec::new()) {
                    Ok(input) => input,
                    Err(error) => {
                        return json_rpc_error(Some(id), -32602, error.to_string());
                    }
                };
                let Some(access) = self.provider_access() else {
                    return tool_error(id, "native-config-unavailable", CONFIG_UNAVAILABLE);
                };
                input.active_providers =
                    active_provider_names(access, PLAN_PROVIDER_ORDER.as_slice());
                tool_result(id, json!(build_search_plan(input)))
            }
            LiteLiteratureHandler::Search => self.literature_search(id, arguments),
            LiteLiteratureHandler::ExportEvidence => {
                let input = match EvidenceInput::from_arguments(arguments) {
                    Ok(input) => input,
                    Err(error) => {
                        return json_rpc_error(Some(id), -32602, error.to_string());
                    }
                };
                tool_result(id, json!(build_evidence_snapshot(input)))
            }
        }
    }

    fn literature_search(&self, id: Value, arguments: &Value) -> Value {
        let request = match SearchRequest::from_arguments(arguments) {
            Ok(request) => request,
            Err(error) => return json_rpc_error(Some(id), -32602, error.to_string()),
        };
        let (access, runtime) = match &self.providers {
            ProviderState::Available(services) => {
                let Some(runtime) = &services.runtime else {
                    return tool_error(
                        id,
                        "provider-runtime-unavailable",
                        PROVIDER_RUNTIME_UNAVAILABLE,
                    );
                };
                (&services.access, runtime)
            }
            ProviderState::ConfigUnavailable => {
                return tool_error(id, "native-config-unavailable", CONFIG_UNAVAILABLE);
            }
        };
        let selected = request.providers();
        let active_providers = SEARCH_PROVIDER_ORDER
            .iter()
            .filter_map(|provider| ProviderId::parse(provider).ok())
            .filter(|provider| selected.is_none_or(|selected| selected.contains(provider)))
            .filter(|provider| access.is_active(*provider))
            .map(|provider| provider.as_str().to_string())
            .collect();
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
            active_providers,
        });
        match crate::providers::search::execute_bounded_search(runtime, &request) {
            Ok(output) => tool_result(
                id,
                json!({
                    "status": output.status,
                    "search_plan": plan,
                    "diagnostics": output.diagnostics,
                    "results": output.results
                }),
            ),
            Err(_) => tool_error(
                id,
                "provider-search-cancelled",
                "provider search was cancelled",
            ),
        }
    }

    fn dispatch_zotero(&self, id: Value, handler: LiteZoteroHandler, arguments: &Value) -> Value {
        match handler {
            LiteZoteroHandler::Status => tool_result(id, json!(ZoteroStatus::disabled())),
            LiteZoteroHandler::ExportImportFiles => {
                let request = match ZoteroExportRequest::from_arguments(arguments) {
                    Ok(request) => request,
                    Err(error) => {
                        return json_rpc_error(Some(id), -32602, error.to_string());
                    }
                };
                match export_selected_import_files(request) {
                    Ok(files) => tool_result(id, json!({"status": "ok", "files": files})),
                    Err(ZoteroExportError::OutputTooLarge | ZoteroExportError::Serialization) => {
                        tool_error(id, "zotero-export-failed", "Zotero export failed")
                    }
                    Err(error) => json_rpc_error(Some(id), -32602, error.to_string()),
                }
            }
        }
    }

    fn dispatch_orchestration(
        &self,
        id: Value,
        handler: LiteOrchestrationHandler,
        arguments: &Value,
    ) -> Value {
        match dispatch_lite_orchestration(handler, arguments) {
            Ok(output) => tool_result(id, json!(output)),
            Err(error) => json_rpc_error(Some(id), -32602, error.to_string()),
        }
    }

    fn provider_access(&self) -> Option<&ProviderAccess> {
        match &self.providers {
            ProviderState::Available(services) => Some(&services.access),
            ProviderState::ConfigUnavailable => None,
        }
    }
}

fn valid_request_id(id: &Value) -> bool {
    id.is_null()
        || id
            .as_str()
            .is_some_and(|value| value.len() <= MAX_REQUEST_ID_BYTES)
        || id.is_number()
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn json_rpc_error(id: Option<Value>, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into()
        }
    })
}

fn tool_result(id: Value, structured_content: Value) -> Value {
    let structured_content = redact_tool_output(structured_content);
    let text = serde_json::to_string_pretty(&structured_content)
        .unwrap_or_else(|_| "{\"status\":\"ok\"}".to_string());
    let response = json_rpc_result(
        id.clone(),
        json!({
            "content": [{"type": "text", "text": text}],
            "structuredContent": structured_content
        }),
    );
    if serde_json::to_vec(&response)
        .is_ok_and(|bytes| bytes.len() <= crate::protocol::MAX_MCP_MESSAGE_BYTES)
    {
        response
    } else {
        tool_error(
            id,
            "tool-output-too-large",
            "tool output exceeds the byte limit",
        )
    }
}

fn tool_error(id: Value, reason_code: &'static str, message: &'static str) -> Value {
    json_rpc_result(
        id,
        json!({
            "isError": true,
            "content": [{"type": "text", "text": message}],
            "structuredContent": {
                "status": "error",
                "error_kind": "tool_error",
                "reason_code": reason_code,
                "message": message
            }
        }),
    )
}

fn first_unknown_key<'a>(arguments: &'a Value, allowed: &[&str]) -> Option<&'a str> {
    arguments
        .as_object()?
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map(String::as_str)
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
        LiteToolId::ZoteroExportImportFiles => &["records", "formats"],
        LiteToolId::OrchestratorRoute => &["request", "platform"],
        LiteToolId::TaskPlan => &["task_id", "paper_type", "topic"],
    }
}

fn validate_optional_context(arguments: &Value) -> Result<(), &'static str> {
    let Some(value) = arguments.get("cwd") else {
        return Ok(());
    };
    let Some(cwd) = value.as_str() else {
        return Err("cwd must be a string");
    };
    if cwd.trim().is_empty() {
        return Err("cwd must not be empty");
    }
    if cwd.chars().count() > MAX_CONTEXT_CHARS {
        return Err("cwd must be at most 4096 characters");
    }
    Ok(())
}

fn validate_save_provider(arguments: &Value) -> Result<(), &'static str> {
    let Some(provider) = arguments.get("provider").and_then(Value::as_str) else {
        return Err("Missing provider");
    };
    let Some(field) = arguments.get("field").and_then(Value::as_str) else {
        return Err("Missing field");
    };
    let Some(value) = arguments.get("value").and_then(Value::as_str) else {
        return Err("Missing value");
    };
    if value.trim().is_empty() {
        return Err("value must not be empty");
    }
    if value.len() > MAX_SECRET_INPUT_BYTES {
        return Err("value exceeds the byte limit");
    }
    if ![
        "openalex",
        "semantic_scholar",
        "semantic-scholar",
        "semanticscholar",
        "s2",
        "crossref",
        "pubmed",
        "ncbi",
    ]
    .contains(&provider)
    {
        return Err("unsupported provider");
    }
    let provider = ProviderId::parse(provider).map_err(|_| "unsupported provider")?;
    let field = ProviderField::parse(field).map_err(|_| "unsupported provider field")?;
    let supported = matches!(
        (provider, field),
        (
            ProviderId::OpenAlex,
            ProviderField::ApiKey | ProviderField::Email
        ) | (ProviderId::SemanticScholar, ProviderField::ApiKey)
            | (ProviderId::Crossref, ProviderField::Email)
            | (ProviderId::PubMed, ProviderField::ApiKey)
    );
    if !supported {
        return Err("unsupported provider field");
    }
    Ok(())
}

fn validate_configure_provider(arguments: &Value) -> Result<(), &'static str> {
    if let Some(provider) = arguments.get("provider") {
        let Some(provider) = provider.as_str() else {
            return Err("provider must be a string");
        };
        if ![
            "openalex",
            "semantic_scholar",
            "semantic-scholar",
            "semanticscholar",
            "s2",
            "crossref",
            "pubmed",
        ]
        .contains(&provider)
        {
            return Err("unsupported provider");
        }
    }
    if let Some(host) = arguments.get("host") {
        let Some(host) = host.as_str() else {
            return Err("host must be a string");
        };
        if !matches!(host, "127.0.0.1" | "localhost") {
            return Err("wizard host must be loopback");
        }
    }
    if let Some(port) = arguments.get("port")
        && port.as_u64().is_none_or(|port| port > u16::MAX as u64)
    {
        return Err("port must be between 0 and 65535");
    }
    Ok(())
}

fn config_status(access: &ProviderAccess) -> Value {
    let projection = provider_projection(access);
    json!({
        "status": "ok",
        "config_path": MANAGED_CONFIG_IDENTIFIER,
        "providers": projection.summary,
        "capability_mode": projection.capability_mode,
        "missing": projection.missing,
        "redacted_config": {"providers": projection.redacted}
    })
}

fn literature_status(access: &ProviderAccess) -> Value {
    let projection = provider_projection(access);
    json!({
        "status": "ok",
        "capability_mode": projection.capability_mode,
        "providers": projection.summary,
        "active_providers": projection.active,
        "missing": projection.missing,
        "provider_capabilities": lite_provider_capabilities(),
        "redacted_config": {"providers": projection.redacted}
    })
}

struct ProviderProjection {
    summary: BTreeMap<String, String>,
    capability_mode: &'static str,
    active: Vec<String>,
    missing: Vec<String>,
    redacted: BTreeMap<String, Value>,
}

fn provider_projection(access: &ProviderAccess) -> ProviderProjection {
    let mut summary = BTreeMap::new();
    let mut active = Vec::new();
    let mut redacted = BTreeMap::new();
    for provider in crate::providers::PROVIDER_ORDER {
        let configured = access.is_active(provider);
        if configured {
            active.push(provider.as_str().to_string());
        }
        summary.insert(
            provider.as_str().to_string(),
            if configured { "configured" } else { "missing" }.to_string(),
        );
        let mut fields = Map::new();
        for field in provider_fields(provider) {
            fields.insert(
                field.as_str().to_string(),
                Value::String(
                    if access.value(provider, *field).is_some() {
                        "configured"
                    } else {
                        "missing"
                    }
                    .to_string(),
                ),
            );
        }
        redacted.insert(
            provider.as_str().to_string(),
            json!({
                "enabled": access.is_enabled(provider),
                "configured": configured,
                "fields": fields
            }),
        );
    }
    let missing = [
        (
            ProviderId::OpenAlex,
            ProviderField::ApiKey,
            "openalex.api_key",
        ),
        (
            ProviderId::SemanticScholar,
            ProviderField::ApiKey,
            "semantic_scholar.api_key",
        ),
        (ProviderId::Crossref, ProviderField::Email, "crossref.email"),
        (ProviderId::PubMed, ProviderField::ApiKey, "pubmed.api_key"),
    ]
    .into_iter()
    .filter(|(provider, field, _)| access.value(*provider, *field).is_none())
    .map(|(_, _, name)| name.to_string())
    .collect();
    ProviderProjection {
        summary,
        capability_mode: if active.is_empty() {
            "strategy_only"
        } else {
            "provider_connected"
        },
        active,
        missing,
        redacted,
    }
}

fn provider_fields(provider: ProviderId) -> &'static [ProviderField] {
    match provider {
        ProviderId::OpenAlex => &[ProviderField::ApiKey, ProviderField::Email],
        ProviderId::SemanticScholar | ProviderId::PubMed => &[ProviderField::ApiKey],
        ProviderId::Crossref => &[ProviderField::Email],
        ProviderId::Arxiv => &[],
    }
}

fn active_provider_names(access: &ProviderAccess, order: &[&str]) -> Vec<String> {
    order
        .iter()
        .filter_map(|provider| ProviderId::parse(provider).ok())
        .filter(|provider| access.is_active(*provider))
        .map(|provider| provider.as_str().to_string())
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
