#![cfg_attr(
    test,
    allow(
        dead_code,
        reason = "the Tauri IPC adapter is excluded from the core library unit-test binary"
    )
)]

use qiongli_project::{
    ACADEMIC_CONSOLIDATION_SCHEMA_VERSION, ARTIFACT_CHANGE_SCHEMA_VERSION, AcademicGraphConfidence,
    AcademicGraphDiagnosticV1, AcademicGraphEdgeStatus, AcademicGraphEdgeV1,
    AcademicGraphEntityKind, AcademicGraphIdentityScope, AcademicGraphLayer, AcademicGraphNodeType,
    AcademicGraphNodeV1, AcademicGraphPathQueryV1, AcademicGraphPathResultV1,
    AcademicGraphPathStatus, AcademicGraphPathStepV1, AcademicGraphPathTraversal,
    AcademicGraphPortfolioSnapshotV1, AcademicGraphQueryResultV1, AcademicGraphQueryV1,
    AcademicGraphRelation, AcademicGraphRevisionComparisonV1, AcademicGraphSnapshotV1,
    AcademicGraphSourceKind, AcademicGraphSourceRefV1, AcademicInferenceStrength,
    ArtifactChangeSnapshotV1, ArtifactChangeState, CAPTURE_COVERAGE_SCHEMA_VERSION,
    CAPTURE_INBOX_SCHEMA_VERSION, CAPTURE_INTAKE_SCHEMA_VERSION, CaptureArea,
    CaptureConsolidationOutcome, CaptureConsolidationPreviewV1, CaptureCoverageDelivery,
    CaptureCoverageSnapshotV1, CaptureCoverageState, CaptureDelivery, CaptureDisposition,
    CaptureInboxSnapshotV1, CaptureIntakeEffect, CaptureIntakePreviewV1, CapturePolicy,
    CaptureSource, CaptureSourceCoverageV1, ContradictionV1, DecisionCandidateV1, DecisionRelation,
    EvidenceLocatorKind, EvidenceReferenceV1, PortableProjectOperation, PortableProjectPreviewV1,
    ProjectBindingV1, ProjectId, ProjectKind, ProjectLifecycle, ProjectMutationEffect,
    ProjectMutationKind, ProjectMutationPreviewV1, ProjectStage, RegisteredArtifact,
    RegisteredArtifactObservationV1, ResearchCaptureDraftV1, ResearchCaptureV1,
    ResearchLibrarySnapshotV1, SemanticChangeV1,
};
use qiongli_ui::{
    AgentBackendSecretChange, DesktopEvent, DesktopIntent, DesktopService, DesktopSnapshotV1,
    IntegrationPathView, IntegrationSelection, IntegrationTarget, IntegrationView,
    OperationApproval, OperationKind, OperationPreview, OperationToken, PrivateText,
    ProductTrustView, ProfileKind, SkillsDestinationPreset, StatusCode, UpdatePhaseView,
    UpdateStreamView, UpdateView,
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::orchestration_control::{OrchestrationRunListViewV1, OrchestrationRunSummaryV1};

pub(crate) const APP_API_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSnapshotV1 {
    schema_version: u32,
    product: AppProductView,
    content: AppContentView,
    mcp: AppMcpView,
    configuration: AppConfigurationView,
    update: AppUpdateView,
    research_library: ResearchLibrarySnapshotV1,
    integrations: Vec<AppIntegrationView>,
    capabilities: AppCapabilityView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProductView {
    version: String,
    build: String,
    operating_system: String,
    architecture: String,
    trust: AppTrustView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppTrustView {
    mode: &'static str,
    label: &'static str,
    can_apply: bool,
    reason_code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppContentView {
    status: &'static str,
    pack_id: String,
    content_version: String,
    entry_count: usize,
    profiles: Vec<AppProfileView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProfileView {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    included_resource_kinds: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppMcpView {
    status: &'static str,
    profile: &'static str,
    public_tool_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConfigurationView {
    status: &'static str,
    revision: Option<u64>,
    legacy_credential: AppLegacyCredentialView,
    cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppLegacyCredentialView {
    reference_present: bool,
    cleanup_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppUpdateView {
    status: &'static str,
    selected_stream: &'static str,
    phase: &'static str,
    available_version: Option<String>,
    archive_size_bytes: Option<u64>,
    progress: Option<AppUpdateProgressView>,
    reason_code: &'static str,
    remediation: &'static str,
    can_select_stream: bool,
    can_check: bool,
    can_prepare: bool,
    can_install: bool,
    can_cancel: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppUpdateProgressView {
    completed_steps: u8,
    total_steps: u8,
    label: &'static str,
    indeterminate: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppIntegrationView {
    target: &'static str,
    label: &'static str,
    connection: AppConnectionView,
    client: AppClientView,
    plugin: AppPluginVersionView,
    discovery: &'static str,
    candidate_required: bool,
    legacy_detected: bool,
    overall: &'static str,
    managed_content: AppManagedContentView,
    symbolic_location: &'static str,
    activation_policy: &'static str,
    ownership: &'static str,
    next_action: &'static str,
    evidence_code: &'static str,
    paths: Vec<AppIntegrationPathView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppClientView {
    detected: bool,
    status: &'static str,
    version: Option<String>,
    compatibility: &'static str,
    minimum_supported_version: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConnectionView {
    state: &'static str,
    label: &'static str,
    reason_code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppPluginVersionView {
    installed_version: Option<String>,
    available_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppManagedContentView {
    source: &'static str,
    skills: &'static str,
    marketplace: &'static str,
    direct_package: Option<&'static str>,
    registration: &'static str,
    activation: &'static str,
    activation_observation: &'static str,
    mcp_attachment: &'static str,
    mcp_attachment_observation: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppIntegrationPathView {
    surface: &'static str,
    scope: &'static str,
    source: &'static str,
    state: &'static str,
    management: &'static str,
    selected: bool,
    symbolic_path: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppCapabilityView {
    refresh: bool,
    skills_materialize: bool,
    integration_discovery: bool,
    integration_preview: bool,
    project_library: bool,
    project_mutation: bool,
    capture_inbox: bool,
    capture_mutation: bool,
    academic_graph: bool,
    orchestration_inspect: bool,
    orchestration_control: bool,
    legacy_credential_cleanup: bool,
    apply: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppIntegrationSelection {
    codex: bool,
    claude_code: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum AppAcademicGraphEntity {
    Node { id: String },
    Edge { id: String },
}

impl AppAcademicGraphEntity {
    pub(crate) fn into_parts(self) -> (AcademicGraphEntityKind, String) {
        match self {
            Self::Node { id } => (AcademicGraphEntityKind::Node, id),
            Self::Edge { id } => (AcademicGraphEntityKind::Edge, id),
        }
    }
}

#[allow(
    dead_code,
    reason = "legacy direct-execution fields remain decode-only during the host-driven migration"
)]
#[derive(Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AppIntent {
    Refresh,
    RefreshResearchLibrary,
    SelectProjectDirectory,
    SelectProjectCreateDestination {
        suggested_name: String,
    },
    PreviewProjectCreate {
        directory_token: String,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
    },
    PreviewProjectRegister {
        directory_token: String,
    },
    OpenProject {
        project_id: String,
    },
    SelectProjectExportDestination {
        project_id: String,
    },
    PreviewProjectExport {
        directory_token: String,
    },
    SelectProjectImportLocations {
        suggested_name: String,
    },
    PreviewProjectImport {
        directory_token: String,
    },
    PreviewProjectRepairManifest {
        project_id: String,
    },
    PreviewProjectArchive {
        project_id: String,
    },
    PreviewProjectRestore {
        project_id: String,
    },
    PreviewProjectRefresh {
        project_id: String,
    },
    PreviewProjectUnregister {
        project_id: String,
    },
    LoadCaptureInbox {
        project_id: String,
    },
    LoadCaptureCoverage {
        project_id: String,
    },
    LoadArtifactChanges {
        project_id: String,
    },
    LoadAcademicGraph {
        project_id: String,
    },
    LoadAcademicGraphPortfolio,
    QueryAcademicGraph {
        project_id: String,
        query: AcademicGraphQueryV1,
    },
    QueryAcademicGraphPath {
        project_id: String,
        query: AcademicGraphPathQueryV1,
    },
    OpenAcademicGraphArtifact {
        project_id: String,
        expected_project_revision: u64,
        expected_projection_id: String,
        entity: AppAcademicGraphEntity,
    },
    ReadCapture {
        project_id: String,
        capture_id: String,
    },
    SelectCaptureFile {
        project_id: String,
    },
    PreviewCaptureIntake {
        file_token: String,
    },
    PreviewCaptureConsolidation {
        project_id: String,
        capture_id: String,
    },
    RefreshIntegrationDiscovery,
    SelectUpdateStream {
        stream: AppUpdateStream,
    },
    CheckForUpdates,
    PrepareUpdate,
    PollUpdate,
    CancelUpdate,
    PreviewUpdateInstall,
    PreviewAgentBackendSettings {
        expected_revision: u64,
        enabled: bool,
    },
    PreviewAgentBackendCredential {
        #[serde(deserialize_with = "deserialize_private_text")]
        api_key: PrivateText,
    },
    PreviewRemoveAgentBackendCredential,
    PreviewAgentRun {
        project_id: String,
        expected_project_revision: u64,
        #[serde(deserialize_with = "deserialize_private_text")]
        prompt: PrivateText,
    },
    LoadOrchestration {
        project_id: String,
        expected_project_revision: u64,
    },
    PreviewOrchestrationTest {
        project_id: String,
        expected_project_revision: u64,
        execution_mode: qiongli_execution::OrchestrationExecutionMode,
    },
    PreviewOrchestrationContinue {
        project_id: String,
        expected_project_revision: u64,
        run_id: String,
        expected_generation: u64,
        expected_document_sha256: String,
    },
    ControlOrchestration {
        project_id: String,
        expected_project_revision: u64,
        run_id: String,
        expected_generation: u64,
        expected_document_sha256: String,
        action_name: AppOrchestrationControlAction,
    },
    TestOpenAiBackend,
    PreviewInstallRecommended,
    PreviewInstallSelected {
        selection: AppIntegrationSelection,
    },
    VerifyIntegrations {
        selection: AppIntegrationSelection,
    },
    PreviewRepairAll,
    PreviewUpdateIntegrations {
        selection: AppIntegrationSelection,
    },
    PreviewRemoveIntegrations {
        selection: AppIntegrationSelection,
    },
    PreviewSkillsPresetMaterialization {
        profile: AppProfileId,
        preset: AppSkillsPreset,
    },
    VerifySkillsPreset {
        preset: AppSkillsPreset,
    },
    PreviewSkillsPresetRemoval {
        preset: AppSkillsPreset,
    },
    ConfirmOperation {
        token: String,
    },
    CancelOperation {
        token: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppProfileId {
    SkillOnly,
    MarketplaceLite,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppUpdateStream {
    Stable,
    Beta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppSkillsPreset {
    QiongliManaged,
    DetectedCodex,
    DetectedClaudeCode,
    CurrentProject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppOrchestrationControlAction {
    Pause,
    Recover,
    Resume,
    Cancel,
}

macro_rules! define_app_events {
    ($(
        $variant:ident { $($field:ident: $field_type:ty),* $(,)? } => $event_type:literal
    ),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
        #[serde(
            tag = "type",
            rename_all = "kebab-case",
            rename_all_fields = "camelCase"
        )]
        pub(crate) enum AppEvent {
            $($variant { $($field: $field_type),* },)+
        }

        impl AppEvent {
            const fn contract_type(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => $event_type,)+
                }
            }
        }

        const APP_EVENT_VARIANT_COUNT: usize = [$($event_type),+].len();
    };
}

define_app_events! {
    Snapshot { snapshot: AppSnapshotV1 } => "snapshot",
    Preview { preview: AppOperationPreview } => "preview",
    CaptureInbox { inbox: CaptureInboxSnapshotV1 } => "capture-inbox",
    CaptureCoverage { coverage: CaptureCoverageSnapshotV1 } => "capture-coverage",
    ArtifactChanges { changes: ArtifactChangeSnapshotV1 } => "artifact-changes",
    AcademicGraph {
        graph: AcademicGraphSnapshotV1,
        comparison: Option<AcademicGraphRevisionComparisonV1>,
    } => "academic-graph",
    AcademicGraphPortfolio {
        portfolio: AcademicGraphPortfolioSnapshotV1,
    } => "academic-graph-portfolio",
    AcademicGraphQuery { result: AcademicGraphQueryResultV1 } => "academic-graph-query",
    AcademicGraphPath { result: AcademicGraphPathResultV1 } => "academic-graph-path",
    AcademicGraphArtifactOpened {
        project_id: ProjectId,
        project_revision: u64,
        projection_id: String,
        entity: AppAcademicGraphEntity,
    } => "academic-graph-artifact-opened",
    CaptureRead { capture: AppResearchCaptureV1 } => "capture-read",
    CaptureFileSelected { token: String, file_label: String } => "capture-file-selected",
    CaptureIntakePreview {
        intake: CaptureIntakePreviewV1,
        preview: AppOperationPreview,
    } => "capture-intake-preview",
    CaptureConsolidationPreview {
        consolidation: CaptureConsolidationPreviewV1,
        preview: AppOperationPreview,
    } => "capture-consolidation-preview",
    ProjectDirectorySelected { token: String, root_label: String } => "project-directory-selected",
    UpdateChanged { update: AppUpdateView, close_requested: bool } => "update-changed",
    OrchestrationLoaded { runs: OrchestrationRunListViewV1 } => "orchestration-loaded",
    OrchestrationRunUpdated {
        run: OrchestrationRunSummaryV1,
        runs: OrchestrationRunListViewV1,
    } => "orchestration-run-updated",
    Completed { code: &'static str, snapshot: AppSnapshotV1 } => "completed",
    CaptureOperationCompleted {
        code: &'static str,
        snapshot: Box<AppSnapshotV1>,
        inbox: CaptureInboxSnapshotV1,
        coverage: CaptureCoverageSnapshotV1,
        changes: ArtifactChangeSnapshotV1,
    } => "capture-operation-completed",
    Cancelled { code: &'static str } => "cancelled",
    ValidationFailed { code: &'static str } => "validation-failed",
    Failed { code: &'static str } => "failed",
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppApiContractFixtureV1 {
    schema_version: u32,
    snapshot: AppSnapshotV1,
    events: Vec<AppEvent>,
}

pub(crate) fn serialize_app_api_contract_fixture(
    snapshot: AppSnapshotV1,
) -> Result<String, &'static str> {
    let project_id = ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051")
        .map_err(|_| "app-api-contract-project-id-invalid")?;
    let capture = canonical_contract_capture(project_id.clone())?;
    let capture_id = capture.capture_id.clone();
    let inbox = canonical_contract_inbox(project_id.clone());
    let coverage = canonical_contract_coverage(project_id.clone());
    let changes = canonical_contract_artifact_changes(project_id.clone());
    let (graph, graph_query, graph_path) = canonical_contract_graph(project_id.clone())?;
    let graph_artifact_opened = AppEvent::AcademicGraphArtifactOpened {
        project_id: graph.project_id.clone(),
        project_revision: graph.project_revision,
        projection_id: graph.projection_id.clone(),
        entity: AppAcademicGraphEntity::Node {
            id: graph.nodes[0].node_id.clone(),
        },
    };
    let project_preview = ProjectMutationPreviewV1 {
        schema_version: 1,
        plan_digest: "c".repeat(64),
        operation: ProjectMutationKind::Refresh,
        effect: ProjectMutationEffect::UpdateSemanticRevision,
        project_id: project_id.clone(),
        display_name: "Canonical article project".to_owned(),
        project_kind: ProjectKind::Article,
        stage: ProjectStage::Writing,
        expected_library_revision: 0,
        expected_project_revision: Some(1),
        root_label: "canonical-project".to_owned(),
        manifest_action: "advance-semantic-revision".to_owned(),
        missing_continuity_artifacts: Vec::new(),
        approvals_required: vec!["filesystem-write".to_owned()],
    };
    let intake = CaptureIntakePreviewV1 {
        schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
        plan_digest: "a".repeat(64),
        capture_id: capture_id.clone(),
        project_id: project_id.clone(),
        disposition: CaptureDisposition::Refinement,
        effect: CaptureIntakeEffect::AppendPendingHistory,
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        expected_library_revision: 0,
        expected_project_revision: 1,
        change_count: 0,
        decision_count: 0,
        evidence_count: 0,
        contradiction_count: 0,
        next_action_count: 0,
        history_entry: format!("captures/history/{}.json", capture_id.as_str()),
        approvals_required: vec!["filesystem-write".to_owned()],
    };
    let consolidation = CaptureConsolidationPreviewV1 {
        schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
        plan_digest: "b".repeat(64),
        capture_id,
        project_id,
        disposition: CaptureDisposition::Refinement,
        outcome: CaptureConsolidationOutcome::Ready,
        expected_library_revision: 0,
        expected_project_revision: 1,
        next_project_revision: Some(2),
        project_stage: ProjectStage::Writing,
        reviewed_at_unix: 1,
        conflicts: Vec::new(),
        artifact_deltas: Vec::new(),
        receipt_entry: "captures/consolidated/canonical.json".to_owned(),
        approvals_required: vec![
            "academic-consolidation".to_owned(),
            "filesystem-write".to_owned(),
        ],
    };
    let intake_operation = app_capture_intake_operation_preview(
        "0000000000000000000000000000002c".to_owned(),
        "canonical-capture.json".to_owned(),
        &intake,
    );
    let consolidation_operation = app_capture_consolidation_operation_preview(
        "0000000000000000000000000000002d".to_owned(),
        &consolidation,
    );
    let project_operation = app_project_operation_preview(
        "0000000000000000000000000000002e".to_owned(),
        &project_preview,
    );
    let orchestration_project_id = ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051")
        .map_err(|_| "app-api-contract-project-id-invalid")?;
    let orchestration_run = OrchestrationRunSummaryV1 {
        run_id: qiongli_execution::RunId::parse(format!("run_{}", "2".repeat(32)))
            .map_err(|_| "app-api-contract-run-id-invalid")?,
        profile_id: "openai-solo-v1".to_owned(),
        execution_mode: qiongli_execution::OrchestrationExecutionMode::Solo,
        status: qiongli_execution::OrchestrationRunStatus::Running,
        generation: 3,
        document_sha256: "3".repeat(64),
        completed_task_count: 1,
        total_task_count: 76,
        next_task_id: Some("A1_5".to_owned()),
        active_task_id: None,
        active_role: None,
        completed_role_count: 0,
        required_role_count: 1,
        host_driven: true,
        recovery_required: false,
        can_continue: true,
        can_pause: true,
        can_resume: false,
        can_recover: false,
        can_cancel: true,
    };
    let orchestration_runs = OrchestrationRunListViewV1 {
        schema_version: 1,
        project_id: orchestration_project_id,
        expected_project_revision: 1,
        runs: vec![orchestration_run.clone()],
    };
    let update = snapshot.update.clone();
    let events = vec![
        AppEvent::Snapshot {
            snapshot: snapshot.clone(),
        },
        AppEvent::Preview {
            preview: project_operation,
        },
        AppEvent::CaptureInbox {
            inbox: inbox.clone(),
        },
        AppEvent::CaptureCoverage {
            coverage: coverage.clone(),
        },
        AppEvent::ArtifactChanges {
            changes: changes.clone(),
        },
        AppEvent::AcademicGraph {
            graph,
            comparison: None,
        },
        AppEvent::AcademicGraphPortfolio {
            portfolio: AcademicGraphPortfolioSnapshotV1 {
                schema_version: 1,
                document_kind: "qiongli-academic-graph-portfolio".to_owned(),
                portfolio_id: format!("gpf_{}", "0".repeat(64)),
                library_revision: 0,
                project_count: 0,
                included_project_count: 0,
                skipped_project_count: 0,
                node_count: 0,
                edge_count: 0,
                projects: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
            },
        },
        AppEvent::AcademicGraphQuery {
            result: graph_query,
        },
        AppEvent::AcademicGraphPath { result: graph_path },
        graph_artifact_opened,
        AppEvent::CaptureRead {
            capture: capture.into(),
        },
        AppEvent::ProjectDirectorySelected {
            token: "0000000000000000000000000000002a".to_owned(),
            root_label: "canonical-project".to_owned(),
        },
        AppEvent::CaptureFileSelected {
            token: "0000000000000000000000000000002b".to_owned(),
            file_label: "canonical-capture.json".to_owned(),
        },
        AppEvent::CaptureIntakePreview {
            intake,
            preview: intake_operation,
        },
        AppEvent::CaptureConsolidationPreview {
            consolidation,
            preview: consolidation_operation,
        },
        AppEvent::UpdateChanged {
            update,
            close_requested: true,
        },
        AppEvent::OrchestrationLoaded {
            runs: orchestration_runs.clone(),
        },
        AppEvent::OrchestrationRunUpdated {
            run: orchestration_run,
            runs: orchestration_runs,
        },
        AppEvent::Completed {
            code: "canonical-operation-completed",
            snapshot: snapshot.clone(),
        },
        AppEvent::CaptureOperationCompleted {
            code: "canonical-capture-operation-completed",
            snapshot: Box::new(snapshot.clone()),
            inbox,
            coverage,
            changes,
        },
        AppEvent::Cancelled {
            code: "canonical-operation-cancelled",
        },
        AppEvent::ValidationFailed {
            code: "canonical-validation-failed",
        },
        AppEvent::Failed {
            code: "canonical-operation-failed",
        },
    ];
    let mut covered_event_types = events
        .iter()
        .map(AppEvent::contract_type)
        .collect::<Vec<_>>();
    covered_event_types.sort_unstable();
    covered_event_types.dedup();
    if covered_event_types.len() != APP_EVENT_VARIANT_COUNT
        || events.iter().any(|event| {
            serde_json::to_value(event)
                .ok()
                .and_then(|value| value.get("type")?.as_str().map(str::to_owned))
                .as_deref()
                != Some(event.contract_type())
        })
    {
        return Err("app-api-contract-event-coverage-incomplete");
    }
    serde_json::to_string_pretty(&AppApiContractFixtureV1 {
        schema_version: APP_API_SCHEMA_VERSION,
        snapshot,
        events,
    })
    .map(|rendered| format!("{rendered}\n"))
    .map_err(|_| "app-api-contract-fixture-serialization-failed")
}

fn canonical_contract_graph(
    project_id: ProjectId,
) -> Result<
    (
        AcademicGraphSnapshotV1,
        AcademicGraphQueryResultV1,
        AcademicGraphPathResultV1,
    ),
    &'static str,
> {
    let project_node = AcademicGraphNodeV1::new(
        &project_id,
        AcademicGraphNodeType::Project,
        AcademicGraphIdentityScope::Project,
        project_id.as_str(),
        "Canonical article project",
        vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
        "context/project_manifest.json",
        "project",
    )
    .map_err(|_| "app-api-contract-graph-node-invalid")?;
    let claim_node = AcademicGraphNodeV1::new(
        &project_id,
        AcademicGraphNodeType::Claim,
        AcademicGraphIdentityScope::Project,
        "CLM-001",
        "Portable article state preserves evidence provenance",
        vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
        "manuscript/claims_evidence_map.md",
        "CLM-001",
    )
    .map_err(|_| "app-api-contract-graph-node-invalid")?;
    let edge = AcademicGraphEdgeV1::new(
        &project_id,
        project_node.node_id.clone(),
        AcademicGraphRelation::Contains,
        claim_node.node_id.clone(),
        vec![AcademicGraphLayer::Combined],
        "The canonical article project contains its manuscript claims.",
        "manuscript/claims_evidence_map.md",
        "CLM-001",
        "The fixture records contract shape rather than empirical support.",
        AcademicInferenceStrength::DirectEvidence,
        AcademicGraphConfidence::High,
        AcademicGraphEdgeStatus::Observed,
        None,
    )
    .map_err(|_| "app-api-contract-graph-edge-invalid")?;
    let projection_id = format!("grp_{}", "a".repeat(64));
    let nodes = vec![project_node, claim_node];
    let edges = vec![edge];
    let snapshot = AcademicGraphSnapshotV1 {
        schema_version: 1,
        document_kind: "qiongli-academic-graph".to_owned(),
        projection_id: projection_id.clone(),
        projection_digest: "b".repeat(64),
        project_id: project_id.clone(),
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        project_lifecycle: ProjectLifecycle::Active,
        project_manifest_digest: "c".repeat(64),
        project_semantic_digest: "d".repeat(64),
        graph_source_digest: "e".repeat(64),
        source_count: 1,
        present_source_count: 1,
        node_count: nodes.len(),
        edge_count: edges.len(),
        diagnostic_count: 0,
        sources: vec![AcademicGraphSourceRefV1 {
            source_kind: AcademicGraphSourceKind::ProjectManifest,
            artifact_path: "context/project_manifest.json".to_owned(),
            present: true,
            content_digest: Some("c".repeat(64)),
            size_bytes: 512,
        }],
        nodes: nodes.clone(),
        edges: edges.clone(),
        diagnostics: Vec::<AcademicGraphDiagnosticV1>::new(),
    };
    let index_id = format!("gix_{}", "f".repeat(64));
    let path = AcademicGraphPathResultV1 {
        schema_version: 1,
        document_kind: "qiongli-academic-graph-explanatory-path".to_owned(),
        index_id: index_id.clone(),
        projection_id: projection_id.clone(),
        project_id: project_id.clone(),
        project_revision: 1,
        source_node_id: nodes[0].node_id.clone(),
        target_node_id: nodes[1].node_id.clone(),
        max_hops: 6,
        status: AcademicGraphPathStatus::Found,
        hop_count: 1,
        nodes: nodes.clone(),
        edges: edges.clone(),
        steps: vec![AcademicGraphPathStepV1 {
            sequence: 1,
            from_node_id: nodes[0].node_id.clone(),
            edge_id: edges[0].edge_id.clone(),
            to_node_id: nodes[1].node_id.clone(),
            traversal: AcademicGraphPathTraversal::Forward,
        }],
    };
    let result = AcademicGraphQueryResultV1 {
        schema_version: 1,
        document_kind: "qiongli-academic-graph-query-result".to_owned(),
        index_id,
        projection_id,
        project_id,
        project_revision: 1,
        matched_node_count: nodes.len(),
        matched_edge_count: edges.len(),
        nodes_truncated: false,
        edges_truncated: false,
        nodes,
        edges,
    };
    Ok((snapshot, result, path))
}

fn canonical_contract_capture(project_id: ProjectId) -> Result<ResearchCaptureV1, &'static str> {
    let binding = ProjectBindingV1::new(
        project_id,
        1,
        ProjectStage::Writing,
        "Validate the canonical desktop contract",
        CapturePolicy::ReviewRequired,
    )
    .map_err(|_| "app-api-contract-binding-invalid")?;
    ResearchCaptureDraftV1 {
        binding,
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        captured_at_unix: 1,
        summary: "Canonical bounded research capture".to_owned(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: Vec::new(),
    }
    .into_capture()
    .map_err(|_| "app-api-contract-capture-invalid")
}

fn canonical_contract_inbox(project_id: ProjectId) -> CaptureInboxSnapshotV1 {
    CaptureInboxSnapshotV1 {
        schema_version: CAPTURE_INBOX_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        pending_review_count: 0,
        stale_count: 0,
        conflicted_count: 0,
        applied_count: 0,
        entries: Vec::new(),
    }
}

fn canonical_contract_coverage(project_id: ProjectId) -> CaptureCoverageSnapshotV1 {
    let sources = [
        CaptureSource::Codex,
        CaptureSource::ClaudeCode,
        CaptureSource::ChatGpt,
        CaptureSource::Cli,
        CaptureSource::Manual,
        CaptureSource::Repository,
        CaptureSource::PortableFile,
    ]
    .into_iter()
    .map(|source| CaptureSourceCoverageV1 {
        source,
        state: CaptureCoverageState::Unknown,
        delivery: CaptureCoverageDelivery::Unknown,
        capture_count: 0,
        pending_review_count: 0,
        current_count: 0,
        stale_count: 0,
        conflicted_count: 0,
        unbound_count: 0,
        latest_capture_id: None,
        last_captured_at_unix: None,
    })
    .collect();
    CaptureCoverageSnapshotV1 {
        schema_version: CAPTURE_COVERAGE_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        capture_count: 0,
        connected_count: 0,
        repository_backed_count: 0,
        portable_count: 0,
        manual_count: 0,
        pending_review_count: 0,
        current_count: 0,
        stale_count: 0,
        conflicted_count: 0,
        unbound_count: 0,
        unknown_source_count: 7,
        sources,
    }
}

fn canonical_contract_artifact_changes(project_id: ProjectId) -> ArtifactChangeSnapshotV1 {
    let artifacts = [
        (
            RegisteredArtifact::ResearchState,
            "context/research_state.md",
        ),
        (RegisteredArtifact::DecisionLog, "context/decision_log.md"),
        (RegisteredArtifact::StageHandoff, "context/stage_handoff.md"),
        (
            RegisteredArtifact::BoundaryReview,
            "context/boundary_review.md",
        ),
        (RegisteredArtifact::IdeaFunnel, "context/idea_funnel.md"),
        (
            RegisteredArtifact::LiteratureMap,
            "literature/literature_map.md",
        ),
        (
            RegisteredArtifact::ClaimEvidenceLedger,
            "evidence/claim-evidence-ledger.csv",
        ),
        (
            RegisteredArtifact::ManuscriptClaimMap,
            "manuscript/claims_evidence_map.md",
        ),
    ]
    .into_iter()
    .map(
        |(artifact, relative_path)| RegisteredArtifactObservationV1 {
            artifact,
            relative_path: relative_path.to_owned(),
            present: false,
        },
    )
    .collect::<Vec<_>>();
    ArtifactChangeSnapshotV1 {
        schema_version: ARTIFACT_CHANGE_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        state: ArtifactChangeState::Current,
        registered_artifact_count: artifacts.len(),
        present_artifact_count: 0,
        change_count: 0,
        unattributed_count: 0,
        changes: Vec::new(),
        artifacts,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppOperationPreview {
    token: String,
    kind: &'static str,
    title: &'static str,
    summary: &'static str,
    display_target: Option<String>,
    plan_digest_sha256: Option<String>,
    approvals_required: Vec<&'static str>,
    can_confirm: bool,
    blocked_reason: Option<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppResearchCaptureV1 {
    schema_version: u32,
    capture_id: String,
    binding: AppCaptureBindingV1,
    source: CaptureSource,
    delivery: CaptureDelivery,
    captured_at_unix: u64,
    summary: String,
    changes: Vec<AppSemanticChangeV1>,
    decisions: Vec<AppDecisionCandidateV1>,
    evidence: Vec<AppEvidenceReferenceV1>,
    contradictions: Vec<AppContradictionV1>,
    next_actions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppCaptureBindingV1 {
    schema_version: u32,
    project_id: String,
    base_revision: u64,
    stage: ProjectStage,
    task: String,
    capture_policy: CapturePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSemanticChangeV1 {
    area: CaptureArea,
    summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppDecisionCandidateV1 {
    relation: DecisionRelation,
    statement: String,
    rationale: String,
    target: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppEvidenceReferenceV1 {
    locator_kind: EvidenceLocatorKind,
    locator: String,
    relevance: String,
    limitation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppContradictionV1 {
    statement: String,
    conflicts_with: String,
    consequence: String,
}

impl AppSnapshotV1 {
    pub(crate) fn from_desktop(
        snapshot: DesktopSnapshotV1,
        research_library: ResearchLibrarySnapshotV1,
    ) -> Result<Self, &'static str> {
        snapshot.validate().map_err(|error| error.code())?;
        let can_apply = snapshot.capabilities.apply;
        let project_available =
            research_library.health != qiongli_project::LibraryHealth::InspectionBlocked;
        let trust = match (snapshot.product.trust, can_apply) {
            (ProductTrustView::PackagedProductControl, _) => AppTrustView {
                mode: "packaged-product",
                label: "Verified packaged product",
                can_apply: true,
                reason_code: "verified-packaged-product-control",
            },
            (ProductTrustView::SourceBuild, true) => AppTrustView {
                mode: "local-installable",
                label: "Local candidate authority",
                can_apply: true,
                reason_code: "local-authority-session",
            },
            (ProductTrustView::SourceBuild, false) => AppTrustView {
                mode: "source-read-only",
                label: "Source build — client changes inspect only",
                can_apply: false,
                reason_code: "source-build-read-only",
            },
        };
        Ok(Self {
            schema_version: APP_API_SCHEMA_VERSION,
            product: AppProductView {
                version: snapshot.product.version,
                build: snapshot.product.build,
                operating_system: snapshot.product.operating_system.label().to_owned(),
                architecture: snapshot.product.architecture.label().to_owned(),
                trust,
            },
            content: AppContentView {
                status: snapshot.content.status.code(),
                pack_id: snapshot.content.pack_id,
                content_version: snapshot.content.content_version,
                entry_count: snapshot.content.entry_count,
                profiles: snapshot
                    .content
                    .profiles
                    .into_iter()
                    .map(|profile| AppProfileView {
                        id: profile.profile.id(),
                        label: profile_label(profile.profile),
                        description: profile.profile.description(),
                        included_resource_kinds: profile.included_resource_kinds,
                    })
                    .collect(),
            },
            mcp: AppMcpView {
                status: snapshot.mcp.status.code(),
                profile: snapshot.mcp.profile.id(),
                public_tool_count: snapshot.mcp.public_tool_count,
            },
            configuration: AppConfigurationView {
                status: snapshot.config.status.code(),
                revision: snapshot.config.revision,
                legacy_credential: AppLegacyCredentialView {
                    reference_present: snapshot.config.openai_backend.secret_reference_present,
                    cleanup_available: snapshot.config.openai_backend.secret_reference_present,
                },
                cleanup_required: snapshot.config.cleanup_required,
            },
            update: app_update_view(snapshot.update),
            research_library,
            integrations: snapshot
                .integrations
                .into_iter()
                .map(app_integration_view)
                .collect(),
            capabilities: AppCapabilityView {
                refresh: snapshot.capabilities.refresh,
                skills_materialize: snapshot.capabilities.skills_materialize,
                integration_discovery: snapshot.capabilities.integration_discovery,
                integration_preview: snapshot.capabilities.integration_preview,
                project_library: project_available,
                project_mutation: project_available,
                capture_inbox: project_available,
                capture_mutation: project_available,
                academic_graph: project_available,
                orchestration_inspect: project_available,
                orchestration_control: project_available,
                legacy_credential_cleanup: snapshot.config.openai_backend.secret_reference_present,
                apply: snapshot.capabilities.apply,
            },
        })
    }
}

impl AppIntent {
    pub(crate) fn into_desktop(self) -> Result<DesktopIntent, &'static str> {
        Ok(match self {
            Self::Refresh => DesktopIntent::Refresh,
            Self::RefreshResearchLibrary
            | Self::SelectProjectDirectory
            | Self::SelectProjectCreateDestination { .. }
            | Self::PreviewProjectCreate { .. }
            | Self::PreviewProjectRegister { .. }
            | Self::OpenProject { .. }
            | Self::SelectProjectExportDestination { .. }
            | Self::PreviewProjectExport { .. }
            | Self::SelectProjectImportLocations { .. }
            | Self::PreviewProjectImport { .. }
            | Self::PreviewProjectRepairManifest { .. }
            | Self::PreviewProjectArchive { .. }
            | Self::PreviewProjectRestore { .. }
            | Self::PreviewProjectRefresh { .. }
            | Self::PreviewProjectUnregister { .. }
            | Self::LoadCaptureInbox { .. }
            | Self::LoadCaptureCoverage { .. }
            | Self::LoadArtifactChanges { .. }
            | Self::LoadAcademicGraph { .. }
            | Self::LoadAcademicGraphPortfolio
            | Self::QueryAcademicGraph { .. }
            | Self::QueryAcademicGraphPath { .. }
            | Self::OpenAcademicGraphArtifact { .. }
            | Self::ReadCapture { .. }
            | Self::SelectCaptureFile { .. }
            | Self::PreviewCaptureIntake { .. }
            | Self::PreviewCaptureConsolidation { .. } => {
                return Err("app-project-intent-not-intercepted");
            }
            Self::LoadOrchestration { .. }
            | Self::PreviewOrchestrationTest { .. }
            | Self::PreviewOrchestrationContinue { .. }
            | Self::ControlOrchestration { .. } => return Err("host-handoff-not-ready"),
            Self::RefreshIntegrationDiscovery => DesktopIntent::RefreshIntegrationDiscovery,
            Self::SelectUpdateStream { stream } => DesktopIntent::SelectUpdateStream {
                stream: stream.into_desktop(),
            },
            Self::CheckForUpdates => DesktopIntent::CheckForUpdates,
            Self::PrepareUpdate => DesktopIntent::PrepareUpdate,
            Self::PollUpdate => DesktopIntent::PollUpdate,
            Self::CancelUpdate => DesktopIntent::CancelUpdate,
            Self::PreviewUpdateInstall => DesktopIntent::PreviewUpdateInstall,
            Self::PreviewAgentBackendSettings { .. }
            | Self::PreviewAgentBackendCredential { .. }
            | Self::PreviewAgentRun { .. }
            | Self::TestOpenAiBackend => return Err("host-driven-execution-required"),
            Self::PreviewRemoveAgentBackendCredential => {
                DesktopIntent::PreviewAgentBackendSecretChange {
                    change: AgentBackendSecretChange::Remove,
                }
            }
            Self::PreviewInstallRecommended => DesktopIntent::PreviewInstallRecommended,
            Self::PreviewInstallSelected { selection } => DesktopIntent::PreviewInstallSelected {
                selection: selection.into_desktop(),
            },
            Self::VerifyIntegrations { selection } => DesktopIntent::VerifyIntegrations {
                selection: selection.into_desktop(),
            },
            Self::PreviewRepairAll => DesktopIntent::PreviewRepairAll,
            Self::PreviewUpdateIntegrations { selection } => {
                DesktopIntent::PreviewUpdateIntegrations {
                    selection: selection.into_desktop(),
                }
            }
            Self::PreviewRemoveIntegrations { selection } => {
                DesktopIntent::PreviewRemoveIntegrations {
                    selection: selection.into_desktop(),
                }
            }
            Self::PreviewSkillsPresetMaterialization { profile, preset } => {
                DesktopIntent::PreviewSkillsPresetMaterialization {
                    profile: profile.into_desktop(),
                    preset: preset.into_desktop(),
                }
            }
            Self::VerifySkillsPreset { preset } => DesktopIntent::VerifySkillsPreset {
                preset: preset.into_desktop(),
            },
            Self::PreviewSkillsPresetRemoval { preset } => {
                DesktopIntent::PreviewSkillsPresetRemoval {
                    preset: preset.into_desktop(),
                }
            }
            Self::ConfirmOperation { token } => DesktopIntent::ConfirmOperation {
                token: parse_operation_token(&token)?,
            },
            Self::CancelOperation { token } => DesktopIntent::CancelOperation {
                token: parse_operation_token(&token)?,
            },
        })
    }
}

pub(crate) fn app_portable_operation_preview(
    token: String,
    preview: &PortableProjectPreviewV1,
) -> AppOperationPreview {
    let (kind, title, summary) = match preview.operation {
        PortableProjectOperation::Export => (
            "project-export",
            "Export portable article project",
            "Copy portable academic artifacts into a verified directory package. Private paths, recognizable credential files, client state, sessions, chats, and transcripts are excluded.",
        ),
        PortableProjectOperation::Import => (
            "project-import",
            "Import portable article project",
            "Verify every packaged artifact, create a new local project directory, and register the preserved project identity in this Research Library.",
        ),
    };
    AppOperationPreview {
        token,
        kind,
        title,
        summary,
        display_target: Some(preview.destination_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
    }
}

pub(crate) fn app_project_operation_preview(
    token: String,
    preview: &ProjectMutationPreviewV1,
) -> AppOperationPreview {
    let (kind, title, summary) = match preview.operation {
        ProjectMutationKind::Register => (
            "project-register",
            "Register article project",
            "Add this portable article project to the private local Research Library. Existing academic artifacts will not be rewritten.",
        ),
        ProjectMutationKind::Archive => (
            "project-archive",
            "Archive article project",
            "Archive this project in the Research Library without deleting its directory or academic artifacts.",
        ),
        ProjectMutationKind::Restore => (
            "project-restore",
            "Restore article project",
            "Return this archived project to the active Research Library.",
        ),
        ProjectMutationKind::Refresh => (
            "project-refresh",
            "Refresh academic revision",
            "Inspect the canonical article artifacts and advance the semantic revision only when their academic content changed.",
        ),
        ProjectMutationKind::Unregister => (
            "project-unregister",
            "Unregister article project",
            "Remove only the private Research Library entry. The portable project manifest and all academic artifacts remain untouched.",
        ),
        ProjectMutationKind::Create => (
            "project-create",
            "Create article project",
            "Create and register a portable article project after explicit confirmation.",
        ),
        ProjectMutationKind::RepairManifest => (
            "project-repair-manifest",
            "Repair portable project manifest",
            "Rebuild the missing portable manifest from the private Research Library entry without changing academic artifacts.",
        ),
    };
    AppOperationPreview {
        token,
        kind,
        title,
        summary,
        display_target: Some(preview.root_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
    }
}

pub(crate) fn app_capture_intake_operation_preview(
    token: String,
    file_label: String,
    preview: &CaptureIntakePreviewV1,
) -> AppOperationPreview {
    let can_confirm = preview.effect == CaptureIntakeEffect::AppendPendingHistory;
    AppOperationPreview {
        token,
        kind: "capture-intake",
        title: "Import research capture",
        summary: "Verify and append this bounded research capture to the selected project's portable review history. No session, transcript, or private host path is retained.",
        display_target: Some(file_label),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: if can_confirm {
            vec!["filesystem-write"]
        } else {
            Vec::new()
        },
        can_confirm,
        blocked_reason: (!can_confirm).then_some("capture-already-intaken"),
    }
}

pub(crate) fn app_capture_consolidation_operation_preview(
    token: String,
    preview: &CaptureConsolidationPreviewV1,
) -> AppOperationPreview {
    let can_confirm = preview.outcome == CaptureConsolidationOutcome::Ready;
    let blocked_reason = match preview.outcome {
        CaptureConsolidationOutcome::Ready => None,
        CaptureConsolidationOutcome::Conflicted => Some("academic-review-conflict"),
        CaptureConsolidationOutcome::AlreadyConsolidated => Some("capture-already-consolidated"),
    };
    AppOperationPreview {
        token,
        kind: "capture-consolidation",
        title: "Consolidate reviewed capture",
        summary: "Apply only the reviewed academic deltas shown in this plan to the canonical research state and decision log, with a portable consolidation receipt.",
        display_target: Some(preview.capture_id.as_str().to_owned()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: if can_confirm {
            vec!["academic-consolidation", "filesystem-write"]
        } else {
            Vec::new()
        },
        can_confirm,
        blocked_reason,
    }
}

impl From<ResearchCaptureV1> for AppResearchCaptureV1 {
    fn from(capture: ResearchCaptureV1) -> Self {
        Self {
            schema_version: capture.schema_version,
            capture_id: capture.capture_id.as_str().to_owned(),
            binding: capture.binding.into(),
            source: capture.source,
            delivery: capture.delivery,
            captured_at_unix: capture.captured_at_unix,
            summary: capture.summary,
            changes: capture.changes.into_iter().map(Into::into).collect(),
            decisions: capture.decisions.into_iter().map(Into::into).collect(),
            evidence: capture.evidence.into_iter().map(Into::into).collect(),
            contradictions: capture.contradictions.into_iter().map(Into::into).collect(),
            next_actions: capture.next_actions,
        }
    }
}

impl From<ProjectBindingV1> for AppCaptureBindingV1 {
    fn from(binding: ProjectBindingV1) -> Self {
        Self {
            schema_version: binding.schema_version,
            project_id: binding.project_id.as_str().to_owned(),
            base_revision: binding.base_revision,
            stage: binding.stage,
            task: binding.task,
            capture_policy: binding.capture_policy,
        }
    }
}

impl From<SemanticChangeV1> for AppSemanticChangeV1 {
    fn from(change: SemanticChangeV1) -> Self {
        Self {
            area: change.area,
            summary: change.summary,
        }
    }
}

impl From<DecisionCandidateV1> for AppDecisionCandidateV1 {
    fn from(decision: DecisionCandidateV1) -> Self {
        Self {
            relation: decision.relation,
            statement: decision.statement,
            rationale: decision.rationale,
            target: decision.target,
        }
    }
}

impl From<EvidenceReferenceV1> for AppEvidenceReferenceV1 {
    fn from(evidence: EvidenceReferenceV1) -> Self {
        Self {
            locator_kind: evidence.locator_kind,
            locator: evidence.locator,
            relevance: evidence.relevance,
            limitation: evidence.limitation,
        }
    }
}

impl From<ContradictionV1> for AppContradictionV1 {
    fn from(contradiction: ContradictionV1) -> Self {
        Self {
            statement: contradiction.statement,
            conflicts_with: contradiction.conflicts_with,
            consequence: contradiction.consequence,
        }
    }
}

impl AppIntegrationSelection {
    const fn into_desktop(self) -> IntegrationSelection {
        IntegrationSelection {
            codex: self.codex,
            claude_code: self.claude_code,
        }
    }
}

impl AppProfileId {
    const fn into_desktop(self) -> ProfileKind {
        match self {
            Self::SkillOnly => ProfileKind::SkillOnly,
            Self::MarketplaceLite => ProfileKind::MarketplaceLite,
            Self::Full => ProfileKind::Full,
        }
    }
}

impl AppSkillsPreset {
    const fn into_desktop(self) -> SkillsDestinationPreset {
        match self {
            Self::QiongliManaged => SkillsDestinationPreset::QiongliManaged,
            Self::DetectedCodex => SkillsDestinationPreset::DetectedCodex,
            Self::DetectedClaudeCode => SkillsDestinationPreset::DetectedClaudeCode,
            Self::CurrentProject => SkillsDestinationPreset::CurrentProject,
        }
    }
}

impl AppUpdateStream {
    const fn into_desktop(self) -> UpdateStreamView {
        match self {
            Self::Stable => UpdateStreamView::Stable,
            Self::Beta => UpdateStreamView::Beta,
        }
    }
}

pub(crate) fn app_event(
    event: DesktopEvent,
    service: &mut dyn DesktopService,
    research_library: ResearchLibrarySnapshotV1,
) -> Result<AppEvent, &'static str> {
    Ok(match event {
        DesktopEvent::SnapshotReplaced(snapshot) => AppEvent::Snapshot {
            snapshot: AppSnapshotV1::from_desktop(*snapshot, research_library)?,
        },
        DesktopEvent::PreviewReady(preview) => AppEvent::Preview {
            preview: app_operation_preview(preview)?,
        },
        DesktopEvent::AgentRunCompleted(_) => AppEvent::Failed {
            code: "app-api-event-unsupported",
        },
        DesktopEvent::Completed { code } => AppEvent::Completed {
            code,
            snapshot: AppSnapshotV1::from_desktop(service.snapshot(), research_library)?,
        },
        DesktopEvent::Cancelled { code } => AppEvent::Cancelled { code },
        DesktopEvent::ValidationFailed { code } => AppEvent::ValidationFailed { code },
        DesktopEvent::Failed { code } => AppEvent::Failed { code },
        DesktopEvent::UpdateChanged {
            update,
            close_requested,
        } => AppEvent::UpdateChanged {
            update: app_update_view(update),
            close_requested,
        },
        DesktopEvent::McpSelfTestUpdated(_) | DesktopEvent::SkillsDestinationSelected { .. } => {
            AppEvent::Failed {
                code: "app-api-event-unsupported",
            }
        }
    })
}

fn app_update_view(update: UpdateView) -> AppUpdateView {
    AppUpdateView {
        status: update.status.code(),
        selected_stream: update_stream_id(update.selected_stream),
        phase: update_phase_id(update.phase),
        available_version: update.available_version,
        archive_size_bytes: update.archive_size_bytes,
        progress: update.progress.map(|progress| AppUpdateProgressView {
            completed_steps: progress.completed_steps,
            total_steps: progress.total_steps,
            label: progress.label,
            indeterminate: progress.indeterminate,
        }),
        reason_code: update.reason_code,
        remediation: update.remediation.code(),
        can_select_stream: update.can_select_stream,
        can_check: update.can_check,
        can_prepare: update.can_prepare,
        can_install: update.can_install,
        can_cancel: update.can_cancel,
    }
}

const fn update_stream_id(stream: UpdateStreamView) -> &'static str {
    match stream {
        UpdateStreamView::Stable => "stable",
        UpdateStreamView::Beta => "beta",
    }
}

const fn update_phase_id(phase: UpdatePhaseView) -> &'static str {
    match phase {
        UpdatePhaseView::Unavailable => "unavailable",
        UpdatePhaseView::Idle => "idle",
        UpdatePhaseView::Checking => "checking",
        UpdatePhaseView::Current => "current",
        UpdatePhaseView::Available => "available",
        UpdatePhaseView::Downloading => "downloading",
        UpdatePhaseView::Verifying => "verifying",
        UpdatePhaseView::Staging => "staging",
        UpdatePhaseView::ReadyToInstall => "ready-to-install",
        UpdatePhaseView::Installing => "installing",
        UpdatePhaseView::AwaitingRestart => "awaiting-restart",
        UpdatePhaseView::Cancelling => "cancelling",
        UpdatePhaseView::Cancelled => "cancelled",
        UpdatePhaseView::RecoveryRequired => "recovery-required",
        UpdatePhaseView::Failed => "failed",
    }
}

fn app_integration_view(integration: IntegrationView) -> AppIntegrationView {
    let connection = app_connection_view(&integration);
    let legacy_detected = integration.paths[..integration.path_count]
        .iter()
        .flatten()
        .any(|path| {
            path.source == qiongli_ui::IntegrationPathSourceView::LegacyObserved
                && path.state != StatusCode::Missing
        });
    AppIntegrationView {
        target: integration_target_id(integration.target),
        label: integration.target.label(),
        connection,
        client: AppClientView {
            detected: integration.client == StatusCode::Ready,
            status: integration.client.code(),
            version: integration.client_version.map(|version| version.label()),
            compatibility: integration.compatibility.code(),
            minimum_supported_version: minimum_supported_client_version(integration.target),
        },
        plugin: AppPluginVersionView {
            installed_version: integration
                .installed_plugin_version
                .map(|version| version.label()),
            available_version: integration.available_plugin_version.label(),
        },
        discovery: integration.discovery.label(),
        candidate_required: integration.candidate_required,
        legacy_detected,
        overall: integration.overall.code(),
        managed_content: AppManagedContentView {
            source: integration.source.code(),
            skills: integration.skills.code(),
            marketplace: integration.marketplace.code(),
            direct_package: integration.direct_package.map(StatusCode::code),
            registration: integration.registration.code(),
            activation: integration.activation_status.code(),
            activation_observation: integration.activation_observation.code(),
            mcp_attachment: integration.mcp_attachment.code(),
            mcp_attachment_observation: integration.mcp_attachment_observation.code(),
        },
        symbolic_location: integration.symbolic_location.label(),
        activation_policy: integration.activation.label(),
        ownership: integration.ownership.label(),
        next_action: integration.next_action.label(),
        evidence_code: integration.evidence_code,
        paths: integration
            .paths
            .into_iter()
            .take(integration.path_count)
            .flatten()
            .map(app_integration_path_view)
            .collect(),
    }
}

fn app_connection_view(integration: &IntegrationView) -> AppConnectionView {
    if integration.compatibility == qiongli_ui::ClientCompatibilityView::Unsupported {
        return AppConnectionView {
            state: "unsupported-client-version",
            label: "Unsupported client version",
            reason_code: "client-version-below-supported-minimum",
        };
    }
    if integration.discovery == qiongli_ui::IntegrationDiscoveryState::Unavailable {
        return AppConnectionView {
            state: "inspection-blocked",
            label: "Inspection blocked",
            reason_code: integration.evidence_code,
        };
    }
    if integration.client != StatusCode::Ready {
        return AppConnectionView {
            state: "client-not-detected",
            label: "Client not detected",
            reason_code: integration.evidence_code,
        };
    }
    if matches!(
        integration.overall,
        StatusCode::Drifted
            | StatusCode::Conflict
            | StatusCode::RecoveryRequired
            | StatusCode::Invalid
            | StatusCode::Insecure
    ) {
        return AppConnectionView {
            state: "needs-repair",
            label: "Needs repair",
            reason_code: integration.evidence_code,
        };
    }
    if integration.registration == StatusCode::Ready
        && integration.activation_observation == qiongli_ui::IntegrationObservationView::Observed
        && integration.mcp_attachment_observation
            == qiongli_ui::IntegrationObservationView::Observed
    {
        return AppConnectionView {
            state: "connected",
            label: "Connected",
            reason_code: "qiongli-plugin-observed-healthy",
        };
    }
    AppConnectionView {
        state: "detected-not-connected",
        label: "Detected, not connected",
        reason_code: integration.evidence_code,
    }
}

const fn minimum_supported_client_version(target: IntegrationTarget) -> &'static str {
    match target {
        IntegrationTarget::Codex => "0.144.1",
        IntegrationTarget::ClaudeCode => "2.1.206",
    }
}

fn app_integration_path_view(path: IntegrationPathView) -> AppIntegrationPathView {
    AppIntegrationPathView {
        surface: path.surface.label(),
        scope: path.scope.label(),
        source: path.source.label(),
        state: path.state.code(),
        management: path.management.label(),
        selected: path.selected,
        symbolic_path: path.symbolic_path,
    }
}

fn app_operation_preview(preview: OperationPreview) -> Result<AppOperationPreview, &'static str> {
    if !preview.validate() {
        return Err("operation-preview-invalid");
    }
    Ok(AppOperationPreview {
        token: format!("{:032x}", preview.token.value()),
        kind: operation_kind_id(preview.kind),
        title: preview.title,
        summary: preview.summary,
        display_target: preview
            .display_target
            .map(|value| value.expose().to_owned()),
        plan_digest_sha256: preview.plan_digest_sha256,
        approvals_required: preview
            .approvals_required
            .into_iter()
            .map(OperationApproval::label)
            .collect(),
        can_confirm: preview.can_confirm,
        blocked_reason: preview.blocked_reason,
    })
}

fn parse_operation_token(value: &str) -> Result<OperationToken, &'static str> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("operation-token-invalid");
    }
    u128::from_str_radix(value, 16)
        .map(OperationToken::new)
        .map_err(|_| "operation-token-invalid")
}

const fn profile_label(profile: ProfileKind) -> &'static str {
    match profile {
        ProfileKind::SkillOnly => "Skills",
        ProfileKind::MarketplaceLite => "Plugin Lite",
        ProfileKind::Full => "Full workflow",
    }
}

const fn integration_target_id(target: IntegrationTarget) -> &'static str {
    match target {
        IntegrationTarget::Codex => "codex",
        IntegrationTarget::ClaudeCode => "claude-code",
    }
}

const fn operation_kind_id(kind: OperationKind) -> &'static str {
    match kind {
        OperationKind::Activation => "activation",
        OperationKind::GlobalSettings => "global-settings",
        OperationKind::ProviderSettings => "provider-settings",
        OperationKind::ProviderSecret => "provider-secret",
        OperationKind::AgentBackendSettings => "agent-backend-settings",
        OperationKind::AgentBackendSecret => "agent-backend-secret",
        OperationKind::AgentRun => "agent-run",
        OperationKind::SkillsMaterialization => "skills-materialization",
        OperationKind::SkillsRemoval => "skills-removal",
        OperationKind::UpdateInstall => "update-install",
    }
}

fn deserialize_private_text<'de, D>(deserializer: D) -> Result<PrivateText, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(PrivateText::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parameterized_app_intents_accept_only_camel_case_fields() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-project-create",
            "directoryToken": "0000000000000000000000000000002a",
            "displayName": "Trustworthy research agents",
            "projectKind": "article",
            "stage": "writing"
        }))
        .expect("the TypeScript App API field casing must deserialize");

        assert!(matches!(
            intent,
            AppIntent::PreviewProjectCreate {
                directory_token,
                display_name,
                project_kind: ProjectKind::Article,
                stage: ProjectStage::Writing,
            } if directory_token == "0000000000000000000000000000002a"
                && display_name == "Trustworthy research agents"
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "read-capture",
                "project_id": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "capture_id": format!("cap_{}", "a".repeat(64))
            }))
            .is_err(),
            "snake_case fields must not become a second IPC contract"
        );

        let graph_open = serde_json::from_value::<AppIntent>(json!({
            "action": "open-academic-graph-artifact",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
            "entity": { "kind": "edge", "id": format!("edg_{}", "b".repeat(64)) }
        }))
        .expect("graph artifact opening must deserialize without accepting a path");
        assert!(matches!(
            graph_open,
            AppIntent::OpenAcademicGraphArtifact {
                expected_project_revision: 12,
                entity: AppAcademicGraphEntity::Edge { .. },
                ..
            }
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "open-academic-graph-artifact",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
                "entity": { "kind": "node", "id": format!("nod_{}", "b".repeat(64)) },
                "artifactPath": "/private/research/context/research_state.md"
            }))
            .is_err()
        );

        let graph_path = serde_json::from_value::<AppIntent>(json!({
            "action": "query-academic-graph-path",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "query": {
                "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
                "sourceNodeId": format!("nod_{}", "b".repeat(64)),
                "targetNodeId": format!("nod_{}", "c".repeat(64)),
                "maxHops": 6
            }
        }))
        .expect("graph path query must deserialize through the typed query contract");
        assert!(matches!(
            graph_path,
            AppIntent::QueryAcademicGraphPath {
                query: AcademicGraphPathQueryV1 { max_hops: 6, .. },
                ..
            }
        ));

        let orchestration_control = serde_json::from_value::<AppIntent>(json!({
            "action": "control-orchestration",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "runId": format!("run_{}", "2".repeat(32)),
            "expectedGeneration": 3,
            "expectedDocumentSha256": "3".repeat(64),
            "actionName": "pause"
        }))
        .expect("orchestration control must deserialize through the checkpoint reference");
        assert!(matches!(
            orchestration_control,
            AppIntent::ControlOrchestration {
                expected_project_revision: 12,
                expected_generation: 3,
                action_name: AppOrchestrationControlAction::Pause,
                ..
            }
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "control-orchestration",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "runId": format!("run_{}", "2".repeat(32)),
                "expectedGeneration": 3,
                "expectedDocumentSha256": "3".repeat(64),
                "action_name": "pause"
            }))
            .is_err(),
            "snake_case orchestration controls must not become a second IPC contract"
        );
    }

    #[test]
    fn legacy_backend_credential_intent_is_parsed_but_rejected_by_default() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-agent-backend-credential",
            "apiKey": "openai-private-api-canary"
        }))
        .expect("the legacy credential request must remain parseable during migration");

        assert_eq!(
            intent.into_desktop().err(),
            Some("host-driven-execution-required")
        );
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "preview-agent-backend-credential",
                "api_key": "wrong-field"
            }))
            .is_err()
        );
    }

    #[test]
    fn legacy_agent_run_intent_is_parsed_but_rejected_by_default() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-agent-run",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "prompt": "private-agent-run-prompt-canary"
        }))
        .expect("the legacy agent run request must remain parseable during migration");

        assert_eq!(
            intent.into_desktop().err(),
            Some("host-driven-execution-required")
        );
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "preview-agent-run",
                "project_id": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "prompt": "wrong-field"
            }))
            .is_err()
        );
    }

    #[test]
    fn legacy_backend_configuration_and_test_intents_are_rejected_by_default() {
        let settings = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-agent-backend-settings",
            "expectedRevision": 4,
            "enabled": true
        }))
        .expect("the legacy settings request must remain parseable during migration");
        assert_eq!(
            settings.into_desktop().err(),
            Some("host-driven-execution-required")
        );

        let test = serde_json::from_value::<AppIntent>(json!({
            "action": "test-open-ai-backend"
        }))
        .expect("the legacy test request must remain parseable during migration");
        assert_eq!(
            test.into_desktop().err(),
            Some("host-driven-execution-required")
        );
    }

    #[test]
    fn legacy_backend_credential_removal_remains_available_for_cleanup() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-remove-agent-backend-credential"
        }))
        .expect("credential cleanup must remain available");

        assert!(matches!(
            intent.into_desktop(),
            Ok(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Remove,
            })
        ));
    }

    #[test]
    fn legacy_orchestration_intents_require_the_host_handoff_boundary() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-orchestration-continue",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "runId": format!("run_{}", "2".repeat(32)),
            "expectedGeneration": 3,
            "expectedDocumentSha256": "3".repeat(64)
        }))
        .expect("the legacy orchestration request must remain parseable during migration");

        assert_eq!(intent.into_desktop().err(), Some("host-handoff-not-ready"));
    }

    #[test]
    fn parameterized_app_events_serialize_camel_case_fields() {
        let selected = serde_json::to_value(AppEvent::ProjectDirectorySelected {
            token: "0000000000000000000000000000002a".to_owned(),
            root_label: "trustworthy-research-agents".to_owned(),
        })
        .expect("app event must serialize");
        assert_eq!(
            selected,
            json!({
                "type": "project-directory-selected",
                "token": "0000000000000000000000000000002a",
                "rootLabel": "trustworthy-research-agents"
            })
        );

        let capture = serde_json::to_value(AppEvent::CaptureFileSelected {
            token: "0000000000000000000000000000002b".to_owned(),
            file_label: "portable-research-capture.json".to_owned(),
        })
        .expect("capture event must serialize");
        assert!(capture.get("fileLabel").is_some());
        assert!(capture.get("file_label").is_none());

        let update = serde_json::to_value(AppEvent::UpdateChanged {
            update: app_update_view(UpdateView {
                status: StatusCode::Ready,
                selected_stream: UpdateStreamView::Stable,
                phase: UpdatePhaseView::Idle,
                available_version: None,
                archive_size_bytes: None,
                progress: None,
                reason_code: "update-ready",
                remediation: qiongli_ui::UpdateRemediation::None,
                can_select_stream: true,
                can_check: true,
                can_prepare: false,
                can_install: false,
                can_cancel: false,
            }),
            close_requested: true,
        })
        .expect("update event must serialize");
        assert_eq!(update.get("closeRequested"), Some(&json!(true)));
        assert!(update.get("close_requested").is_none());
    }

    #[test]
    fn main_window_capability_is_limited_to_update_handoff_close() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json"))
                .expect("main window capability must be valid JSON");

        assert_eq!(
            capability.get("permissions"),
            Some(&json!(["core:window:allow-close"]))
        );
    }

    #[test]
    fn operation_tokens_have_one_canonical_ipc_encoding() {
        let token = parse_operation_token("0000000000000000000000000000002a").unwrap();
        assert_eq!(token.value(), 42);
        assert_eq!(parse_operation_token("2a"), Err("operation-token-invalid"));
        assert_eq!(
            parse_operation_token("0000000000000000000000000000002A"),
            Err("operation-token-invalid")
        );
    }

    #[test]
    fn integration_selection_maps_without_hidden_defaults() {
        assert_eq!(
            AppIntegrationSelection {
                codex: true,
                claude_code: false,
            }
            .into_desktop(),
            IntegrationSelection {
                codex: true,
                claude_code: false,
            }
        );
    }

    #[test]
    fn update_state_maps_to_the_bounded_app_contract() {
        let update = app_update_view(UpdateView {
            status: StatusCode::Ready,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::Available,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: Some(24_600_000),
            progress: None,
            reason_code: "trusted-update-available",
            remediation: qiongli_ui::UpdateRemediation::RetryPreparation,
            can_select_stream: true,
            can_check: true,
            can_prepare: true,
            can_install: false,
            can_cancel: false,
        });

        assert_eq!(update.selected_stream, "beta");
        assert_eq!(update.phase, "available");
        assert_eq!(update.available_version.as_deref(), Some("2.0.0-alpha.2"));
        assert_eq!(update.remediation, "retry-update-preparation");
        assert!(update.can_prepare);
    }
}
