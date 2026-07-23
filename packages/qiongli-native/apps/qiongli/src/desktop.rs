#![cfg_attr(
    test,
    allow(
        dead_code,
        reason = "the Tauri shell is excluded from the core library unit-test binary"
    )
)]

use qiongli_config::{
    ConfigError, ConfigState, EmailAddress, GlobalSettings, ProviderReadiness,
    RedactedProviderStatus, SecretRef, SecretStore, SecretStoreStatus, SecretValue,
    UpdateStateStore, UpdateStreamPreference, UpdateTransactionPhase,
};
use qiongli_content::{
    EmbeddedContent, MaterializationReceiptV1, MaterializationTarget, ProfileId,
    approve_materialization_target, remove_materialization, verify_materialization,
};
use qiongli_execution::{
    AgentFinishReason, AgentRunResultV1, BackendControlService, BackendReadinessV1,
    CancellationToken as AgentCancellationToken, OrchestrationExecutionMode, openai_backend_status,
};
use qiongli_platform::{
    ApprovalRequirement, Architecture, ClientActionReadiness, ClientActivationCoordinator,
    ClientActivationDisposition, ClientActivationHandle, ClientActivationPreview,
    ClientActivationTarget, ClientComponentState, ClientDiscoveryState, ClientInventoryEntryV1,
    ClientKind, ClientOwnershipState, ClientPathManagement, ClientPathScope, ClientPathSource,
    ClientPathState, ClientPathSurface, InstallPlanMetadataV1, OperatingSystem,
    PackagedProductBatchInstallPreview, PackagedProductInstallEffect,
    PackagedProductInstallPreview, PackagedProductInstallVerification,
    PackagedProductVerificationInput, TrustedPublicKey, VerifiedLaunchGrant,
    VerifiedNativeReleaseCandidate, VerifiedPackagedProduct, apply_native_release_candidate_local,
    apply_packaged_product_batch_install, apply_packaged_product_install, approve_install_plan,
    packaged_product_control_path, preview_client_activation,
    preview_packaged_product_batch_install, preview_packaged_product_install,
    remove_packaged_product_install, verify_packaged_product, verify_packaged_product_install,
};
use qiongli_runtime::mcp::{LiteMcpServer, MCP_PROTOCOL_VERSION};
use qiongli_runtime::providers::{ProviderAccess, ProviderAvailability, ProviderId};
use qiongli_runtime::{FullProjectToolRegistry, LITE_PUBLIC_TOOL_NAMES, LiteToolRegistry};
use qiongli_ui::{
    ActivationPolicy, AgentBackendReadinessView, AgentBackendSecretChange,
    AgentBackendSettingsPatch, AgentBackendView, AgentRunDraft, AgentRunResultView,
    ArchitectureView, CapabilityView, ClientCompatibilityView, ClientVersionView, ConfigView,
    ContentView, DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, DiagnosticPathView,
    EMPTY_INTEGRATION_PATHS, GlobalSettingsPatch, IntegrationActionView, IntegrationDiscoveryState,
    IntegrationObservationView, IntegrationOwnershipView, IntegrationPathManagementView,
    IntegrationPathScopeView, IntegrationPathSourceView, IntegrationPathSurfaceView,
    IntegrationPathView, IntegrationSelection, IntegrationTarget, IntegrationView,
    MAX_INTEGRATION_PATHS, McpSelfTestCheckId, McpSelfTestCheckView, McpSelfTestState,
    McpSelfTestView, McpView, OperatingSystemView, OperationApproval, OperationKind,
    OperationPreview, OperationToken, PrivateDisplayText, ProductTrustView,
    ProductVersionChannelView, ProductVersionView, ProductView, ProfileKind, ProfileView,
    ProviderKind, ProviderReadinessView, ProviderSecretChange, ProviderSettingsPatch, ProviderView,
    PublicSettingChange, RemediationCode, SkillsDestinationPreset, StatusCode, SymbolicLocation,
    UpdatePhaseView, UpdateProgressView, UpdateRemediation, UpdateStreamView, UpdateView,
};
use serde_json::json;
use sha2::{Digest, Sha256};

use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::agent_run::{FullAgentRunRequest, FullAgentRunService, readiness_reason_code};
use crate::command::{CommandEnvironment, config_root, config_store};
use crate::desktop_api::{
    AppOperationPreview, AppResearchCaptureV1, AppSnapshotV1,
    app_capture_consolidation_operation_preview, app_capture_intake_operation_preview,
    app_orchestration_operation_preview, app_portable_operation_preview,
    app_project_operation_preview, serialize_app_api_contract_fixture,
};
use crate::orchestration_control::{
    FullOrchestrationService, OrchestrationControlAction, OrchestrationDoctorViewV1,
    OrchestrationExecutionViewV1, OrchestrationRunListViewV1, OrchestrationRunReference,
    OrchestrationRunSummaryV1,
};
use qiongli_project::{
    AcademicGraphArtifactTarget, AcademicGraphComparisonService, AcademicGraphEntityKind,
    AcademicGraphIndexService, AcademicGraphPathQueryV1, AcademicGraphPathResultV1,
    AcademicGraphPortfolioService, AcademicGraphPortfolioSnapshotV1, AcademicGraphQueryResultV1,
    AcademicGraphQueryV1, AcademicGraphRevisionComparisonV1, AcademicGraphService,
    AcademicGraphSnapshotV1, ApprovedCaptureConsolidation, ApprovedCaptureIntake,
    ApprovedProjectMutation, ArtifactChangeSnapshotV1, CaptureConsolidationPreviewV1,
    CaptureCoverageSnapshotV1, CaptureId, CaptureInboxSnapshotV1, CaptureIntakePreviewV1,
    LibraryHealth, ProjectId, ProjectKind, ProjectMutationKind, ProjectRegistrationOptions,
    ProjectStage, ProjectStateService, ResearchLibrarySnapshotV1, VerifiedCaptureConsolidation,
    VerifiedCaptureIntake, VerifiedPortableProjectOperation, VerifiedProjectMutation,
    read_portable_capture_packet,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLaunchError;

#[cfg(not(test))]
mod tauri_adapter;
#[cfg(not(test))]
use tauri_adapter::run_tauri_application;

#[cfg(test)]
fn run_tauri_application(
    _service: NativeDesktopService,
    _project_service: Option<ProjectStateService>,
) -> Result<(), DesktopLaunchError> {
    Err(DesktopLaunchError)
}

const ACTIVATION_PLAN_TTL_SECONDS: u64 = 600;
const MCP_SELF_TEST_TIMEOUT: Duration = Duration::from_secs(5);
const ACTIVATION_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];

pub fn run_desktop(
    mut environment: CommandEnvironment,
    content: EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let product_control = running_packaged_product(&environment, &content);
    let service = NativeDesktopService::new_with_packaged_product(
        environment,
        content,
        Vec::new(),
        product_control,
    );
    run_tauri_application(service, project_service)
}

pub fn run_desktop_with_activation_sessions(
    mut environment: CommandEnvironment,
    content: EmbeddedContent,
    sessions: Vec<DesktopActivationSession>,
) -> Result<(), DesktopLaunchError> {
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    if sessions.len() > 2
        || sessions.iter().enumerate().any(|(index, session)| {
            sessions[..index]
                .iter()
                .any(|prior| prior.target == session.target)
        })
    {
        return Err(DesktopLaunchError);
    }
    let service = NativeDesktopService::new(environment, content, sessions);
    run_tauri_application(service, project_service)
}

pub fn run_desktop_with_candidate_sessions(
    mut environment: CommandEnvironment,
    content: EmbeddedContent,
    sessions: Vec<DesktopCandidateSession>,
) -> Result<(), DesktopLaunchError> {
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    if sessions.len() > 2
        || sessions.iter().enumerate().any(|(index, session)| {
            sessions[..index]
                .iter()
                .any(|prior| prior.target == session.target)
        })
    {
        return Err(DesktopLaunchError);
    }
    let service = NativeDesktopService::new_with_candidate_sessions(environment, content, sessions);
    run_tauri_application(service, project_service)
}

struct ProjectDesktopState {
    service: Option<ProjectStateService>,
    selected_location: Option<SelectedProjectLocation>,
    pending: Option<PendingProjectOperation>,
    academic_graph_history: BTreeMap<ProjectId, AcademicGraphSnapshotV1>,
}

struct OrchestrationDesktopState {
    service: Option<FullOrchestrationService>,
    pending: Option<PendingOrchestrationOperation>,
}

enum PendingOrchestrationOperation {
    Test {
        token: String,
        project_id: ProjectId,
        expected_project_revision: u64,
        execution_mode: OrchestrationExecutionMode,
    },
    Continue {
        token: String,
        reference: OrchestrationRunReference,
    },
}

impl OrchestrationDesktopState {
    fn new(projects: Option<ProjectStateService>, content: &EmbeddedContent) -> Self {
        let service = projects.and_then(|projects| {
            FullProjectToolRegistry::from_embedded_content(content)
                .ok()
                .and_then(|registry| {
                    FullOrchestrationService::from_embedded_content(projects, registry, content)
                        .ok()
                })
        });
        Self {
            service,
            pending: None,
        }
    }

    fn load(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        settings: &GlobalSettings,
        secrets: &dyn SecretStore,
    ) -> Result<(OrchestrationDoctorViewV1, OrchestrationRunListViewV1), &'static str> {
        let service = self
            .service
            .as_ref()
            .ok_or("orchestration-service-unavailable")?;
        let doctor = service
            .doctor_openai(project_id, expected_project_revision, settings, secrets)
            .map_err(|error| error.reason_code())?;
        let runs = service
            .list_runs(project_id, expected_project_revision)
            .map_err(|error| error.reason_code())?;
        Ok((doctor, runs))
    }

    fn preview_test(
        &mut self,
        project_id: ProjectId,
        expected_project_revision: u64,
        execution_mode: OrchestrationExecutionMode,
    ) -> Result<AppOperationPreview, &'static str> {
        if self.service.is_none() || expected_project_revision == 0 {
            return Err("orchestration-service-unavailable");
        }
        let token = project_app_token()?;
        let preview = app_orchestration_operation_preview(
            token.clone(),
            false,
            format!(
                "{} · revision {} · {}",
                project_id.as_str(),
                expected_project_revision,
                orchestration_mode_name(execution_mode)
            ),
            None,
        );
        self.pending = Some(PendingOrchestrationOperation::Test {
            token,
            project_id,
            expected_project_revision,
            execution_mode,
        });
        Ok(preview)
    }

    fn preview_continue(
        &mut self,
        reference: OrchestrationRunReference,
    ) -> Result<AppOperationPreview, &'static str> {
        if self.service.is_none() {
            return Err("orchestration-service-unavailable");
        }
        let token = project_app_token()?;
        let preview = app_orchestration_operation_preview(
            token.clone(),
            true,
            format!(
                "{} · generation {}",
                reference.run_id.as_str(),
                reference.expected_generation
            ),
            Some(reference.expected_document_sha256.clone()),
        );
        self.pending = Some(PendingOrchestrationOperation::Continue { token, reference });
        Ok(preview)
    }

    fn confirm(
        &mut self,
        token: &str,
        settings: &GlobalSettings,
        secrets: Arc<dyn SecretStore>,
    ) -> Option<Result<(OrchestrationExecutionViewV1, ProjectId, u64), &'static str>> {
        if self
            .pending
            .as_ref()
            .map(PendingOrchestrationOperation::token)
            != Some(token)
        {
            return None;
        }
        let pending = self.pending.take().expect("pending token checked above");
        let result = (|| {
            let service = self
                .service
                .as_ref()
                .ok_or("orchestration-service-unavailable")?;
            match pending {
                PendingOrchestrationOperation::Test {
                    project_id,
                    expected_project_revision,
                    execution_mode,
                    ..
                } => service
                    .start_openai_test(
                        project_id.clone(),
                        expected_project_revision,
                        execution_mode,
                        true,
                        settings,
                        secrets,
                    )
                    .map(|execution| (execution, project_id, expected_project_revision)),
                PendingOrchestrationOperation::Continue { reference, .. } => {
                    let project_id = reference.project_id.clone();
                    let expected_project_revision = reference.expected_project_revision;
                    service
                        .continue_openai(&reference, true, settings, secrets)
                        .map(|execution| (execution, project_id, expected_project_revision))
                }
            }
            .map_err(|error| error.reason_code())
        })();
        Some(result)
    }

    fn control_and_load(
        &self,
        reference: &OrchestrationRunReference,
        action: OrchestrationControlAction,
        settings: &GlobalSettings,
        secrets: &dyn SecretStore,
    ) -> Result<
        (
            OrchestrationRunSummaryV1,
            OrchestrationDoctorViewV1,
            OrchestrationRunListViewV1,
        ),
        &'static str,
    > {
        let service = self
            .service
            .as_ref()
            .ok_or("orchestration-service-unavailable")?;
        let run = service
            .control(reference, action)
            .map_err(|error| error.reason_code())?;
        let doctor = service
            .doctor_openai(
                &reference.project_id,
                reference.expected_project_revision,
                settings,
                secrets,
            )
            .map_err(|error| error.reason_code())?;
        let runs = service
            .list_runs(&reference.project_id, reference.expected_project_revision)
            .map_err(|error| error.reason_code())?;
        Ok((run, doctor, runs))
    }

    fn cancel(&mut self, token: &str) -> bool {
        if self
            .pending
            .as_ref()
            .map(PendingOrchestrationOperation::token)
            != Some(token)
        {
            return false;
        }
        self.pending = None;
        true
    }

    fn owns(&self, token: &str) -> bool {
        self.pending
            .as_ref()
            .map(PendingOrchestrationOperation::token)
            == Some(token)
    }
}

impl PendingOrchestrationOperation {
    fn token(&self) -> &str {
        match self {
            Self::Test { token, .. } | Self::Continue { token, .. } => token,
        }
    }
}

const fn orchestration_mode_name(mode: OrchestrationExecutionMode) -> &'static str {
    match mode {
        OrchestrationExecutionMode::Solo => "solo",
        OrchestrationExecutionMode::Duo => "duo",
        OrchestrationExecutionMode::Triad => "triad",
    }
}

enum SelectedProjectLocation {
    Register {
        token: String,
        root: PathBuf,
    },
    Create {
        token: String,
        root: PathBuf,
    },
    Export {
        token: String,
        project_id: ProjectId,
        destination: PathBuf,
    },
    Import {
        token: String,
        source: PathBuf,
        destination: PathBuf,
    },
    CaptureIntake {
        token: String,
        project_id: ProjectId,
        source: PathBuf,
    },
}

enum PendingProjectOperation {
    Mutation {
        token: String,
        plan: VerifiedProjectMutation,
    },
    Portable {
        token: String,
        plan: VerifiedPortableProjectOperation,
    },
    CaptureIntake {
        token: String,
        plan: Box<VerifiedCaptureIntake>,
    },
    CaptureConsolidation {
        token: String,
        plan: Box<VerifiedCaptureConsolidation>,
    },
}

#[derive(Debug, Eq, PartialEq)]
struct ConfirmedProjectOperation {
    code: &'static str,
    capture_project_id: Option<ProjectId>,
}

impl ProjectDesktopState {
    const fn new(service: Option<ProjectStateService>) -> Self {
        Self {
            service,
            selected_location: None,
            pending: None,
            academic_graph_history: BTreeMap::new(),
        }
    }

    fn snapshot(&self) -> ResearchLibrarySnapshotV1 {
        project_snapshot(&self.service)
    }

    fn capture_inbox(
        &self,
        project_id: &ProjectId,
    ) -> Result<CaptureInboxSnapshotV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .capture_inbox(project_id)
            .map_err(|error| error.reason_code())
    }

    fn capture_coverage(
        &self,
        project_id: &ProjectId,
    ) -> Result<CaptureCoverageSnapshotV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .capture_coverage(project_id)
            .map_err(|error| error.reason_code())
    }

    fn artifact_changes(
        &self,
        project_id: &ProjectId,
    ) -> Result<ArtifactChangeSnapshotV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .artifact_changes(project_id)
            .map_err(|error| error.reason_code())
    }

    fn academic_graph(
        &mut self,
        project_id: &ProjectId,
    ) -> Result<
        (
            AcademicGraphSnapshotV1,
            Option<AcademicGraphRevisionComparisonV1>,
        ),
        &'static str,
    > {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let graph = AcademicGraphService::new(projects.clone())
            .rebuild(project_id)
            .map_err(|error| error.reason_code())?;
        let comparison = self
            .academic_graph_history
            .get(project_id)
            .filter(|before| before.project_revision <= graph.project_revision)
            .map(|before| AcademicGraphComparisonService::compare(before, &graph))
            .transpose()
            .map_err(|error| error.reason_code())?;
        self.academic_graph_history
            .insert(project_id.clone(), graph.clone());
        Ok((graph, comparison))
    }

    fn query_academic_graph(
        &self,
        project_id: &ProjectId,
        query: &AcademicGraphQueryV1,
    ) -> Result<AcademicGraphQueryResultV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphIndexService::new(projects.clone())
            .rebuild(project_id)
            .and_then(|index| index.query(query))
            .map_err(|error| error.reason_code())
    }

    fn academic_graph_portfolio(&self) -> Result<AcademicGraphPortfolioSnapshotV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphPortfolioService::new(projects.clone())
            .rebuild()
            .map_err(|error| error.reason_code())
    }

    fn query_academic_graph_path(
        &self,
        project_id: &ProjectId,
        query: &AcademicGraphPathQueryV1,
    ) -> Result<AcademicGraphPathResultV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphIndexService::new(projects.clone())
            .rebuild(project_id)
            .and_then(|index| index.explanatory_path(query))
            .map_err(|error| error.reason_code())
    }

    fn resolve_academic_graph_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        expected_projection_id: &str,
        entity_kind: AcademicGraphEntityKind,
        entity_id: &str,
    ) -> Result<AcademicGraphArtifactTarget, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphService::new(projects.clone())
            .resolve_artifact(
                project_id,
                expected_project_revision,
                expected_projection_id,
                entity_kind,
                entity_id,
            )
            .map_err(|error| error.reason_code())
    }

    fn read_capture(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<AppResearchCaptureV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .read_capture(project_id, capture_id)
            .map_err(|error| error.reason_code())?
            .map(Into::into)
            .ok_or("capture-not-found")
    }

    fn select_register_root(&mut self, root: PathBuf) -> Result<(String, String), &'static str> {
        let root = resolve_selected_article_project_root(root)?;
        let token = project_app_token()?;
        let root_label = project_app_root_label(&root);
        self.selected_location = Some(SelectedProjectLocation::Register {
            token: token.clone(),
            root,
        });
        Ok((token, root_label))
    }

    fn select_create_root(&mut self, root: PathBuf) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&root);
        self.selected_location = Some(SelectedProjectLocation::Create {
            token: token.clone(),
            root,
        });
        Ok((token, root_label))
    }

    fn select_export_destination(
        &mut self,
        project_id: ProjectId,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::Export {
            token: token.clone(),
            project_id,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_import_locations(
        &mut self,
        source: PathBuf,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::Import {
            token: token.clone(),
            source,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_capture_file(
        &mut self,
        project_id: ProjectId,
        source: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let file_label = project_app_root_label(&source);
        self.selected_location = Some(SelectedProjectLocation::CaptureIntake {
            token: token.clone(),
            project_id,
            source,
        });
        Ok((token, file_label))
    }

    fn preview_create(
        &mut self,
        directory_token: &str,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Create { token, root }) = self.selected_location.take()
        else {
            return Err("project-create-destination-selection-invalid");
        };
        if token != directory_token {
            return Err("project-create-destination-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_create(
                root,
                ProjectRegistrationOptions::new(display_name, project_kind).with_stage(stage),
                now_unix()?,
            )
            .map_err(|error| error.reason_code())?;
        self.store_preview(plan)
    }

    fn preview_register(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Register { token, root }) = self.selected_location.take()
        else {
            return Err("project-directory-selection-invalid");
        };
        if token != directory_token {
            return Err("project-directory-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_register(root, ProjectRegistrationOptions::existing(), now_unix()?)
            .map_err(|error| error.reason_code())?;
        self.store_preview(plan)
    }

    fn preview_export(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Export {
            token,
            project_id,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-export-destination-selection-invalid");
        };
        if token != directory_token {
            return Err("project-export-destination-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_export(&project_id, destination)
            .map_err(|error| error.reason_code())?;
        self.store_portable_preview(plan)
    }

    fn preview_import(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Import {
            token,
            source,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-import-location-selection-invalid");
        };
        if token != directory_token {
            return Err("project-import-location-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_import(source, destination)
            .map_err(|error| error.reason_code())?;
        self.store_portable_preview(plan)
    }

    fn preview_capture_intake(
        &mut self,
        file_token: &str,
    ) -> Result<(CaptureIntakePreviewV1, AppOperationPreview), &'static str> {
        let Some(SelectedProjectLocation::CaptureIntake {
            token,
            project_id,
            source,
        }) = self.selected_location.take()
        else {
            return Err("capture-file-selection-invalid");
        };
        if token != file_token {
            return Err("capture-file-selection-invalid");
        }
        let file_label = project_app_root_label(&source);
        let capture = read_portable_capture_packet(source).map_err(|error| error.reason_code())?;
        if capture.binding.project_id != project_id {
            return Err("capture-project-mismatch");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture(capture)
            .map_err(|error| error.reason_code())?;
        let intake = plan.preview().clone();
        let token = project_app_token()?;
        let preview = app_capture_intake_operation_preview(token.clone(), file_label, &intake);
        self.pending = Some(PendingProjectOperation::CaptureIntake {
            token,
            plan: Box::new(plan),
        });
        Ok((intake, preview))
    }

    fn preview_capture_consolidation(
        &mut self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<(CaptureConsolidationPreviewV1, AppOperationPreview), &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture_consolidation(project_id, capture_id, now_unix()?)
            .map_err(|error| error.reason_code())?;
        let consolidation = plan.preview().clone();
        let token = project_app_token()?;
        let preview = app_capture_consolidation_operation_preview(token.clone(), &consolidation);
        self.pending = Some(PendingProjectOperation::CaptureConsolidation {
            token,
            plan: Box::new(plan),
        });
        Ok((consolidation, preview))
    }

    fn resolve_root(
        &self,
        project_id: &ProjectId,
    ) -> Result<qiongli_project::RegisteredProjectRoot, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .resolve_project_root(project_id)
            .map_err(|error| error.reason_code())
    }

    fn preview_lifecycle(
        &mut self,
        project_id: &ProjectId,
        operation: ProjectMutationKind,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = match operation {
            ProjectMutationKind::Archive => service.preview_archive(project_id),
            ProjectMutationKind::Restore => service.preview_restore(project_id),
            ProjectMutationKind::Refresh => service.preview_refresh(project_id, now_unix()?),
            ProjectMutationKind::RepairManifest => service.preview_repair_manifest(project_id),
            ProjectMutationKind::Unregister => service.preview_unregister(project_id),
            ProjectMutationKind::Register | ProjectMutationKind::Create => {
                return Err("project-operation-invalid");
            }
        }
        .map_err(|error| error.reason_code())?;
        self.store_preview(plan)
    }

    fn store_preview(
        &mut self,
        plan: VerifiedProjectMutation,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview = app_project_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::Mutation { token, plan });
        Ok(preview)
    }

    fn store_portable_preview(
        &mut self,
        plan: VerifiedPortableProjectOperation,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview = app_portable_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::Portable { token, plan });
        Ok(preview)
    }

    fn confirm(&mut self, token: &str) -> Option<Result<ConfirmedProjectOperation, &'static str>> {
        if self.pending.as_ref().map(PendingProjectOperation::token) != Some(token) {
            return None;
        }
        let pending = self.pending.take().expect("pending token checked above");
        let result = (|| {
            let service = self.service.as_ref().ok_or("project-service-unavailable")?;
            match pending {
                PendingProjectOperation::Mutation { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let commit = service
                        .apply(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: project_completion_code(commit.operation),
                        capture_project_id: None,
                    })
                }
                PendingProjectOperation::Portable { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let commit = service
                        .apply_portable(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: match commit.operation {
                            qiongli_project::PortableProjectOperation::Export => {
                                "project-export-completed"
                            }
                            qiongli_project::PortableProjectOperation::Import => {
                                "project-import-completed"
                            }
                        },
                        capture_project_id: None,
                    })
                }
                PendingProjectOperation::CaptureIntake { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let project_id = plan.preview().project_id.clone();
                    service
                        .apply_capture(
                            &plan,
                            &ApprovedCaptureIntake::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "capture-intake-completed",
                        capture_project_id: Some(project_id),
                    })
                }
                PendingProjectOperation::CaptureConsolidation { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let project_id = plan.preview().project_id.clone();
                    service
                        .apply_capture_consolidation(
                            &plan,
                            &ApprovedCaptureConsolidation::new(digest, true, true),
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "capture-consolidation-completed",
                        capture_project_id: Some(project_id),
                    })
                }
            }
        })();
        Some(result)
    }

    fn cancel(&mut self, token: &str) -> bool {
        if self.pending.as_ref().map(PendingProjectOperation::token) != Some(token) {
            return false;
        }
        self.pending = None;
        true
    }
}

impl PendingProjectOperation {
    fn token(&self) -> &str {
        match self {
            Self::Mutation { token, .. }
            | Self::Portable { token, .. }
            | Self::CaptureIntake { token, .. }
            | Self::CaptureConsolidation { token, .. } => token,
        }
    }
}

fn project_app_token() -> Result<String, &'static str> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "project-random-unavailable")?;
    let mut token = String::with_capacity(32);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut token, "{byte:02x}").map_err(|_| "project-token-invalid")?;
    }
    Ok(token)
}

fn project_app_root_label(root: &Path) -> String {
    root.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| {
            !value.is_empty() && value.len() <= 160 && !value.chars().any(char::is_control)
        })
        .unwrap_or("Article project")
        .to_owned()
}

fn resolve_selected_article_project_root(selected: PathBuf) -> Result<PathBuf, &'static str> {
    if selected
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        return Ok(selected);
    }

    let research_root = if selected
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        selected.clone()
    } else {
        selected.join("RESEARCH")
    };
    let metadata = match std::fs::symlink_metadata(&research_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(selected),
        Err(_) => return Err("article-project-discovery-failed"),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("article-project-research-root-unsafe");
    }

    let entries =
        std::fs::read_dir(&research_root).map_err(|_| "article-project-discovery-failed")?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| "article-project-discovery-failed")?;
        let file_type = entry
            .file_type()
            .map_err(|_| "article-project-discovery-failed")?;
        if file_type.is_dir() && !file_type.is_symlink() {
            candidates.push(entry.path());
        }
    }
    candidates.sort();
    match candidates.as_slice() {
        [root] => Ok(root.clone()),
        [] => Err("article-project-not-found-under-research"),
        _ => Err("multiple-article-projects-found-select-topic"),
    }
}

fn article_project_root_in_workspace(workspace: &Path, suggested_name: &str) -> PathBuf {
    if workspace
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        workspace.join(suggested_name)
    } else {
        workspace.join("RESEARCH").join(suggested_name)
    }
}

fn validate_project_dialog_name(value: &str) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > 160
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains(['/', '\\'])
    {
        Err("project-name-invalid")
    } else {
        Ok(())
    }
}

const fn project_completion_code(operation: ProjectMutationKind) -> &'static str {
    match operation {
        ProjectMutationKind::Register => "project-registration-completed",
        ProjectMutationKind::Create => "project-creation-completed",
        ProjectMutationKind::RepairManifest => "project-manifest-repair-completed",
        ProjectMutationKind::Archive => "project-archive-completed",
        ProjectMutationKind::Restore => "project-restore-completed",
        ProjectMutationKind::Refresh => "project-refresh-completed",
        ProjectMutationKind::Unregister => "project-unregister-completed",
    }
}

pub(crate) fn app_snapshot_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
) -> Result<String, &'static str> {
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    if content.pack().pack_sha256() != expected_content.pack().pack_sha256() {
        return Err("desktop-content-identity-mismatch");
    }
    let mut environment = environment.clone();
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let product_control = running_packaged_product(&environment, &content);
    let mut service = NativeDesktopService::new_with_packaged_product(
        environment,
        content,
        Vec::new(),
        product_control,
    );
    let snapshot =
        AppSnapshotV1::from_desktop(service.snapshot(), project_snapshot(&project_service))?;
    serde_json::to_string_pretty(&snapshot)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|_| "app-snapshot-serialization-failed")
}

/// Generates the deterministic, path-free Rust side of the frontend IPC contract gate.
///
/// This intentionally uses an empty environment rather than process discovery so the
/// fixture cannot depend on user configuration, installed clients, or network state.
#[doc(hidden)]
pub fn app_api_contract_fixture_json() -> Result<String, &'static str> {
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    let mut service = NativeDesktopService::new(CommandEnvironment::default(), content, Vec::new());
    let research_library = ResearchLibrarySnapshotV1 {
        schema_version: qiongli_project::RESEARCH_LIBRARY_SCHEMA_VERSION,
        revision: 0,
        health: LibraryHealth::Empty,
        projects: Vec::new(),
    };
    let snapshot = AppSnapshotV1::from_desktop(service.snapshot(), research_library)?;
    serialize_app_api_contract_fixture(snapshot)
}

pub(crate) fn validate_desktop_startup(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    let _window_entrypoint: fn(
        CommandEnvironment,
        EmbeddedContent,
    ) -> Result<(), DesktopLaunchError> = run_desktop;
    let owned_content = crate::embedded_content().map_err(|_| DesktopLaunchError)?;
    if owned_content.pack().pack_sha256() != content.pack().pack_sha256() {
        return Err(DesktopLaunchError);
    }
    let mut detected_environment = environment.clone();
    detected_environment.detect_client_versions();
    let project_service = project_state_service(&detected_environment);
    let mut service = NativeDesktopService::new(detected_environment, owned_content, Vec::new());
    service
        .snapshot()
        .validate()
        .map_err(|_| DesktopLaunchError)?;
    AppSnapshotV1::from_desktop(service.snapshot(), project_snapshot(&project_service))
        .map_err(|_| DesktopLaunchError)?;
    Ok(())
}

fn project_state_service(environment: &CommandEnvironment) -> Option<ProjectStateService> {
    crate::command::config_root(environment)
        .ok()
        .map(ProjectStateService::new)
}

fn project_snapshot(service: &Option<ProjectStateService>) -> ResearchLibrarySnapshotV1 {
    service
        .as_ref()
        .and_then(|service| service.snapshot().ok())
        .unwrap_or(ResearchLibrarySnapshotV1 {
            schema_version: qiongli_project::RESEARCH_LIBRARY_SCHEMA_VERSION,
            revision: 0,
            health: LibraryHealth::InspectionBlocked,
            projects: Vec::new(),
        })
}

pub struct DesktopActivationSession {
    target: IntegrationTarget,
    handle: ClientActivationHandle,
    grant: VerifiedLaunchGrant,
    trusted_keys: Vec<TrustedPublicKey>,
    minimum_generation: u64,
    pending: Option<ClientActivationPreview>,
}

impl DesktopActivationSession {
    #[must_use]
    pub fn new(
        handle: ClientActivationHandle,
        grant: VerifiedLaunchGrant,
        trusted_keys: Vec<TrustedPublicKey>,
        minimum_generation: u64,
    ) -> Self {
        let target = integration_target(handle.target());
        Self {
            target,
            handle,
            grant,
            trusted_keys,
            minimum_generation,
            pending: None,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        now_unix: u64,
    ) -> Result<OperationPreview, &'static str> {
        let expires_at_unix = now_unix
            .saturating_add(ACTIVATION_PLAN_TTL_SECONDS)
            .min(self.grant.grant().expires_at_unix);
        let plan_id = match self.target {
            IntegrationTarget::Codex => "desktop-activate-codex",
            IntegrationTarget::ClaudeCode => "desktop-activate-claude-code",
        };
        let preview = preview_client_activation(
            &self.handle,
            InstallPlanMetadataV1 {
                plan_id: plan_id.to_owned(),
                created_at_unix: now_unix,
                expires_at_unix,
            },
            &self.grant,
            &self.trusted_keys,
            self.minimum_generation,
            now_unix,
        )
        .map_err(|error| error.reason_code())?;
        let digest = preview.plan().plan().semantic_digest_sha256.clone();
        self.pending = Some(preview);
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match self.target {
                IntegrationTarget::Codex => "Codex activation preview",
                IntegrationTarget::ClaudeCode => "Claude Code activation preview",
            },
            summary: "Register the verified local Qiongli source. Client-owned enablement remains a host action.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(&mut self, now_unix: u64) -> Result<&'static str, &'static str> {
        let preview = self
            .pending
            .take()
            .ok_or("desktop-activation-preview-missing")?;
        let approval = approve_install_plan(preview.plan(), &ACTIVATION_APPROVALS, now_unix)
            .map_err(|error| error.reason_code())?;
        let commit = ClientActivationCoordinator::new(self.handle.clone())
            .apply(&preview, &approval, now_unix)
            .map_err(|error| error.reason_code())?;
        Ok(match commit.disposition {
            ClientActivationDisposition::Activated => "client-activation-applied",
            ClientActivationDisposition::AlreadyActive => "client-activation-already-active",
            ClientActivationDisposition::Repaired => "client-activation-repaired",
            ClientActivationDisposition::AlreadyHealthy => "client-activation-already-healthy",
        })
    }

    fn cancel(&mut self) {
        self.pending = None;
    }
}

impl Debug for DesktopActivationSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopActivationSession")
            .field("target", &self.target)
            .field("pending", &self.pending.is_some())
            .finish()
    }
}

pub struct DesktopCandidateSession {
    target: IntegrationTarget,
    candidate: VerifiedNativeReleaseCandidate,
    pending: bool,
}

impl DesktopCandidateSession {
    #[must_use]
    pub fn new(candidate: VerifiedNativeReleaseCandidate) -> Self {
        Self {
            target: integration_target(candidate.target()),
            candidate,
            pending: false,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        now_unix: u64,
    ) -> Result<OperationPreview, &'static str> {
        if now_unix < self.candidate.candidate().not_before_unix {
            return Err("native-release-candidate-not-yet-valid");
        }
        if now_unix >= self.candidate.candidate().expires_at_unix {
            return Err("native-release-candidate-expired");
        }
        self.pending = true;
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match self.target {
                IntegrationTarget::Codex => "Codex candidate installation preview",
                IntegrationTarget::ClaudeCode => "Claude Code candidate installation preview",
            },
            summary: "Install the verified native payload and fixed local Qiongli source, then register it. Client-owned enablement remains a host action.",
            display_target: None,
            plan_digest_sha256: Some(crate::candidate_cli::candidate_approval_digest(
                self.candidate.signed_payload_sha256(),
                self.candidate.target(),
            )),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(
        &mut self,
        content: &EmbeddedContent,
        home: &std::path::Path,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        if !std::mem::take(&mut self.pending) {
            return Err("desktop-candidate-preview-missing");
        }
        let commit =
            apply_native_release_candidate_local(content.pack(), &self.candidate, home, now_unix)
                .map_err(|error| error.reason_code())?;
        Ok(match commit.payload.disposition {
            qiongli_platform::InstallDisposition::Applied => "native-candidate-install-applied",
            qiongli_platform::InstallDisposition::AlreadyApplied => {
                "native-candidate-install-already-applied"
            }
            qiongli_platform::InstallDisposition::Repaired => "native-candidate-install-repaired",
            qiongli_platform::InstallDisposition::AlreadyHealthy => {
                "native-candidate-install-already-healthy"
            }
        })
    }

    fn cancel(&mut self) {
        self.pending = false;
    }
}

impl Debug for DesktopCandidateSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopCandidateSession")
            .field("target", &self.target)
            .field("candidate_digest", &"<verified-candidate>")
            .field("pending", &self.pending)
            .finish()
    }
}

#[derive(Clone, Copy)]
struct McpSelfTestCounts {
    enabled_providers: usize,
    ready_providers: usize,
    discovered_clients: usize,
    registered_clients: usize,
}

struct McpSelfTestInput {
    server: LiteMcpServer,
    counts: McpSelfTestCounts,
}

trait McpSelfTestExecutor: Send + Sync {
    fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView;
}

struct NativeMcpSelfTestExecutor;

impl McpSelfTestExecutor for NativeMcpSelfTestExecutor {
    fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView {
        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }

        let initialize = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }));
        let initialize_ready = initialize.as_ref().is_some_and(|response| {
            response
                .pointer("/result/protocolVersion")
                .and_then(|value| value.as_str())
                == Some(MCP_PROTOCOL_VERSION)
                && response.pointer("/result/capabilities/tools").is_some()
        });

        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }
        let tools = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }));
        let tools_ready = tools.as_ref().is_some_and(|response| {
            response
                .pointer("/result/tools")
                .and_then(|value| value.as_array())
                .is_some_and(|tools| {
                    tools.len() == LITE_PUBLIC_TOOL_NAMES.len()
                        && tools
                            .iter()
                            .zip(LITE_PUBLIC_TOOL_NAMES)
                            .all(|(tool, expected)| {
                                tool.get("name").and_then(|value| value.as_str()) == Some(expected)
                            })
                })
        });

        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }
        let offline_dispatch = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "qiongli_task_plan",
                "arguments": {
                    "task_id": "desktop-self-test",
                    "paper_type": "review",
                    "topic": "offline self-test"
                }
            }
        }));
        let offline_ready = offline_dispatch.as_ref().is_some_and(|response| {
            response.get("result").is_some() && response.get("error").is_none()
        });

        let provider_check = if input.counts.enabled_providers == 0 {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Attention,
                "no-provider-enabled",
            )
        } else if input.counts.ready_providers == input.counts.enabled_providers {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Ready,
                "enabled-providers-ready",
            )
        } else {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Attention,
                "provider-configuration-attention",
            )
        };
        let client_check = if input.counts.discovered_clients == 0 {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Attention,
                "no-client-discovered",
            )
        } else if input.counts.registered_clients == input.counts.discovered_clients {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Ready,
                "discovered-clients-registered",
            )
        } else {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Attention,
                "client-registration-attention",
            )
        };
        let checks = [
            mcp_check(
                McpSelfTestCheckId::EmbeddedContract,
                StatusCode::Ready,
                "embedded-contract-ready",
            ),
            mcp_check(
                McpSelfTestCheckId::Initialize,
                if initialize_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if initialize_ready {
                    "mcp-initialize-ready"
                } else {
                    "mcp-initialize-invalid"
                },
            ),
            mcp_check(
                McpSelfTestCheckId::ToolRegistry,
                if tools_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if tools_ready {
                    "exact-tool-registry-ready"
                } else {
                    "tool-registry-drifted"
                },
            ),
            mcp_check(
                McpSelfTestCheckId::OfflineDispatch,
                if offline_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if offline_ready {
                    "offline-dispatch-ready"
                } else {
                    "offline-dispatch-failed"
                },
            ),
            provider_check,
            client_check,
        ];
        let state = if initialize_ready && tools_ready && offline_ready {
            McpSelfTestState::Passed
        } else {
            McpSelfTestState::Failed
        };
        mcp_self_test_view(state, checks, input.counts)
    }
}

struct ActiveMcpSelfTest {
    receiver: mpsc::Receiver<McpSelfTestView>,
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
    running: McpSelfTestView,
}

struct ActiveDesktopUpdate {
    receiver: mpsc::Receiver<DesktopUpdateWorkerMessage>,
}

enum DesktopUpdateWorkerMessage {
    Progress(UpdateView),
    Finished(Result<DesktopUpdateOutcome, &'static str>),
}

enum DesktopUpdateOutcome {
    Checked(crate::update_cli::DesktopUpdateCheck),
    Prepared(crate::update_cli::DesktopPreparedUpdate),
    Cancelled(UpdateStreamView),
    InstallHandoff(crate::update_cli::DesktopInstallHandoff),
}

fn mcp_check(
    check: McpSelfTestCheckId,
    status: StatusCode,
    code: &'static str,
) -> McpSelfTestCheckView {
    McpSelfTestCheckView {
        check,
        status,
        code,
        remediation: mcp_check_remediation(check, status),
    }
}

const fn mcp_check_remediation(check: McpSelfTestCheckId, status: StatusCode) -> &'static str {
    if matches!(status, StatusCode::Ready | StatusCode::Missing) {
        return "none";
    }
    match check {
        McpSelfTestCheckId::EmbeddedContract | McpSelfTestCheckId::ToolRegistry => {
            "reinstall-qiongli"
        }
        McpSelfTestCheckId::Initialize => "upgrade-qiongli",
        McpSelfTestCheckId::OfflineDispatch => "retry-mcp-self-test",
        McpSelfTestCheckId::ProviderReadiness => "configure-enabled-providers",
        McpSelfTestCheckId::ClientRegistration => "refresh-integration-discovery",
    }
}

fn mcp_self_test_view(
    state: McpSelfTestState,
    checks: [McpSelfTestCheckView; 6],
    counts: McpSelfTestCounts,
) -> McpSelfTestView {
    McpSelfTestView {
        state,
        checks,
        public_tool_count: LITE_PUBLIC_TOOL_NAMES.len(),
        enabled_provider_count: counts.enabled_providers,
        ready_provider_count: counts.ready_providers,
        discovered_client_count: counts.discovered_clients,
        registered_client_count: counts.registered_clients,
    }
}

fn pending_mcp_self_test(counts: McpSelfTestCounts) -> McpSelfTestView {
    mcp_self_test_view(
        McpSelfTestState::Running,
        McpSelfTestCheckId::ALL
            .map(|check| mcp_check(check, StatusCode::Missing, "self-test-check-pending")),
        counts,
    )
}

fn terminal_mcp_self_test(state: McpSelfTestState, counts: McpSelfTestCounts) -> McpSelfTestView {
    let (status, code) = match state {
        McpSelfTestState::Cancelled => (StatusCode::Missing, "self-test-cancelled"),
        McpSelfTestState::TimedOut => (StatusCode::Blocked, "self-test-timed-out"),
        McpSelfTestState::Failed => (StatusCode::Invalid, "self-test-failed"),
        McpSelfTestState::Running | McpSelfTestState::Passed => {
            (StatusCode::Invalid, "self-test-state-invalid")
        }
    };
    let mut view = mcp_self_test_view(
        state,
        McpSelfTestCheckId::ALL.map(|check| mcp_check(check, status, code)),
        counts,
    );
    for check in &mut view.checks {
        check.remediation = if state == McpSelfTestState::Cancelled {
            "none"
        } else {
            "retry-mcp-self-test"
        };
    }
    view
}

fn contract_failure_mcp_self_test(counts: McpSelfTestCounts) -> McpSelfTestView {
    let mut view = terminal_mcp_self_test(McpSelfTestState::Failed, counts);
    view.public_tool_count = 0;
    view.checks[0] = mcp_check(
        McpSelfTestCheckId::EmbeddedContract,
        StatusCode::Invalid,
        "embedded-contract-invalid",
    );
    view
}

trait FolderPicker: Send {
    fn pick_folder(&mut self) -> Option<PathBuf>;

    fn pick_project_folder(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_create_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_export_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_import_source(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_import_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_capture_file(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }
}

struct NativeFolderPicker;

impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a Qiongli Skills destination")
            .pick_folder()
    }

    fn pick_project_folder(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a workspace, RESEARCH folder, or article topic folder")
            .pick_folder()
    }

    fn pick_project_create_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the workspace for the new Qiongli article project")
            .pick_folder()
            .map(|workspace| article_project_root_in_workspace(&workspace, suggested_name))
    }

    fn pick_project_export_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a destination for the portable Qiongli project")
            .set_file_name(suggested_name)
            .save_file()
    }

    fn pick_project_import_source(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a portable Qiongli project package")
            .pick_folder()
    }

    fn pick_project_import_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a location for the imported Qiongli project")
            .set_file_name(suggested_name)
            .save_file()
    }

    fn pick_capture_file(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a portable Qiongli research capture")
            .add_filter("Qiongli research capture", &["json"])
            .pick_file()
    }
}

struct NativeDesktopService {
    environment: CommandEnvironment,
    content: EmbeddedContent,
    active_operation: Option<PendingDesktopOperation>,
    folder_picker: Box<dyn FolderPicker>,
    selected_skills_target: Option<MaterializationTarget>,
    secret_store: Arc<dyn SecretStore>,
    mcp_self_test: Option<ActiveMcpSelfTest>,
    mcp_self_test_executor: Arc<dyn McpSelfTestExecutor>,
    mcp_self_test_timeout: Duration,
    update_view: UpdateView,
    active_update: Option<ActiveDesktopUpdate>,
    activation_sessions: Vec<DesktopActivationSession>,
    candidate_sessions: Vec<DesktopCandidateSession>,
    packaged_product: PackagedProductState,
}

struct PackagedProductState {
    product: Option<VerifiedPackagedProduct>,
    blocked_reason: &'static str,
    pending: Option<PendingPackagedProductOperation>,
}

enum PendingPackagedProductOperation {
    Install(PackagedProductInstallPreview),
    BatchInstall(PackagedProductBatchInstallPreview),
    Remove {
        selection: IntegrationSelection,
        verifications: Vec<PackagedProductInstallVerification>,
    },
}

impl PackagedProductState {
    fn read_only(blocked_reason: &'static str) -> Self {
        Self {
            product: None,
            blocked_reason,
            pending: None,
        }
    }

    fn verified(product: VerifiedPackagedProduct) -> Self {
        Self {
            product: Some(product),
            blocked_reason: "none",
            pending: None,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        target: IntegrationTarget,
    ) -> Result<OperationPreview, &'static str> {
        let Some(product) = self.product.as_ref() else {
            return Ok(blocked_product_preview(token, target, self.blocked_reason));
        };
        let preview = preview_packaged_product_install(product, activation_target(target))
            .map_err(|error| error.reason_code())?;
        let can_confirm = preview.can_apply;
        let blocked_reason = match preview.effect {
            PackagedProductInstallEffect::Install
            | PackagedProductInstallEffect::Repair
            | PackagedProductInstallEffect::AlreadyCurrent => None,
            PackagedProductInstallEffect::ReplaceRequired => {
                Some("packaged-product-replace-required")
            }
            PackagedProductInstallEffect::RecoveryRequired => {
                Some("packaged-product-recovery-required")
            }
        };
        let operation = OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match target {
                IntegrationTarget::Codex => "Codex packaged installation preview",
                IntegrationTarget::ClaudeCode => "Claude Code packaged installation preview",
            },
            summary: match preview.effect {
                PackagedProductInstallEffect::Install => {
                    "Install the receipt-owned qiongli-next Lite source and registration from this verified App."
                }
                PackagedProductInstallEffect::Repair => {
                    "Repair the missing qiongli-next registration from its exact receipt-owned Lite source."
                }
                PackagedProductInstallEffect::AlreadyCurrent => {
                    "The receipt-owned qiongli-next Lite installation is already current."
                }
                PackagedProductInstallEffect::ReplaceRequired => {
                    "An unmanaged or drifted qiongli-next installation was preserved and requires an explicit replacement workflow."
                }
                PackagedProductInstallEffect::RecoveryRequired => {
                    "A prior qiongli-next transaction requires recovery before installation can continue."
                }
            },
            display_target: None,
            plan_digest_sha256: can_confirm.then(|| preview.plan_digest_sha256.clone()),
            approvals_required: if can_confirm {
                OperationApproval::ACTIVATION.to_vec()
            } else {
                Vec::new()
            },
            can_confirm,
            blocked_reason,
        };
        self.pending = can_confirm.then_some(PendingPackagedProductOperation::Install(preview));
        Ok(operation)
    }

    fn preview_batch(
        &mut self,
        token: OperationToken,
        selection: IntegrationSelection,
        title: &'static str,
        summary: &'static str,
    ) -> Result<OperationPreview, &'static str> {
        let Some(product) = self.product.as_ref() else {
            return Ok(blocked_batch_product_preview(
                token,
                title,
                self.blocked_reason,
            ));
        };
        let targets = selected_activation_targets(selection)?;
        let preview = preview_packaged_product_batch_install(product, &targets)
            .map_err(|error| error.reason_code())?;
        let blocked_reason = (!preview.can_apply).then_some(
            if preview
                .installs
                .iter()
                .any(|install| install.effect == PackagedProductInstallEffect::RecoveryRequired)
            {
                "packaged-product-recovery-required"
            } else {
                "packaged-product-replace-required"
            },
        );
        let operation = OperationPreview {
            token,
            kind: OperationKind::Activation,
            title,
            summary,
            display_target: None,
            plan_digest_sha256: preview
                .can_apply
                .then(|| preview.plan_digest_sha256.clone()),
            approvals_required: if preview.can_apply {
                OperationApproval::ACTIVATION.to_vec()
            } else {
                Vec::new()
            },
            can_confirm: preview.can_apply,
            blocked_reason,
        };
        self.pending = preview
            .can_apply
            .then_some(PendingPackagedProductOperation::BatchInstall(preview));
        Ok(operation)
    }

    fn verify(&self, selection: IntegrationSelection) -> Result<&'static str, &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        for target in selected_activation_targets(selection)? {
            verify_packaged_product_install(product, target)
                .map_err(|error| error.reason_code())?;
        }
        Ok("packaged-product-install-verified")
    }

    fn preview_remove(
        &mut self,
        token: OperationToken,
        selection: IntegrationSelection,
    ) -> Result<OperationPreview, &'static str> {
        let Some(product) = self.product.as_ref() else {
            return Ok(blocked_batch_product_preview(
                token,
                "Remove selected integrations",
                self.blocked_reason,
            ));
        };
        let verifications = selected_activation_targets(selection)?
            .into_iter()
            .map(|target| {
                verify_packaged_product_install(product, target)
                    .map_err(|error| error.reason_code())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let digest = packaged_product_removal_digest(product, &verifications);
        self.pending = Some(PendingPackagedProductOperation::Remove {
            selection,
            verifications,
        });
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: "Remove selected integrations",
            summary: "Remove only receipt-owned qiongli-next registrations and plugin sources for the selected clients.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(
        &mut self,
        content: &EmbeddedContent,
        target: IntegrationTarget,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        let pending = self
            .pending
            .take()
            .ok_or("packaged-product-preview-missing")?;
        let PendingPackagedProductOperation::Install(preview) = pending else {
            return Err("packaged-product-preview-invalid");
        };
        if preview.target != activation_target(target) {
            return Err("packaged-product-preview-invalid");
        }
        let commit = apply_packaged_product_install(content.pack(), product, &preview, now_unix)
            .map_err(|error| error.reason_code())?;
        Ok(match commit.disposition {
            qiongli_platform::PackagedProductInstallDisposition::Installed => {
                "packaged-product-install-applied"
            }
            qiongli_platform::PackagedProductInstallDisposition::AlreadyCurrent => {
                "packaged-product-install-already-current"
            }
        })
    }

    fn confirm_batch(
        &mut self,
        content: &EmbeddedContent,
        selection: IntegrationSelection,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        let pending = self
            .pending
            .take()
            .ok_or("packaged-product-preview-missing")?;
        let PendingPackagedProductOperation::BatchInstall(preview) = pending else {
            return Err("packaged-product-preview-invalid");
        };
        if preview
            .installs
            .iter()
            .map(|install| install.target)
            .collect::<Vec<_>>()
            != selected_activation_targets(selection)?
        {
            return Err("packaged-product-preview-invalid");
        }
        let commit =
            apply_packaged_product_batch_install(content.pack(), product, &preview, now_unix)
                .map_err(|error| error.reason_code())?;
        Ok(
            if commit.installs.iter().all(|install| {
                install.disposition
                    == qiongli_platform::PackagedProductInstallDisposition::AlreadyCurrent
            }) {
                "packaged-product-batch-already-current"
            } else {
                "packaged-product-batch-applied"
            },
        )
    }

    fn confirm_remove(
        &mut self,
        selection: IntegrationSelection,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        let pending = self
            .pending
            .take()
            .ok_or("packaged-product-preview-missing")?;
        let PendingPackagedProductOperation::Remove {
            selection: expected_selection,
            verifications,
        } = pending
        else {
            return Err("packaged-product-preview-invalid");
        };
        if expected_selection != selection {
            return Err("packaged-product-preview-invalid");
        }
        let targets = selected_activation_targets(selection)?;
        let current = targets
            .iter()
            .copied()
            .map(|target| {
                verify_packaged_product_install(product, target)
                    .map_err(|error| error.reason_code())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if current != verifications {
            return Err("packaged-product-preview-invalid");
        }
        for target in targets {
            remove_packaged_product_install(product, target, now_unix)
                .map_err(|error| error.reason_code())?;
        }
        Ok("packaged-product-install-removed")
    }

    fn cancel(&mut self) {
        self.pending = None;
    }
}

enum PendingDesktopOperation {
    Blocked(OperationToken),
    GlobalSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    ProviderSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    ProviderSecret {
        token: OperationToken,
        expected_revision: u64,
        provider: ProviderKind,
        replacement: GlobalSettings,
        secret_ref: SecretRef,
        replacement_value: Option<SecretValue>,
        previous_value: Option<SecretValue>,
    },
    AgentBackendSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    AgentBackendSecret {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
        secret_ref: SecretRef,
        replacement_value: Option<SecretValue>,
        previous_value: Option<SecretValue>,
    },
    AgentRun {
        token: OperationToken,
        request: FullAgentRunRequest,
    },
    SkillsMaterialization {
        token: OperationToken,
        profile: ProfileKind,
        target: MaterializationTarget,
    },
    SkillsRemoval {
        token: OperationToken,
        target: MaterializationTarget,
        expected_receipt: MaterializationReceiptV1,
    },
    Activation {
        token: OperationToken,
        target: IntegrationTarget,
    },
    Candidate {
        token: OperationToken,
        target: IntegrationTarget,
    },
    PackagedProduct {
        token: OperationToken,
        target: IntegrationTarget,
    },
    PackagedProductBatch {
        token: OperationToken,
        selection: IntegrationSelection,
    },
    PackagedProductRemoval {
        token: OperationToken,
        selection: IntegrationSelection,
    },
    UpdateInstall {
        token: OperationToken,
        expected_revision: u64,
    },
}

impl Drop for NativeDesktopService {
    fn drop(&mut self) {
        if let Some(active) = &self.mcp_self_test {
            active.cancelled.store(true, Ordering::Release);
        }
    }
}

impl PendingDesktopOperation {
    const fn token(&self) -> OperationToken {
        match self {
            Self::Blocked(token)
            | Self::GlobalSettings { token, .. }
            | Self::ProviderSettings { token, .. }
            | Self::ProviderSecret { token, .. }
            | Self::AgentBackendSettings { token, .. }
            | Self::AgentBackendSecret { token, .. }
            | Self::AgentRun { token, .. }
            | Self::SkillsMaterialization { token, .. }
            | Self::SkillsRemoval { token, .. }
            | Self::Activation { token, .. }
            | Self::Candidate { token, .. }
            | Self::PackagedProduct { token, .. }
            | Self::PackagedProductBatch { token, .. }
            | Self::PackagedProductRemoval { token, .. }
            | Self::UpdateInstall { token, .. } => *token,
        }
    }
}

impl NativeDesktopService {
    fn new(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        activation_sessions: Vec<DesktopActivationSession>,
    ) -> Self {
        Self::new_with_packaged_product(
            environment,
            content,
            activation_sessions,
            PackagedProductState::read_only("source-build-read-only"),
        )
    }

    fn new_with_packaged_product(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        activation_sessions: Vec<DesktopActivationSession>,
        packaged_product: PackagedProductState,
    ) -> Self {
        let update_view = update_snapshot(&environment);
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker: Box::new(NativeFolderPicker),
            selected_skills_target: None,
            secret_store: crate::credential_store::native_secret_store(),
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            update_view,
            active_update: None,
            activation_sessions,
            candidate_sessions: Vec::new(),
            packaged_product,
        }
    }

    fn new_with_candidate_sessions(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        candidate_sessions: Vec<DesktopCandidateSession>,
    ) -> Self {
        let update_view = update_snapshot(&environment);
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker: Box::new(NativeFolderPicker),
            selected_skills_target: None,
            secret_store: crate::credential_store::native_secret_store(),
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            update_view,
            active_update: None,
            activation_sessions: Vec::new(),
            candidate_sessions,
            packaged_product: PackagedProductState::read_only("candidate-session-only"),
        }
    }

    #[cfg(test)]
    fn new_with_folder_picker(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        folder_picker: Box<dyn FolderPicker>,
    ) -> Self {
        let update_view = update_snapshot(&environment);
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker,
            selected_skills_target: None,
            secret_store: crate::credential_store::native_secret_store(),
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            update_view,
            active_update: None,
            activation_sessions: Vec::new(),
            candidate_sessions: Vec::new(),
            packaged_product: PackagedProductState::read_only("source-build-read-only"),
        }
    }

    fn start_mcp_self_test(&mut self) -> DesktopEvent {
        if let Some(active) = &self.mcp_self_test {
            return DesktopEvent::McpSelfTestUpdated(active.running.clone());
        }
        let snapshot = build_snapshot(&self.environment, &self.content, self.secret_store.as_ref());
        let counts = mcp_self_test_counts(&snapshot);
        let registry = match LiteToolRegistry::from_embedded_content(&self.content) {
            Ok(registry) => registry,
            Err(_) => {
                return DesktopEvent::McpSelfTestUpdated(contract_failure_mcp_self_test(counts));
            }
        };
        // The bounded offline test exercises only the protocol contract, exact tool registry,
        // and an offline planning tool. Provider readiness is reported from the snapshot above,
        // so resolving credentials here would add no coverage and may synchronously prompt the
        // OS credential store on the UI thread.
        let server = LiteMcpServer::production(
            "qiongli",
            env!("CARGO_PKG_VERSION"),
            registry,
            ProviderAccess::builder().build(),
        );
        let running = pending_mcp_self_test(counts);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let executor = Arc::clone(&self.mcp_self_test_executor);
        let (sender, receiver) = mpsc::sync_channel(1);
        let spawn = thread::Builder::new()
            .name("qiongli-mcp-self-test".to_owned())
            .spawn(move || {
                let result = executor.run(McpSelfTestInput { server, counts }, worker_cancelled);
                let _ = sender.send(result);
            });
        if spawn.is_err() {
            return DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                McpSelfTestState::Failed,
                counts,
            ));
        }
        let now = Instant::now();
        self.mcp_self_test = Some(ActiveMcpSelfTest {
            receiver,
            cancelled,
            deadline: now.checked_add(self.mcp_self_test_timeout).unwrap_or(now),
            running: running.clone(),
        });
        DesktopEvent::McpSelfTestUpdated(running)
    }

    fn poll_mcp_self_test(&mut self) -> DesktopEvent {
        let Some(active) = self.mcp_self_test.as_ref() else {
            return DesktopEvent::Failed {
                code: "mcp-self-test-not-running",
            };
        };
        if Instant::now() >= active.deadline {
            let active = self
                .mcp_self_test
                .take()
                .expect("validated MCP self-test remains active");
            active.cancelled.store(true, Ordering::Release);
            return DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                McpSelfTestState::TimedOut,
                mcp_counts_from_view(&active.running),
            ));
        }
        match active.receiver.try_recv() {
            Ok(result) => {
                self.mcp_self_test = None;
                DesktopEvent::McpSelfTestUpdated(result)
            }
            Err(mpsc::TryRecvError::Empty) => {
                DesktopEvent::McpSelfTestUpdated(active.running.clone())
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                let counts = mcp_counts_from_view(&active.running);
                self.mcp_self_test = None;
                DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                    McpSelfTestState::Failed,
                    counts,
                ))
            }
        }
    }

    fn cancel_mcp_self_test(&mut self) -> DesktopEvent {
        let Some(active) = self.mcp_self_test.take() else {
            return DesktopEvent::Failed {
                code: "mcp-self-test-not-running",
            };
        };
        active.cancelled.store(true, Ordering::Release);
        DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
            McpSelfTestState::Cancelled,
            mcp_counts_from_view(&active.running),
        ))
    }

    fn select_update_stream(&mut self, stream: UpdateStreamView) -> DesktopEvent {
        if self.active_update.is_some() || !self.update_view.can_select_stream {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let result = with_native_update_context(&self.environment, |store, authority, content| {
            crate::update_cli::desktop_select_stream(
                store,
                update_stream_preference(stream),
                authority,
                crate::embedded_macos_team_id(),
                &self.environment,
                content,
            )
        });
        self.update_view = match result {
            Ok(()) => update_snapshot(&self.environment),
            Err(code) => update_failure_view(&self.environment, code),
        };
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn start_update_check(&mut self) -> DesktopEvent {
        if self.active_update.is_some() || !self.update_view.can_check {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let running = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Checking,
            1,
            "Checking signed update metadata",
        );
        let environment = self.environment.clone();
        let (sender, receiver) = mpsc::channel();
        let spawn = thread::Builder::new()
            .name("qiongli-update-check".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_check(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                        )
                        .map(DesktopUpdateOutcome::Checked)
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.update_view = running;
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn start_update_preparation(&mut self) -> DesktopEvent {
        if self.active_update.is_some() || !self.update_view.can_prepare {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let running = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Downloading,
            1,
            "Downloading signed package",
        );
        let environment = self.environment.clone();
        let target_version = self.update_view.available_version.clone();
        let archive_size_bytes = self.update_view.archive_size_bytes;
        let selected_stream = self.update_view.selected_stream;
        let (sender, receiver) = mpsc::channel();
        let worker_sender = sender.clone();
        let spawn = thread::Builder::new()
            .name("qiongli-update-prepare".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_prepare(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                            |stage| {
                                let update = update_preparation_progress_view(
                                    selected_stream,
                                    target_version.clone(),
                                    archive_size_bytes,
                                    stage,
                                );
                                let _ = worker_sender
                                    .send(DesktopUpdateWorkerMessage::Progress(update));
                            },
                        )
                        .map(DesktopUpdateOutcome::Prepared)
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.update_view = running;
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn poll_update(&mut self) -> DesktopEvent {
        let Some(result) = self
            .active_update
            .as_ref()
            .map(|active| active.receiver.try_recv())
        else {
            self.update_view = update_snapshot(&self.environment);
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        };
        match result {
            Ok(DesktopUpdateWorkerMessage::Progress(update)) => {
                self.update_view = update;
                DesktopEvent::UpdateChanged {
                    update: self.update_view.clone(),
                    close_requested: false,
                }
            }
            Ok(DesktopUpdateWorkerMessage::Finished(result)) => {
                self.active_update = None;
                let (update, close_requested) = match result {
                    Ok(DesktopUpdateOutcome::Checked(checked)) => {
                        (update_checked_view(checked), false)
                    }
                    Ok(DesktopUpdateOutcome::Prepared(prepared)) => {
                        (update_prepared_view(&self.environment, prepared), false)
                    }
                    Ok(DesktopUpdateOutcome::Cancelled(stream)) => {
                        (update_cancelled_view(stream), false)
                    }
                    Ok(DesktopUpdateOutcome::InstallHandoff(handoff)) => {
                        (update_install_handoff_view(handoff), true)
                    }
                    Err(code) => (update_failure_view(&self.environment, code), false),
                };
                self.update_view = update;
                DesktopEvent::UpdateChanged {
                    update: self.update_view.clone(),
                    close_requested,
                }
            }
            Err(mpsc::TryRecvError::Empty) => DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                self.active_update = None;
                self.update_view =
                    update_failure_view(&self.environment, "native-update-worker-unavailable");
                DesktopEvent::UpdateChanged {
                    update: self.update_view.clone(),
                    close_requested: false,
                }
            }
        }
    }

    fn cancel_update(&mut self) -> DesktopEvent {
        if !self.update_view.can_cancel {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let selected_stream = self.update_view.selected_stream;
        self.update_view = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Cancelling,
            1,
            "Removing staged update bytes",
        );
        let environment = self.environment.clone();
        let (sender, receiver) = mpsc::channel();
        let spawn = thread::Builder::new()
            .name("qiongli-update-cancel".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_cancel(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                        )
                        .map(|()| DesktopUpdateOutcome::Cancelled(selected_stream))
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        self.active_update = None;
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn preview_update_install(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        if self.active_update.is_some() {
            return DesktopEvent::Failed {
                code: "native-update-transaction-active",
            };
        }
        let store = match update_store(&self.environment) {
            Ok(store) => store,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let Some(transaction) = loaded.state.active_transaction else {
            return DesktopEvent::Failed {
                code: "native-update-transaction-missing",
            };
        };
        if !matches!(
            transaction.phase,
            UpdateTransactionPhase::Staged | UpdateTransactionPhase::ReconciliationPrepared
        ) {
            return DesktopEvent::Failed {
                code: "native-update-transaction-not-installable",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = update_install_digest(
            loaded.revision,
            &transaction.transaction_id,
            &transaction.target_version,
        );
        self.active_operation = Some(PendingDesktopOperation::UpdateInstall {
            token,
            expected_revision: loaded.revision,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::UpdateInstall,
            title: "Install Qiongli update",
            summary: "Quit Qiongli, atomically activate the verified application and managed content, then roll back automatically if the new runtime fails health checks.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn start_update_install(&mut self, expected_revision: u64) -> DesktopEvent {
        let loaded = match update_store(&self.environment)
            .and_then(|store| store.load().map_err(|error| error.reason_code()))
        {
            Ok(value) => value,
            Err(code) => {
                return DesktopEvent::UpdateChanged {
                    update: update_failure_view(&self.environment, code),
                    close_requested: false,
                };
            }
        };
        if loaded.revision != expected_revision {
            return DesktopEvent::UpdateChanged {
                update: update_failure_view(&self.environment, "revision-conflict"),
                close_requested: false,
            };
        }
        let running = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Installing,
            4,
            "Handing off to the native update helper",
        );
        let environment = self.environment.clone();
        let (sender, receiver) = mpsc::channel();
        let spawn = thread::Builder::new()
            .name("qiongli-update-install".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_install(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                        )
                        .map(DesktopUpdateOutcome::InstallHandoff)
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.update_view = running;
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn next_operation_token() -> Result<OperationToken, &'static str> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| "operation-token-unavailable")?;
        Ok(OperationToken::new(u128::from_le_bytes(bytes)))
    }

    fn issue_preview(
        &mut self,
        title: &'static str,
        summary: &'static str,
        blocked_reason: &'static str,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        self.active_operation = Some(PendingDesktopOperation::Blocked(token));
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::GlobalSettings,
            title,
            summary,
            display_target: None,
            plan_digest_sha256: None,
            approvals_required: Vec::new(),
            can_confirm: false,
            blocked_reason: Some(blocked_reason),
        })
    }

    fn preview_global_settings(&mut self, patch: GlobalSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }

        let mut replacement = loaded.settings.clone();
        replacement.default_profile = profile_to_content(patch.default_profile);
        if replacement == loaded.settings {
            return self.issue_preview(
                "Global settings preview",
                "The current product-wide default profile is already active. No configuration change will be made.",
                "global-settings-unchanged",
            );
        }

        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = global_settings_patch_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::GlobalSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::GlobalSettings,
            title: "Global settings preview",
            summary: "Atomically update the product-wide default profile without changing literature provider configuration.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_provider_settings(&mut self, patch: ProviderSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }
        let mut replacement = loaded.settings.clone();
        replacement.providers.openalex.enabled = patch.providers_enabled[0];
        replacement.providers.semantic_scholar.enabled = patch.providers_enabled[1];
        replacement.providers.crossref.enabled = patch.providers_enabled[2];
        replacement.providers.pubmed.enabled = patch.providers_enabled[3];
        replacement.providers.arxiv.enabled = patch.providers_enabled[4];
        if let Err(code) = apply_public_email_change(
            &mut replacement.providers.openalex.email,
            patch.openalex_email,
        ) {
            return DesktopEvent::ValidationFailed { code };
        }
        if let Err(code) = apply_public_email_change(
            &mut replacement.providers.crossref.email,
            patch.crossref_email,
        ) {
            return DesktopEvent::ValidationFailed { code };
        }
        if replacement == loaded.settings {
            return DesktopEvent::ValidationFailed {
                code: "provider-settings-unchanged",
            };
        }

        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = global_settings_patch_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::ProviderSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::ProviderSettings,
            title: "Literature provider settings preview",
            summary: "Atomically update provider enablement and supported public contact settings without changing global product defaults.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_provider_secret(
        &mut self,
        provider: ProviderKind,
        change: ProviderSecretChange,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        if self.secret_store.status() != SecretStoreStatus::Available
            || !matches!(
                provider,
                ProviderKind::OpenAlex | ProviderKind::SemanticScholar
            )
        {
            return DesktopEvent::ValidationFailed {
                code: "provider-secret-store-unavailable",
            };
        }
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let current_ref = match provider {
            ProviderKind::OpenAlex => loaded.settings.providers.openalex.api_key_ref.as_ref(),
            ProviderKind::SemanticScholar => loaded
                .settings
                .providers
                .semantic_scholar
                .api_key_ref
                .as_ref(),
            ProviderKind::Crossref | ProviderKind::PubMed | ProviderKind::Arxiv => None,
        };
        let (secret_ref, replacement_value, previous_value) = match change {
            ProviderSecretChange::Replace(value) => {
                let replacement_value = match SecretValue::new(value.expose().as_bytes().to_vec()) {
                    Ok(value) => value,
                    Err(_) => {
                        return DesktopEvent::ValidationFailed {
                            code: "provider-secret-invalid",
                        };
                    }
                };
                let secret_ref = match current_ref.cloned().map_or_else(new_secret_ref, Ok) {
                    Ok(reference) => reference,
                    Err(code) => return DesktopEvent::Failed { code },
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, Some(replacement_value), previous_value)
            }
            ProviderSecretChange::Remove => {
                let Some(secret_ref) = current_ref.cloned() else {
                    return DesktopEvent::ValidationFailed {
                        code: "provider-secret-not-configured",
                    };
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, None, previous_value)
            }
        };
        let mut replacement = loaded.settings.clone();
        match provider {
            ProviderKind::OpenAlex => {
                replacement.providers.openalex.api_key_ref =
                    replacement_value.as_ref().map(|_| secret_ref.clone());
            }
            ProviderKind::SemanticScholar => {
                replacement.providers.semantic_scholar.api_key_ref =
                    replacement_value.as_ref().map(|_| secret_ref.clone());
            }
            ProviderKind::Crossref | ProviderKind::PubMed | ProviderKind::Arxiv => {
                return DesktopEvent::ValidationFailed {
                    code: "provider-secret-unsupported",
                };
            }
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = provider_secret_digest(
            loaded.revision,
            provider,
            &secret_ref,
            replacement_value.is_some(),
        );
        self.active_operation = Some(PendingDesktopOperation::ProviderSecret {
            token,
            expected_revision: loaded.revision,
            provider,
            replacement,
            secret_ref,
            replacement_value,
            previous_value,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::ProviderSecret,
            title: "Provider credential preview",
            summary: "Save, replace, or remove the selected API key in the OS credential store while persisting only its opaque reference in configuration.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![
                OperationApproval::SecretStoreWrite,
                OperationApproval::ClientConfigChange,
            ],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn test_literature_provider(&mut self, provider: ProviderKind) -> DesktopEvent {
        self.cancel_active_operation();
        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let access =
            ProviderAccess::from_global_settings(&loaded.settings, self.secret_store.as_ref());
        let provider_id = match provider {
            ProviderKind::OpenAlex => ProviderId::OpenAlex,
            ProviderKind::SemanticScholar => ProviderId::SemanticScholar,
            ProviderKind::Crossref => ProviderId::Crossref,
            ProviderKind::PubMed => ProviderId::PubMed,
            ProviderKind::Arxiv => ProviderId::Arxiv,
        };
        match access.availability(provider_id) {
            ProviderAvailability::Ready => DesktopEvent::Completed {
                code: "literature-provider-ready",
            },
            ProviderAvailability::Disabled => DesktopEvent::Failed {
                code: "literature-provider-disabled",
            },
            ProviderAvailability::NeedsSecret => DesktopEvent::Failed {
                code: "literature-provider-key-required",
            },
            ProviderAvailability::NeedsPublicSetting => DesktopEvent::Failed {
                code: "literature-provider-email-required",
            },
            ProviderAvailability::SecretStoreUnavailable => DesktopEvent::Failed {
                code: "literature-provider-secret-store-unavailable",
            },
        }
    }

    fn preview_agent_backend_settings(&mut self, patch: AgentBackendSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }
        let mut replacement = loaded.settings.clone();
        replacement.agent_backends.openai.enabled = patch.openai_enabled;
        if replacement == loaded.settings {
            return DesktopEvent::ValidationFailed {
                code: "agent-backend-settings-unchanged",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = agent_backend_settings_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::AgentBackendSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::AgentBackendSettings,
            title: "Agent backend settings preview",
            summary: "Enable or disable the direct OpenAI backend without testing a connection or changing its credential.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_agent_backend_secret(&mut self, change: AgentBackendSecretChange) -> DesktopEvent {
        self.cancel_active_operation();
        if self.secret_store.status() != SecretStoreStatus::Available {
            return DesktopEvent::ValidationFailed {
                code: "agent-backend-secret-store-unavailable",
            };
        }
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let current_ref = loaded.settings.agent_backends.openai.api_key_ref.as_ref();
        let (secret_ref, replacement_value, previous_value) = match change {
            AgentBackendSecretChange::Replace(value) => {
                let replacement_value = match SecretValue::new(value.expose().as_bytes().to_vec()) {
                    Ok(value) => value,
                    Err(_) => {
                        return DesktopEvent::ValidationFailed {
                            code: "agent-backend-secret-invalid",
                        };
                    }
                };
                let secret_ref = match current_ref
                    .cloned()
                    .map_or_else(new_agent_backend_secret_ref, Ok)
                {
                    Ok(reference) => reference,
                    Err(code) => return DesktopEvent::Failed { code },
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, Some(replacement_value), previous_value)
            }
            AgentBackendSecretChange::Remove => {
                let Some(secret_ref) = current_ref.cloned() else {
                    return DesktopEvent::ValidationFailed {
                        code: "agent-backend-secret-not-configured",
                    };
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, None, previous_value)
            }
        };
        let mut replacement = loaded.settings.clone();
        replacement.agent_backends.openai.api_key_ref =
            replacement_value.as_ref().map(|_| secret_ref.clone());
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest =
            agent_backend_secret_digest(loaded.revision, &secret_ref, replacement_value.is_some());
        self.active_operation = Some(PendingDesktopOperation::AgentBackendSecret {
            token,
            expected_revision: loaded.revision,
            replacement,
            secret_ref,
            replacement_value,
            previous_value,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::AgentBackendSecret,
            title: "OpenAI credential preview",
            summary: "Save, replace, or remove the OpenAI API key in the OS credential store while persisting only its opaque reference in configuration.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![
                OperationApproval::SecretStoreWrite,
                OperationApproval::ClientConfigChange,
            ],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn test_openai_backend(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let control = BackendControlService::from_global_settings(
            &loaded.settings,
            Arc::clone(&self.secret_store),
        );
        match control.test_openai_connection(&AgentCancellationToken::new()) {
            Ok(_) => DesktopEvent::Completed {
                code: "openai-backend-connection-passed",
            },
            Err(error) => DesktopEvent::Failed {
                code: error.reason_code(),
            },
        }
    }

    fn preview_agent_run(&mut self, draft: AgentRunDraft) -> DesktopEvent {
        self.cancel_active_operation();
        let project_id = match ProjectId::parse(draft.project_id) {
            Ok(project_id) => project_id,
            Err(_) => {
                return DesktopEvent::ValidationFailed {
                    code: "agent-run-request-invalid",
                };
            }
        };
        let Some(projects) = project_state_service(&self.environment) else {
            return DesktopEvent::Failed {
                code: "project-service-unavailable",
            };
        };
        let snapshot = match projects.snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let Some(project) = snapshot
            .projects
            .iter()
            .find(|project| project.project_id == project_id)
        else {
            return DesktopEvent::Failed {
                code: "project-not-registered",
            };
        };
        if project.semantic_revision != draft.expected_project_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }
        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let readiness =
            openai_backend_status(&loaded.settings, self.secret_store.as_ref()).readiness;
        if readiness != BackendReadinessV1::Ready {
            return DesktopEvent::Failed {
                code: readiness_reason_code(readiness),
            };
        }
        if FullProjectToolRegistry::from_embedded_content(&self.content).is_err() {
            return DesktopEvent::Failed {
                code: "agent-run-tools-unavailable",
            };
        }
        let digest = agent_run_digest(
            &project_id,
            draft.expected_project_revision,
            draft.prompt.expose(),
        );
        let request = match FullAgentRunRequest::new(
            project_id,
            draft.expected_project_revision,
            draft.prompt,
            true,
        ) {
            Ok(request) => request,
            Err(error) => {
                return DesktopEvent::ValidationFailed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        self.active_operation = Some(PendingDesktopOperation::AgentRun { token, request });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::AgentRun,
            title: "Run project query with OpenAI",
            summary: "Send this prompt and any redacted read-only project tool results to OpenAI. The run is bound to the selected project revision and cannot write project files.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::NetworkRequest],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn select_skills_destination(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(path) = self.folder_picker.pick_folder() else {
            return DesktopEvent::Cancelled {
                code: "skills-destination-selection-cancelled",
            };
        };
        let target = match approve_materialization_target(&path) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let display_path = PrivateDisplayText::new(display_path(&path));
        self.selected_skills_target = Some(target);
        DesktopEvent::SkillsDestinationSelected { display_path }
    }

    fn preview_skills_materialization(&mut self, profile: ProfileKind) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target = PrivateDisplayText::new(display_path(target.path()));
        let digest = skills_materialization_digest(
            self.content.pack().pack_sha256(),
            profile,
            target.path(),
        );
        self.active_operation = Some(PendingDesktopOperation::SkillsMaterialization {
            token,
            profile,
            target,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsMaterialization,
            title: "Skills materialization preview",
            summary: "Write the selected embedded profile to the explicitly selected folder and verify its managed receipt.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn verify_skills_materialization(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        match verify_materialization(&target) {
            Ok(_) => DesktopEvent::Completed {
                code: "skills-materialization-verified",
            },
            Err(error) => DesktopEvent::Failed {
                code: error.reason_code(),
            },
        }
    }

    fn preview_skills_removal(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let receipt = match verify_materialization(&target) {
            Ok(receipt) => receipt,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target = PrivateDisplayText::new(display_path(target.path()));
        let digest = skills_removal_digest(&receipt, target.path());
        self.active_operation = Some(PendingDesktopOperation::SkillsRemoval {
            token,
            target,
            expected_receipt: receipt,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsRemoval,
            title: "Skills removal preview",
            summary: "Remove only the selected Qiongli-managed materialization after re-verifying its complete receipt.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn select_skills_preset_target(
        &self,
        preset: SkillsDestinationPreset,
    ) -> Result<MaterializationTarget, &'static str> {
        let path = match preset {
            SkillsDestinationPreset::QiongliManaged => self
                .environment
                .platform_home()
                .ok_or("skills-home-unavailable")?
                .join(".qiongli-skills"),
            SkillsDestinationPreset::CurrentProject => self
                .environment
                .project_root()
                .ok_or("skills-project-unavailable")?
                .join(".qiongli-skills"),
            SkillsDestinationPreset::CustomFolder => self
                .selected_skills_target
                .as_ref()
                .ok_or("skills-destination-required")?
                .path()
                .to_path_buf(),
            SkillsDestinationPreset::DetectedCodex
            | SkillsDestinationPreset::DetectedClaudeCode => {
                return Err("skills-preset-client-managed");
            }
        };
        approve_materialization_target(path).map_err(|error| error.reason_code())
    }

    fn preview_skills_preset_materialization(
        &mut self,
        profile: ProfileKind,
        preset: SkillsDestinationPreset,
    ) -> DesktopEvent {
        match preset {
            SkillsDestinationPreset::DetectedCodex => {
                self.preview_activation(IntegrationTarget::Codex)
            }
            SkillsDestinationPreset::DetectedClaudeCode => {
                self.preview_activation(IntegrationTarget::ClaudeCode)
            }
            _ => match self.select_skills_preset_target(preset) {
                Ok(target) => {
                    self.selected_skills_target = Some(target);
                    self.preview_skills_materialization(profile)
                }
                Err(code) => DesktopEvent::ValidationFailed { code },
            },
        }
    }

    fn verify_skills_preset(&mut self, preset: SkillsDestinationPreset) -> DesktopEvent {
        match preset {
            SkillsDestinationPreset::DetectedCodex => {
                self.verify_packaged_integrations(IntegrationSelection {
                    codex: true,
                    claude_code: false,
                })
            }
            SkillsDestinationPreset::DetectedClaudeCode => {
                self.verify_packaged_integrations(IntegrationSelection {
                    codex: false,
                    claude_code: true,
                })
            }
            _ => match self.select_skills_preset_target(preset) {
                Ok(target) => {
                    self.selected_skills_target = Some(target);
                    self.verify_skills_materialization()
                }
                Err(code) => DesktopEvent::ValidationFailed { code },
            },
        }
    }

    fn preview_skills_preset_removal(&mut self, preset: SkillsDestinationPreset) -> DesktopEvent {
        match preset {
            SkillsDestinationPreset::DetectedCodex => {
                self.preview_packaged_product_removal(IntegrationSelection {
                    codex: true,
                    claude_code: false,
                })
            }
            SkillsDestinationPreset::DetectedClaudeCode => {
                self.preview_packaged_product_removal(IntegrationSelection {
                    codex: false,
                    claude_code: true,
                })
            }
            _ => match self.select_skills_preset_target(preset) {
                Ok(target) => {
                    self.selected_skills_target = Some(target);
                    self.preview_skills_removal()
                }
                Err(code) => DesktopEvent::ValidationFailed { code },
            },
        }
    }

    fn preview_packaged_product_batch(
        &mut self,
        selection: IntegrationSelection,
        title: &'static str,
        summary: &'static str,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        if selection.is_empty() {
            return DesktopEvent::ValidationFailed {
                code: "integration-selection-required",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        match self
            .packaged_product
            .preview_batch(token, selection, title, summary)
        {
            Ok(preview) => {
                self.active_operation = if preview.can_confirm {
                    Some(PendingDesktopOperation::PackagedProductBatch { token, selection })
                } else {
                    Some(PendingDesktopOperation::Blocked(token))
                };
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn recommended_integration_selection(&mut self) -> IntegrationSelection {
        let integrations = self.snapshot().integrations;
        IntegrationSelection {
            codex: integrations[0].client == StatusCode::Ready
                && integrations[0].next_action != IntegrationActionView::ResolveConflict,
            claude_code: integrations[1].client == StatusCode::Ready
                && integrations[1].next_action != IntegrationActionView::ResolveConflict,
        }
    }

    fn repair_integration_selection(&mut self) -> IntegrationSelection {
        let integrations = self.snapshot().integrations;
        IntegrationSelection {
            codex: integrations[0].next_action == IntegrationActionView::RepairReady,
            claude_code: integrations[1].next_action == IntegrationActionView::RepairReady,
        }
    }

    fn verify_packaged_integrations(&mut self, selection: IntegrationSelection) -> DesktopEvent {
        self.cancel_active_operation();
        match self.packaged_product.verify(selection) {
            Ok(code) => DesktopEvent::Completed { code },
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_packaged_product_removal(
        &mut self,
        selection: IntegrationSelection,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        if selection.is_empty() {
            return DesktopEvent::ValidationFailed {
                code: "integration-selection-required",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        match self.packaged_product.preview_remove(token, selection) {
            Ok(preview) => {
                self.active_operation = if preview.can_confirm {
                    Some(PendingDesktopOperation::PackagedProductRemoval { token, selection })
                } else {
                    Some(PendingDesktopOperation::Blocked(token))
                };
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_activation(&mut self, target: IntegrationTarget) -> DesktopEvent {
        self.cancel_active_operation();
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let now_unix = match now_unix() {
            Ok(now_unix) => now_unix,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if let Some(session) = self
            .candidate_sessions
            .iter_mut()
            .find(|session| session.target == target)
        {
            return match session.preview(token, now_unix) {
                Ok(preview) => {
                    self.active_operation =
                        Some(PendingDesktopOperation::Candidate { token, target });
                    DesktopEvent::PreviewReady(preview)
                }
                Err(code) => DesktopEvent::Failed { code },
            };
        }
        let Some(session) = self
            .activation_sessions
            .iter_mut()
            .find(|session| session.target == target)
        else {
            return match self.packaged_product.preview(token, target) {
                Ok(preview) => {
                    self.active_operation = if preview.can_confirm {
                        Some(PendingDesktopOperation::PackagedProduct { token, target })
                    } else {
                        Some(PendingDesktopOperation::Blocked(token))
                    };
                    DesktopEvent::PreviewReady(preview)
                }
                Err(code) => DesktopEvent::Failed { code },
            };
        };
        match session.preview(token, now_unix) {
            Ok(preview) => {
                self.active_operation = Some(PendingDesktopOperation::Activation { token, target });
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn cancel_active_operation(&mut self) {
        if let Some(PendingDesktopOperation::Activation { target, .. }) = &self.active_operation
            && let Some(session) = self
                .activation_sessions
                .iter_mut()
                .find(|session| session.target == *target)
        {
            session.cancel();
        }
        if let Some(PendingDesktopOperation::Candidate { target, .. }) = &self.active_operation
            && let Some(session) = self
                .candidate_sessions
                .iter_mut()
                .find(|session| session.target == *target)
        {
            session.cancel();
        }
        if matches!(
            self.active_operation,
            Some(
                PendingDesktopOperation::PackagedProduct { .. }
                    | PendingDesktopOperation::PackagedProductBatch { .. }
                    | PendingDesktopOperation::PackagedProductRemoval { .. }
            )
        ) {
            self.packaged_product.cancel();
        }
        self.active_operation = None;
    }
}

fn apply_public_email_change(
    current: &mut Option<EmailAddress>,
    change: PublicSettingChange,
) -> Result<(), &'static str> {
    match change {
        PublicSettingChange::Keep => Ok(()),
        PublicSettingChange::Clear => {
            *current = None;
            Ok(())
        }
        PublicSettingChange::Replace(value) => {
            *current = Some(
                EmailAddress::parse(value.expose())
                    .map_err(|_| "provider-public-setting-invalid")?,
            );
            Ok(())
        }
    }
}

fn global_settings_patch_digest(expected_revision: u64, settings: &GlobalSettings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-GLOBAL-SETTINGS-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hash_component(
        &mut hasher,
        profile_from_content(settings.default_profile)
            .id()
            .as_bytes(),
    );
    hasher.update([
        u8::from(settings.providers.openalex.enabled),
        u8::from(settings.providers.semantic_scholar.enabled),
        u8::from(settings.providers.crossref.enabled),
        u8::from(settings.providers.pubmed.enabled),
        u8::from(settings.providers.arxiv.enabled),
        u8::from(settings.providers.openalex.api_key_ref.is_some()),
        u8::from(settings.providers.semantic_scholar.api_key_ref.is_some()),
        u8::from(settings.providers.pubmed.api_key_ref.is_some()),
    ]);
    hash_optional_email(&mut hasher, settings.providers.openalex.email.as_ref());
    hash_optional_email(&mut hasher, settings.providers.crossref.email.as_ref());
    lower_hex(&hasher.finalize())
}

fn new_secret_ref() -> Result<SecretRef, &'static str> {
    let mut identifier = [0_u8; 16];
    getrandom::fill(&mut identifier).map_err(|_| "provider-secret-reference-unavailable")?;
    SecretRef::parse(&format!("qsr1_{}", lower_hex(&identifier)))
        .map_err(|_| "provider-secret-reference-unavailable")
}

fn new_agent_backend_secret_ref() -> Result<SecretRef, &'static str> {
    let mut identifier = [0_u8; 16];
    getrandom::fill(&mut identifier).map_err(|_| "agent-backend-secret-reference-unavailable")?;
    SecretRef::parse(&format!("qsr1_{}", lower_hex(&identifier)))
        .map_err(|_| "agent-backend-secret-reference-unavailable")
}

fn provider_secret_digest(
    expected_revision: u64,
    provider: ProviderKind,
    secret_ref: &SecretRef,
    replacing: bool,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-PROVIDER-SECRET-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hasher.update([provider as u8, u8::from(replacing)]);
    hash_component(&mut hasher, secret_ref.storage_key().as_bytes());
    lower_hex(&hasher.finalize())
}

fn agent_backend_settings_digest(expected_revision: u64, settings: &GlobalSettings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-AGENT-BACKEND-SETTINGS-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hasher.update([
        u8::from(settings.agent_backends.openai.enabled),
        u8::from(settings.agent_backends.openai.api_key_ref.is_some()),
    ]);
    lower_hex(&hasher.finalize())
}

fn agent_backend_secret_digest(
    expected_revision: u64,
    secret_ref: &SecretRef,
    replacing: bool,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-AGENT-BACKEND-SECRET-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hasher.update([u8::from(replacing)]);
    hash_component(&mut hasher, secret_ref.storage_key().as_bytes());
    lower_hex(&hasher.finalize())
}

fn agent_run_digest(project_id: &ProjectId, project_revision: u64, prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-AGENT-RUN-V1\0");
    hash_component(&mut hasher, project_id.as_str().as_bytes());
    hasher.update(project_revision.to_be_bytes());
    hash_component(&mut hasher, prompt.as_bytes());
    lower_hex(&hasher.finalize())
}

fn agent_run_result_view(result: AgentRunResultV1) -> AgentRunResultView {
    AgentRunResultView {
        schema_version: result.schema_version,
        run_id: result.run_id.as_str().to_owned(),
        backend_id: result.backend_id.as_str().to_owned(),
        model: result.model,
        finish_reason: match result.finish_reason {
            AgentFinishReason::Stop => "stop",
            AgentFinishReason::Length => "length",
            AgentFinishReason::ToolRequest => "tool-request",
        },
        content: PrivateDisplayText::new(result.content),
        input_tokens: result.provider_usage.input_tokens,
        output_tokens: result.provider_usage.output_tokens,
        cached_input_tokens: result.provider_usage.cached_input_tokens,
        model_turns: result.execution_usage.model_turns,
        tool_calls: result.execution_usage.tool_calls,
        network_requests: result.execution_usage.network_requests,
        audited_tool_calls: result.tool_audits.len(),
    }
}

fn skills_materialization_digest(pack_sha256: &str, profile: ProfileKind, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-MATERIALIZATION-V1\0");
    hash_component(&mut hasher, pack_sha256.as_bytes());
    hash_component(&mut hasher, profile.id().as_bytes());
    hash_path(&mut hasher, path);
    lower_hex(&hasher.finalize())
}

fn skills_removal_digest(receipt: &MaterializationReceiptV1, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-REMOVAL-V1\0");
    hash_component(&mut hasher, receipt.pack_sha256.as_bytes());
    hash_component(&mut hasher, receipt.content_root_sha256.as_bytes());
    hash_component(
        &mut hasher,
        profile_from_content(receipt.profile).id().as_bytes(),
    );
    hasher.update(
        u64::try_from(receipt.entries.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for entry in &receipt.entries {
        hash_component(&mut hasher, entry.path.as_bytes());
        hash_component(&mut hasher, entry.sha256.as_bytes());
    }
    hash_path(&mut hasher, path);
    lower_hex(&hasher.finalize())
}

fn packaged_product_removal_digest(
    product: &VerifiedPackagedProduct,
    verifications: &[PackagedProductInstallVerification],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-PACKAGED-PRODUCT-REMOVAL-V1\0");
    hash_component(&mut hasher, product.control_sha256().as_bytes());
    for verification in verifications {
        hasher.update([verification.target as u8]);
        hash_component(
            &mut hasher,
            verification.activation_transaction_id.as_bytes(),
        );
        hash_component(&mut hasher, verification.source.binary_sha256.as_bytes());
        hash_component(
            &mut hasher,
            verification.source.resource_pack_sha256.as_bytes(),
        );
    }
    lower_hex(&hasher.finalize())
}

fn hash_optional_email(hasher: &mut Sha256, value: Option<&EmailAddress>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hash_component(hasher, value.as_str().as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn hash_component(hasher: &mut Sha256, value: &[u8]) {
    hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(value);
}

#[cfg(unix)]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    use std::os::unix::ffi::OsStrExt;

    hash_component(hasher, path.as_os_str().as_bytes());
}

#[cfg(windows)]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    use std::os::windows::ffi::OsStrExt;

    let encoded = path
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    hash_component(hasher, &encoded);
}

#[cfg(not(any(unix, windows)))]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    hash_component(hasher, path.to_string_lossy().as_bytes());
}

fn display_path(path: &Path) -> String {
    let mut display = path
        .to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_control() {
                '�'
            } else {
                character
            }
        })
        .take(1_024)
        .collect::<String>();
    if display.is_empty() {
        display.push_str("<selected-folder>");
    }
    display
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}

fn mcp_self_test_counts(snapshot: &DesktopSnapshotV1) -> McpSelfTestCounts {
    McpSelfTestCounts {
        enabled_providers: snapshot
            .config
            .providers
            .iter()
            .filter(|provider| provider.enabled)
            .count(),
        ready_providers: snapshot
            .config
            .providers
            .iter()
            .filter(|provider| {
                provider.enabled && provider.readiness == ProviderReadinessView::Ready
            })
            .count(),
        discovered_clients: snapshot
            .integrations
            .iter()
            .filter(|integration| {
                !matches!(
                    integration.discovery,
                    IntegrationDiscoveryState::NotDiscovered
                        | IntegrationDiscoveryState::Unavailable
                )
            })
            .count(),
        registered_clients: snapshot
            .integrations
            .iter()
            .filter(|integration| {
                integration.registration == StatusCode::Ready
                    && !matches!(
                        integration.discovery,
                        IntegrationDiscoveryState::NotDiscovered
                            | IntegrationDiscoveryState::Unavailable
                    )
            })
            .count(),
    }
}

const fn mcp_counts_from_view(view: &McpSelfTestView) -> McpSelfTestCounts {
    McpSelfTestCounts {
        enabled_providers: view.enabled_provider_count,
        ready_providers: view.ready_provider_count,
        discovered_clients: view.discovered_client_count,
        registered_clients: view.registered_client_count,
    }
}

fn update_store(environment: &CommandEnvironment) -> Result<UpdateStateStore, &'static str> {
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    Ok(UpdateStateStore::new(root, default_update_stream()))
}

fn default_update_stream() -> UpdateStreamPreference {
    if env!("CARGO_PKG_VERSION").contains("-alpha.") || env!("CARGO_PKG_VERSION").contains("-beta.")
    {
        UpdateStreamPreference::Beta
    } else {
        UpdateStreamPreference::Stable
    }
}

fn with_native_update_context<T>(
    environment: &CommandEnvironment,
    operation: impl FnOnce(
        &UpdateStateStore,
        Option<&qiongli_platform::NativeReleaseAuthority>,
        &EmbeddedContent,
    ) -> Result<T, &'static str>,
) -> Result<T, &'static str> {
    let store = update_store(environment)?;
    let authority = crate::embedded_release_authority()
        .map_err(|_| "native-update-release-authority-invalid")?;
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    operation(&store, authority.as_ref(), &content)
}

fn update_snapshot(environment: &CommandEnvironment) -> UpdateView {
    let stream = update_stream_view(default_update_stream());
    if OperatingSystem::current() != Some(OperatingSystem::Macos)
        || Architecture::current() != Some(Architecture::Aarch64)
    {
        return update_unavailable_view(
            stream,
            "native-update-target-unsupported",
            UpdateRemediation::UseSupportedPlatform,
        );
    }
    let authority = match crate::embedded_release_authority() {
        Ok(Some(authority)) => authority,
        Ok(None) => {
            return update_unavailable_view(
                stream,
                "native-update-release-authority-unavailable",
                UpdateRemediation::InstallTrustedRelease,
            );
        }
        Err(_) => {
            return update_unavailable_view(
                stream,
                "native-update-release-authority-invalid",
                UpdateRemediation::ReinstallApplication,
            );
        }
    };
    if authority
        .validate_product_version(env!("CARGO_PKG_VERSION"))
        .is_err()
        || crate::embedded_macos_team_id().is_none()
    {
        return update_unavailable_view(
            stream,
            "native-update-release-authority-invalid",
            UpdateRemediation::ReinstallApplication,
        );
    }
    let store = match update_store(environment) {
        Ok(store) => store,
        Err(code) => return update_failure_base(stream, None, code, false, false),
    };
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(ConfigError::RecoveryRequired) => {
            return UpdateView {
                status: StatusCode::RecoveryRequired,
                selected_stream: stream,
                phase: UpdatePhaseView::RecoveryRequired,
                available_version: None,
                archive_size_bytes: None,
                progress: None,
                reason_code: "native-update-recovery-required",
                remediation: UpdateRemediation::RestartApplication,
                can_select_stream: false,
                can_check: false,
                can_prepare: false,
                can_install: false,
                can_cancel: false,
            };
        }
        Err(error) => {
            return update_failure_base(stream, None, error.reason_code(), false, false);
        }
    };
    let stream = update_stream_view(loaded.state.selected_stream);
    let Some(transaction) = loaded.state.active_transaction else {
        return update_idle_view(stream);
    };
    match transaction.phase {
        UpdateTransactionPhase::Downloading | UpdateTransactionPhase::Cancelling => {
            update_failure_base(
                stream,
                Some(transaction.target_version),
                "native-update-preparation-interrupted",
                true,
                false,
            )
        }
        UpdateTransactionPhase::Downloaded | UpdateTransactionPhase::Verified => {
            update_failure_base(
                stream,
                Some(transaction.target_version),
                "native-update-preparation-interrupted",
                true,
                true,
            )
        }
        UpdateTransactionPhase::Staged | UpdateTransactionPhase::ReconciliationPrepared => {
            UpdateView {
                status: StatusCode::Attention,
                selected_stream: stream,
                phase: UpdatePhaseView::ReadyToInstall,
                available_version: Some(transaction.target_version),
                archive_size_bytes: None,
                progress: Some(UpdateProgressView {
                    completed_steps: 3,
                    total_steps: 4,
                    label: "Verified and prepared",
                    indeterminate: false,
                }),
                reason_code: "update-ready-to-install",
                remediation: UpdateRemediation::None,
                can_select_stream: false,
                can_check: false,
                can_prepare: false,
                can_install: true,
                can_cancel: true,
            }
        }
        UpdateTransactionPhase::AwaitingExit
        | UpdateTransactionPhase::Activating
        | UpdateTransactionPhase::HealthWindow => UpdateView {
            status: StatusCode::Busy,
            selected_stream: stream,
            phase: UpdatePhaseView::AwaitingRestart,
            available_version: Some(transaction.target_version),
            archive_size_bytes: None,
            progress: Some(UpdateProgressView {
                completed_steps: 4,
                total_steps: 4,
                label: "Completing application replacement",
                indeterminate: true,
            }),
            reason_code: "update-restart-in-progress",
            remediation: UpdateRemediation::RestartApplication,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
        UpdateTransactionPhase::RecoveryRequired => UpdateView {
            status: StatusCode::RecoveryRequired,
            selected_stream: stream,
            phase: UpdatePhaseView::RecoveryRequired,
            available_version: Some(transaction.target_version),
            archive_size_bytes: None,
            progress: None,
            reason_code: "native-update-recovery-required",
            remediation: UpdateRemediation::ReinstallApplication,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
    }
}

fn update_idle_view(stream: UpdateStreamView) -> UpdateView {
    UpdateView {
        status: StatusCode::Ready,
        selected_stream: stream,
        phase: UpdatePhaseView::Idle,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code: "update-ready",
        remediation: UpdateRemediation::None,
        can_select_stream: true,
        can_check: true,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_unavailable_view(
    stream: UpdateStreamView,
    reason_code: &'static str,
    remediation: UpdateRemediation,
) -> UpdateView {
    UpdateView {
        status: StatusCode::Unavailable,
        selected_stream: stream,
        phase: UpdatePhaseView::Unavailable,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code,
        remediation,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_busy_view(
    previous: &UpdateView,
    phase: UpdatePhaseView,
    completed_steps: u8,
    label: &'static str,
) -> UpdateView {
    UpdateView {
        status: StatusCode::Busy,
        selected_stream: previous.selected_stream,
        phase,
        available_version: previous.available_version.clone(),
        archive_size_bytes: previous.archive_size_bytes,
        progress: Some(UpdateProgressView {
            completed_steps,
            total_steps: 4,
            label,
            indeterminate: matches!(
                phase,
                UpdatePhaseView::Checking
                    | UpdatePhaseView::Downloading
                    | UpdatePhaseView::Installing
                    | UpdatePhaseView::AwaitingRestart
                    | UpdatePhaseView::Cancelling
            ),
        }),
        reason_code: match phase {
            UpdatePhaseView::Checking => "update-checking",
            UpdatePhaseView::Downloading => "update-downloading",
            UpdatePhaseView::Verifying => "update-verifying",
            UpdatePhaseView::Staging => "update-staging",
            UpdatePhaseView::Installing => "update-installing",
            UpdatePhaseView::AwaitingRestart => "update-restarting",
            UpdatePhaseView::Cancelling => "update-cancelling",
            _ => "update-busy",
        },
        remediation: UpdateRemediation::None,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        // The in-process preparation worker does not yet expose cooperative
        // cancellation. Advertising Cancel here would start a second worker
        // while download/verification/staging still owns the transaction.
        // Interrupted or fully staged transactions remain cancellable through
        // `update_snapshot`, after no preparation worker is active.
        can_cancel: false,
    }
}

fn update_preparation_progress_view(
    stream: UpdateStreamView,
    available_version: Option<String>,
    archive_size_bytes: Option<u64>,
    stage: crate::update_cli::DesktopUpdatePreparationStage,
) -> UpdateView {
    let base = UpdateView {
        status: StatusCode::Busy,
        selected_stream: stream,
        phase: UpdatePhaseView::Downloading,
        available_version,
        archive_size_bytes,
        progress: None,
        reason_code: "update-downloading",
        remediation: UpdateRemediation::None,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    };
    match stage {
        crate::update_cli::DesktopUpdatePreparationStage::Downloading => update_busy_view(
            &base,
            UpdatePhaseView::Downloading,
            1,
            "Downloading signed package",
        ),
        crate::update_cli::DesktopUpdatePreparationStage::Verifying => update_busy_view(
            &base,
            UpdatePhaseView::Verifying,
            2,
            "Verifying signatures and evidence",
        ),
        crate::update_cli::DesktopUpdatePreparationStage::Staging => update_busy_view(
            &base,
            UpdatePhaseView::Staging,
            3,
            "Preparing the native application",
        ),
    }
}

fn update_checked_view(checked: crate::update_cli::DesktopUpdateCheck) -> UpdateView {
    let stream = update_stream_view(checked.selected_stream);
    match checked.disposition {
        crate::update_cli::DesktopUpdateCheckDisposition::Current => UpdateView {
            status: StatusCode::Ready,
            selected_stream: stream,
            phase: UpdatePhaseView::Current,
            available_version: Some(checked.target_version),
            archive_size_bytes: None,
            progress: None,
            reason_code: "update-current",
            remediation: UpdateRemediation::None,
            can_select_stream: true,
            can_check: true,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
        crate::update_cli::DesktopUpdateCheckDisposition::Available => UpdateView {
            status: StatusCode::Attention,
            selected_stream: stream,
            phase: UpdatePhaseView::Available,
            available_version: Some(checked.target_version),
            archive_size_bytes: Some(checked.archive_size_bytes),
            progress: None,
            reason_code: "update-available",
            remediation: UpdateRemediation::None,
            can_select_stream: true,
            can_check: true,
            can_prepare: true,
            can_install: false,
            can_cancel: false,
        },
    }
}

fn update_prepared_view(
    environment: &CommandEnvironment,
    prepared: crate::update_cli::DesktopPreparedUpdate,
) -> UpdateView {
    let mut view = update_snapshot(environment);
    view.available_version = Some(prepared.target_version);
    view
}

fn update_install_handoff_view(handoff: crate::update_cli::DesktopInstallHandoff) -> UpdateView {
    UpdateView {
        status: StatusCode::Busy,
        selected_stream: update_stream_view(handoff.selected_stream),
        phase: UpdatePhaseView::AwaitingRestart,
        available_version: Some(handoff.target_version),
        archive_size_bytes: None,
        progress: Some(UpdateProgressView {
            completed_steps: 4,
            total_steps: 4,
            label: "Closing Qiongli for atomic replacement",
            indeterminate: true,
        }),
        reason_code: "update-helper-launched",
        remediation: UpdateRemediation::RestartApplication,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_cancelled_view(stream: UpdateStreamView) -> UpdateView {
    UpdateView {
        status: StatusCode::Missing,
        selected_stream: stream,
        phase: UpdatePhaseView::Cancelled,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code: "update-cancelled",
        remediation: UpdateRemediation::RetryCheck,
        can_select_stream: true,
        can_check: true,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_failure_view(environment: &CommandEnvironment, code: &'static str) -> UpdateView {
    let store = update_store(environment).ok();
    let loaded = store.as_ref().and_then(|store| store.load().ok());
    let stream = loaded.as_ref().map_or_else(
        || update_stream_view(default_update_stream()),
        |loaded| update_stream_view(loaded.state.selected_stream),
    );
    let transaction = loaded
        .as_ref()
        .and_then(|loaded| loaded.state.active_transaction.as_ref());
    let available_version = transaction.map(|transaction| transaction.target_version.clone());
    let can_cancel = transaction.is_some_and(|transaction| {
        matches!(
            transaction.phase,
            UpdateTransactionPhase::Downloading
                | UpdateTransactionPhase::Downloaded
                | UpdateTransactionPhase::Verified
                | UpdateTransactionPhase::Staged
                | UpdateTransactionPhase::ReconciliationPrepared
                | UpdateTransactionPhase::Cancelling
        )
    });
    let can_prepare = transaction.is_some_and(|transaction| {
        matches!(
            transaction.phase,
            UpdateTransactionPhase::Downloaded
                | UpdateTransactionPhase::Verified
                | UpdateTransactionPhase::Staged
                | UpdateTransactionPhase::ReconciliationPrepared
        )
    });
    update_failure_base(stream, available_version, code, can_cancel, can_prepare)
}

fn update_failure_base(
    stream: UpdateStreamView,
    available_version: Option<String>,
    code: &'static str,
    can_cancel: bool,
    can_prepare: bool,
) -> UpdateView {
    let remediation = update_remediation(code, can_cancel, can_prepare);
    UpdateView {
        status: if code.contains("recovery-required") {
            StatusCode::RecoveryRequired
        } else {
            StatusCode::Blocked
        },
        selected_stream: stream,
        phase: if code.contains("recovery-required") {
            UpdatePhaseView::RecoveryRequired
        } else {
            UpdatePhaseView::Failed
        },
        available_version,
        archive_size_bytes: None,
        progress: None,
        reason_code: code,
        remediation,
        can_select_stream: !can_cancel && !can_prepare,
        can_check: !can_cancel && !can_prepare,
        can_prepare,
        can_install: false,
        can_cancel,
    }
}

fn update_remediation(
    code: &'static str,
    can_cancel: bool,
    can_prepare: bool,
) -> UpdateRemediation {
    if code == "native-update-target-unsupported" {
        UpdateRemediation::UseSupportedPlatform
    } else if code.contains("release-authority") || code.contains("macos-team-id") {
        UpdateRemediation::InstallTrustedRelease
    } else if code == "native-update-installation-location-not-writable"
        || code == "native-update-installation-location-unsafe"
    {
        UpdateRemediation::MoveToApplications
    } else if code.contains("recovery-required")
        || code.contains("atomic-replacement")
        || code.contains("health-check")
    {
        UpdateRemediation::ReinstallApplication
    } else if can_prepare {
        UpdateRemediation::RetryPreparation
    } else if can_cancel {
        UpdateRemediation::CancelAndRetry
    } else {
        UpdateRemediation::RetryCheck
    }
}

const fn update_stream_view(stream: UpdateStreamPreference) -> UpdateStreamView {
    match stream {
        UpdateStreamPreference::Stable => UpdateStreamView::Stable,
        UpdateStreamPreference::Beta => UpdateStreamView::Beta,
    }
}

const fn update_stream_preference(stream: UpdateStreamView) -> UpdateStreamPreference {
    match stream {
        UpdateStreamView::Stable => UpdateStreamPreference::Stable,
        UpdateStreamView::Beta => UpdateStreamPreference::Beta,
    }
}

fn update_install_digest(revision: u64, transaction_id: &str, target_version: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-UPDATE-INSTALL-V1\0");
    hasher.update(revision.to_be_bytes());
    hash_component(&mut hasher, transaction_id.as_bytes());
    hash_component(&mut hasher, target_version.as_bytes());
    lower_hex(&hasher.finalize())
}

impl DesktopService for NativeDesktopService {
    fn snapshot(&mut self) -> DesktopSnapshotV1 {
        let mut snapshot =
            build_snapshot(&self.environment, &self.content, self.secret_store.as_ref());
        snapshot.update = self.update_view.clone();
        snapshot.capabilities.apply = !self.activation_sessions.is_empty()
            || !self.candidate_sessions.is_empty()
            || self.packaged_product.product.is_some();
        snapshot.product.trust = if self.packaged_product.product.is_some() {
            ProductTrustView::PackagedProductControl
        } else {
            ProductTrustView::SourceBuild
        };
        for integration in &mut snapshot.integrations {
            let authority_available = self
                .activation_sessions
                .iter()
                .any(|session| session.target == integration.target)
                || self
                    .candidate_sessions
                    .iter()
                    .any(|session| session.target == integration.target)
                || self
                    .packaged_product
                    .product
                    .as_ref()
                    .and_then(|product| product.capability(activation_target(integration.target)))
                    .is_some();
            integration.candidate_required = integration.discovery
                == IntegrationDiscoveryState::DiscoveredUnmanaged
                && !authority_available;
        }
        snapshot
    }

    fn execute(&mut self, intent: DesktopIntent) -> DesktopEvent {
        match intent {
            DesktopIntent::Refresh => {
                self.environment.detect_client_versions();
                DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
            }
            DesktopIntent::RunLiteMcpSelfTest => self.start_mcp_self_test(),
            DesktopIntent::PollLiteMcpSelfTest => self.poll_mcp_self_test(),
            DesktopIntent::CancelLiteMcpSelfTest => self.cancel_mcp_self_test(),
            DesktopIntent::SelectUpdateStream { stream } => self.select_update_stream(stream),
            DesktopIntent::CheckForUpdates => self.start_update_check(),
            DesktopIntent::PrepareUpdate => self.start_update_preparation(),
            DesktopIntent::PollUpdate => self.poll_update(),
            DesktopIntent::CancelUpdate => self.cancel_update(),
            DesktopIntent::PreviewUpdateInstall => self.preview_update_install(),
            DesktopIntent::RefreshIntegrationDiscovery => {
                self.environment.detect_client_versions();
                DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
            }
            DesktopIntent::PreviewGlobalSettingsPatch(patch) => self.preview_global_settings(patch),
            DesktopIntent::PreviewProviderSettingsPatch(patch) => {
                self.preview_provider_settings(patch)
            }
            DesktopIntent::PreviewProviderSecretChange { provider, change } => {
                self.preview_provider_secret(provider, change)
            }
            DesktopIntent::PreviewAgentBackendSettingsPatch(patch) => {
                self.preview_agent_backend_settings(patch)
            }
            DesktopIntent::PreviewAgentBackendSecretChange { change } => {
                self.preview_agent_backend_secret(change)
            }
            DesktopIntent::PreviewAgentRun(draft) => self.preview_agent_run(draft),
            DesktopIntent::TestOpenAiBackend => self.test_openai_backend(),
            DesktopIntent::TestLiteratureProvider { provider } => {
                self.test_literature_provider(provider)
            }
            DesktopIntent::SelectSkillsDestination => self.select_skills_destination(),
            DesktopIntent::PreviewSkillsMaterialization { profile } => {
                self.preview_skills_materialization(profile)
            }
            DesktopIntent::VerifySkillsMaterialization => self.verify_skills_materialization(),
            DesktopIntent::PreviewSkillsRemoval => self.preview_skills_removal(),
            DesktopIntent::PreviewSkillsPresetMaterialization { profile, preset } => {
                self.preview_skills_preset_materialization(profile, preset)
            }
            DesktopIntent::VerifySkillsPreset { preset } => {
                self.verify_skills_preset(preset)
            }
            DesktopIntent::PreviewSkillsPresetRemoval { preset } => {
                self.preview_skills_preset_removal(preset)
            }
            DesktopIntent::PreviewProviderPublicSetting {
                provider,
                public_email,
            } => {
                if !matches!(provider, ProviderKind::Crossref | ProviderKind::OpenAlex)
                    || EmailAddress::parse(public_email.expose()).is_err()
                {
                    return DesktopEvent::ValidationFailed {
                        code: "provider-public-setting-invalid",
                    };
                }
                self.issue_preview(
                    "Provider setting preview",
                    "The public contact email is valid. No value was stored.",
                    "config-write-unavailable",
                )
            }
            DesktopIntent::PreviewIntegration { target } => self.preview_activation(target),
            DesktopIntent::PreviewInstallRecommended => {
                let selection = self.recommended_integration_selection();
                self.preview_packaged_product_batch(
                    selection,
                    "Install recommended integrations",
                    "Install the receipt-owned Qiongli Lite source, Skills, MCP attachment, and registration for every detected supported client in one compensating transaction.",
                )
            }
            DesktopIntent::PreviewInstallSelected { selection } => {
                self.preview_packaged_product_batch(
                    selection,
                    "Install selected integrations",
                    "Install the receipt-owned Qiongli Lite source, Skills, MCP attachment, and registration for the selected clients in one compensating transaction.",
                )
            }
            DesktopIntent::VerifyIntegrations { selection } => {
                self.verify_packaged_integrations(selection)
            }
            DesktopIntent::PreviewRepairAll => {
                let selection = self.repair_integration_selection();
                if selection.is_empty() {
                    DesktopEvent::Completed {
                        code: "packaged-product-repair-not-required",
                    }
                } else {
                    self.preview_packaged_product_batch(
                        selection,
                        "Repair all integrations",
                        "Repair every detected receipt-owned integration that is missing registration while preserving unmanaged content.",
                    )
                }
            }
            DesktopIntent::PreviewUpdateIntegrations { selection } => {
                self.preview_packaged_product_batch(
                    selection,
                    "Update selected integrations",
                    "Reconcile selected receipt-owned integrations with the exact Skills and Lite MCP content embedded in this App.",
                )
            }
            DesktopIntent::PreviewRemoveIntegrations { selection } => {
                self.preview_packaged_product_removal(selection)
            }
            DesktopIntent::ConfirmOperation { token } => {
                let Some(operation) = self.active_operation.as_ref() else {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                };
                if operation.token() != token {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                let operation = self
                    .active_operation
                    .take()
                    .expect("validated active operation remains available");
                match operation {
                    PendingDesktopOperation::Blocked(_) => DesktopEvent::Failed {
                        code: "desktop-apply-unavailable",
                    },
                    PendingDesktopOperation::GlobalSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "global-settings-updated-cleanup-required"
                                } else {
                                    "global-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::ProviderSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "provider-settings-updated-cleanup-required"
                                } else {
                                    "provider-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::AgentBackendSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "agent-backend-settings-updated-cleanup-required"
                                } else {
                                    "agent-backend-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::ProviderSecret {
                        expected_revision,
                        provider,
                        replacement,
                        secret_ref,
                        replacement_value,
                        previous_value,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let credential_write = if let Some(value) = replacement_value.as_ref() {
                            self.secret_store.store(&secret_ref, value)
                        } else if previous_value.is_some() {
                            self.secret_store.remove(&secret_ref)
                        } else {
                            Ok(())
                        };
                        if let Err(error) = credential_write {
                            return DesktopEvent::Failed {
                                code: error.remediation_code(),
                            };
                        }
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: match (provider, replacement_value.is_some(), outcome.cleanup_required) {
                                    (_, _, true) => "provider-secret-updated-cleanup-required",
                                    (ProviderKind::OpenAlex, true, false) => "openalex-api-key-saved",
                                    (ProviderKind::SemanticScholar, true, false) => {
                                        "semantic-scholar-api-key-saved"
                                    }
                                    (ProviderKind::OpenAlex, false, false) => {
                                        "openalex-api-key-removed"
                                    }
                                    (ProviderKind::SemanticScholar, false, false) => {
                                        "semantic-scholar-api-key-removed"
                                    }
                                    (ProviderKind::Crossref | ProviderKind::PubMed | ProviderKind::Arxiv, _, false) => {
                                        "provider-secret-updated"
                                    }
                                },
                            },
                            Err(error) => {
                                let compensated = if let Some(previous) = previous_value.as_ref() {
                                    self.secret_store.store(&secret_ref, previous).is_ok()
                                } else if replacement_value.is_some() {
                                    self.secret_store.remove(&secret_ref).is_ok()
                                } else {
                                    true
                                };
                                DesktopEvent::Failed {
                                    code: if compensated {
                                        error.reason_code()
                                    } else {
                                        "provider-secret-recovery-required"
                                    },
                                }
                            }
                        }
                    }
                    PendingDesktopOperation::AgentBackendSecret {
                        expected_revision,
                        replacement,
                        secret_ref,
                        replacement_value,
                        previous_value,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let credential_write = if let Some(value) = replacement_value.as_ref() {
                            self.secret_store.store(&secret_ref, value)
                        } else if previous_value.is_some() {
                            self.secret_store.remove(&secret_ref)
                        } else {
                            Ok(())
                        };
                        if let Err(error) = credential_write {
                            return DesktopEvent::Failed {
                                code: error.remediation_code(),
                            };
                        }
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: match (replacement_value.is_some(), outcome.cleanup_required) {
                                    (_, true) => {
                                        "agent-backend-secret-updated-cleanup-required"
                                    }
                                    (true, false) => "openai-api-key-saved",
                                    (false, false) => "openai-api-key-removed",
                                },
                            },
                            Err(error) => {
                                let compensated = if let Some(previous) = previous_value.as_ref() {
                                    self.secret_store.store(&secret_ref, previous).is_ok()
                                } else if replacement_value.is_some() {
                                    self.secret_store.remove(&secret_ref).is_ok()
                                } else {
                                    true
                                };
                                DesktopEvent::Failed {
                                    code: if compensated {
                                        error.reason_code()
                                    } else {
                                        "agent-backend-secret-recovery-required"
                                    },
                                }
                            }
                        }
                    }
                    PendingDesktopOperation::AgentRun { request, .. } => {
                        let Some(projects) = project_state_service(&self.environment) else {
                            return DesktopEvent::Failed {
                                code: "project-service-unavailable",
                            };
                        };
                        let registry = match FullProjectToolRegistry::from_embedded_content(
                            &self.content,
                        ) {
                            Ok(registry) => registry,
                            Err(_) => {
                                return DesktopEvent::Failed {
                                    code: "agent-run-tools-unavailable",
                                };
                            }
                        };
                        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
                            Ok(loaded) => loaded,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let service = FullAgentRunService::new(projects, registry);
                        match service.run_openai(
                            request,
                            &loaded.settings,
                            Arc::clone(&self.secret_store),
                        ) {
                            Ok(result) => {
                                DesktopEvent::AgentRunCompleted(agent_run_result_view(result))
                            }
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::SkillsMaterialization {
                        profile, target, ..
                    } => {
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let previous = verify_materialization(&target).ok();
                        match self.content.materialize_profile(profile.id(), &target) {
                        Ok(receipt) => {
                            match crate::managed_content::register_managed_materialization(
                                root.state_root(),
                                &target,
                                &receipt,
                            ) {
                                Ok(()) => DesktopEvent::Completed {
                                    code: "skills-materialization-completed",
                                },
                                Err(code) => {
                                    match crate::managed_content::compensate_unregistered_materialization(
                                        &self.content,
                                        &target,
                                        &receipt,
                                        previous.as_ref(),
                                    ) {
                                        Ok(()) => DesktopEvent::Failed { code },
                                        Err(recovery) => DesktopEvent::Failed { code: recovery },
                                    }
                                }
                            }
                        }
                        Err(error) => DesktopEvent::Failed {
                            code: error.reason_code(),
                        },
                    }
                    }
                    PendingDesktopOperation::SkillsRemoval {
                        target,
                        expected_receipt,
                        ..
                    } => {
                        let observed = match verify_materialization(&target) {
                            Ok(observed) => observed,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        if observed != expected_receipt {
                            return DesktopEvent::Failed {
                                code: "materialization-target-changed",
                            };
                        }
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match remove_materialization(&target) {
                            Ok(removed) if removed == expected_receipt => {
                                match crate::managed_content::unregister_managed_materialization(
                                    root.state_root(),
                                    &target,
                                    &expected_receipt,
                                ) {
                                    Ok(()) => DesktopEvent::Completed {
                                        code: "skills-materialization-removed",
                                    },
                                    Err(code) => {
                                        match crate::managed_content::restore_managed_materialization(
                                            &self.content,
                                            &target,
                                            &expected_receipt,
                                        ) {
                                            Ok(()) => DesktopEvent::Failed { code },
                                            Err(recovery) => {
                                                DesktopEvent::Failed { code: recovery }
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(_) => DesktopEvent::Failed {
                                code: "materialization-target-changed",
                            },
                            Err(error) => {
                                DesktopEvent::Failed {
                                    code: error.reason_code(),
                                }
                            }
                        }
                    }
                    PendingDesktopOperation::Activation { target, .. } => {
                        let Some(session) = self
                            .activation_sessions
                            .iter_mut()
                            .find(|session| session.target == target)
                        else {
                            return DesktopEvent::Failed {
                                code: "desktop-activation-session-missing",
                            };
                        };
                        match now_unix().and_then(|now_unix| session.confirm(now_unix)) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::Candidate { target, .. } => {
                        let Some(home) = self.environment.platform_home() else {
                            return DesktopEvent::Failed {
                                code: "native-candidate-home-unavailable",
                            };
                        };
                        let Some(session) = self
                            .candidate_sessions
                            .iter_mut()
                            .find(|session| session.target == target)
                        else {
                            return DesktopEvent::Failed {
                                code: "desktop-candidate-session-missing",
                            };
                        };
                        match now_unix()
                            .and_then(|now_unix| session.confirm(&self.content, home, now_unix))
                        {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::PackagedProduct { target, .. } => {
                        match now_unix().and_then(|now_unix| {
                            self.packaged_product
                                .confirm(&self.content, target, now_unix)
                        }) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::PackagedProductBatch { selection, .. } => {
                        match now_unix().and_then(|now_unix| {
                            self.packaged_product
                                .confirm_batch(&self.content, selection, now_unix)
                        }) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::PackagedProductRemoval { selection, .. } => {
                        match now_unix().and_then(|now_unix| {
                            self.packaged_product.confirm_remove(selection, now_unix)
                        }) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::UpdateInstall {
                        expected_revision, ..
                    } => self.start_update_install(expected_revision),
                }
            }
            DesktopIntent::CancelOperation { token } => {
                if self
                    .active_operation
                    .as_ref()
                    .map(PendingDesktopOperation::token)
                    != Some(token)
                {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                self.cancel_active_operation();
                DesktopEvent::Cancelled {
                    code: "operation-preview-cancelled",
                }
            }
        }
    }
}

fn build_snapshot(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store: &dyn SecretStore,
) -> DesktopSnapshotV1 {
    let manifest = content.pack().manifest();
    let profiles = ProfileKind::ALL.map(|profile| ProfileView {
        profile,
        included_resource_kinds: content
            .profiles()
            .iter()
            .find(|candidate| profile_from_content(candidate.id) == profile)
            .map_or(0, |candidate| candidate.included_resource_kinds.len()),
    });
    let (config, _config_diagnostic) = config_snapshot(environment, secret_store);
    let [(codex, _codex_diagnostic), (claude, _claude_diagnostic)] =
        integration_snapshots(environment);
    let inspection =
        crate::product_diagnostics::inspect_product(environment, content, secret_store.status());
    let diagnostics = inspection.checks.map(diagnostic_check_view);
    let diagnostic_paths = inspection
        .paths
        .into_iter()
        .map(diagnostic_path_view)
        .collect();
    let config_edit = matches!(config.status, StatusCode::Missing | StatusCode::Ready)
        && config.revision.is_some();
    DesktopSnapshotV1 {
        schema_version: DESKTOP_SNAPSHOT_SCHEMA_VERSION,
        product: ProductView {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            build: crate::embedded_source_commit()
                .unwrap_or("source-build")
                .to_owned(),
            operating_system: operating_system_view(OperatingSystem::current()),
            architecture: architecture_view(Architecture::current()),
            trust: ProductTrustView::SourceBuild,
        },
        content: ContentView {
            status: StatusCode::Ready,
            pack_id: manifest.pack_id.clone(),
            content_version: manifest.content_version.clone(),
            entry_count: manifest.entries.len(),
            profiles,
        },
        mcp: McpView {
            status: StatusCode::Ready,
            profile: ProfileKind::MarketplaceLite,
            public_tool_count: LITE_PUBLIC_TOOL_NAMES.len(),
        },
        config,
        update: update_snapshot(environment),
        integrations: [codex, claude],
        diagnostics,
        diagnostic_paths,
        capabilities: CapabilityView {
            refresh: true,
            config_edit,
            skills_materialize: OperatingSystem::current().is_some(),
            provider_preview: true,
            mcp_self_test: true,
            integration_discovery: true,
            integration_preview: true,
            apply: false,
        },
    }
}

fn diagnostic_check_view(
    check: crate::product_diagnostics::ProductDoctorCheckV1,
) -> DiagnosticCheckView {
    use crate::product_diagnostics::{ProductDoctorCheckId as Check, ProductDoctorStatus as State};

    let id = match check.id {
        Check::EmbeddedContent => DiagnosticCheckId::EmbeddedContent,
        Check::GlobalConfig => DiagnosticCheckId::GlobalConfig,
        Check::SecureStore => DiagnosticCheckId::SecureStore,
        Check::ManagedContent => DiagnosticCheckId::ManagedContent,
        Check::CodexLocal => DiagnosticCheckId::CodexLocal,
        Check::ClaudeCodeLocal => DiagnosticCheckId::ClaudeCodeLocal,
        Check::LiteMcp => DiagnosticCheckId::LiteMcp,
        Check::LiteratureProviders => DiagnosticCheckId::LiteratureProviders,
        Check::UpdateRecovery => DiagnosticCheckId::UpdateRecovery,
        Check::FullRuntime => DiagnosticCheckId::FullRuntime,
    };
    DiagnosticCheckView {
        check: id,
        status: match check.status {
            State::Ready => StatusCode::Ready,
            State::Attention => StatusCode::Attention,
            State::Missing => StatusCode::Missing,
            State::Unavailable => StatusCode::Unavailable,
            State::Invalid => StatusCode::Invalid,
            State::FutureSchema => StatusCode::FutureSchema,
            State::Insecure => StatusCode::Insecure,
            State::Busy => StatusCode::Busy,
            State::WriteUnsupported => StatusCode::WriteUnsupported,
            State::RecoveryRequired => StatusCode::RecoveryRequired,
            State::Deferred => StatusCode::Disabled,
        },
        blocking: check.blocking,
        remediation: diagnostic_remediation(id, check.remediation),
    }
}

fn diagnostic_remediation(check: DiagnosticCheckId, remediation: &'static str) -> RemediationCode {
    match remediation {
        "none" => RemediationCode::None,
        "inspect-global-config" | "create-global-config" => RemediationCode::InspectGlobalConfig,
        "upgrade-qiongli" => RemediationCode::UpgradeQiongli,
        "repair-global-config-permissions" => RemediationCode::RepairGlobalConfigPermissions,
        "retry-global-config" => RemediationCode::RetryGlobalConfig,
        "recover-global-config" => RemediationCode::RecoverGlobalConfig,
        "use-supported-platform" => RemediationCode::UseSupportedPlatform,
        "use-supported-secure-store" => RemediationCode::UseSupportedSecureStore,
        "inspect-managed-content" => RemediationCode::InspectManagedContent,
        "install-supported-client" => RemediationCode::InstallSupportedClient,
        "install-client-integration" => RemediationCode::InstallClientIntegration,
        "resolve-client-conflict" => RemediationCode::ResolveClientConflict,
        "repair-client-integration" => RemediationCode::RepairClientIntegration,
        "configure-literature-providers" => RemediationCode::ConfigureLiteratureProviders,
        "inspect-update-state" => RemediationCode::InspectUpdateState,
        "reinstall-qiongli" => RemediationCode::ReinstallQiongli,
        "upgrade-to-r4-full-runtime" => RemediationCode::UpgradeToFullRuntime,
        "retry-mcp-self-test" => RemediationCode::RetryLiteMcpSelfTest,
        "inspect-client-paths" if check == DiagnosticCheckId::CodexLocal => {
            RemediationCode::InspectCodexLocal
        }
        "inspect-client-paths" if check == DiagnosticCheckId::ClaudeCodeLocal => {
            RemediationCode::InspectClaudeCodeLocal
        }
        _ if check == DiagnosticCheckId::CodexLocal => RemediationCode::InspectCodexLocal,
        _ if check == DiagnosticCheckId::ClaudeCodeLocal => RemediationCode::InspectClaudeCodeLocal,
        _ => RemediationCode::UseSupportedPlatform,
    }
}

fn diagnostic_path_view(
    path: crate::product_diagnostics::ProductPathInspectionV1,
) -> DiagnosticPathView {
    use crate::product_diagnostics::{
        ProductPathFileType as FileType, ProductPathSafety as Safety,
    };

    let status = match (path.safety, path.file_type, path.type_matches_expected) {
        (Safety::Unsafe, _, _) | (_, FileType::Other, _) | (_, _, Some(false)) => {
            StatusCode::Invalid
        }
        (Safety::Unavailable, _, _) | (_, FileType::Unavailable, _) => StatusCode::Unavailable,
        (_, FileType::Missing, _) => StatusCode::Missing,
        _ => StatusCode::Ready,
    };
    let exact = Path::new(&path.exact_path);
    let reveal_path = display_path(if path.file_type == FileType::Directory {
        exact
    } else {
        exact.parent().unwrap_or(exact)
    });
    DiagnosticPathView {
        id: path.id,
        label: path.label,
        symbolic_path: path.symbolic_path,
        exact_path: PrivateDisplayText::new(path.exact_path),
        reveal_path: PrivateDisplayText::new(reveal_path),
        details: format!(
            "Group: {:?} · Scope: {:?} · Source: {:?} · Type: {:?} (expected {}, match {:?}) · Owner: {:?} · Writability: {:?} · Safety: {:?}",
            path.group,
            path.scope,
            path.source,
            path.file_type,
            path.expected_type,
            path.type_matches_expected,
            path.owner,
            path.writability,
            path.safety,
        ),
        selected: path.selected,
        status,
        resolved_target: path.resolved_target.map(PrivateDisplayText::new),
    }
}

const fn integration_target(target: ClientActivationTarget) -> IntegrationTarget {
    match target {
        ClientActivationTarget::Codex => IntegrationTarget::Codex,
        ClientActivationTarget::ClaudeCode => IntegrationTarget::ClaudeCode,
    }
}

const fn activation_target(target: IntegrationTarget) -> ClientActivationTarget {
    match target {
        IntegrationTarget::Codex => ClientActivationTarget::Codex,
        IntegrationTarget::ClaudeCode => ClientActivationTarget::ClaudeCode,
    }
}

fn selected_activation_targets(
    selection: IntegrationSelection,
) -> Result<Vec<ClientActivationTarget>, &'static str> {
    let mut targets = Vec::with_capacity(2);
    if selection.codex {
        targets.push(ClientActivationTarget::Codex);
    }
    if selection.claude_code {
        targets.push(ClientActivationTarget::ClaudeCode);
    }
    if targets.is_empty() {
        return Err("integration-selection-required");
    }
    Ok(targets)
}

fn blocked_batch_product_preview(
    token: OperationToken,
    title: &'static str,
    blocked_reason: &'static str,
) -> OperationPreview {
    OperationPreview {
        token,
        kind: OperationKind::Activation,
        title,
        summary: "The selected clients were inspected. This process has no verified packaged-product installation authority.",
        display_target: None,
        plan_digest_sha256: None,
        approvals_required: Vec::new(),
        can_confirm: false,
        blocked_reason: Some(blocked_reason),
    }
}

fn blocked_product_preview(
    token: OperationToken,
    target: IntegrationTarget,
    blocked_reason: &'static str,
) -> OperationPreview {
    OperationPreview {
        token,
        kind: OperationKind::Activation,
        title: match target {
            IntegrationTarget::Codex => "Codex installation preview",
            IntegrationTarget::ClaudeCode => "Claude Code installation preview",
        },
        summary: "The local target was inspected. This process has no verified packaged-product installation authority.",
        display_target: None,
        plan_digest_sha256: None,
        approvals_required: Vec::new(),
        can_confirm: false,
        blocked_reason: Some(blocked_reason),
    }
}

fn running_packaged_product(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> PackagedProductState {
    let authority = match crate::embedded_release_authority() {
        Ok(Some(authority)) => authority,
        Ok(None) => return PackagedProductState::read_only("source-build-read-only"),
        Err(error) => return PackagedProductState::read_only(error.reason_code()),
    };
    let Some(source_commit) = crate::embedded_source_commit() else {
        return PackagedProductState::read_only("source-build-read-only");
    };
    let Some(home) = environment.platform_home() else {
        return PackagedProductState::read_only("packaged-product-home-invalid");
    };
    let current_executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            return PackagedProductState::read_only("packaged-product-executable-invalid");
        }
    };
    let desktop_manifest_path = running_desktop_manifest_path(&current_executable);
    if !desktop_manifest_path.is_file() {
        return PackagedProductState::read_only("source-build-read-only");
    }
    let control_path = match packaged_product_control_path(&desktop_manifest_path) {
        Ok(path) => path,
        Err(error) => return PackagedProductState::read_only(error.reason_code()),
    };
    let now_unix = match now_unix() {
        Ok(value) => value,
        Err(code) => return PackagedProductState::read_only(code),
    };
    match verify_packaged_product(&PackagedProductVerificationInput {
        current_executable: &current_executable,
        desktop_manifest_path: &desktop_manifest_path,
        control_path: &control_path,
        release_authority: &authority,
        pack: content.pack(),
        product_version: env!("CARGO_PKG_VERSION"),
        product_source_commit: source_commit,
        home,
        now_unix,
    }) {
        Ok(product) => PackagedProductState::verified(product),
        Err(error) => PackagedProductState::read_only(error.reason_code()),
    }
}

fn running_desktop_manifest_path(current_executable: &Path) -> PathBuf {
    let parent = current_executable.parent().unwrap_or(current_executable);
    if cfg!(target_os = "macos") {
        parent
            .join("../Resources")
            .join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
    } else {
        parent.join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
    }
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system-clock-unavailable")
}

fn config_snapshot(
    environment: &CommandEnvironment,
    secret_store: &dyn SecretStore,
) -> (ConfigView, DiagnosticCheckView) {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return unavailable_config(error),
    };
    let loaded = store.load().ok();
    let status = store.status();
    let view_status = config_status(status.state);
    let providers = status
        .providers
        .as_ref()
        .map_or_else(unavailable_providers, |providers| {
            [
                provider_view(
                    ProviderKind::OpenAlex,
                    &providers.openalex,
                    loaded
                        .as_ref()
                        .is_some_and(|loaded| loaded.settings.providers.openalex.email.is_some()),
                ),
                provider_view(
                    ProviderKind::SemanticScholar,
                    &providers.semantic_scholar,
                    false,
                ),
                provider_view(
                    ProviderKind::Crossref,
                    &providers.crossref,
                    loaded
                        .as_ref()
                        .is_some_and(|loaded| loaded.settings.providers.crossref.email.is_some()),
                ),
                provider_view(ProviderKind::PubMed, &providers.pubmed, false),
                provider_view(ProviderKind::Arxiv, &providers.arxiv, false),
            ]
        });
    let openai_backend = loaded.as_ref().map_or(
        AgentBackendView {
            enabled: false,
            readiness: AgentBackendReadinessView::Disabled,
            secret_reference_present: false,
            test_available: false,
        },
        |loaded| {
            let backend = openai_backend_status(&loaded.settings, secret_store);
            AgentBackendView {
                enabled: backend.enabled,
                readiness: match backend.readiness {
                    BackendReadinessV1::Disabled => AgentBackendReadinessView::Disabled,
                    BackendReadinessV1::NeedsSecretReference => {
                        AgentBackendReadinessView::NeedsSecretReference
                    }
                    BackendReadinessV1::SecretStoreUnavailable => {
                        AgentBackendReadinessView::SecretStoreUnavailable
                    }
                    BackendReadinessV1::CredentialMissing => {
                        AgentBackendReadinessView::CredentialMissing
                    }
                    BackendReadinessV1::CredentialInvalid => {
                        AgentBackendReadinessView::CredentialInvalid
                    }
                    BackendReadinessV1::Ready => AgentBackendReadinessView::Ready,
                },
                secret_reference_present: loaded
                    .settings
                    .agent_backends
                    .openai
                    .api_key_ref
                    .is_some(),
                test_available: backend.test_available,
            }
        },
    );
    let diagnostic = DiagnosticCheckView {
        check: DiagnosticCheckId::GlobalConfig,
        status: view_status,
        blocking: config_is_blocking(status.state),
        remediation: config_remediation(status.state),
    };
    (
        ConfigView {
            status: view_status,
            revision: status.revision,
            default_profile: status.default_profile.map(profile_from_content),
            secret_store: match secret_store.status() {
                SecretStoreStatus::Available => StatusCode::Ready,
                SecretStoreStatus::Unavailable => StatusCode::Unavailable,
            },
            providers,
            openai_backend,
            cleanup_required: status.cleanup_required,
        },
        diagnostic,
    )
}

fn unavailable_config(error: ConfigError) -> (ConfigView, DiagnosticCheckView) {
    let (status, remediation) = if error == ConfigError::HomeUnavailable {
        (StatusCode::Unavailable, RemediationCode::HomeUnavailable)
    } else {
        (StatusCode::Invalid, RemediationCode::InspectGlobalConfig)
    };
    (
        ConfigView {
            status,
            revision: None,
            default_profile: None,
            secret_store: StatusCode::Unavailable,
            providers: unavailable_providers(),
            openai_backend: AgentBackendView {
                enabled: false,
                readiness: AgentBackendReadinessView::SecretStoreUnavailable,
                secret_reference_present: false,
                test_available: false,
            },
            cleanup_required: false,
        },
        DiagnosticCheckView {
            check: DiagnosticCheckId::GlobalConfig,
            status,
            blocking: false,
            remediation,
        },
    )
}

fn provider_view(
    provider: ProviderKind,
    status: &RedactedProviderStatus,
    public_setting_present: bool,
) -> ProviderView {
    ProviderView {
        provider,
        enabled: status.enabled,
        readiness: match status.readiness {
            ProviderReadiness::Disabled => ProviderReadinessView::Disabled,
            ProviderReadiness::Ready => ProviderReadinessView::Ready,
            ProviderReadiness::NeedsSecret => ProviderReadinessView::NeedsSecret,
            ProviderReadiness::NeedsPublicSetting => ProviderReadinessView::NeedsPublicSetting,
        },
        public_setting_present,
        secret_reference_present: status.secret_ref_present,
    }
}

fn unavailable_providers() -> [ProviderView; 5] {
    ProviderKind::ALL.map(|provider| ProviderView {
        provider,
        enabled: false,
        readiness: ProviderReadinessView::Unavailable,
        public_setting_present: false,
        secret_reference_present: false,
    })
}

fn integration_snapshots(
    environment: &CommandEnvironment,
) -> [(IntegrationView, DiagnosticCheckView); 2] {
    let Some(inventory) = environment.client_inventory() else {
        return [
            unavailable_integration(
                IntegrationTarget::Codex,
                StatusCode::Unavailable,
                RemediationCode::HomeUnavailable,
            ),
            unavailable_integration(
                IntegrationTarget::ClaudeCode,
                StatusCode::Unavailable,
                RemediationCode::HomeUnavailable,
            ),
        ];
    };
    let clients = &inventory.summary().clients;
    [
        integration_snapshot(&clients[0], environment.codex_host_version()),
        integration_snapshot(&clients[1], environment.claude_host_version()),
    ]
}

fn integration_snapshot(
    inventory: &ClientInventoryEntryV1,
    version: Option<crate::command::DetectedClientVersion>,
) -> (IntegrationView, DiagnosticCheckView) {
    let (target, check, symbolic_location, activation, remediation) = match inventory.client {
        ClientKind::Codex => (
            IntegrationTarget::Codex,
            DiagnosticCheckId::CodexLocal,
            SymbolicLocation::CodexMarketplace,
            ActivationPolicy::ClientActionRequired,
            RemediationCode::InspectCodexLocal,
        ),
        ClientKind::ClaudeCode => (
            IntegrationTarget::ClaudeCode,
            DiagnosticCheckId::ClaudeCodeLocal,
            SymbolicLocation::ClaudeMarketplace,
            ActivationPolicy::ReloadOrClientActionRequired,
            RemediationCode::InspectClaudeCodeLocal,
        ),
    };
    let source = component_status(inventory.components.plugin_source);
    let marketplace = component_status(inventory.components.marketplace);
    let registration = component_status(inventory.components.registration);
    let compatibility = client_compatibility(inventory.client, inventory.discovery, version);
    let direct_package = (inventory.client == ClientKind::ClaudeCode)
        .then(|| component_status(inventory.components.skills));
    let overall = match (inventory.discovery, compatibility) {
        (ClientDiscoveryState::Detected, ClientCompatibilityView::Unsupported) => {
            StatusCode::Blocked
        }
        (ClientDiscoveryState::NotDetected, _) => StatusCode::Missing,
        (ClientDiscoveryState::Unavailable, _) => StatusCode::Unavailable,
        (ClientDiscoveryState::Detected, _) => {
            integration_overall(source, marketplace, direct_package, registration)
        }
    };
    let (activation_status, activation_observation) = integration_activation(registration);
    let (mcp_attachment, mcp_attachment_observation) =
        integration_mcp_attachment(inventory.discovery, source, registration);
    let (paths, path_count) = integration_paths(inventory);
    integration_result(
        IntegrationView {
            target,
            client_version: version.map(|version| ClientVersionView {
                major: version.major,
                minor: version.minor,
                patch: version.patch,
            }),
            compatibility,
            installed_plugin_version: inventory
                .installed_plugin_version
                .as_deref()
                .and_then(product_version_view),
            available_plugin_version: available_product_version_view(),
            discovery: match inventory.discovery {
                ClientDiscoveryState::NotDetected => IntegrationDiscoveryState::NotDiscovered,
                ClientDiscoveryState::Unavailable => IntegrationDiscoveryState::Unavailable,
                ClientDiscoveryState::Detected => integration_discovery(true, registration),
            },
            candidate_required: false,
            client: match inventory.discovery {
                ClientDiscoveryState::Detected => StatusCode::Ready,
                ClientDiscoveryState::NotDetected => StatusCode::Missing,
                ClientDiscoveryState::Unavailable => StatusCode::Unavailable,
            },
            overall,
            source,
            skills: component_status(inventory.components.skills),
            marketplace,
            direct_package,
            registration,
            activation_status,
            activation_observation,
            mcp_attachment,
            mcp_attachment_observation,
            symbolic_location,
            activation,
            ownership: ownership_view(inventory.ownership),
            next_action: action_view(inventory.readiness),
            evidence_code: integration_evidence_code(&inventory.reason_code),
            path_count,
            paths,
        },
        check,
        remediation,
    )
}

fn client_compatibility(
    client: ClientKind,
    discovery: ClientDiscoveryState,
    version: Option<crate::command::DetectedClientVersion>,
) -> ClientCompatibilityView {
    if discovery != ClientDiscoveryState::Detected {
        return ClientCompatibilityView::NotEvaluated;
    }
    let Some(version) = version else {
        return ClientCompatibilityView::NotEvaluated;
    };
    let minimum = match client {
        ClientKind::Codex => (0, 144, 1),
        ClientKind::ClaudeCode => (2, 1, 206),
    };
    if (version.major, version.minor, version.patch) >= minimum {
        ClientCompatibilityView::Supported
    } else {
        ClientCompatibilityView::Unsupported
    }
}

const fn integration_activation(
    registration: StatusCode,
) -> (StatusCode, IntegrationObservationView) {
    match registration {
        StatusCode::Ready => (
            StatusCode::Attention,
            IntegrationObservationView::ClientActionRequired,
        ),
        StatusCode::Missing => (StatusCode::Missing, IntegrationObservationView::Missing),
        StatusCode::Unavailable | StatusCode::Blocked | StatusCode::Insecure => {
            (registration, IntegrationObservationView::InspectionBlocked)
        }
        status => (status, IntegrationObservationView::NotObservable),
    }
}

const fn integration_mcp_attachment(
    discovery: ClientDiscoveryState,
    source: StatusCode,
    registration: StatusCode,
) -> (StatusCode, IntegrationObservationView) {
    if matches!(discovery, ClientDiscoveryState::Unavailable) {
        return (
            StatusCode::Unavailable,
            IntegrationObservationView::InspectionBlocked,
        );
    }
    if !matches!(source, StatusCode::Ready) || !matches!(registration, StatusCode::Ready) {
        return (StatusCode::Missing, IntegrationObservationView::Missing);
    }
    (
        StatusCode::Attention,
        IntegrationObservationView::NotObservable,
    )
}

fn available_product_version_view() -> ProductVersionView {
    product_version_view(crate::DESKTOP_PRODUCT_VERSION).unwrap_or(ProductVersionView {
        major: 0,
        minor: 0,
        patch: 0,
        channel: ProductVersionChannelView::Alpha,
        prerelease_number: None,
    })
}

fn product_version_view(value: &str) -> Option<ProductVersionView> {
    let version = semver::Version::parse(value).ok()?;
    let prerelease = version.pre.as_str();
    let (channel, prerelease_number) = if prerelease.is_empty() {
        (ProductVersionChannelView::Stable, None)
    } else if let Some(number) = prerelease.strip_prefix("alpha.") {
        (ProductVersionChannelView::Alpha, Some(number.parse().ok()?))
    } else {
        let number = prerelease.strip_prefix("beta.")?;
        (ProductVersionChannelView::Beta, Some(number.parse().ok()?))
    };
    Some(ProductVersionView {
        major: version.major,
        minor: version.minor,
        patch: version.patch,
        channel,
        prerelease_number,
    })
}

fn integration_paths(
    inventory: &ClientInventoryEntryV1,
) -> ([Option<IntegrationPathView>; MAX_INTEGRATION_PATHS], usize) {
    let mut paths = EMPTY_INTEGRATION_PATHS;
    for (slot, candidate) in paths.iter_mut().zip(&inventory.paths) {
        *slot = Some(IntegrationPathView {
            surface: match candidate.surface {
                ClientPathSurface::ClientConfig => IntegrationPathSurfaceView::ClientConfig,
                ClientPathSurface::SkillsRoot => IntegrationPathSurfaceView::SkillsRoot,
                ClientPathSurface::SkillsPackage => IntegrationPathSurfaceView::SkillsPackage,
                ClientPathSurface::PluginMarketplace => {
                    IntegrationPathSurfaceView::PluginMarketplace
                }
                ClientPathSurface::PluginSource => IntegrationPathSurfaceView::PluginSource,
            },
            scope: match candidate.scope {
                ClientPathScope::User => IntegrationPathScopeView::User,
                ClientPathScope::Project => IntegrationPathScopeView::Project,
                ClientPathScope::Managed => IntegrationPathScopeView::Managed,
                ClientPathScope::Custom => IntegrationPathScopeView::Custom,
                ClientPathScope::Legacy => IntegrationPathScopeView::Legacy,
            },
            source: match candidate.source {
                ClientPathSource::EnvironmentOverride => {
                    IntegrationPathSourceView::EnvironmentOverride
                }
                ClientPathSource::OfficialDefault => IntegrationPathSourceView::OfficialDefault,
                ClientPathSource::ProjectContext => IntegrationPathSourceView::ProjectContext,
                ClientPathSource::QiongliManaged => IntegrationPathSourceView::QiongliManaged,
                ClientPathSource::ExplicitCustom => IntegrationPathSourceView::ExplicitCustom,
                ClientPathSource::LegacyObserved => IntegrationPathSourceView::LegacyObserved,
            },
            state: path_status(candidate.state),
            management: match candidate.management {
                ClientPathManagement::Supported => IntegrationPathManagementView::Supported,
                ClientPathManagement::InspectOnly => IntegrationPathManagementView::InspectOnly,
                ClientPathManagement::LegacyOnly => IntegrationPathManagementView::LegacyOnly,
                ClientPathManagement::Unsafe => IntegrationPathManagementView::Unsafe,
                ClientPathManagement::Unavailable => IntegrationPathManagementView::Unavailable,
            },
            selected: candidate.selected,
            symbolic_path: candidate.symbolic_path.display(),
        });
    }
    (paths, inventory.paths.len())
}

const fn component_status(state: ClientComponentState) -> StatusCode {
    match state {
        ClientComponentState::Missing => StatusCode::Missing,
        ClientComponentState::Ready => StatusCode::Ready,
        ClientComponentState::Conflict => StatusCode::Conflict,
        ClientComponentState::Drifted => StatusCode::Drifted,
        ClientComponentState::RecoveryRequired => StatusCode::RecoveryRequired,
        ClientComponentState::Unavailable => StatusCode::Unavailable,
    }
}

const fn path_status(state: ClientPathState) -> StatusCode {
    match state {
        ClientPathState::Missing => StatusCode::Missing,
        ClientPathState::Directory | ClientPathState::File | ClientPathState::Symlink => {
            StatusCode::Ready
        }
        ClientPathState::Invalid | ClientPathState::Unsafe => StatusCode::Invalid,
        ClientPathState::Unavailable => StatusCode::Unavailable,
    }
}

const fn ownership_view(state: ClientOwnershipState) -> IntegrationOwnershipView {
    match state {
        ClientOwnershipState::NotInstalled => IntegrationOwnershipView::NotInstalled,
        ClientOwnershipState::QiongliManaged => IntegrationOwnershipView::QiongliManaged,
        ClientOwnershipState::Unmanaged => IntegrationOwnershipView::Unmanaged,
        ClientOwnershipState::Mixed => IntegrationOwnershipView::Mixed,
        ClientOwnershipState::Unknown => IntegrationOwnershipView::Unknown,
    }
}

const fn action_view(state: ClientActionReadiness) -> IntegrationActionView {
    match state {
        ClientActionReadiness::InspectOnly => IntegrationActionView::InspectOnly,
        ClientActionReadiness::InstallReady => IntegrationActionView::InstallReady,
        ClientActionReadiness::Current => IntegrationActionView::Current,
        ClientActionReadiness::RepairReady => IntegrationActionView::RepairReady,
        ClientActionReadiness::ResolveConflict => IntegrationActionView::ResolveConflict,
        ClientActionReadiness::Unavailable => IntegrationActionView::Unavailable,
    }
}

fn integration_evidence_code(reason: &str) -> &'static str {
    match reason {
        "client-not-detected" => "client-not-detected",
        "client-detected-install-ready" => "client-detected-install-ready",
        "client-managed-current" => "client-managed-current",
        "client-managed-repair-ready" => "client-managed-repair-ready",
        "client-registration-conflict" => "client-registration-conflict",
        "client-inventory-unavailable" => "client-inventory-unavailable",
        _ => "client-adapter-discovery-unavailable",
    }
}

fn integration_result(
    view: IntegrationView,
    check: DiagnosticCheckId,
    remediation: RemediationCode,
) -> (IntegrationView, DiagnosticCheckView) {
    (
        view,
        DiagnosticCheckView {
            check,
            status: view.overall,
            blocking: integration_is_blocking(view.overall),
            remediation: if view.overall == StatusCode::Ready {
                RemediationCode::None
            } else {
                remediation
            },
        },
    )
}

const fn integration_discovery(
    client_discovered: bool,
    registration: StatusCode,
) -> IntegrationDiscoveryState {
    if !client_discovered {
        return IntegrationDiscoveryState::NotDiscovered;
    }
    match registration {
        StatusCode::Ready => IntegrationDiscoveryState::Managed,
        StatusCode::Drifted => IntegrationDiscoveryState::Drifted,
        StatusCode::Conflict => IntegrationDiscoveryState::Conflict,
        StatusCode::RecoveryRequired => IntegrationDiscoveryState::RecoveryRequired,
        StatusCode::Missing => IntegrationDiscoveryState::DiscoveredUnmanaged,
        StatusCode::Attention
        | StatusCode::Unavailable
        | StatusCode::Disabled
        | StatusCode::Blocked
        | StatusCode::Invalid
        | StatusCode::FutureSchema
        | StatusCode::Insecure
        | StatusCode::Busy
        | StatusCode::WriteUnsupported => IntegrationDiscoveryState::Unavailable,
    }
}

fn unavailable_integration(
    target: IntegrationTarget,
    status: StatusCode,
    remediation: RemediationCode,
) -> (IntegrationView, DiagnosticCheckView) {
    let (check, symbolic_location, activation, direct_package) = match target {
        IntegrationTarget::Codex => (
            DiagnosticCheckId::CodexLocal,
            SymbolicLocation::CodexMarketplace,
            ActivationPolicy::ClientActionRequired,
            None,
        ),
        IntegrationTarget::ClaudeCode => (
            DiagnosticCheckId::ClaudeCodeLocal,
            SymbolicLocation::ClaudeMarketplace,
            ActivationPolicy::ReloadOrClientActionRequired,
            Some(status),
        ),
    };
    let view = IntegrationView {
        target,
        client_version: None,
        compatibility: ClientCompatibilityView::NotEvaluated,
        installed_plugin_version: None,
        available_plugin_version: available_product_version_view(),
        discovery: IntegrationDiscoveryState::Unavailable,
        candidate_required: false,
        client: status,
        overall: status,
        source: status,
        skills: status,
        marketplace: status,
        direct_package,
        registration: status,
        activation_status: status,
        activation_observation: IntegrationObservationView::InspectionBlocked,
        mcp_attachment: status,
        mcp_attachment_observation: IntegrationObservationView::InspectionBlocked,
        symbolic_location,
        activation,
        ownership: IntegrationOwnershipView::Unknown,
        next_action: IntegrationActionView::Unavailable,
        evidence_code: "client-inventory-home-unavailable",
        path_count: 0,
        paths: EMPTY_INTEGRATION_PATHS,
    };
    (
        view,
        DiagnosticCheckView {
            check,
            status,
            blocking: false,
            remediation,
        },
    )
}

const fn profile_from_content(profile: ProfileId) -> ProfileKind {
    match profile {
        ProfileId::SkillOnly => ProfileKind::SkillOnly,
        ProfileId::MarketplaceLite => ProfileKind::MarketplaceLite,
        ProfileId::Full => ProfileKind::Full,
    }
}

const fn profile_to_content(profile: ProfileKind) -> ProfileId {
    match profile {
        ProfileKind::SkillOnly => ProfileId::SkillOnly,
        ProfileKind::MarketplaceLite => ProfileId::MarketplaceLite,
        ProfileKind::Full => ProfileId::Full,
    }
}

const fn operating_system_view(os: Option<OperatingSystem>) -> OperatingSystemView {
    match os {
        Some(OperatingSystem::Linux) => OperatingSystemView::Linux,
        Some(OperatingSystem::Macos) => OperatingSystemView::MacOs,
        Some(OperatingSystem::Windows) => OperatingSystemView::Windows,
        None => OperatingSystemView::Unsupported,
    }
}

const fn architecture_view(architecture: Option<Architecture>) -> ArchitectureView {
    match architecture {
        Some(Architecture::Aarch64) => ArchitectureView::Aarch64,
        Some(Architecture::X86_64) => ArchitectureView::X86_64,
        None => ArchitectureView::Unsupported,
    }
}

const fn config_status(state: ConfigState) -> StatusCode {
    match state {
        ConfigState::Missing => StatusCode::Missing,
        ConfigState::Ready => StatusCode::Ready,
        ConfigState::Invalid => StatusCode::Invalid,
        ConfigState::FutureSchema => StatusCode::FutureSchema,
        ConfigState::Insecure => StatusCode::Insecure,
        ConfigState::Busy => StatusCode::Busy,
        ConfigState::RecoveryRequired => StatusCode::RecoveryRequired,
        ConfigState::WriteUnsupported => StatusCode::WriteUnsupported,
    }
}

const fn config_is_blocking(state: ConfigState) -> bool {
    !matches!(state, ConfigState::Missing | ConfigState::Ready)
}

const fn config_remediation(state: ConfigState) -> RemediationCode {
    match state {
        ConfigState::Missing | ConfigState::Ready => RemediationCode::None,
        ConfigState::Invalid => RemediationCode::InspectGlobalConfig,
        ConfigState::FutureSchema => RemediationCode::UpgradeQiongli,
        ConfigState::Insecure => RemediationCode::RepairGlobalConfigPermissions,
        ConfigState::Busy => RemediationCode::RetryGlobalConfig,
        ConfigState::RecoveryRequired => RemediationCode::RecoverGlobalConfig,
        ConfigState::WriteUnsupported => RemediationCode::UseSupportedPlatform,
    }
}

fn integration_overall(
    source: StatusCode,
    marketplace: StatusCode,
    direct_package: Option<StatusCode>,
    registration: StatusCode,
) -> StatusCode {
    let states = [
        source,
        marketplace,
        direct_package.unwrap_or(StatusCode::Ready),
        registration,
    ];
    for priority in [
        StatusCode::RecoveryRequired,
        StatusCode::Conflict,
        StatusCode::Drifted,
        StatusCode::Invalid,
        StatusCode::Busy,
        StatusCode::Unavailable,
    ] {
        if states.contains(&priority) {
            return priority;
        }
    }
    if states.iter().all(|state| *state == StatusCode::Ready) {
        StatusCode::Ready
    } else {
        StatusCode::Missing
    }
}

const fn integration_is_blocking(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::RecoveryRequired
            | StatusCode::Conflict
            | StatusCode::Drifted
            | StatusCode::Invalid
            | StatusCode::Insecure
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::SecretRef;

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct FakeFolderPicker {
        path: Option<PathBuf>,
    }

    struct CancelAwareExecutor;

    fn json_string_value_containing<'a>(
        value: &'a serde_json::Value,
        needle: &str,
    ) -> Option<&'a str> {
        match value {
            serde_json::Value::String(observed) => {
                observed.contains(needle).then_some(observed.as_str())
            }
            serde_json::Value::Array(values) => values
                .iter()
                .find_map(|value| json_string_value_containing(value, needle)),
            serde_json::Value::Object(values) => values
                .values()
                .find_map(|value| json_string_value_containing(value, needle)),
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
                None
            }
        }
    }

    #[derive(Default)]
    struct TestSecretStore {
        values: Mutex<BTreeMap<String, Vec<u8>>>,
    }

    #[derive(Default)]
    struct ResolveCountingSecretStore {
        calls: AtomicU64,
    }

    impl SecretStore for ResolveCountingSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(
            &self,
            _secret_ref: &SecretRef,
        ) -> Result<SecretValue, qiongli_config::SecretStoreError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(qiongli_config::SecretStoreError::NotFound)
        }
    }

    impl SecretStore for TestSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(
            &self,
            secret_ref: &SecretRef,
        ) -> Result<SecretValue, qiongli_config::SecretStoreError> {
            let values = self
                .values
                .lock()
                .map_err(|_| qiongli_config::SecretStoreError::Unavailable)?;
            let value = values
                .get(secret_ref.storage_key())
                .cloned()
                .ok_or(qiongli_config::SecretStoreError::NotFound)?;
            SecretValue::new(value).map_err(|_| qiongli_config::SecretStoreError::PersistenceFailed)
        }

        fn store(
            &self,
            secret_ref: &SecretRef,
            value: &SecretValue,
        ) -> Result<(), qiongli_config::SecretStoreError> {
            self.values
                .lock()
                .map_err(|_| qiongli_config::SecretStoreError::Unavailable)?
                .insert(
                    secret_ref.storage_key().to_owned(),
                    value.as_bytes().to_vec(),
                );
            Ok(())
        }

        fn remove(&self, secret_ref: &SecretRef) -> Result<(), qiongli_config::SecretStoreError> {
            self.values
                .lock()
                .map_err(|_| qiongli_config::SecretStoreError::Unavailable)?
                .remove(secret_ref.storage_key())
                .map(|_| ())
                .ok_or(qiongli_config::SecretStoreError::NotFound)
        }
    }

    impl McpSelfTestExecutor for CancelAwareExecutor {
        fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView {
            while !cancelled.load(Ordering::Acquire) {
                thread::yield_now();
            }
            terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts)
        }
    }

    impl FolderPicker for FakeFolderPicker {
        fn pick_folder(&mut self) -> Option<PathBuf> {
            self.path.take()
        }
    }

    #[test]
    fn startup_validation_is_repeatable_without_a_config_home() {
        let environment = CommandEnvironment::with_paths(None, None, None);
        let content = crate::embedded_content().expect("embedded content must load");

        assert_eq!(validate_desktop_startup(&environment, &content), Ok(()));
        assert_eq!(validate_desktop_startup(&environment, &content), Ok(()));
    }

    #[test]
    fn app_api_contract_fixture_is_deterministic_and_covers_every_event_variant() {
        let first = app_api_contract_fixture_json().expect("contract fixture must serialize");
        let second = app_api_contract_fixture_json().expect("contract fixture must be repeatable");
        assert_eq!(first, second);

        let fixture: serde_json::Value =
            serde_json::from_str(&first).expect("contract fixture must be JSON");
        assert_eq!(
            fixture["schemaVersion"],
            json!(crate::desktop_api::APP_API_SCHEMA_VERSION)
        );
        assert_eq!(
            fixture["snapshot"]["schemaVersion"],
            json!(crate::desktop_api::APP_API_SCHEMA_VERSION)
        );
        let event_types = fixture["events"]
            .as_array()
            .expect("contract events must be an array")
            .iter()
            .map(|event| {
                event["type"]
                    .as_str()
                    .expect("every contract event must be tagged")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            event_types,
            [
                "snapshot",
                "preview",
                "capture-inbox",
                "capture-coverage",
                "artifact-changes",
                "academic-graph",
                "academic-graph-portfolio",
                "academic-graph-query",
                "academic-graph-path",
                "academic-graph-artifact-opened",
                "capture-read",
                "project-directory-selected",
                "capture-file-selected",
                "capture-intake-preview",
                "capture-consolidation-preview",
                "update-changed",
                "agent-run-completed",
                "orchestration-loaded",
                "orchestration-executed",
                "orchestration-run-updated",
                "completed",
                "capture-operation-completed",
                "cancelled",
                "validation-failed",
                "failed",
            ]
        );

        let mut host_path_canaries = vec![
            (
                "current directory",
                std::env::current_dir().expect("test directory must resolve"),
            ),
            (
                "current executable",
                std::env::current_exe().expect("test executable must resolve"),
            ),
        ];
        for (label, variable) in [
            ("home directory", "HOME"),
            ("Windows home directory", "USERPROFILE"),
            ("Qiongli config directory", "QIONGLI_CONFIG_HOME"),
            ("Codex config directory", "CODEX_HOME"),
            ("Claude config directory", "CLAUDE_CONFIG_DIR"),
            ("XDG config directory", "XDG_CONFIG_HOME"),
            ("Windows roaming config directory", "APPDATA"),
            ("Windows local config directory", "LOCALAPPDATA"),
        ] {
            let Some(path) = std::env::var_os(variable).map(PathBuf::from) else {
                continue;
            };
            if path.is_absolute() && path.components().count() > 1 {
                host_path_canaries.push((label, path));
            }
        }
        for (label, path) in host_path_canaries {
            let path = path.to_string_lossy();
            assert!(
                json_string_value_containing(&fixture, path.as_ref()).is_none(),
                "contract fixture must not expose the test-process {label}"
            );
        }
    }

    #[test]
    fn app_api_contract_path_canary_search_handles_escaped_windows_values() {
        let fixture: serde_json::Value = serde_json::from_str(
            r#"{"nested":[{"value":"prefix C:\\Users\\Researcher\\AppData suffix"}]}"#,
        )
        .expect("escaped Windows fixture must parse");

        assert_eq!(
            json_string_value_containing(&fixture, r"C:\Users\Researcher\AppData"),
            Some(r"prefix C:\Users\Researcher\AppData suffix")
        );
    }

    #[test]
    fn read_only_snapshot_is_valid_and_creates_no_state() {
        let root = isolated_root("snapshot");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(
            Some(OsString::from(&config)),
            Some(home.clone()),
            Some(root.join("claude-config")),
        );
        let content = crate::embedded_content().unwrap();

        let snapshot = build_snapshot(
            &environment,
            &content,
            &qiongli_config::UnavailableSecretStore,
        );

        assert_eq!(snapshot.validate(), Ok(()));
        assert_eq!(snapshot.mcp.public_tool_count, LITE_PUBLIC_TOOL_NAMES.len());
        assert!(!config.exists());
        assert!(!home.join(".qiongli").exists());
        assert!(!home.join(".agents").exists());
        assert!(!home.join(".claude").exists());
        assert!(!root.join("claude-config").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn snapshot_exposes_detected_client_versions_without_dynamic_output() {
        let root = isolated_root("client-versions");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None)
            .with_inventory_context(None, None, true, true)
            .with_client_versions(
                Some(crate::command::DetectedClientVersion {
                    major: 0,
                    minor: 144,
                    patch: 4,
                }),
                Some(crate::command::DetectedClientVersion {
                    major: 2,
                    minor: 1,
                    patch: 209,
                }),
            );
        let content = crate::embedded_content().unwrap();

        let snapshot = build_snapshot(
            &environment,
            &content,
            &qiongli_config::UnavailableSecretStore,
        );

        assert_eq!(
            snapshot.integrations[0].client_version,
            Some(ClientVersionView {
                major: 0,
                minor: 144,
                patch: 4,
            })
        );
        assert_eq!(
            snapshot.integrations[1].client_version,
            Some(ClientVersionView {
                major: 2,
                minor: 1,
                patch: 209,
            })
        );
        assert_eq!(
            snapshot
                .integrations
                .map(|integration| integration.compatibility),
            [
                ClientCompatibilityView::Supported,
                ClientCompatibilityView::Supported,
            ]
        );
        assert_eq!(
            snapshot
                .integrations
                .map(|integration| integration.mcp_attachment_observation),
            [
                IntegrationObservationView::Missing,
                IntegrationObservationView::Missing,
            ],
            "plugin-source discovery must not be reused as Lite MCP attachment evidence"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn client_compatibility_policy_rejects_versions_below_the_accepted_floor() {
        let codex = crate::command::DetectedClientVersion {
            major: 0,
            minor: 144,
            patch: 0,
        };
        let claude = crate::command::DetectedClientVersion {
            major: 2,
            minor: 1,
            patch: 205,
        };

        assert_eq!(
            client_compatibility(
                ClientKind::Codex,
                ClientDiscoveryState::Detected,
                Some(codex),
            ),
            ClientCompatibilityView::Unsupported
        );
        assert_eq!(
            client_compatibility(
                ClientKind::ClaudeCode,
                ClientDiscoveryState::Detected,
                Some(claude),
            ),
            ClientCompatibilityView::Unsupported
        );
        assert_eq!(
            client_compatibility(ClientKind::Codex, ClientDiscoveryState::Detected, None),
            ClientCompatibilityView::NotEvaluated
        );
    }

    #[test]
    fn production_previews_never_enable_apply_or_echo_private_input() {
        let environment = CommandEnvironment::default();
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        assert!(!service.snapshot().capabilities.apply);

        let event = service.execute(DesktopIntent::PreviewProviderPublicSetting {
            provider: ProviderKind::Crossref,
            public_email: qiongli_ui::PrivateText::new("private@example.org".to_owned()),
        });

        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid public setting should produce a preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("config-write-unavailable"));
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        assert!(!format!("{preview:?}").contains("private@example.org"));

        let wrong_token = if preview.token == OperationToken::new(99) {
            OperationToken::new(100)
        } else {
            OperationToken::new(99)
        };
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation { token: wrong_token }),
            DesktopEvent::Failed {
                code: "operation-token-invalid",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::CancelOperation {
                token: preview.token,
            }),
            DesktopEvent::Cancelled {
                code: "operation-preview-cancelled",
            }
        );
    }

    #[test]
    fn source_build_integration_preview_is_explicitly_read_only() {
        let root = isolated_root("source-build-integration-preview");
        let home = root.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        let event = service.execute(DesktopIntent::PreviewIntegration {
            target: IntegrationTarget::Codex,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("source builds must return a bounded read-only preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("source-build-read-only"));
        assert_ne!(
            preview.blocked_reason,
            Some("production-activation-session-unavailable")
        );
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        assert!(!root.join("home/.qiongli").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_session_discovers_clients_without_install_authority() {
        assert_eq!(
            integration_discovery(true, StatusCode::Ready),
            IntegrationDiscoveryState::Managed
        );
        assert_eq!(
            integration_discovery(true, StatusCode::Drifted),
            IntegrationDiscoveryState::Drifted
        );
        assert_eq!(
            integration_discovery(true, StatusCode::Conflict),
            IntegrationDiscoveryState::Conflict
        );
        assert_eq!(
            integration_discovery(true, StatusCode::RecoveryRequired),
            IntegrationDiscoveryState::RecoveryRequired
        );
        let root = isolated_root("source-discovery");
        let home = root.join("home");
        let claude_config = home.join(".claude");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(None, Some(home.clone()), Some(claude_config.clone()));
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment.clone(), content, Vec::new());

        let missing = service.snapshot();
        assert!(missing.integrations.iter().all(|integration| {
            integration.discovery == IntegrationDiscoveryState::NotDiscovered
                && !integration.candidate_required
                && integration.ownership == IntegrationOwnershipView::NotInstalled
                && integration.next_action == IntegrationActionView::InspectOnly
                && integration.path_count >= 5
        }));

        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(&claude_config).unwrap();
        let discovered = service.snapshot();
        assert!(discovered.integrations.iter().all(|integration| {
            integration.discovery == IntegrationDiscoveryState::DiscoveredUnmanaged
                && integration.candidate_required
                && integration.registration == StatusCode::Missing
                && integration.next_action == IntegrationActionView::InstallReady
                && integration.paths[..integration.path_count]
                    .iter()
                    .all(Option::is_some)
        }));
        assert!(!discovered.capabilities.apply);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lite_mcp_self_test_uses_exact_registry_and_offline_dispatch() {
        let root = isolated_root("mcp-self-test");
        let home = root.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        let DesktopEvent::McpSelfTestUpdated(started) =
            service.execute(DesktopIntent::RunLiteMcpSelfTest)
        else {
            panic!("self-test must return bounded progress");
        };
        assert_eq!(started.state, McpSelfTestState::Running);
        let completed = (0..1_000)
            .find_map(|_| {
                let DesktopEvent::McpSelfTestUpdated(view) =
                    service.execute(DesktopIntent::PollLiteMcpSelfTest)
                else {
                    panic!("self-test poll must return typed progress");
                };
                if view.state == McpSelfTestState::Running {
                    thread::sleep(Duration::from_millis(1));
                    None
                } else {
                    Some(view)
                }
            })
            .expect("bounded self-test must complete");
        assert_eq!(completed.state, McpSelfTestState::Passed);
        assert!(completed.validate());
        assert_eq!(completed.public_tool_count, LITE_PUBLIC_TOOL_NAMES.len());
        assert!(
            completed.checks[..4]
                .iter()
                .all(|check| check.status == StatusCode::Ready)
        );
        assert_eq!(completed.discovered_client_count, 2);
        assert_eq!(completed.registered_client_count, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lite_mcp_self_test_does_not_resolve_provider_credentials() {
        let root = isolated_root("mcp-self-test-no-credential-read");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let mut settings = GlobalSettings::default();
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.api_key_ref =
            Some(SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap());
        store.replace(0, settings).unwrap();

        let credential_store = Arc::new(ResolveCountingSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        service.secret_store = credential_store.clone();

        let DesktopEvent::McpSelfTestUpdated(started) =
            service.execute(DesktopIntent::RunLiteMcpSelfTest)
        else {
            panic!("self-test must start without resolving credentials");
        };
        assert_eq!(started.state, McpSelfTestState::Running);
        let completed = (0..1_000)
            .find_map(|_| {
                let DesktopEvent::McpSelfTestUpdated(view) =
                    service.execute(DesktopIntent::PollLiteMcpSelfTest)
                else {
                    panic!("self-test poll must return typed progress");
                };
                if view.state == McpSelfTestState::Running {
                    thread::sleep(Duration::from_millis(1));
                    None
                } else {
                    Some(view)
                }
            })
            .expect("bounded self-test must complete");
        assert_eq!(completed.state, McpSelfTestState::Passed);
        assert_eq!(credential_store.calls.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lite_mcp_self_test_supports_cancel_and_fixed_timeout() {
        let root = isolated_root("mcp-self-test-timeout");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        service.mcp_self_test_executor = Arc::new(CancelAwareExecutor);

        let _ = service.execute(DesktopIntent::RunLiteMcpSelfTest);
        let DesktopEvent::McpSelfTestUpdated(cancelled) =
            service.execute(DesktopIntent::CancelLiteMcpSelfTest)
        else {
            panic!("cancellation must return a typed result");
        };
        assert_eq!(cancelled.state, McpSelfTestState::Cancelled);

        service.mcp_self_test_timeout = Duration::ZERO;
        let _ = service.execute(DesktopIntent::RunLiteMcpSelfTest);
        let DesktopEvent::McpSelfTestUpdated(timed_out) =
            service.execute(DesktopIntent::PollLiteMcpSelfTest)
        else {
            panic!("timeout must return a typed result");
        };
        assert_eq!(timed_out.state, McpSelfTestState::TimedOut);
        assert!(timed_out.validate());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_settings_preview_and_confirm_preserve_secret_references() {
        let root = isolated_root("global-settings");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let mut initial = GlobalSettings::default();
        initial.providers.openalex.api_key_ref =
            Some(SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap());
        let initial_outcome = store.replace(0, initial.clone()).unwrap();
        assert_eq!(initial_outcome.revision, 1);

        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        let event = service.execute(DesktopIntent::PreviewProviderSettingsPatch(
            ProviderSettingsPatch {
                expected_revision: 1,
                providers_enabled: [true, false, true, false, true],
                openalex_email: PublicSettingChange::Replace(qiongli_ui::PrivateText::new(
                    "openalex-private-canary@example.org".to_owned(),
                )),
                crossref_email: PublicSettingChange::Replace(qiongli_ui::PrivateText::new(
                    "crossref-private-canary@example.org".to_owned(),
                )),
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid settings must produce a preview");
        };
        assert_eq!(preview.kind, OperationKind::ProviderSettings);
        assert_eq!(
            preview.approvals_required,
            vec![OperationApproval::ClientConfigChange]
        );
        assert!(!format!("{preview:?}").contains("private-canary"));

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "provider-settings-updated",
            }
        );
        let committed = store.load().unwrap();
        assert_eq!(committed.revision, 2);
        assert_eq!(
            committed.settings.default_profile,
            ProfileId::MarketplaceLite
        );
        assert_eq!(
            committed.settings.providers.openalex.api_key_ref,
            initial.providers.openalex.api_key_ref
        );
        assert_eq!(
            committed
                .settings
                .providers
                .openalex
                .email
                .as_ref()
                .map(EmailAddress::as_str),
            Some("openalex-private-canary@example.org")
        );
        assert_eq!(
            committed
                .settings
                .providers
                .crossref
                .email
                .as_ref()
                .map(EmailAddress::as_str),
            Some("crossref-private-canary@example.org")
        );
        let DesktopEvent::PreviewReady(global_preview) = service.execute(
            DesktopIntent::PreviewGlobalSettingsPatch(GlobalSettingsPatch {
                expected_revision: 2,
                default_profile: ProfileKind::Full,
            }),
        ) else {
            panic!("global defaults must preview independently");
        };
        assert!(matches!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: global_preview.token,
            }),
            DesktopEvent::Completed { .. }
        ));
        let separated = store.load().unwrap();
        assert_eq!(separated.settings.default_profile, ProfileId::Full);
        assert_eq!(
            separated.settings.providers.openalex.api_key_ref,
            initial.providers.openalex.api_key_ref
        );
        assert!(separated.settings.providers.openalex.enabled);
        assert!(separated.settings.providers.crossref.enabled);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_secret_save_replace_restart_and_remove_never_persist_raw_values() {
        let root = isolated_root("provider-secret-lifecycle");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let settings_store = config_store(&environment).unwrap();
        let mut initial = GlobalSettings::default();
        initial.providers.openalex.enabled = true;
        settings_store.replace(0, initial).unwrap();
        let credential_store = Arc::new(TestSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment.clone(),
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        service.secret_store = credential_store.clone();

        let first_secret = "openalex-first-private-canary";
        let event = service.execute(DesktopIntent::PreviewProviderSecretChange {
            provider: ProviderKind::OpenAlex,
            change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                first_secret.to_owned(),
            )),
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("a valid provider secret must preview");
        };
        assert_eq!(preview.kind, OperationKind::ProviderSecret);
        assert!(!format!("{preview:?}").contains(first_secret));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "openalex-api-key-saved",
            }
        );
        let first = settings_store.load().unwrap();
        let reference = first
            .settings
            .providers
            .openalex
            .api_key_ref
            .clone()
            .expect("configuration stores an opaque reference");
        let settings_bytes = fs::read(
            config_root(&environment)
                .unwrap()
                .state_root()
                .join(qiongli_config::GLOBAL_SETTINGS_FILE),
        )
        .unwrap();
        assert!(
            !settings_bytes
                .windows(first_secret.len())
                .any(|bytes| bytes == first_secret.as_bytes())
        );

        let content = crate::embedded_content().unwrap();
        let mut restarted = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        restarted.secret_store = credential_store.clone();
        assert_eq!(
            restarted.execute(DesktopIntent::TestLiteratureProvider {
                provider: ProviderKind::OpenAlex,
            }),
            DesktopEvent::Completed {
                code: "literature-provider-ready",
            }
        );

        let replacement_secret = "openalex-second-private-canary";
        let DesktopEvent::PreviewReady(replace_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::OpenAlex,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    replacement_secret.to_owned(),
                )),
            })
        else {
            panic!("credential replacement must preview");
        };
        assert!(matches!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: replace_preview.token,
            }),
            DesktopEvent::Completed { .. }
        ));
        assert_eq!(
            credential_store.resolve(&reference).unwrap().as_bytes(),
            replacement_secret.as_bytes()
        );

        let rollback_secret = "openalex-rollback-private-canary";
        let DesktopEvent::PreviewReady(rollback_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::OpenAlex,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    rollback_secret.to_owned(),
                )),
            })
        else {
            panic!("rollback credential must preview");
        };
        let external = settings_store.load().unwrap();
        settings_store
            .replace(external.revision, external.settings)
            .unwrap();
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: rollback_preview.token,
            }),
            DesktopEvent::Failed {
                code: "revision-conflict",
            }
        );
        assert_eq!(
            credential_store.resolve(&reference).unwrap().as_bytes(),
            replacement_secret.as_bytes()
        );

        let DesktopEvent::PreviewReady(remove_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::OpenAlex,
                change: ProviderSecretChange::Remove,
            })
        else {
            panic!("credential removal must preview");
        };
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: remove_preview.token,
            }),
            DesktopEvent::Completed {
                code: "openalex-api-key-removed",
            }
        );
        assert!(
            settings_store
                .load()
                .unwrap()
                .settings
                .providers
                .openalex
                .api_key_ref
                .is_none()
        );
        assert!(matches!(
            credential_store.resolve(&reference),
            Err(qiongli_config::SecretStoreError::NotFound)
        ));
        let DesktopEvent::PreviewReady(semantic_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::SemanticScholar,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    "semantic-scholar-private-canary".to_owned(),
                )),
            })
        else {
            panic!("Semantic Scholar credential must preview");
        };
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: semantic_preview.token,
            }),
            DesktopEvent::Completed {
                code: "semantic-scholar-api-key-saved",
            }
        );
        assert!(
            settings_store
                .load()
                .unwrap()
                .settings
                .providers
                .semantic_scholar
                .api_key_ref
                .is_some()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_build_agent_backend_settings_and_key_use_the_secure_store_transaction() {
        let root = isolated_root("agent-backend-secret-lifecycle");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let settings_store = config_store(&environment).unwrap();
        let credential_store = Arc::new(TestSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment.clone(),
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        service.secret_store = credential_store.clone();

        let DesktopEvent::PreviewReady(settings_preview) = service.execute(
            DesktopIntent::PreviewAgentBackendSettingsPatch(AgentBackendSettingsPatch {
                expected_revision: 0,
                openai_enabled: true,
            }),
        ) else {
            panic!("backend enablement must preview");
        };
        assert_eq!(settings_preview.kind, OperationKind::AgentBackendSettings);
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: settings_preview.token,
            }),
            DesktopEvent::Completed {
                code: "agent-backend-settings-updated",
            }
        );
        assert_eq!(
            service.snapshot().config.openai_backend.readiness,
            AgentBackendReadinessView::NeedsSecretReference
        );
        assert_eq!(
            service.execute(DesktopIntent::TestOpenAiBackend),
            DesktopEvent::Failed {
                code: "agent-backend-secret-reference-missing",
            }
        );

        let secret = "openai-source-build-private-canary";
        let DesktopEvent::PreviewReady(secret_preview) =
            service.execute(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Replace(qiongli_ui::PrivateText::new(
                    secret.to_owned(),
                )),
            })
        else {
            panic!("backend credential must preview");
        };
        assert_eq!(secret_preview.kind, OperationKind::AgentBackendSecret);
        assert!(!format!("{secret_preview:?}").contains(secret));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: secret_preview.token,
            }),
            DesktopEvent::Completed {
                code: "openai-api-key-saved",
            }
        );

        let committed = settings_store.load().unwrap();
        let reference = committed
            .settings
            .agent_backends
            .openai
            .api_key_ref
            .clone()
            .expect("configuration must retain only an opaque reference");
        assert_eq!(
            credential_store.resolve(&reference).unwrap().as_bytes(),
            secret.as_bytes()
        );
        let settings_bytes = fs::read(
            config_root(&environment)
                .unwrap()
                .state_root()
                .join(qiongli_config::GLOBAL_SETTINGS_FILE),
        )
        .unwrap();
        assert!(
            !settings_bytes
                .windows(secret.len())
                .any(|bytes| bytes == secret.as_bytes())
        );
        let backend = service.snapshot().config.openai_backend;
        assert_eq!(backend.readiness, AgentBackendReadinessView::Ready);
        assert!(backend.test_available);

        let projects = project_state_service(&environment).unwrap();
        let project_plan = projects
            .preview_create(
                root.join("agent-run-article"),
                ProjectRegistrationOptions::new("Agent Run Article", ProjectKind::Article),
                1,
            )
            .unwrap();
        let project_id = project_plan.preview().project_id.clone();
        projects
            .apply(
                &project_plan,
                &ApprovedProjectMutation::new(project_plan.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        let prompt = "private-agent-run-preview-canary";
        let DesktopEvent::PreviewReady(run_preview) =
            service.execute(DesktopIntent::PreviewAgentRun(AgentRunDraft {
                project_id: project_id.as_str().to_owned(),
                expected_project_revision: 1,
                prompt: qiongli_ui::PrivateText::new(prompt.to_owned()),
            }))
        else {
            panic!("ready project and backend must produce a run preview");
        };
        assert_eq!(run_preview.kind, OperationKind::AgentRun);
        assert_eq!(
            run_preview.approvals_required,
            vec![OperationApproval::NetworkRequest]
        );
        assert!(run_preview.can_confirm);
        assert!(!format!("{run_preview:?}").contains(prompt));
        assert_eq!(
            service.execute(DesktopIntent::CancelOperation {
                token: run_preview.token,
            }),
            DesktopEvent::Cancelled {
                code: "operation-preview-cancelled",
            }
        );

        let DesktopEvent::PreviewReady(remove_preview) =
            service.execute(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Remove,
            })
        else {
            panic!("backend credential removal must preview");
        };
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: remove_preview.token,
            }),
            DesktopEvent::Completed {
                code: "openai-api-key-removed",
            }
        );
        assert!(matches!(
            credential_store.resolve(&reference),
            Err(qiongli_config::SecretStoreError::NotFound)
        ));
        assert!(
            settings_store
                .load()
                .unwrap()
                .settings
                .agent_backends
                .openai
                .api_key_ref
                .is_none()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn global_settings_confirmation_rejects_a_stale_revision() {
        let root = isolated_root("global-settings-stale");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );

        let event = service.execute(DesktopIntent::PreviewGlobalSettingsPatch(
            GlobalSettingsPatch {
                expected_revision: 0,
                default_profile: ProfileKind::Full,
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("missing settings must preview from revision zero");
        };
        let external = GlobalSettings {
            default_profile: ProfileId::SkillOnly,
            ..GlobalSettings::default()
        };
        store.replace(0, external.clone()).unwrap();

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Failed {
                code: "revision-conflict",
            }
        );
        assert_eq!(store.load().unwrap().settings, external);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unchanged_global_settings_produce_a_read_only_preview() {
        let root = isolated_root("global-settings-read-only-preview");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );

        let event = service.execute(DesktopIntent::PreviewGlobalSettingsPatch(
            GlobalSettingsPatch {
                expected_revision: 0,
                default_profile: ProfileKind::MarketplaceLite,
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("unchanged settings must produce a visible read-only preview");
        };
        assert_eq!(preview.kind, OperationKind::GlobalSettings);
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("global-settings-unchanged"));
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selected_skills_destination_materializes_and_verifies_without_debug_path() {
        let root = isolated_root("skills-destination");
        let home = root.join("home");
        let target = root.join("selected-private-canary");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&target).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker {
                path: Some(target.clone()),
            }),
        );

        let selected = service.execute(DesktopIntent::SelectSkillsDestination);
        assert!(matches!(
            selected,
            DesktopEvent::SkillsDestinationSelected { .. }
        ));
        assert!(!format!("{selected:?}").contains("selected-private-canary"));

        let event = service.execute(DesktopIntent::PreviewSkillsPresetMaterialization {
            profile: ProfileKind::SkillOnly,
            preset: SkillsDestinationPreset::CustomFolder,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("selected Skills destination must preview");
        };
        assert_eq!(preview.kind, OperationKind::SkillsMaterialization);
        assert_eq!(
            preview.approvals_required,
            vec![OperationApproval::FilesystemWrite]
        );
        assert!(!format!("{preview:?}").contains("selected-private-canary"));

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::VerifySkillsPreset {
                preset: SkillsDestinationPreset::CustomFolder,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-verified",
            }
        );
        let approved = approve_materialization_target(&target).unwrap();
        let receipt = verify_materialization(&approved).unwrap();
        assert_eq!(receipt.profile, ProfileId::SkillOnly);

        let removal = service.execute(DesktopIntent::PreviewSkillsPresetRemoval {
            preset: SkillsDestinationPreset::CustomFolder,
        });
        let DesktopEvent::PreviewReady(removal_preview) = removal else {
            panic!("verified Skills destination must preview removal");
        };
        assert_eq!(removal_preview.kind, OperationKind::SkillsRemoval);
        assert!(!format!("{removal_preview:?}").contains("selected-private-canary"));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: removal_preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-removed",
            }
        );
        assert!(!target.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn qiongli_managed_skills_preset_needs_no_folder_picker() {
        let root = isolated_root("skills-managed-preset");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );

        let event = service.execute(DesktopIntent::PreviewSkillsPresetMaterialization {
            profile: ProfileKind::SkillOnly,
            preset: SkillsDestinationPreset::QiongliManaged,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("Qiongli Managed must preview without a folder picker");
        };
        assert_eq!(preview.kind, OperationKind::SkillsMaterialization);
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::VerifySkillsPreset {
                preset: SkillsDestinationPreset::QiongliManaged,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-verified",
            }
        );
        assert!(home.join(".qiongli-skills").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_update_views_are_bounded_and_recovery_oriented() {
        let current = update_checked_view(crate::update_cli::DesktopUpdateCheck {
            disposition: crate::update_cli::DesktopUpdateCheckDisposition::Current,
            selected_stream: UpdateStreamPreference::Stable,
            target_version: "2.0.0".to_owned(),
            archive_size_bytes: 24 * 1024 * 1024,
        });
        assert!(current.validate());
        assert_eq!(current.phase, UpdatePhaseView::Current);
        assert!(!current.can_prepare);

        let available = update_checked_view(crate::update_cli::DesktopUpdateCheck {
            disposition: crate::update_cli::DesktopUpdateCheckDisposition::Available,
            selected_stream: UpdateStreamPreference::Beta,
            target_version: "2.0.0-alpha.2".to_owned(),
            archive_size_bytes: 24 * 1024 * 1024,
        });
        assert!(available.validate());
        assert_eq!(available.phase, UpdatePhaseView::Available);
        assert!(available.can_prepare);

        let offline = update_failure_base(
            UpdateStreamView::Beta,
            None,
            "native-update-manifest-timeout",
            false,
            false,
        );
        assert!(offline.validate());
        assert_eq!(offline.remediation, UpdateRemediation::RetryCheck);
        assert!(offline.can_check);

        let expired = update_failure_base(
            UpdateStreamView::Beta,
            None,
            "native-update-manifest-expired",
            false,
            false,
        );
        assert!(expired.validate());
        assert_eq!(expired.remediation, UpdateRemediation::RetryCheck);

        let corrupt = update_failure_base(
            UpdateStreamView::Beta,
            Some("2.0.0-alpha.2".to_owned()),
            "native-update-archive-digest-mismatch",
            true,
            false,
        );
        assert!(corrupt.validate());
        assert_eq!(corrupt.remediation, UpdateRemediation::CancelAndRetry);
        assert!(corrupt.can_cancel);

        let read_only = update_failure_base(
            UpdateStreamView::Beta,
            Some("2.0.0-alpha.2".to_owned()),
            "native-update-installation-location-not-writable",
            true,
            true,
        );
        assert!(read_only.validate());
        assert_eq!(read_only.remediation, UpdateRemediation::MoveToApplications);

        let health_failure = update_failure_base(
            UpdateStreamView::Beta,
            Some("2.0.0-alpha.2".to_owned()),
            "native-update-health-check-failed",
            false,
            false,
        );
        assert!(health_failure.validate());
        assert_eq!(
            health_failure.remediation,
            UpdateRemediation::ReinstallApplication
        );

        let cancelling = update_busy_view(
            &available,
            UpdatePhaseView::Cancelling,
            1,
            "Removing staged update bytes",
        );
        assert!(cancelling.validate());
        assert_eq!(cancelling.phase, UpdatePhaseView::Cancelling);

        for phase in [
            UpdatePhaseView::Downloading,
            UpdatePhaseView::Verifying,
            UpdatePhaseView::Staging,
        ] {
            let preparing = update_busy_view(&available, phase, 1, "Preparing update");
            assert!(preparing.validate());
            assert!(
                !preparing.can_cancel,
                "an active preparation worker must not race a cancellation worker"
            );
        }

        let restarting = UpdateView {
            status: StatusCode::Busy,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::AwaitingRestart,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: None,
            progress: Some(UpdateProgressView {
                completed_steps: 4,
                total_steps: 4,
                label: "Completing application replacement",
                indeterminate: true,
            }),
            reason_code: "update-restart-in-progress",
            remediation: UpdateRemediation::RestartApplication,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        };
        assert!(restarting.validate());
    }

    #[test]
    fn desktop_update_install_confirmation_is_revision_and_transaction_bound() {
        let first = update_install_digest(
            7,
            "update-0123456789abcdef0123456789abcdef",
            "2.0.0-alpha.2",
        );
        assert_eq!(first.len(), 64);
        assert_eq!(
            first,
            update_install_digest(
                7,
                "update-0123456789abcdef0123456789abcdef",
                "2.0.0-alpha.2",
            )
        );
        assert_ne!(
            first,
            update_install_digest(
                8,
                "update-0123456789abcdef0123456789abcdef",
                "2.0.0-alpha.2",
            )
        );
        assert_ne!(
            first,
            update_install_digest(
                7,
                "update-fedcba9876543210fedcba9876543210",
                "2.0.0-alpha.2",
            )
        );
    }

    #[test]
    fn project_workspace_selection_resolves_one_canonical_article_root() {
        let root = isolated_root("project-workspace-selection");
        let workspace = root.join("workspace");
        let research = workspace.join("RESEARCH");
        let article = research.join("article-topic");
        create_private_directory(&workspace);
        create_private_directory(&research);
        create_private_directory(&article);

        assert_eq!(
            resolve_selected_article_project_root(workspace.clone()).unwrap(),
            article
        );
        assert_eq!(
            resolve_selected_article_project_root(research.clone()).unwrap(),
            article
        );
        assert_eq!(
            resolve_selected_article_project_root(article.clone()).unwrap(),
            article
        );
        assert_eq!(
            article_project_root_in_workspace(&workspace, "new-article"),
            research.join("new-article")
        );
        assert_eq!(
            article_project_root_in_workspace(&research, "new-article"),
            research.join("new-article")
        );

        let second = research.join("second-topic");
        create_private_directory(&second);
        assert_eq!(
            resolve_selected_article_project_root(workspace),
            Err("multiple-article-projects-found-select-topic")
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_registers_and_archives_through_previewed_mutations() {
        let root = isolated_root("project-library");
        let home = root.join("home");
        let configured = root.join("configured");
        let project = root.join("article-project");
        create_private_directory(&home);
        create_private_directory(&project);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (directory_token, root_label) = state.select_register_root(project.clone()).unwrap();
        assert_eq!(root_label, "article-project");
        assert_eq!(directory_token.len(), 32);
        assert!(!directory_token.contains("article-project"));

        state.preview_register(&directory_token).unwrap();
        let register_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&register_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-registration-completed");
        assert_eq!(confirmed.capture_project_id, None);
        let registered = state.snapshot();
        assert_eq!(registered.projects.len(), 1);
        let project_id = registered.projects[0].project_id.clone();

        state
            .preview_lifecycle(&project_id, ProjectMutationKind::Archive)
            .unwrap();
        let archive_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&archive_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-archive-completed");
        assert_eq!(confirmed.capture_project_id, None);
        assert_eq!(
            state.snapshot().projects[0].lifecycle,
            qiongli_project::ProjectLifecycle::Archived
        );

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_compares_only_against_its_validated_graph_baseline() {
        let root = isolated_root("project-graph-comparison");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root).unwrap();
        state
            .preview_create(
                &create_token,
                "Graph comparison article".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();

        let (first, baseline) = state.academic_graph(&project_id).unwrap();
        assert!(baseline.is_none());
        let (second, comparison) = state.academic_graph(&project_id).unwrap();
        let comparison = comparison.expect("second load compares the validated baseline");
        assert_eq!(first, second);
        assert_eq!(comparison.before_projection_id, first.projection_id);
        assert_eq!(comparison.after_projection_id, second.projection_id);
        assert!(!comparison.has_changes);

        let portfolio = state.academic_graph_portfolio().unwrap();
        assert_eq!(portfolio.included_project_count, 1);
        assert_eq!(portfolio.project_count, 1);
        assert_eq!(portfolio.node_count, 1);
        assert_eq!(portfolio.edge_count, 0);

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_creates_exports_and_imports_portable_projects() {
        let root = isolated_root("project-portable");
        let source_home = root.join("source-home");
        let source_configured = root.join("source-configured");
        let source_project = root.join("source-project");
        let portable_package = root.join("portable-package");
        let imported_project = root.join("imported-project");
        create_private_directory(&source_home);

        let source_config_root =
            qiongli_config::resolve_config_root(Some(source_configured.as_os_str()), &source_home)
                .unwrap();
        let mut source_state =
            ProjectDesktopState::new(Some(ProjectStateService::new(source_config_root)));

        let (create_directory_token, _) = source_state
            .select_create_root(source_project.clone())
            .unwrap();
        source_state
            .preview_create(
                &create_directory_token,
                "Portable article".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_token = source_state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = source_state.confirm(&create_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-creation-completed");
        assert_eq!(confirmed.capture_project_id, None);
        let project_id = source_state.snapshot().projects[0].project_id.clone();

        let (export_directory_token, _) = source_state
            .select_export_destination(project_id.clone(), portable_package.clone())
            .unwrap();
        source_state
            .preview_export(&export_directory_token)
            .unwrap();
        let export_token = source_state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = source_state.confirm(&export_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-export-completed");
        assert_eq!(confirmed.capture_project_id, None);
        assert!(
            portable_package
                .join("qiongli-portable-project.json")
                .is_file()
        );

        let destination_home = root.join("destination-home");
        let destination_configured = root.join("destination-configured");
        create_private_directory(&destination_home);
        let destination_config_root = qiongli_config::resolve_config_root(
            Some(destination_configured.as_os_str()),
            &destination_home,
        )
        .unwrap();
        let mut destination_state =
            ProjectDesktopState::new(Some(ProjectStateService::new(destination_config_root)));

        let (import_directory_token, _) = destination_state
            .select_import_locations(portable_package, imported_project.clone())
            .unwrap();
        destination_state
            .preview_import(&import_directory_token)
            .unwrap();
        let import_token = destination_state
            .pending
            .as_ref()
            .unwrap()
            .token()
            .to_owned();
        let confirmed = destination_state.confirm(&import_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-import-completed");
        assert_eq!(confirmed.capture_project_id, None);

        let imported = destination_state.snapshot();
        assert_eq!(imported.projects.len(), 1);
        assert_eq!(imported.projects[0].project_id, project_id);
        assert!(
            imported_project
                .join("context/project_manifest.json")
                .is_file()
        );

        drop(source_state);
        drop(destination_state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_intakes_reads_and_consolidates_capture_without_exposing_path() {
        let root = isolated_root("capture-inbox");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        let capture_path = root.join("private-capture-packet.json");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root).unwrap();
        state
            .preview_create(
                &create_token,
                "Capture article".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();

        let capture = qiongli_project::ResearchCaptureDraftV1 {
            binding: qiongli_project::ProjectBindingV1::new(
                project_id.clone(),
                1,
                ProjectStage::Idea,
                "Refine the article framing",
                qiongli_project::CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: qiongli_project::CaptureSource::Codex,
            delivery: qiongli_project::CaptureDelivery::Portable,
            captured_at_unix: now_unix().unwrap(),
            summary: "Separate the literature synthesis from the working thesis.".to_owned(),
            changes: vec![qiongli_project::SemanticChangeV1 {
                area: qiongli_project::CaptureArea::Literature,
                summary: "Organize the literature around cross-client research continuity."
                    .to_owned(),
            }],
            decisions: vec![qiongli_project::DecisionCandidateV1 {
                relation: qiongli_project::DecisionRelation::Candidate,
                statement: "Treat the article project as the durable unit.".to_owned(),
                rationale: "Sessions remain execution context rather than research memory."
                    .to_owned(),
                target: None,
            }],
            evidence: vec![qiongli_project::EvidenceReferenceV1 {
                locator_kind: qiongli_project::EvidenceLocatorKind::Doi,
                locator: "10.1000/capture-inbox".to_owned(),
                relevance: "Provides a bounded citation anchor for the refinement.".to_owned(),
                limitation: Some("Architecture evidence only.".to_owned()),
            }],
            contradictions: Vec::new(),
            next_actions: vec!["Review the framing against current literature.".to_owned()],
        }
        .into_capture()
        .unwrap();
        fs::write(&capture_path, capture.to_canonical_json().unwrap()).unwrap();

        let (file_token, file_label) = state
            .select_capture_file(project_id.clone(), capture_path.clone())
            .unwrap();
        assert_eq!(file_label, "private-capture-packet.json");
        assert!(!file_token.contains("private-capture-packet"));
        let (intake, preview) = state.preview_capture_intake(&file_token).unwrap();
        assert_eq!(
            intake.effect,
            qiongli_project::CaptureIntakeEffect::AppendPendingHistory
        );
        let preview_json = serde_json::to_value(&preview).unwrap();
        assert_eq!(preview_json["canConfirm"], true);
        assert!(!format!("{preview:?}").contains(&capture_path.to_string_lossy().to_string()));

        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&operation_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-intake-completed");
        assert_eq!(confirmed.capture_project_id, Some(project_id.clone()));
        let inbox = state.capture_inbox(&project_id).unwrap();
        assert_eq!(inbox.pending_review_count, 1);
        assert_eq!(inbox.entries[0].capture_id, capture.capture_id);

        let read = state
            .read_capture(&project_id, &capture.capture_id)
            .unwrap();
        let read_json = serde_json::to_value(read).unwrap();
        assert_eq!(read_json["schemaVersion"], 1);
        assert_eq!(read_json["binding"]["projectId"], project_id.as_str());
        assert!(read_json.get("document_kind").is_none());
        assert!(
            !read_json
                .to_string()
                .contains(&capture_path.to_string_lossy().to_string())
        );

        let (consolidation, preview) = state
            .preview_capture_consolidation(&project_id, &capture.capture_id)
            .unwrap();
        assert_eq!(
            consolidation.outcome,
            qiongli_project::CaptureConsolidationOutcome::Ready
        );
        let preview_json = serde_json::to_value(&preview).unwrap();
        assert_eq!(preview_json["canConfirm"], true);
        assert_eq!(
            preview_json["approvalsRequired"],
            serde_json::json!(["academic-consolidation", "filesystem-write"])
        );
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&operation_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-consolidation-completed");
        assert_eq!(confirmed.capture_project_id, Some(project_id.clone()));
        let inbox = state.capture_inbox(&project_id).unwrap();
        assert_eq!(inbox.applied_count, 1);
        assert_eq!(inbox.project_revision, 2);

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    fn isolated_root(name: &str) -> PathBuf {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let fixture_base = native_root.join("target/qiongli-desktop-service-tests");
        fs::create_dir_all(&fixture_base).expect("desktop fixture base must be created");
        let root = fixture_base.join(format!(
            "qiongli-r3f-{name}-{}-{sequence}",
            std::process::id()
        ));
        create_private_directory(&root);
        root
    }

    #[cfg(unix)]
    fn create_private_directory(path: &Path) {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .expect("private directory must be created");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("private directory mode must be retained");
    }

    #[cfg(windows)]
    fn create_private_directory(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path)
            .expect("owner-only Windows directory must be created");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_private_directory(path: &Path) {
        fs::create_dir(path).expect("private directory must be created");
    }
}
