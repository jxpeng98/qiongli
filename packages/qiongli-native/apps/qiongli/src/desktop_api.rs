use qiongli_project::{
    PortableProjectOperation, PortableProjectPreviewV1, ProjectKind, ProjectMutationKind,
    ProjectMutationPreviewV1, ProjectStage, ResearchLibrarySnapshotV1,
};
use qiongli_ui::{
    DesktopEvent, DesktopIntent, DesktopService, DesktopSnapshotV1, IntegrationPathView,
    IntegrationSelection, IntegrationTarget, IntegrationView, OperationApproval, OperationKind,
    OperationPreview, OperationToken, ProductTrustView, ProfileKind, SkillsDestinationPreset,
    StatusCode,
};
use serde::{Deserialize, Serialize};

pub(crate) const APP_API_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSnapshotV1 {
    schema_version: u32,
    product: AppProductView,
    content: AppContentView,
    mcp: AppMcpView,
    configuration: AppConfigurationView,
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
    cleanup_required: bool,
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
    apply: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppIntegrationSelection {
    codex: bool,
    claude_code: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
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
    RefreshIntegrationDiscovery,
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
pub(crate) enum AppSkillsPreset {
    QiongliManaged,
    DetectedCodex,
    DetectedClaudeCode,
    CurrentProject,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum AppEvent {
    Snapshot {
        snapshot: AppSnapshotV1,
    },
    Preview {
        preview: AppOperationPreview,
    },
    ProjectDirectorySelected {
        token: String,
        root_label: String,
    },
    Completed {
        code: &'static str,
        snapshot: AppSnapshotV1,
    },
    Cancelled {
        code: &'static str,
    },
    ValidationFailed {
        code: &'static str,
    },
    Failed {
        code: &'static str,
    },
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
                cleanup_required: snapshot.config.cleanup_required,
            },
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
            | Self::PreviewProjectUnregister { .. } => {
                return Err("app-project-intent-not-intercepted");
            }
            Self::RefreshIntegrationDiscovery => DesktopIntent::RefreshIntegrationDiscovery,
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
        DesktopEvent::Completed { code } => AppEvent::Completed {
            code,
            snapshot: AppSnapshotV1::from_desktop(service.snapshot(), research_library)?,
        },
        DesktopEvent::Cancelled { code } => AppEvent::Cancelled { code },
        DesktopEvent::ValidationFailed { code } => AppEvent::ValidationFailed { code },
        DesktopEvent::Failed { code } => AppEvent::Failed { code },
        DesktopEvent::McpSelfTestUpdated(_)
        | DesktopEvent::UpdateChanged { .. }
        | DesktopEvent::SkillsDestinationSelected { .. } => AppEvent::Failed {
            code: "app-api-event-unsupported",
        },
    })
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
        OperationKind::SkillsMaterialization => "skills-materialization",
        OperationKind::SkillsRemoval => "skills-removal",
        OperationKind::UpdateInstall => "update-install",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
