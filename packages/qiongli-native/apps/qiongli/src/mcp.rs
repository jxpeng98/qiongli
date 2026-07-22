use std::io::{BufRead, Write};

use qiongli_content::EmbeddedContent;
use qiongli_project::{
    AcademicGraphDirection, AcademicGraphIndexService, AcademicGraphLayer, AcademicGraphNodeType,
    AcademicGraphQueryV1, AcademicGraphRelation, AcademicGraphService, ApprovedCaptureIntake,
    CaptureDelivery, ProjectId, ProjectStateService, ResearchCaptureV1,
};
use qiongli_runtime::mcp::LiteMcpServer;
use qiongli_runtime::protocol::{read_message, write_message};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{FullProjectToolId, FullProjectToolRegistry, LiteToolRegistry, RuntimeError};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::command::{CommandEnvironment, config_root, config_store};
use crate::credential_store::native_secret_store;

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
    FullMcpServer::new(
        lite_server(environment, content)?,
        FullProjectToolRegistry::from_embedded_content(content)?,
        projects,
    )
    .serve(reader, writer)
}

fn lite_server(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<LiteMcpServer, RuntimeError> {
    let registry = LiteToolRegistry::from_embedded_content(content)?;
    let server = match config_store(environment).and_then(|store| store.load()) {
        Ok(loaded) => {
            let secret_store = native_secret_store();
            let access =
                ProviderAccess::from_global_settings(&loaded.settings, secret_store.as_ref());
            LiteMcpServer::production("qiongli", env!("CARGO_PKG_VERSION"), registry, access)
        }
        Err(_) => LiteMcpServer::config_unavailable("qiongli", env!("CARGO_PKG_VERSION"), registry),
    };
    Ok(server)
}

#[derive(Clone)]
pub struct FullMcpServer {
    lite: LiteMcpServer,
    registry: FullProjectToolRegistry,
    projects: Option<ProjectStateService>,
}

impl FullMcpServer {
    #[must_use]
    pub const fn new(
        lite: LiteMcpServer,
        registry: FullProjectToolRegistry,
        projects: Option<ProjectStateService>,
    ) -> Self {
        Self {
            lite,
            registry,
            projects,
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
            Some("tools/list") => self.list_tools(request),
            Some("tools/call") => self.handle_tool_call(request),
            _ => self.lite.handle(request),
        }
    }

    fn list_tools(&self, request: Value) -> Option<Value> {
        let mut response = self.lite.handle(request)?;
        let tools = response
            .pointer_mut("/result/tools")
            .and_then(Value::as_array_mut);
        if let Some(tools) = tools {
            tools.extend(self.registry.tools().iter().map(|tool| json!(tool)));
        }
        Some(response)
    }

    fn handle_tool_call(&self, request: Value) -> Option<Value> {
        let name = request
            .pointer("/params/name")
            .and_then(Value::as_str)
            .and_then(|name| self.registry.resolve(name));
        let Some(tool) = name else {
            return self.lite.handle(request);
        };
        let validation = self.lite.handle(request.clone())?;
        if validation.pointer("/error/code").and_then(Value::as_i64) != Some(-32601) {
            return Some(validation);
        }
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let arguments = request
            .pointer("/params/arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        Some(self.dispatch_project(id, tool, &arguments))
    }

    fn dispatch_project(&self, id: Value, tool: FullProjectToolId, arguments: &Value) -> Value {
        let Some(arguments) = arguments.as_object() else {
            return json_rpc_error(Some(id), -32602, "Tool arguments must be an object");
        };
        let Some(projects) = self.projects.as_ref() else {
            return tool_error(
                id,
                "project-service-unavailable",
                "native Research Library is unavailable",
            );
        };
        match tool {
            FullProjectToolId::List => {
                if !arguments.is_empty() {
                    return json_rpc_error(Some(id), -32602, "Unsupported argument");
                }
                match projects.snapshot() {
                    Ok(snapshot) => tool_result(id, json!(snapshot)),
                    Err(error) => tool_error(
                        id,
                        error.reason_code(),
                        "Research Library inspection failed",
                    ),
                }
            }
            FullProjectToolId::Read => {
                if arguments.len() != 1 {
                    return json_rpc_error(Some(id), -32602, "Invalid project read arguments");
                }
                let Some(project_id) = arguments.get("project_id").and_then(Value::as_str) else {
                    return json_rpc_error(Some(id), -32602, "project_id is required");
                };
                let Ok(project_id) = ProjectId::parse(project_id.to_string()) else {
                    return json_rpc_error(Some(id), -32602, "project_id is invalid");
                };
                match projects.snapshot().and_then(|snapshot| {
                    let revision = snapshot.revision;
                    snapshot
                        .projects
                        .into_iter()
                        .find(|project| project.project_id == project_id)
                        .map(|project| (revision, project))
                        .ok_or(qiongli_project::ProjectError::ProjectNotRegistered)
                }) {
                    Ok((library_revision, project)) => tool_result(
                        id,
                        json!({
                            "schemaVersion": 1,
                            "libraryRevision": library_revision,
                            "project": project
                        }),
                    ),
                    Err(error) => tool_error(
                        id,
                        error.reason_code(),
                        "registered project inspection failed",
                    ),
                }
            }
            FullProjectToolId::GraphSnapshot => {
                let Some(project_id) = parse_project_id_argument(arguments) else {
                    return json_rpc_error(Some(id), -32602, "Invalid graph snapshot arguments");
                };
                match AcademicGraphService::new(projects.clone()).rebuild(&project_id) {
                    Ok(snapshot) => tool_result(id, json!(snapshot)),
                    Err(error) => {
                        tool_error(id, error.reason_code(), "Academic Graph projection failed")
                    }
                }
            }
            FullProjectToolId::GraphQuery => {
                let Some((project_id, query)) = parse_graph_query_arguments(arguments) else {
                    return json_rpc_error(Some(id), -32602, "Invalid graph query arguments");
                };
                match AcademicGraphIndexService::new(projects.clone())
                    .rebuild(&project_id)
                    .and_then(|index| index.query(&query))
                {
                    Ok(result) => tool_result(id, json!(result)),
                    Err(qiongli_project::ProjectError::InvalidGraphQuery) => {
                        json_rpc_error(Some(id), -32602, "Invalid graph query arguments")
                    }
                    Err(error) => {
                        tool_error(id, error.reason_code(), "Academic Graph query failed")
                    }
                }
            }
            FullProjectToolId::ArtifactChanges => {
                if arguments.len() != 1 {
                    return json_rpc_error(Some(id), -32602, "Invalid artifact change arguments");
                }
                let Some(project_id) = arguments.get("project_id").and_then(Value::as_str) else {
                    return json_rpc_error(Some(id), -32602, "project_id is required");
                };
                let Ok(project_id) = ProjectId::parse(project_id.to_string()) else {
                    return json_rpc_error(Some(id), -32602, "project_id is invalid");
                };
                match projects.artifact_changes(&project_id) {
                    Ok(changes) => tool_result(id, json!(changes)),
                    Err(error) => tool_error(
                        id,
                        error.reason_code(),
                        "registered artifact change inspection failed",
                    ),
                }
            }
            FullProjectToolId::CaptureCoverage => {
                if arguments.len() != 1 {
                    return json_rpc_error(Some(id), -32602, "Invalid capture coverage arguments");
                }
                let Some(project_id) = arguments.get("project_id").and_then(Value::as_str) else {
                    return json_rpc_error(Some(id), -32602, "project_id is required");
                };
                let Ok(project_id) = ProjectId::parse(project_id.to_string()) else {
                    return json_rpc_error(Some(id), -32602, "project_id is invalid");
                };
                match projects.capture_coverage(&project_id) {
                    Ok(coverage) => tool_result(id, json!(coverage)),
                    Err(error) => tool_error(
                        id,
                        error.reason_code(),
                        "capture coverage inspection failed",
                    ),
                }
            }
            FullProjectToolId::CapturePreview => {
                if arguments.len() != 1 {
                    return json_rpc_error(Some(id), -32602, "Invalid capture preview arguments");
                }
                let Some(capture) = arguments
                    .get("capture")
                    .and_then(|capture| parse_connected_capture(capture).ok())
                else {
                    return json_rpc_error(Some(id), -32602, "capture is invalid");
                };
                match projects.preview_capture(capture) {
                    Ok(plan) => tool_result(id, json!(plan.preview())),
                    Err(error) => {
                        tool_error(id, error.reason_code(), "connected capture preview failed")
                    }
                }
            }
            FullProjectToolId::CaptureApply => {
                if arguments.len() != 3 {
                    return json_rpc_error(Some(id), -32602, "Invalid capture apply arguments");
                }
                let Some(capture) = arguments
                    .get("capture")
                    .and_then(|capture| parse_connected_capture(capture).ok())
                else {
                    return json_rpc_error(Some(id), -32602, "capture is invalid");
                };
                let Some(plan_digest) = arguments
                    .get("plan_digest")
                    .and_then(Value::as_str)
                    .filter(|digest| valid_sha256(digest))
                else {
                    return json_rpc_error(Some(id), -32602, "plan_digest is invalid");
                };
                let Some(filesystem_write) = arguments
                    .get("approve_filesystem_write")
                    .and_then(Value::as_bool)
                else {
                    return json_rpc_error(
                        Some(id),
                        -32602,
                        "approve_filesystem_write is required",
                    );
                };
                let plan = match projects.preview_capture(capture) {
                    Ok(plan) => plan,
                    Err(error) => {
                        return tool_error(
                            id,
                            error.reason_code(),
                            "connected capture revalidation failed",
                        );
                    }
                };
                let now_unix = match crate::candidate_cli::now_unix() {
                    Ok(now_unix) => now_unix,
                    Err(reason_code) => {
                        return tool_error(id, reason_code, "native system clock is unavailable");
                    }
                };
                match projects.apply_capture(
                    &plan,
                    &ApprovedCaptureIntake::new(plan_digest, filesystem_write),
                    now_unix,
                ) {
                    Ok(commit) => tool_result(id, json!(commit)),
                    Err(error) => {
                        tool_error(id, error.reason_code(), "connected capture intake failed")
                    }
                }
            }
        }
    }
}

fn parse_project_id_argument(arguments: &serde_json::Map<String, Value>) -> Option<ProjectId> {
    if arguments.len() != 1 {
        return None;
    }
    ProjectId::parse(arguments.get("project_id")?.as_str()?.to_string()).ok()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphQueryArguments {
    project_id: String,
    expected_projection_id: String,
    focus_node_id: Option<String>,
    direction: Option<AcademicGraphDirection>,
    node_types: Option<Vec<AcademicGraphNodeType>>,
    relations: Option<Vec<AcademicGraphRelation>>,
    layers: Option<Vec<AcademicGraphLayer>>,
    canonical_id: Option<String>,
    text: Option<String>,
    max_nodes: Option<usize>,
    max_edges: Option<usize>,
}

fn parse_graph_query_arguments(
    arguments: &serde_json::Map<String, Value>,
) -> Option<(ProjectId, AcademicGraphQueryV1)> {
    let parsed =
        serde_json::from_value::<GraphQueryArguments>(Value::Object(arguments.clone())).ok()?;
    let project_id = ProjectId::parse(parsed.project_id).ok()?;
    let mut query = AcademicGraphQueryV1::new(parsed.expected_projection_id)
        .with_node_types(parsed.node_types.unwrap_or_default())
        .with_relations(parsed.relations.unwrap_or_default())
        .with_layers(parsed.layers.unwrap_or_default())
        .with_limits(
            parsed.max_nodes.unwrap_or(100),
            parsed.max_edges.unwrap_or(200),
        );
    if let Some(focus) = parsed.focus_node_id {
        query = query.with_focus(
            focus,
            parsed.direction.unwrap_or(AcademicGraphDirection::Both),
        );
    } else if parsed.direction.is_some() {
        return None;
    }
    if let Some(canonical_id) = parsed.canonical_id {
        query = query.with_canonical_id(canonical_id);
    }
    if let Some(text) = parsed.text {
        query = query.with_text(text);
    }
    Some((project_id, query))
}

fn parse_connected_capture(value: &Value) -> Result<ResearchCaptureV1, &'static str> {
    let bytes = serde_json::to_vec(value).map_err(|_| "research-capture-document-invalid")?;
    let capture = ResearchCaptureV1::from_json_slice(&bytes)
        .map_err(|_| "research-capture-document-invalid")?;
    if capture.delivery != CaptureDelivery::Connected {
        return Err("research-capture-delivery-invalid");
    }
    Ok(capture)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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
