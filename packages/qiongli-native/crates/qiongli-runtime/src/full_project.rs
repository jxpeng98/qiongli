use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    AcademicGraphDirection, AcademicGraphIndexService, AcademicGraphLayer, AcademicGraphNodeType,
    AcademicGraphPortfolioService, AcademicGraphQueryV1, AcademicGraphRelation,
    AcademicGraphService, ApprovedCaptureIntake, CaptureDelivery, ProjectError, ProjectId,
    ProjectStateService, ResearchCaptureV1,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::FullProjectToolId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullProjectServiceErrorKind {
    InvalidArguments,
    OperationFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullProjectServiceError {
    kind: FullProjectServiceErrorKind,
    reason_code: &'static str,
    public_message: &'static str,
}

impl FullProjectServiceError {
    #[must_use]
    pub const fn kind(self) -> FullProjectServiceErrorKind {
        self.kind
    }

    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        self.reason_code
    }

    #[must_use]
    pub const fn public_message(self) -> &'static str {
        self.public_message
    }

    const fn invalid(public_message: &'static str) -> Self {
        Self {
            kind: FullProjectServiceErrorKind::InvalidArguments,
            reason_code: "full-project-arguments-invalid",
            public_message,
        }
    }

    const fn domain(error: ProjectError, public_message: &'static str) -> Self {
        Self {
            kind: FullProjectServiceErrorKind::OperationFailed,
            reason_code: error.reason_code(),
            public_message,
        }
    }

    const fn clock() -> Self {
        Self {
            kind: FullProjectServiceErrorKind::OperationFailed,
            reason_code: "system-clock-unavailable",
            public_message: "native system clock is unavailable",
        }
    }
}

impl Display for FullProjectServiceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code)
    }
}

impl Error for FullProjectServiceError {}

/// Shared Full-profile project service used by MCP and native ToolHost adapters.
#[derive(Clone)]
pub struct FullProjectService {
    projects: ProjectStateService,
}

impl FullProjectService {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn verify_project_scope(
        &self,
        project_id: &ProjectId,
        expected_semantic_revision: u64,
        expected_root: &Path,
    ) -> Result<(), FullProjectServiceError> {
        let snapshot = self.projects.snapshot().map_err(|error| {
            FullProjectServiceError::domain(error, "registered project scope validation failed")
        })?;
        let project = snapshot
            .projects
            .iter()
            .find(|project| &project.project_id == project_id)
            .ok_or_else(|| {
                FullProjectServiceError::domain(
                    ProjectError::ProjectNotRegistered,
                    "registered project scope validation failed",
                )
            })?;
        if project.semantic_revision != expected_semantic_revision {
            return Err(FullProjectServiceError::domain(
                ProjectError::RevisionConflict,
                "registered project revision changed",
            ));
        }
        let registered = self
            .projects
            .resolve_project_root(project_id)
            .map_err(|error| {
                FullProjectServiceError::domain(error, "registered project scope validation failed")
            })?;
        if registered.path() != expected_root {
            return Err(FullProjectServiceError::domain(
                ProjectError::ProjectIdentityConflict,
                "registered project scope changed",
            ));
        }
        Ok(())
    }

    pub fn dispatch(
        &self,
        tool: FullProjectToolId,
        arguments: &Value,
    ) -> Result<Value, FullProjectServiceError> {
        let arguments = arguments
            .as_object()
            .ok_or_else(|| FullProjectServiceError::invalid("Tool arguments must be an object"))?;
        match tool {
            FullProjectToolId::List => self.list(arguments),
            FullProjectToolId::Read => self.read(arguments),
            FullProjectToolId::GraphSnapshot => self.graph_snapshot(arguments),
            FullProjectToolId::GraphPortfolio => self.graph_portfolio(arguments),
            FullProjectToolId::GraphQuery => self.graph_query(arguments),
            FullProjectToolId::ArtifactChanges => self.artifact_changes(arguments),
            FullProjectToolId::CaptureCoverage => self.capture_coverage(arguments),
            FullProjectToolId::CapturePreview => self.capture_preview(arguments),
            FullProjectToolId::CaptureApply => self.capture_apply(arguments),
        }
    }

    fn list(&self, arguments: &Map<String, Value>) -> Result<Value, FullProjectServiceError> {
        if !arguments.is_empty() {
            return Err(FullProjectServiceError::invalid("Unsupported argument"));
        }
        self.projects
            .snapshot()
            .map(|snapshot| json!(snapshot))
            .map_err(|error| {
                FullProjectServiceError::domain(error, "Research Library inspection failed")
            })
    }

    fn read(&self, arguments: &Map<String, Value>) -> Result<Value, FullProjectServiceError> {
        if arguments.len() != 1 {
            return Err(FullProjectServiceError::invalid(
                "Invalid project read arguments",
            ));
        }
        let project_id = required_project_id(arguments)?;
        self.projects
            .snapshot()
            .and_then(|snapshot| {
                let revision = snapshot.revision;
                snapshot
                    .projects
                    .into_iter()
                    .find(|project| project.project_id == project_id)
                    .map(|project| (revision, project))
                    .ok_or(ProjectError::ProjectNotRegistered)
            })
            .map(|(library_revision, project)| {
                json!({
                    "schemaVersion": 1,
                    "libraryRevision": library_revision,
                    "project": project
                })
            })
            .map_err(|error| {
                FullProjectServiceError::domain(error, "registered project inspection failed")
            })
    }

    fn graph_snapshot(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        let project_id = parse_only_project_id(arguments)
            .ok_or_else(|| FullProjectServiceError::invalid("Invalid graph snapshot arguments"))?;
        let projection = AcademicGraphService::new(self.projects.clone())
            .rebuild_projection(&project_id)
            .map_err(|error| {
                FullProjectServiceError::domain(error, "Academic Graph projection failed")
            })?;
        let mut result = json!(projection.graph);
        result
            .as_object_mut()
            .expect("serialized graph projection is an object")
            .insert("readiness".to_owned(), json!(projection.readiness));
        Ok(result)
    }

    fn graph_portfolio(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        if !arguments.is_empty() {
            return Err(FullProjectServiceError::invalid("Unsupported argument"));
        }
        AcademicGraphPortfolioService::new(self.projects.clone())
            .rebuild()
            .map(|portfolio| json!(portfolio))
            .map_err(|error| {
                FullProjectServiceError::domain(error, "Academic Graph Portfolio failed")
            })
    }

    fn graph_query(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        let (project_id, query) = parse_graph_query_arguments(arguments)
            .ok_or_else(|| FullProjectServiceError::invalid("Invalid graph query arguments"))?;
        let projection = AcademicGraphService::new(self.projects.clone())
            .rebuild_projection(&project_id)
            .map_err(|error| {
                FullProjectServiceError::domain(error, "Academic Graph projection failed")
            })?;
        let result = AcademicGraphIndexService::new(self.projects.clone())
            .rebuild(&project_id)
            .and_then(|index| index.query(&query))
            .map_err(|error| {
                if error == ProjectError::InvalidGraphQuery {
                    FullProjectServiceError::invalid("Invalid graph query arguments")
                } else {
                    FullProjectServiceError::domain(error, "Academic Graph query failed")
                }
            })?;
        let mut result = json!(result);
        result
            .as_object_mut()
            .expect("serialized graph query result is an object")
            .insert("readiness".to_owned(), json!(projection.readiness));
        Ok(result)
    }

    fn artifact_changes(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        if arguments.len() != 1 {
            return Err(FullProjectServiceError::invalid(
                "Invalid artifact change arguments",
            ));
        }
        let project_id = required_project_id(arguments)?;
        self.projects
            .artifact_changes(&project_id)
            .map(|changes| json!(changes))
            .map_err(|error| {
                FullProjectServiceError::domain(
                    error,
                    "registered artifact change inspection failed",
                )
            })
    }

    fn capture_coverage(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        if arguments.len() != 1 {
            return Err(FullProjectServiceError::invalid(
                "Invalid capture coverage arguments",
            ));
        }
        let project_id = required_project_id(arguments)?;
        self.projects
            .capture_coverage(&project_id)
            .map(|coverage| json!(coverage))
            .map_err(|error| {
                FullProjectServiceError::domain(error, "capture coverage inspection failed")
            })
    }

    fn capture_preview(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        if arguments.len() != 1 {
            return Err(FullProjectServiceError::invalid(
                "Invalid capture preview arguments",
            ));
        }
        let capture = arguments
            .get("capture")
            .and_then(|capture| parse_connected_capture(capture).ok())
            .ok_or_else(|| FullProjectServiceError::invalid("capture is invalid"))?;
        self.projects
            .preview_capture(capture)
            .map(|plan| json!(plan.preview()))
            .map_err(|error| {
                FullProjectServiceError::domain(error, "connected capture preview failed")
            })
    }

    fn capture_apply(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Value, FullProjectServiceError> {
        if arguments.len() != 3 {
            return Err(FullProjectServiceError::invalid(
                "Invalid capture apply arguments",
            ));
        }
        let capture = arguments
            .get("capture")
            .and_then(|capture| parse_connected_capture(capture).ok())
            .ok_or_else(|| FullProjectServiceError::invalid("capture is invalid"))?;
        let plan_digest = arguments
            .get("plan_digest")
            .and_then(Value::as_str)
            .filter(|digest| valid_sha256(digest))
            .ok_or_else(|| FullProjectServiceError::invalid("plan_digest is invalid"))?;
        let filesystem_write = arguments
            .get("approve_filesystem_write")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                FullProjectServiceError::invalid("approve_filesystem_write is required")
            })?;
        let plan = self.projects.preview_capture(capture).map_err(|error| {
            FullProjectServiceError::domain(error, "connected capture revalidation failed")
        })?;
        let now_unix = now_unix()?;
        self.projects
            .apply_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan_digest, filesystem_write),
                now_unix,
            )
            .map(|commit| json!(commit))
            .map_err(|error| {
                FullProjectServiceError::domain(error, "connected capture intake failed")
            })
    }
}

fn parse_project_id(arguments: &Map<String, Value>, key: &str) -> Option<ProjectId> {
    ProjectId::parse(arguments.get(key)?.as_str()?.to_string()).ok()
}

fn required_project_id(
    arguments: &Map<String, Value>,
) -> Result<ProjectId, FullProjectServiceError> {
    let project_id = arguments
        .get("project_id")
        .and_then(Value::as_str)
        .ok_or_else(|| FullProjectServiceError::invalid("project_id is required"))?;
    ProjectId::parse(project_id.to_string())
        .map_err(|_| FullProjectServiceError::invalid("project_id is invalid"))
}

fn parse_only_project_id(arguments: &Map<String, Value>) -> Option<ProjectId> {
    if arguments.len() != 1 {
        return None;
    }
    parse_project_id(arguments, "project_id")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphQueryArguments {
    project_id: String,
    expected_projection_id: String,
    focus_node_id: Option<String>,
    direction: Option<AcademicGraphDirection>,
    max_depth: Option<usize>,
    node_types: Option<Vec<AcademicGraphNodeType>>,
    relations: Option<Vec<AcademicGraphRelation>>,
    layers: Option<Vec<AcademicGraphLayer>>,
    canonical_id: Option<String>,
    text: Option<String>,
    max_nodes: Option<usize>,
    max_edges: Option<usize>,
}

fn parse_graph_query_arguments(
    arguments: &Map<String, Value>,
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
        query = query
            .with_focus(
                focus,
                parsed.direction.unwrap_or(AcademicGraphDirection::Both),
            )
            .with_max_depth(parsed.max_depth.unwrap_or(1));
    } else if parsed.direction.is_some() || parsed.max_depth.is_some() {
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

fn now_unix() -> Result<u64, FullProjectServiceError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| FullProjectServiceError::clock())
}
