use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};

use qiongli_content::EmbeddedContent;
use qiongli_execution::{
    HOST_CANDIDATE_SCHEMA_VERSION, HOST_HANDOFF_SCHEMA_VERSION, HostCandidateEnvelopeV1,
    HostEvidenceReferenceV1, HostRuntimeDescriptorV1, OrchestrationExecutionMode, RunId,
    ToolCallId, ToolId,
};
use qiongli_project::ProjectStateService;
use qiongli_runtime::mcp::LiteMcpServer;
use qiongli_runtime::protocol::{read_message, write_message};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::zotero::companion::CompanionClient;
use qiongli_runtime::{
    FullProjectService, FullProjectServiceErrorKind, FullProjectToolId, FullProjectToolRegistry,
    LiteToolRegistry, RuntimeError, RuntimeErrorCode,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::command::{CommandEnvironment, config_root, config_store};
use crate::credential_store::native_secret_store;
use crate::orchestration_control::{
    FullOrchestrationService, OrchestrationControlAction, OrchestrationRunReference,
    WorkerOrchestrationControlAction, WorkerOrchestrationRunReference,
};

const MAX_HOST_EVIDENCE_RECORDS: usize = 128;
pub const FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES: [&str; 9] = [
    "qiongli_orchestration_doctor",
    "qiongli_orchestration_start",
    "qiongli_orchestration_next",
    "qiongli_orchestration_read",
    "qiongli_orchestration_submit",
    "qiongli_orchestration_runs",
    "qiongli_orchestration_action",
    "qiongli_worker_orchestration_runs",
    "qiongli_worker_orchestration_action",
];

#[derive(Clone)]
struct HostEvidenceLedgerRecord {
    project_id: qiongli_project::ProjectId,
    expected_project_revision: u64,
    run_id: RunId,
    handoff_sha256: String,
    evidence: HostEvidenceReferenceV1,
}

#[derive(Default)]
struct HostEvidenceLedger {
    records: Vec<HostEvidenceLedgerRecord>,
}

#[derive(Clone, Debug)]
struct McpClientInfo {
    name: String,
    version: String,
}

pub fn serve_lite_mcp<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), RuntimeError> {
    lite_server(environment, content)?.serve(reader, writer)
}

pub fn serve_full_mcp<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), RuntimeError> {
    let projects = config_root(environment).ok().map(ProjectStateService::new);
    let registry = FullProjectToolRegistry::from_embedded_content(content)?;
    let orchestration = projects
        .as_ref()
        .map(|projects| {
            FullOrchestrationService::from_embedded_content(
                projects.clone(),
                registry.clone(),
                content,
            )
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::InvalidFullProjectContract))
        })
        .transpose()?;
    FullMcpServer::new(
        lite_server(environment, content)?,
        registry,
        projects,
        orchestration,
    )
    .serve(reader, writer)
}

fn lite_server(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<LiteMcpServer, RuntimeError> {
    let registry = LiteToolRegistry::from_embedded_content(content)?;
    let mut server = match config_store(environment).and_then(|store| store.load()) {
        Ok(loaded) => {
            let settings = Arc::new(loaded.settings);
            let preview = ProviderAccess::from_global_settings_metadata(settings.as_ref());
            let secret_store = native_secret_store();
            LiteMcpServer::production_deferred(
                "qiongli",
                env!("CARGO_PKG_VERSION"),
                registry,
                preview,
                Arc::new(move || {
                    ProviderAccess::from_global_settings(settings.as_ref(), secret_store.as_ref())
                }),
            )
        }
        Err(_) => LiteMcpServer::config_unavailable("qiongli", env!("CARGO_PKG_VERSION"), registry),
    };
    if let Some(url) = environment.zotero_connector_url() {
        let client = CompanionClient::new(url)
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::InvalidLiteContract))?;
        server = server.with_zotero_client(client);
    }
    Ok(server)
}

#[derive(Clone)]
pub struct FullMcpServer {
    lite: LiteMcpServer,
    registry: FullProjectToolRegistry,
    projects: Option<FullProjectService>,
    orchestration: Option<FullOrchestrationService>,
    evidence: Arc<Mutex<HostEvidenceLedger>>,
    client_info: Arc<Mutex<Option<McpClientInfo>>>,
}

impl FullMcpServer {
    #[must_use]
    pub fn new(
        lite: LiteMcpServer,
        registry: FullProjectToolRegistry,
        projects: Option<ProjectStateService>,
        orchestration: Option<FullOrchestrationService>,
    ) -> Self {
        Self {
            lite,
            registry,
            projects: projects.map(FullProjectService::new),
            orchestration,
            evidence: Arc::new(Mutex::new(HostEvidenceLedger::default())),
            client_info: Arc::new(Mutex::new(None)),
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
        match request.get("method").and_then(Value::as_str) {
            Some("initialize") => self.handle_initialize(request),
            Some("tools/list") => self.list_tools(request),
            Some("tools/call") => self.handle_tool_call(request),
            _ => self.lite.handle(request),
        }
    }

    fn handle_initialize(&self, request: Value) -> Option<Value> {
        if let (Some(name), Some(version)) = (
            request
                .pointer("/params/clientInfo/name")
                .and_then(Value::as_str),
            request
                .pointer("/params/clientInfo/version")
                .and_then(Value::as_str),
        ) && valid_client_info_token(name)
            && valid_client_info_token(version)
            && let Ok(mut client_info) = self.client_info.lock()
        {
            *client_info = Some(McpClientInfo {
                name: name.to_owned(),
                version: version.to_owned(),
            });
        }
        self.lite.handle(request)
    }

    fn list_tools(&self, request: Value) -> Option<Value> {
        let mut response = self.lite.handle(request)?;
        let tools = response
            .pointer_mut("/result/tools")
            .and_then(Value::as_array_mut);
        if let Some(tools) = tools {
            tools.extend(self.registry.tools().iter().map(|tool| json!(tool)));
            tools.extend(orchestration_control_tools());
            debug_assert_eq!(
                tools
                    .iter()
                    .rev()
                    .take(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES.len())
                    .rev()
                    .filter_map(|tool| tool.get("name").and_then(Value::as_str))
                    .collect::<Vec<_>>(),
                FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES
            );
        }
        Some(response)
    }

    fn handle_tool_call(&self, request: Value) -> Option<Value> {
        let requested_name = request.pointer("/params/name").and_then(Value::as_str);
        let project_tool = requested_name.and_then(|name| self.registry.resolve(name));
        let orchestration_tool = requested_name.and_then(OrchestrationControlTool::resolve);
        if project_tool.is_none() && orchestration_tool.is_none() {
            return self.lite.handle(request);
        }
        let validation = self.lite.handle(request.clone())?;
        if validation.pointer("/error/code").and_then(Value::as_i64) != Some(-32601) {
            return Some(validation);
        }
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let arguments = request
            .pointer("/params/arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(tool) = project_tool {
            Some(self.dispatch_project(id, tool, &arguments))
        } else {
            Some(self.dispatch_orchestration(
                id,
                orchestration_tool.expect("validated orchestration tool remains present"),
                &arguments,
            ))
        }
    }

    fn dispatch_project(&self, id: Value, tool: FullProjectToolId, arguments: &Value) -> Value {
        let Some(projects) = self.projects.as_ref() else {
            return tool_error(
                id,
                "project-service-unavailable",
                "native Research Library is unavailable",
            );
        };
        match projects.dispatch(tool, arguments) {
            Ok(result) => tool_result(id, result),
            Err(error) if error.kind() == FullProjectServiceErrorKind::InvalidArguments => {
                json_rpc_error(Some(id), -32602, error.public_message())
            }
            Err(error) => tool_error(id, error.reason_code(), error.public_message()),
        }
    }

    fn dispatch_orchestration(
        &self,
        id: Value,
        tool: OrchestrationControlTool,
        arguments: &Value,
    ) -> Value {
        let Some(service) = self.orchestration.as_ref() else {
            return tool_error(
                id,
                "orchestration-service-unavailable",
                "native orchestration service is unavailable",
            );
        };
        let Some(arguments) = arguments.as_object() else {
            return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
        };
        if matches!(tool, OrchestrationControlTool::EvidenceRead) {
            return self.dispatch_host_evidence_read(id, service, arguments);
        }
        let result = match tool {
            OrchestrationControlTool::Doctor => {
                let (project_id, revision, host) = match parse_host_doctor(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .doctor_host(&project_id, revision, host)
                    .and_then(serialize_orchestration)
                    .map(|value| self.attach_client_info(value))
            }
            OrchestrationControlTool::Start => {
                let (project_id, revision, execution_mode, host) = match parse_host_start(arguments)
                {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .start_host(project_id, revision, execution_mode, host)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Next => {
                let (reference, host) = match parse_host_next(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .next_host(&reference, host)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Submit => {
                let (reference, host, candidate) = match parse_host_submit(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                let evidence = match self.authenticated_evidence(
                    &reference,
                    &candidate.handoff_sha256,
                    &candidate.evidence,
                ) {
                    Ok(evidence) => evidence,
                    Err(reason_code) => {
                        return tool_error(id, reason_code, "host evidence ledger is unavailable");
                    }
                };
                match service.submit_host(&reference, host, &candidate, &evidence) {
                    Ok(view) => {
                        if self
                            .consume_evidence(
                                &reference,
                                &candidate.handoff_sha256,
                                &candidate.evidence,
                            )
                            .is_err()
                        {
                            return tool_error(
                                id,
                                "host-evidence-ledger-unavailable",
                                "host evidence ledger is unavailable",
                            );
                        }
                        serialize_orchestration(view)
                    }
                    Err(error) => Err(error),
                }
            }
            OrchestrationControlTool::Runs => {
                let (project_id, revision) = match parse_project_reference(arguments) {
                    Ok(reference) => reference,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .list_runs(&project_id, revision)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Action => {
                let (reference, action) = match parse_orchestration_action(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .control(&reference, action)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::WorkerRuns => {
                let (project_id, revision) = match parse_project_reference(arguments) {
                    Ok(reference) => reference,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .list_worker_runs(&project_id, revision)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::WorkerAction => {
                let (reference, action) = match parse_worker_orchestration_action(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .control_worker(&reference, action)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::EvidenceRead => unreachable!("handled before dispatch"),
        };
        match result {
            Ok(value) => tool_result(id, value),
            Err(error) => tool_error(id, error.reason_code(), "orchestration operation failed"),
        }
    }

    fn dispatch_host_evidence_read(
        &self,
        id: Value,
        service: &FullOrchestrationService,
        arguments: &serde_json::Map<String, Value>,
    ) -> Value {
        let (reference, host, handoff_sha256, tool_name, tool_arguments) =
            match parse_host_evidence_read(arguments) {
                Ok(request) => request,
                Err(()) => {
                    return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                }
            };
        let handoff = match service.current_host_handoff(&reference, host) {
            Ok(handoff) => handoff,
            Err(error) => {
                return tool_error(id, error.reason_code(), "orchestration operation failed");
            }
        };
        if handoff.digest().ok().as_deref() != Some(&handoff_sha256) {
            return tool_error(
                id,
                "host-handoff-reference-stale",
                "orchestration operation failed",
            );
        }
        let Some(project_tool) = self.registry.resolve(&tool_name) else {
            return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
        };
        if !project_tool.is_read_only()
            || !handoff
                .allowed_tool_ids
                .iter()
                .any(|tool_id| tool_id.as_str() == tool_name)
            || !host_tool_arguments_match_scope(project_tool, &reference, &tool_arguments)
        {
            return tool_error(
                id,
                "host-handoff-tool-not-allowed",
                "orchestration operation failed",
            );
        }
        let Some(projects) = self.projects.as_ref() else {
            return tool_error(
                id,
                "project-service-unavailable",
                "native Research Library is unavailable",
            );
        };
        let result = match projects.dispatch(project_tool, &tool_arguments) {
            Ok(result) => result,
            Err(error) if error.kind() == FullProjectServiceErrorKind::InvalidArguments => {
                return json_rpc_error(Some(id), -32602, error.public_message());
            }
            Err(error) => {
                return tool_error(id, error.reason_code(), error.public_message());
            }
        };
        let evidence = match self.record_evidence(
            &reference,
            &handoff_sha256,
            &tool_name,
            &tool_arguments,
            &result,
        ) {
            Ok(evidence) => evidence,
            Err(reason_code) => {
                return tool_error(id, reason_code, "host evidence ledger is unavailable");
            }
        };
        let visible_result = attach_host_evidence(result, &evidence, &handoff_sha256);
        tool_result_with_meta(
            id,
            visible_result,
            json!({"qiongli/evidence": evidence, "qiongli/handoffSha256": handoff_sha256}),
        )
    }

    fn attach_client_info(&self, mut value: Value) -> Value {
        let client_info = self
            .client_info
            .lock()
            .ok()
            .and_then(|client_info| client_info.clone());
        if let Some(client_info) = client_info {
            value["mcpClientInfo"] = json!({
                "name": client_info.name,
                "version": client_info.version,
                "trust": "display-only"
            });
        }
        value
    }

    fn record_evidence(
        &self,
        reference: &OrchestrationRunReference,
        handoff_sha256: &str,
        tool_name: &str,
        arguments: &Value,
        result: &Value,
    ) -> Result<HostEvidenceReferenceV1, &'static str> {
        let request_sha256 = canonical_sha256(&json!({
            "handoffSha256": handoff_sha256,
            "toolName": tool_name,
            "arguments": arguments
        }))?;
        let decision_sha256 = canonical_sha256(&json!({
            "outcome": "allow",
            "reasonCode": "host-handoff-project-read",
            "projectId": reference.project_id,
            "expectedProjectRevision": reference.expected_project_revision,
            "runId": reference.run_id,
            "handoffSha256": handoff_sha256
        }))?;
        let result_sha256 = canonical_sha256(result)?;
        let call_id = new_evidence_call_id()?;
        let evidence = HostEvidenceReferenceV1::try_new(
            reference.run_id.clone(),
            call_id,
            ToolId::parse(tool_name.to_owned()).map_err(|_| "host-evidence-ledger-unavailable")?,
            request_sha256,
            decision_sha256,
            result_sha256,
        )
        .map_err(|_| "host-evidence-ledger-unavailable")?;
        let mut ledger = self
            .evidence
            .lock()
            .map_err(|_| "host-evidence-ledger-unavailable")?;
        if ledger.records.len() >= MAX_HOST_EVIDENCE_RECORDS {
            ledger.records.remove(0);
        }
        ledger.records.push(HostEvidenceLedgerRecord {
            project_id: reference.project_id.clone(),
            expected_project_revision: reference.expected_project_revision,
            run_id: reference.run_id.clone(),
            handoff_sha256: handoff_sha256.to_owned(),
            evidence: evidence.clone(),
        });
        Ok(evidence)
    }

    fn authenticated_evidence(
        &self,
        reference: &OrchestrationRunReference,
        handoff_sha256: &str,
        requested: &[HostEvidenceReferenceV1],
    ) -> Result<Vec<HostEvidenceReferenceV1>, &'static str> {
        let ledger = self
            .evidence
            .lock()
            .map_err(|_| "host-evidence-ledger-unavailable")?;
        Ok(requested
            .iter()
            .filter(|evidence| {
                ledger.records.iter().any(|record| {
                    record.project_id == reference.project_id
                        && record.expected_project_revision == reference.expected_project_revision
                        && record.run_id == reference.run_id
                        && record.handoff_sha256 == handoff_sha256
                        && &record.evidence == *evidence
                })
            })
            .cloned()
            .collect())
    }

    fn consume_evidence(
        &self,
        reference: &OrchestrationRunReference,
        handoff_sha256: &str,
        consumed: &[HostEvidenceReferenceV1],
    ) -> Result<(), ()> {
        let mut ledger = self.evidence.lock().map_err(|_| ())?;
        ledger.records.retain(|record| {
            !(record.project_id == reference.project_id
                && record.expected_project_revision == reference.expected_project_revision
                && record.run_id == reference.run_id
                && record.handoff_sha256 == handoff_sha256
                && consumed.contains(&record.evidence))
        });
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum OrchestrationControlTool {
    Doctor,
    Start,
    Next,
    Submit,
    EvidenceRead,
    Runs,
    Action,
    WorkerRuns,
    WorkerAction,
}

impl OrchestrationControlTool {
    fn resolve(name: &str) -> Option<Self> {
        match name {
            "qiongli_orchestration_doctor" => Some(Self::Doctor),
            "qiongli_orchestration_start" => Some(Self::Start),
            "qiongli_orchestration_next" => Some(Self::Next),
            "qiongli_orchestration_submit" => Some(Self::Submit),
            "qiongli_orchestration_read" => Some(Self::EvidenceRead),
            "qiongli_orchestration_runs" => Some(Self::Runs),
            "qiongli_orchestration_action" => Some(Self::Action),
            "qiongli_worker_orchestration_runs" => Some(Self::WorkerRuns),
            "qiongli_worker_orchestration_action" => Some(Self::WorkerAction),
            _ => None,
        }
    }
}

fn orchestration_control_tools() -> impl Iterator<Item = Value> {
    let project_properties = json!({
        "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
        "expectedProjectRevision": {"type": "integer", "minimum": 1}
    });
    vec![
        json!({
            "name": "qiongli_orchestration_doctor",
            "description": "Verify the registered project, embedded workflow, and explicit host/plugin activation state without contacting a model provider.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "host": host_runtime_schema()
                },
                "required": ["projectId", "expectedProjectRevision", "host"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_start",
            "description": "Create a host-bound local checkpoint and return the first bounded handoff; model execution remains in the calling host.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "executionMode": {"type": "string", "enum": ["solo", "duo", "triad"]},
                    "host": host_runtime_schema()
                },
                "required": ["projectId", "expectedProjectRevision", "executionMode", "host"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_next",
            "description": "Return or reissue the current host-bound handoff using an exact checkpoint generation and document digest.",
            "inputSchema": host_run_input_schema(false),
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_read",
            "description": "Run one handoff-authorized project read and return an authenticated evidence reference for candidate submission in structuredContent.qiongliOrchestration.evidence and MCP _meta.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
                    "expectedGeneration": {"type": "integer", "minimum": 0},
                    "expectedDocumentSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                    "host": host_runtime_schema(),
                    "handoffSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                    "toolName": {
                        "type": "string",
                        "enum": [
                            "qiongli_project_read",
                            "qiongli_project_graph_snapshot",
                            "qiongli_project_graph_query",
                            "qiongli_project_artifact_changes",
                            "qiongli_project_capture_coverage",
                            "qiongli_project_capture_preview"
                        ]
                    },
                    "toolArguments": {"type": "object"}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "runId",
                    "expectedGeneration",
                    "expectedDocumentSha256",
                    "host",
                    "handoffSha256",
                    "toolName",
                    "toolArguments"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_submit",
            "description": "Validate one host-produced candidate and authenticated project-read evidence, persist only its digest, and return the next handoff.",
            "inputSchema": host_run_input_schema(true),
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_runs",
            "description": "List redacted revision-bound host-orchestration checkpoints and their available local recovery actions.",
            "inputSchema": {
                "type": "object",
                "properties": project_properties.clone(),
                "required": ["projectId", "expectedProjectRevision"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_action",
            "description": "Pause, explicitly recover, resume, or terminally cancel an unchanged host-orchestration checkpoint without executing a model.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
                    "expectedGeneration": {"type": "integer", "minimum": 0},
                    "expectedDocumentSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                    "action": {"type": "string", "enum": ["pause", "recover", "resume", "cancel"]}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "runId",
                    "expectedGeneration",
                    "expectedDocumentSha256",
                    "action"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_worker_orchestration_runs",
            "description": "List redacted revision-bound worker handoff, barrier, synthesis, and review checkpoints.",
            "inputSchema": {
                "type": "object",
                "properties": project_properties,
                "required": ["projectId", "expectedProjectRevision"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_worker_orchestration_action",
            "description": "Explicitly recover a hash-only interrupted worker handoff or terminally cancel an unchanged checkpoint without executing a model.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
                    "expectedGeneration": {"type": "integer", "minimum": 0},
                    "expectedDocumentSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                    "action": {"type": "string", "enum": ["recover", "cancel"]}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "runId",
                    "expectedGeneration",
                    "expectedDocumentSha256",
                    "action"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
    ]
    .into_iter()
}

fn host_runtime_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "schemaVersion": {"type": "integer", "const": HOST_HANDOFF_SCHEMA_VERSION},
            "family": {
                "type": "string",
                "enum": ["codex", "claude-code", "claude-desktop", "other-local"]
            },
            "hostVersion": {"type": "string", "minLength": 1, "maxLength": 64},
            "adapterVersion": {"type": "string", "minLength": 1, "maxLength": 64},
            "fullMcpProtocol": {"type": "string", "const": "qiongli-full-mcp/1"},
            "capabilities": {
                "type": "array",
                "minItems": 1,
                "maxItems": 4,
                "uniqueItems": true,
                "items": {
                    "type": "string",
                    "enum": ["single-agent", "native-subagents", "attachments", "structured-output"]
                }
            },
            "pluginState": {"type": "string", "enum": host_component_states()},
            "registrationState": {"type": "string", "enum": host_component_states()},
            "enablementState": {"type": "string", "enum": host_component_states()},
            "trustState": {"type": "string", "enum": host_component_states()},
            "activationState": {"type": "string", "enum": host_component_states()}
        },
        "required": [
            "schemaVersion",
            "family",
            "hostVersion",
            "adapterVersion",
            "fullMcpProtocol",
            "capabilities",
            "pluginState",
            "registrationState",
            "enablementState",
            "trustState",
            "activationState"
        ],
        "additionalProperties": false
    })
}

fn host_component_states() -> Value {
    json!([
        "missing",
        "present",
        "host-action-required",
        "ready",
        "unsupported",
        "unknown"
    ])
}

fn host_run_input_schema(with_candidate: bool) -> Value {
    let mut properties = json!({
        "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
        "expectedProjectRevision": {"type": "integer", "minimum": 1},
        "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
        "expectedGeneration": {"type": "integer", "minimum": 0},
        "expectedDocumentSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
        "host": host_runtime_schema()
    });
    let mut required = vec![
        "projectId",
        "expectedProjectRevision",
        "runId",
        "expectedGeneration",
        "expectedDocumentSha256",
        "host",
    ];
    if with_candidate {
        properties["candidate"] = host_candidate_schema();
        required.push("candidate");
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn host_candidate_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "schemaVersion": {"type": "integer", "const": HOST_CANDIDATE_SCHEMA_VERSION},
            "handoffSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
            "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
            "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
            "expectedProjectRevision": {"type": "integer", "minimum": 1},
            "taskId": {"type": "string", "minLength": 1, "maxLength": 64},
            "role": {"type": "string", "enum": ["primary", "reviewer", "verifier"]},
            "attempt": {"type": "integer", "minimum": 1, "maximum": 3},
            "candidateKind": {
                "type": "string",
                "enum": ["research-task", "review", "verification", "worker", "synthesis"]
            },
            "content": {"type": "string", "minLength": 1, "maxLength": 65536},
            "evidence": {
                "type": "array",
                "maxItems": 32,
                "items": {
                    "type": "object",
                    "properties": {
                        "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
                        "callId": {"type": "string", "pattern": "^call_[0-9a-f]{32}$"},
                        "toolId": {"type": "string", "minLength": 1, "maxLength": 96},
                        "requestSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                        "decisionSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                        "resultSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"}
                    },
                    "required": [
                        "runId",
                        "callId",
                        "toolId",
                        "requestSha256",
                        "decisionSha256",
                        "resultSha256"
                    ],
                    "additionalProperties": false
                }
            },
            "knownFactDigests": {
                "type": "array",
                "minItems": 1,
                "maxItems": 32,
                "uniqueItems": true,
                "items": {"type": "string", "pattern": "^[0-9a-f]{64}$"}
            },
            "reviewResult": {
                "type": "string",
                "enum": ["not-applicable", "pass", "changes-requested", "blocked"]
            },
            "conflicts": {
                "type": "array",
                "maxItems": 16,
                "items": {"type": "string", "minLength": 1, "maxLength": 1024}
            },
            "evidenceGaps": {
                "type": "array",
                "maxItems": 16,
                "items": {"type": "string", "minLength": 1, "maxLength": 1024}
            }
        },
        "required": [
            "schemaVersion",
            "handoffSha256",
            "runId",
            "projectId",
            "expectedProjectRevision",
            "taskId",
            "role",
            "attempt",
            "candidateKind",
            "content",
            "evidence",
            "knownFactDigests",
            "reviewResult",
            "conflicts",
            "evidenceGaps"
        ],
        "additionalProperties": false
    })
}

fn parse_project_reference(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(qiongli_project::ProjectId, u64), ()> {
    if arguments.len() != 2 {
        return Err(());
    }
    let project_id = arguments
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or(())?;
    let revision = arguments
        .get("expectedProjectRevision")
        .and_then(Value::as_u64)
        .filter(|revision| *revision > 0)
        .ok_or(())?;
    Ok((
        qiongli_project::ProjectId::parse(project_id.to_owned()).map_err(|_| ())?,
        revision,
    ))
}

fn parse_run_reference(
    arguments: &serde_json::Map<String, Value>,
) -> Result<OrchestrationRunReference, ()> {
    if arguments.len() != 5 {
        return Err(());
    }
    let project_id = arguments
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or(())?;
    let revision = arguments
        .get("expectedProjectRevision")
        .and_then(Value::as_u64)
        .filter(|revision| *revision > 0)
        .ok_or(())?;
    let run_id = arguments.get("runId").and_then(Value::as_str).ok_or(())?;
    let generation = arguments
        .get("expectedGeneration")
        .and_then(Value::as_u64)
        .ok_or(())?;
    let document_sha256 = arguments
        .get("expectedDocumentSha256")
        .and_then(Value::as_str)
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or(())?;
    Ok(OrchestrationRunReference {
        project_id: qiongli_project::ProjectId::parse(project_id.to_owned()).map_err(|_| ())?,
        expected_project_revision: revision,
        run_id: RunId::parse(run_id.to_owned()).map_err(|_| ())?,
        expected_generation: generation,
        expected_document_sha256: document_sha256.to_owned(),
    })
}

fn parse_host(value: Value) -> Result<HostRuntimeDescriptorV1, ()> {
    serde_json::from_value(value).map_err(|_| ())
}

fn parse_execution_mode(value: Option<&Value>) -> Result<OrchestrationExecutionMode, ()> {
    match value.and_then(Value::as_str) {
        Some("solo") => Ok(OrchestrationExecutionMode::Solo),
        Some("duo") => Ok(OrchestrationExecutionMode::Duo),
        Some("triad") => Ok(OrchestrationExecutionMode::Triad),
        _ => Err(()),
    }
}

fn parse_host_doctor(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(qiongli_project::ProjectId, u64, HostRuntimeDescriptorV1), ()> {
    if arguments.len() != 3 {
        return Err(());
    }
    let mut project_arguments = arguments.clone();
    let host = parse_host(project_arguments.remove("host").ok_or(())?)?;
    let (project_id, revision) = parse_project_reference(&project_arguments)?;
    Ok((project_id, revision, host))
}

fn parse_host_start(
    arguments: &serde_json::Map<String, Value>,
) -> Result<
    (
        qiongli_project::ProjectId,
        u64,
        OrchestrationExecutionMode,
        HostRuntimeDescriptorV1,
    ),
    (),
> {
    if arguments.len() != 4 {
        return Err(());
    }
    let mut project_arguments = arguments.clone();
    let host = parse_host(project_arguments.remove("host").ok_or(())?)?;
    let execution_mode = parse_execution_mode(project_arguments.get("executionMode"))?;
    project_arguments.remove("executionMode");
    let (project_id, revision) = parse_project_reference(&project_arguments)?;
    Ok((project_id, revision, execution_mode, host))
}

fn parse_host_next(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(OrchestrationRunReference, HostRuntimeDescriptorV1), ()> {
    if arguments.len() != 6 {
        return Err(());
    }
    let mut reference_arguments = arguments.clone();
    let host = parse_host(reference_arguments.remove("host").ok_or(())?)?;
    Ok((parse_run_reference(&reference_arguments)?, host))
}

fn parse_host_submit(
    arguments: &serde_json::Map<String, Value>,
) -> Result<
    (
        OrchestrationRunReference,
        HostRuntimeDescriptorV1,
        HostCandidateEnvelopeV1,
    ),
    (),
> {
    if arguments.len() != 7 {
        return Err(());
    }
    let mut reference_arguments = arguments.clone();
    let host = parse_host(reference_arguments.remove("host").ok_or(())?)?;
    let candidate = serde_json::from_value(reference_arguments.remove("candidate").ok_or(())?)
        .map_err(|_| ())?;
    Ok((parse_run_reference(&reference_arguments)?, host, candidate))
}

type HostEvidenceReadRequest = (
    OrchestrationRunReference,
    HostRuntimeDescriptorV1,
    String,
    String,
    Value,
);

fn parse_host_evidence_read(
    arguments: &serde_json::Map<String, Value>,
) -> Result<HostEvidenceReadRequest, ()> {
    if arguments.len() != 9 {
        return Err(());
    }
    let mut reference_arguments = arguments.clone();
    let host = parse_host(reference_arguments.remove("host").ok_or(())?)?;
    let handoff_sha256 = reference_arguments
        .remove("handoffSha256")
        .and_then(|value| value.as_str().map(str::to_owned))
        .filter(|digest| valid_sha256(digest))
        .ok_or(())?;
    let tool_name = reference_arguments
        .remove("toolName")
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(())?;
    let tool_arguments = reference_arguments.remove("toolArguments").ok_or(())?;
    if !tool_arguments.is_object() {
        return Err(());
    }
    Ok((
        parse_run_reference(&reference_arguments)?,
        host,
        handoff_sha256,
        tool_name,
        tool_arguments,
    ))
}

fn host_tool_arguments_match_scope(
    tool: FullProjectToolId,
    reference: &OrchestrationRunReference,
    arguments: &Value,
) -> bool {
    match tool {
        FullProjectToolId::Read
        | FullProjectToolId::GraphSnapshot
        | FullProjectToolId::GraphQuery
        | FullProjectToolId::ArtifactChanges
        | FullProjectToolId::CaptureCoverage => {
            arguments.get("project_id").and_then(Value::as_str)
                == Some(reference.project_id.as_str())
        }
        FullProjectToolId::CapturePreview => {
            arguments
                .pointer("/capture/binding/project_id")
                .and_then(Value::as_str)
                == Some(reference.project_id.as_str())
                && arguments
                    .pointer("/capture/binding/base_revision")
                    .and_then(Value::as_u64)
                    == Some(reference.expected_project_revision)
        }
        FullProjectToolId::List
        | FullProjectToolId::GraphPortfolio
        | FullProjectToolId::CaptureApply => false,
    }
}

fn valid_sha256(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_client_info_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| !character.is_control() && character != '<' && character != '>')
}

fn parse_orchestration_action(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(OrchestrationRunReference, OrchestrationControlAction), ()> {
    if arguments.len() != 6 {
        return Err(());
    }
    let mut reference_arguments = arguments.clone();
    let action = match reference_arguments
        .remove("action")
        .and_then(|value| value.as_str().map(str::to_owned))
        .as_deref()
    {
        Some("pause") => OrchestrationControlAction::Pause,
        Some("recover") => OrchestrationControlAction::Recover,
        Some("resume") => OrchestrationControlAction::Resume,
        Some("cancel") => OrchestrationControlAction::Cancel,
        _ => return Err(()),
    };
    Ok((parse_run_reference(&reference_arguments)?, action))
}

fn parse_worker_run_reference(
    arguments: &serde_json::Map<String, Value>,
) -> Result<WorkerOrchestrationRunReference, ()> {
    let reference = parse_run_reference(arguments)?;
    Ok(WorkerOrchestrationRunReference {
        project_id: reference.project_id,
        expected_project_revision: reference.expected_project_revision,
        run_id: reference.run_id,
        expected_generation: reference.expected_generation,
        expected_document_sha256: reference.expected_document_sha256,
    })
}

fn parse_worker_orchestration_action(
    arguments: &serde_json::Map<String, Value>,
) -> Result<
    (
        WorkerOrchestrationRunReference,
        WorkerOrchestrationControlAction,
    ),
    (),
> {
    if arguments.len() != 6 {
        return Err(());
    }
    let mut reference_arguments = arguments.clone();
    let action = match reference_arguments
        .remove("action")
        .and_then(|value| value.as_str().map(str::to_owned))
        .as_deref()
    {
        Some("recover") => WorkerOrchestrationControlAction::Recover,
        Some("cancel") => WorkerOrchestrationControlAction::Cancel,
        _ => return Err(()),
    };
    Ok((parse_worker_run_reference(&reference_arguments)?, action))
}

fn serialize_orchestration<T: serde::Serialize>(
    value: T,
) -> Result<Value, crate::orchestration_control::FullOrchestrationError> {
    serde_json::to_value(value).map_err(|_| {
        crate::orchestration_control::FullOrchestrationError::new(
            "orchestration-output-serialization-failed",
        )
    })
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn json_rpc_error(id: Option<Value>, code: i64, message: &'static str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}

fn tool_result(id: Value, structured_content: Value) -> Value {
    tool_result_with_optional_meta(id, structured_content, None)
}

fn tool_result_with_meta(id: Value, structured_content: Value, metadata: Value) -> Value {
    tool_result_with_optional_meta(id, structured_content, Some(metadata))
}

fn tool_result_with_optional_meta(
    id: Value,
    structured_content: Value,
    metadata: Option<Value>,
) -> Value {
    let text = serde_json::to_string_pretty(&structured_content)
        .unwrap_or_else(|_| "{\"status\":\"ok\"}".to_string());
    let mut result = json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured_content
    });
    if let Some(metadata) = metadata {
        result["_meta"] = metadata;
    }
    let response = json_rpc_result(id.clone(), result);
    if serde_json::to_vec(&response)
        .is_ok_and(|bytes| bytes.len() <= qiongli_runtime::protocol::MAX_MCP_MESSAGE_BYTES)
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

fn attach_host_evidence(
    mut project_result: Value,
    evidence: &HostEvidenceReferenceV1,
    handoff_sha256: &str,
) -> Value {
    let orchestration = json!({
        "evidence": evidence,
        "handoffSha256": handoff_sha256,
    });
    if let Some(object) = project_result.as_object_mut() {
        object.insert("qiongliOrchestration".to_owned(), orchestration);
        project_result
    } else {
        json!({
            "projectResult": project_result,
            "qiongliOrchestration": orchestration,
        })
    }
}

fn canonical_sha256(value: &Value) -> Result<String, &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| "host-evidence-ledger-unavailable")?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn new_evidence_call_id() -> Result<ToolCallId, &'static str> {
    let mut identifier = [0_u8; 16];
    getrandom::fill(&mut identifier).map_err(|_| "host-evidence-ledger-unavailable")?;
    let mut value = String::with_capacity(37);
    value.push_str("call_");
    for byte in identifier {
        use std::fmt::Write as _;
        write!(value, "{byte:02x}").map_err(|_| "host-evidence-ledger-unavailable")?;
    }
    ToolCallId::parse(value).map_err(|_| "host-evidence-ledger-unavailable")
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
