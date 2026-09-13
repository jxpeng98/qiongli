use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ProjectError;
use crate::capture::{
    CaptureArea, CaptureDisposition, CaptureId, CapturePolicy, CaptureSource, DecisionRelation,
    EvidenceLocatorKind, ResearchCaptureV1, classify_capture,
};
use crate::json::parse_unique_json;
use crate::model::{
    ArticleProjectManifestV1, MAX_SEMANTIC_REVISION, ProjectId, ProjectLifecycle, ProjectStage,
    RegisteredProjectV1, valid_lower_hex,
};
use crate::service::ProjectStateService;
use crate::storage::{
    ProjectFileTransaction, ProjectFileUpdate, consolidation_relative_path,
    encode_project_document, project_root_from_string, project_root_string, read_capture_document,
    read_consolidation_document, read_manifest, read_semantic_artifact,
    semantic_digest_with_overrides, sha256_bytes, validate_existing_project_root,
};

pub const ACADEMIC_CONSOLIDATION_SCHEMA_VERSION: u32 = 1;
const CONSOLIDATION_DOCUMENT_KIND: &str = "qiongli-capture-consolidation";
const RESEARCH_STATE_PATH: &str = "context/research_state.md";
const DECISION_LOG_PATH: &str = "context/decision_log.md";
const PROJECT_MANIFEST_PATH: &str = "context/project_manifest.json";
const MAX_CONSOLIDATED_ARTIFACTS: usize = 2;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureConsolidationOutcome {
    Ready,
    Conflicted,
    AlreadyConsolidated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureConsolidationConflictKind {
    ProjectArchived,
    StaleProjectRevision,
    StageChanged,
    HistoryOnlyPolicy,
    ScopeBoundaryChange,
    LockedDecisionGuard,
    ContradictionRequiresResolution,
    UnsupportedEvidence,
    ArtifactNotUtf8,
    ArtifactLineageConflict,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsolidationArtifact {
    ResearchState,
    DecisionLog,
}

impl ConsolidationArtifact {
    const fn relative_path(self) -> &'static str {
        match self {
            Self::ResearchState => RESEARCH_STATE_PATH,
            Self::DecisionLog => DECISION_LOG_PATH,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsolidationArtifactEffect {
    Create,
    Update,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureConsolidationConflictV1 {
    pub kind: CaptureConsolidationConflictKind,
    pub artifact: Option<ConsolidationArtifact>,
    pub resolution: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConsolidationArtifactDeltaV1 {
    pub artifact: ConsolidationArtifact,
    pub relative_path: String,
    pub effect: ConsolidationArtifactEffect,
    pub previous_digest: Option<String>,
    pub next_digest: String,
    pub previous_bytes: usize,
    pub next_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureConsolidationPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub capture_id: CaptureId,
    pub project_id: ProjectId,
    pub disposition: CaptureDisposition,
    pub outcome: CaptureConsolidationOutcome,
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub next_project_revision: Option<u64>,
    pub project_stage: ProjectStage,
    pub reviewed_at_unix: u64,
    pub conflicts: Vec<CaptureConsolidationConflictV1>,
    pub artifact_deltas: Vec<ConsolidationArtifactDeltaV1>,
    pub receipt_entry: String,
    pub approvals_required: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConsolidatedArtifactV1 {
    pub artifact: ConsolidationArtifact,
    pub relative_path: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureConsolidationReceiptV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub capture_id: CaptureId,
    pub project_id: ProjectId,
    pub source_capture_digest: String,
    pub plan_digest: String,
    pub disposition: CaptureDisposition,
    pub from_project_revision: u64,
    pub to_project_revision: u64,
    pub project_stage: ProjectStage,
    pub consolidated_at_unix: u64,
    pub artifacts: Vec<ConsolidatedArtifactV1>,
    pub acknowledgement: String,
}

impl CaptureConsolidationReceiptV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        CaptureId::parse(self.capture_id.as_str().to_string())?;
        self.project_id.validate()?;
        if self.schema_version != ACADEMIC_CONSOLIDATION_SCHEMA_VERSION
            || self.document_kind != CONSOLIDATION_DOCUMENT_KIND
            || !valid_lower_hex(&self.source_capture_digest, 64)
            || !valid_lower_hex(&self.plan_digest, 64)
            || self.from_project_revision == 0
            || self.from_project_revision >= MAX_SEMANTIC_REVISION
            || self.to_project_revision != self.from_project_revision.saturating_add(1)
            || self.to_project_revision > MAX_SEMANTIC_REVISION
            || self.consolidated_at_unix > MAX_SEMANTIC_REVISION
            || self.artifacts.is_empty()
            || self.artifacts.len() > MAX_CONSOLIDATED_ARTIFACTS
            || !self
                .acknowledgement
                .strip_prefix("ack_")
                .is_some_and(|value| valid_lower_hex(value, 64))
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let mut paths = Vec::new();
        for artifact in &self.artifacts {
            if artifact.relative_path != artifact.artifact.relative_path()
                || !valid_lower_hex(&artifact.digest, 64)
                || paths.contains(&artifact.relative_path.as_str())
            {
                return Err(ProjectError::InvalidProjectDocument);
            }
            paths.push(artifact.relative_path.as_str());
        }
        if acknowledgement(self)? != self.acknowledgement {
            return Err(ProjectError::InvalidProjectDocument);
        }
        Ok(())
    }
}

#[derive(Clone)]
struct PlannedArtifact {
    artifact: ConsolidationArtifact,
    previous_digest: Option<String>,
    previous_bytes: usize,
    next_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct VerifiedCaptureConsolidation {
    preview: CaptureConsolidationPreviewV1,
    capture: ResearchCaptureV1,
    capture_document_digest: String,
    root: PathBuf,
    root_reference_digest: String,
    observed_manifest_digest: String,
    observed_receipt_digest: Option<String>,
    artifacts: Vec<PlannedArtifact>,
    next_manifest: Option<ArticleProjectManifestV1>,
}

impl VerifiedCaptureConsolidation {
    #[must_use]
    pub const fn preview(&self) -> &CaptureConsolidationPreviewV1 {
        &self.preview
    }
}

impl Debug for VerifiedCaptureConsolidation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedCaptureConsolidation")
            .field("preview", &self.preview)
            .field("capture", &"<bounded-research-capture>")
            .field("root", &"<registered-project-root>")
            .field("artifacts", &"<reviewed-academic-deltas>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedCaptureConsolidation {
    expected_plan_digest: String,
    filesystem_write: bool,
    academic_review: bool,
}

impl ApprovedCaptureConsolidation {
    #[must_use]
    pub fn new(
        expected_plan_digest: impl Into<String>,
        filesystem_write: bool,
        academic_review: bool,
    ) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            filesystem_write,
            academic_review,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureConsolidationCommitV1 {
    pub schema_version: u32,
    pub capture_id: CaptureId,
    pub project_id: ProjectId,
    pub disposition: CaptureDisposition,
    pub library_revision: u64,
    pub semantic_revision: u64,
    pub artifacts_updated: Vec<ConsolidationArtifact>,
    pub receipt_entry: String,
    pub acknowledgement: String,
    pub index_rebuild_required: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConsolidationPlanSemantics<'a> {
    schema_version: u32,
    capture_id: &'a CaptureId,
    project_id: &'a ProjectId,
    disposition: CaptureDisposition,
    outcome: CaptureConsolidationOutcome,
    expected_library_revision: u64,
    expected_project_revision: u64,
    next_project_revision: Option<u64>,
    project_stage: ProjectStage,
    reviewed_at_unix: u64,
    root_reference_digest: &'a str,
    observed_manifest_digest: &'a str,
    observed_receipt_digest: Option<&'a str>,
    capture_document_digest: &'a str,
    conflicts: &'a [CaptureConsolidationConflictV1],
    artifact_deltas: &'a [ConsolidationArtifactDeltaV1],
}

impl ProjectStateService {
    pub fn preview_capture_consolidation(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
        reviewed_at_unix: u64,
    ) -> Result<VerifiedCaptureConsolidation, ProjectError> {
        if reviewed_at_unix > MAX_SEMANTIC_REVISION {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        let (manifest, observed_manifest_digest) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        validate_registered_manifest(entry, &manifest, project_id)?;
        if reviewed_at_unix < manifest.academically_updated_at_unix {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let (capture, capture_document_digest) =
            read_capture_document(&root, capture_id)?.ok_or(ProjectError::CaptureNotFound)?;
        if capture.binding.project_id != *project_id {
            return Err(ProjectError::CaptureIdentityConflict);
        }

        let existing_receipt = read_consolidation_receipt(&root, capture_id)?;
        let observed_receipt_digest = existing_receipt
            .as_ref()
            .map(|(_, bytes)| sha256_bytes(bytes));
        if let Some((receipt, _)) = &existing_receipt
            && (receipt.project_id != *project_id
                || receipt.source_capture_digest != capture_document_digest)
        {
            return Err(ProjectError::CaptureIdentityConflict);
        }

        let disposition = classify_capture(&capture, false);
        let root_reference_digest = sha256_bytes(project_root_string(&root)?.as_bytes());
        let mut conflicts = Vec::new();
        let mut artifacts = Vec::new();
        let mut next_manifest = None;
        let outcome = if existing_receipt.is_some() {
            CaptureConsolidationOutcome::AlreadyConsolidated
        } else {
            collect_conflicts(&capture, entry, &manifest, disposition, &mut conflicts);
            if conflicts.is_empty() {
                match plan_artifacts(&root, &capture) {
                    Ok(planned) => artifacts = planned,
                    Err(ArtifactPlanError::Conflict(RenderConflict { kind, artifact })) => {
                        conflicts.push(conflict(kind, artifact))
                    }
                    Err(ArtifactPlanError::Project(error)) => return Err(error),
                }
            }
            if conflicts.is_empty() {
                let updates = artifacts
                    .iter()
                    .map(|artifact| ProjectFileUpdate {
                        relative_path: artifact.artifact.relative_path().to_string(),
                        expected_digest: artifact.previous_digest.clone(),
                        next_bytes: artifact.next_bytes.clone(),
                    })
                    .collect::<Vec<_>>();
                let mut next = manifest.clone();
                next.semantic_revision = next
                    .semantic_revision
                    .checked_add(1)
                    .filter(|revision| *revision <= MAX_SEMANTIC_REVISION)
                    .ok_or(ProjectError::RevisionConflict)?;
                next.semantic_digest = semantic_digest_with_overrides(&root, &updates)?;
                next.academically_updated_at_unix = reviewed_at_unix;
                next.validate()?;
                next_manifest = Some(next);
                CaptureConsolidationOutcome::Ready
            } else {
                artifacts.clear();
                CaptureConsolidationOutcome::Conflicted
            }
        };

        let artifact_deltas = artifacts.iter().map(artifact_delta).collect::<Vec<_>>();
        let next_project_revision = next_manifest
            .as_ref()
            .map(|manifest| manifest.semantic_revision);
        let semantics = ConsolidationPlanSemantics {
            schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
            capture_id,
            project_id,
            disposition,
            outcome,
            expected_library_revision: library.revision,
            expected_project_revision: manifest.semantic_revision,
            next_project_revision,
            project_stage: manifest.stage,
            reviewed_at_unix,
            root_reference_digest: &root_reference_digest,
            observed_manifest_digest: &observed_manifest_digest,
            observed_receipt_digest: observed_receipt_digest.as_deref(),
            capture_document_digest: &capture_document_digest,
            conflicts: &conflicts,
            artifact_deltas: &artifact_deltas,
        };
        let preview = CaptureConsolidationPreviewV1 {
            schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
            plan_digest: canonical_digest(&semantics)?,
            capture_id: capture_id.clone(),
            project_id: project_id.clone(),
            disposition,
            outcome,
            expected_library_revision: library.revision,
            expected_project_revision: manifest.semantic_revision,
            next_project_revision,
            project_stage: manifest.stage,
            reviewed_at_unix,
            conflicts,
            artifact_deltas,
            receipt_entry: consolidation_relative_path(capture_id),
            approvals_required: if outcome == CaptureConsolidationOutcome::Ready {
                vec![
                    "academic-consolidation".to_string(),
                    "filesystem-write".to_string(),
                ]
            } else {
                Vec::new()
            },
        };
        Ok(VerifiedCaptureConsolidation {
            preview,
            capture,
            capture_document_digest,
            root,
            root_reference_digest,
            observed_manifest_digest,
            observed_receipt_digest,
            artifacts,
            next_manifest,
        })
    }

    pub fn apply_capture_consolidation(
        &self,
        plan: &VerifiedCaptureConsolidation,
        approval: &ApprovedCaptureConsolidation,
    ) -> Result<CaptureConsolidationCommitV1, ProjectError> {
        validate_plan(plan)?;
        match plan.preview.outcome {
            CaptureConsolidationOutcome::AlreadyConsolidated => {
                return Err(ProjectError::ConsolidationAlreadyApplied);
            }
            CaptureConsolidationOutcome::Conflicted => {
                return Err(ProjectError::ConsolidationConflict);
            }
            CaptureConsolidationOutcome::Ready => {}
        }
        if !approval.filesystem_write || !approval.academic_review {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview.plan_digest {
            return Err(ProjectError::PlanMismatch);
        }

        let mut mutation = self.store.begin(plan.preview.expected_library_revision)?;
        let prior_entry;
        {
            let entry = mutation
                .document
                .projects
                .iter()
                .find(|entry| entry.project_id == plan.preview.project_id)
                .ok_or(ProjectError::RevisionConflict)?;
            revalidate_apply_state(plan, entry)?;
            prior_entry = entry.clone();
        }

        let next_manifest = plan
            .next_manifest
            .as_ref()
            .ok_or(ProjectError::PlanMismatch)?;
        let mut receipt = build_receipt(plan)?;
        receipt.acknowledgement = acknowledgement(&receipt)?;
        receipt.validate()?;
        let receipt_bytes = encode_project_document(&receipt)?;
        let mut updates = plan
            .artifacts
            .iter()
            .map(|artifact| ProjectFileUpdate {
                relative_path: artifact.artifact.relative_path().to_string(),
                expected_digest: artifact.previous_digest.clone(),
                next_bytes: artifact.next_bytes.clone(),
            })
            .collect::<Vec<_>>();
        updates.push(ProjectFileUpdate {
            relative_path: plan.preview.receipt_entry.clone(),
            expected_digest: None,
            next_bytes: receipt_bytes,
        });
        updates.push(ProjectFileUpdate {
            relative_path: PROJECT_MANIFEST_PATH.to_string(),
            expected_digest: Some(plan.observed_manifest_digest.clone()),
            next_bytes: encode_project_document(next_manifest)?,
        });
        let transaction = ProjectFileTransaction::apply(&plan.root, &updates)?;

        let next_entry = if let Some(entry) = mutation
            .document
            .projects
            .iter_mut()
            .find(|entry| entry.project_id == plan.preview.project_id)
        {
            entry.semantic_revision = next_manifest.semantic_revision;
            entry
                .semantic_digest
                .clone_from(&next_manifest.semantic_digest);
            entry.academically_updated_at_unix = next_manifest.academically_updated_at_unix;
            entry.clone()
        } else {
            return transaction
                .rollback()
                .and(Err(ProjectError::RecoveryRequired));
        };
        let expected_next_library_revision = plan
            .preview
            .expected_library_revision
            .checked_add(1)
            .ok_or(ProjectError::RevisionConflict)?;
        let library_revision = match mutation.commit() {
            Ok(revision) => revision,
            Err(error) => match self.store.load() {
                Ok(document)
                    if library_observation_matches(
                        &document,
                        expected_next_library_revision,
                        &next_entry,
                    ) =>
                {
                    expected_next_library_revision
                }
                Ok(document)
                    if library_observation_matches(
                        &document,
                        plan.preview.expected_library_revision,
                        &prior_entry,
                    ) =>
                {
                    return match transaction.rollback() {
                        Ok(()) => Err(error),
                        Err(_) => Err(ProjectError::RecoveryRequired),
                    };
                }
                Ok(_) | Err(_) => {
                    transaction.preserve_for_recovery();
                    return Err(ProjectError::RecoveryRequired);
                }
            },
        };
        transaction.commit()?;

        Ok(CaptureConsolidationCommitV1 {
            schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
            capture_id: plan.preview.capture_id.clone(),
            project_id: plan.preview.project_id.clone(),
            disposition: plan.preview.disposition,
            library_revision,
            semantic_revision: next_manifest.semantic_revision,
            artifacts_updated: plan
                .artifacts
                .iter()
                .map(|artifact| artifact.artifact)
                .collect(),
            receipt_entry: plan.preview.receipt_entry.clone(),
            acknowledgement: receipt.acknowledgement,
            index_rebuild_required: true,
        })
    }
}

fn library_observation_matches(
    document: &crate::model::ResearchLibraryDocumentV1,
    expected_revision: u64,
    expected_entry: &RegisteredProjectV1,
) -> bool {
    document.revision == expected_revision
        && document
            .projects
            .iter()
            .find(|entry| entry.project_id == expected_entry.project_id)
            == Some(expected_entry)
}

fn validate_registered_manifest(
    entry: &RegisteredProjectV1,
    manifest: &ArticleProjectManifestV1,
    project_id: &ProjectId,
) -> Result<(), ProjectError> {
    if entry.project_id != *project_id
        || manifest.project_id != *project_id
        || entry.display_name != manifest.display_name
        || entry.project_kind != manifest.project_kind
        || entry.stage != manifest.stage
        || entry.lifecycle != manifest.lifecycle
        || entry.semantic_revision != manifest.semantic_revision
        || entry.semantic_digest != manifest.semantic_digest
        || entry.academically_updated_at_unix != manifest.academically_updated_at_unix
    {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

fn collect_conflicts(
    capture: &ResearchCaptureV1,
    entry: &RegisteredProjectV1,
    manifest: &ArticleProjectManifestV1,
    disposition: CaptureDisposition,
    conflicts: &mut Vec<CaptureConsolidationConflictV1>,
) {
    if entry.lifecycle != ProjectLifecycle::Active || manifest.lifecycle != ProjectLifecycle::Active
    {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::ProjectArchived,
            None,
        ));
    }
    if capture.binding.base_revision != manifest.semantic_revision {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::StaleProjectRevision,
            None,
        ));
    }
    if capture.binding.stage != manifest.stage {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::StageChanged,
            None,
        ));
    }
    if capture.binding.capture_policy == CapturePolicy::HistoryOnly {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::HistoryOnlyPolicy,
            None,
        ));
    }
    if capture
        .changes
        .iter()
        .any(|change| change.area == CaptureArea::Scope)
    {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::ScopeBoundaryChange,
            None,
        ));
    }
    if capture
        .decisions
        .iter()
        .any(|decision| decision.relation != DecisionRelation::Candidate)
    {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::LockedDecisionGuard,
            Some(ConsolidationArtifact::DecisionLog),
        ));
    }
    if !capture.contradictions.is_empty() {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::ContradictionRequiresResolution,
            None,
        ));
    }
    if disposition == CaptureDisposition::UnsupportedGap {
        conflicts.push(conflict(
            CaptureConsolidationConflictKind::UnsupportedEvidence,
            None,
        ));
    }
}

fn conflict(
    kind: CaptureConsolidationConflictKind,
    artifact: Option<ConsolidationArtifact>,
) -> CaptureConsolidationConflictV1 {
    let resolution = match kind {
        CaptureConsolidationConflictKind::ProjectArchived => "restore-project-before-consolidation",
        CaptureConsolidationConflictKind::StaleProjectRevision => {
            "rebase-capture-on-current-revision"
        }
        CaptureConsolidationConflictKind::StageChanged => "review-capture-against-current-stage",
        CaptureConsolidationConflictKind::HistoryOnlyPolicy => {
            "preserve-as-history-or-create-reviewed-capture"
        }
        CaptureConsolidationConflictKind::ScopeBoundaryChange => {
            "review-boundary-change-explicitly"
        }
        CaptureConsolidationConflictKind::LockedDecisionGuard => {
            "resolve-target-decision-transition-explicitly"
        }
        CaptureConsolidationConflictKind::ContradictionRequiresResolution => {
            "resolve-contradiction-before-merge"
        }
        CaptureConsolidationConflictKind::UnsupportedEvidence => {
            "attach-qualified-evidence-or-retain-as-gap"
        }
        CaptureConsolidationConflictKind::ArtifactNotUtf8 => {
            "repair-artifact-encoding-before-merge"
        }
        CaptureConsolidationConflictKind::ArtifactLineageConflict => {
            "repair-duplicate-capture-lineage"
        }
    };
    CaptureConsolidationConflictV1 {
        kind,
        artifact,
        resolution: resolution.to_string(),
    }
}

struct RenderConflict {
    kind: CaptureConsolidationConflictKind,
    artifact: Option<ConsolidationArtifact>,
}

enum ArtifactPlanError {
    Project(ProjectError),
    Conflict(RenderConflict),
}

impl From<ProjectError> for ArtifactPlanError {
    fn from(error: ProjectError) -> Self {
        Self::Project(error)
    }
}

impl From<RenderConflict> for ArtifactPlanError {
    fn from(error: RenderConflict) -> Self {
        Self::Conflict(error)
    }
}

fn plan_artifacts(
    root: &std::path::Path,
    capture: &ResearchCaptureV1,
) -> Result<Vec<PlannedArtifact>, ArtifactPlanError> {
    let mut artifacts = Vec::new();
    artifacts.push(plan_artifact(
        root,
        capture,
        ConsolidationArtifact::ResearchState,
        render_research_state,
    )?);
    if !capture.decisions.is_empty() {
        artifacts.push(plan_artifact(
            root,
            capture,
            ConsolidationArtifact::DecisionLog,
            render_decision_log,
        )?);
    }
    Ok(artifacts)
}

fn plan_artifact(
    root: &std::path::Path,
    capture: &ResearchCaptureV1,
    artifact: ConsolidationArtifact,
    render: fn(&str, &ResearchCaptureV1) -> String,
) -> Result<PlannedArtifact, ArtifactPlanError> {
    let observed = read_semantic_artifact(root, artifact.relative_path())?;
    let (previous, previous_digest, previous_bytes) = match observed {
        Some((bytes, digest)) => {
            let previous_bytes = bytes.len();
            let text = String::from_utf8(bytes).map_err(|_| {
                ArtifactPlanError::Conflict(RenderConflict {
                    kind: CaptureConsolidationConflictKind::ArtifactNotUtf8,
                    artifact: Some(artifact),
                })
            })?;
            (text, Some(digest), previous_bytes)
        }
        None => (String::new(), None, 0),
    };
    let marker = format!(
        "<!-- qiongli:capture {} begin -->",
        capture.capture_id.as_str()
    );
    if previous.contains(&marker) {
        return Err(RenderConflict {
            kind: CaptureConsolidationConflictKind::ArtifactLineageConflict,
            artifact: Some(artifact),
        }
        .into());
    }
    let next_bytes = render(&previous, capture).into_bytes();
    if next_bytes.len() > 4 * 1024 * 1024 {
        return Err(ArtifactPlanError::Project(ProjectError::DocumentTooLarge));
    }
    Ok(PlannedArtifact {
        artifact,
        previous_digest,
        previous_bytes,
        next_bytes,
    })
}

fn artifact_delta(artifact: &PlannedArtifact) -> ConsolidationArtifactDeltaV1 {
    ConsolidationArtifactDeltaV1 {
        artifact: artifact.artifact,
        relative_path: artifact.artifact.relative_path().to_string(),
        effect: if artifact.previous_digest.is_some() {
            ConsolidationArtifactEffect::Update
        } else {
            ConsolidationArtifactEffect::Create
        },
        previous_digest: artifact.previous_digest.clone(),
        next_digest: sha256_bytes(&artifact.next_bytes),
        previous_bytes: artifact.previous_bytes,
        next_bytes: artifact.next_bytes.len(),
    }
}

fn render_research_state(previous: &str, capture: &ResearchCaptureV1) -> String {
    let mut output = prepare_document(previous, "# Research State");
    let id = capture.capture_id.as_str();
    output.push_str(&format!("<!-- qiongli:capture {id} begin -->\n"));
    output.push_str(&format!("## Reviewed capture `{id}`\n\n"));
    output.push_str(&format!("- Source: {}\n", source_name(capture.source)));
    output.push_str(&format!(
        "- Bound task: {}\n",
        escape_markdown(&capture.binding.task)
    ));
    output.push_str(&format!(
        "- Summary: {}\n",
        escape_markdown(&capture.summary)
    ));
    if !capture.changes.is_empty() {
        output.push_str("\n### Reviewed academic changes\n\n");
        for change in &capture.changes {
            output.push_str(&format!(
                "- **{}:** {}\n",
                area_name(change.area),
                escape_markdown(&change.summary)
            ));
        }
    }
    if !capture.evidence.is_empty() {
        output.push_str("\n### Qualified evidence references\n\n");
        for evidence in &capture.evidence {
            output.push_str(&format!(
                "- **{}:** `{}` — {}",
                locator_name(evidence.locator_kind),
                escape_markdown(&evidence.locator),
                escape_markdown(&evidence.relevance)
            ));
            if let Some(limitation) = &evidence.limitation {
                output.push_str(&format!("; limitation: {}", escape_markdown(limitation)));
            }
            output.push('\n');
        }
    }
    if !capture.next_actions.is_empty() {
        output.push_str("\n### Next actions\n\n");
        for action in &capture.next_actions {
            output.push_str(&format!("- {}\n", escape_markdown(action)));
        }
    }
    output.push_str(&format!("\n<!-- qiongli:capture {id} end -->\n"));
    output
}

fn render_decision_log(previous: &str, capture: &ResearchCaptureV1) -> String {
    let mut output = prepare_document(previous, "# Decision Log");
    let id = capture.capture_id.as_str();
    output.push_str(&format!("<!-- qiongli:capture {id} begin -->\n"));
    output.push_str(&format!("## Reviewed capture `{id}`\n\n"));
    output.push_str("| Decision ID | Stage | Status | Decision | Rationale | Alternatives Rejected | Evidence Basis | Revisit Trigger | Downstream Impact |\n");
    output.push_str("|---|---|---|---|---|---|---|---|---|\n");
    let evidence_basis = if capture.evidence.is_empty() {
        "Not supplied".to_string()
    } else {
        capture
            .evidence
            .iter()
            .map(|evidence| escape_table(&evidence.locator))
            .collect::<Vec<_>>()
            .join("; ")
    };
    let downstream = if capture.changes.is_empty() {
        "Not specified".to_string()
    } else {
        capture
            .changes
            .iter()
            .map(|change| area_name(change.area))
            .collect::<Vec<_>>()
            .join(", ")
    };
    for (index, decision) in capture.decisions.iter().enumerate() {
        let decision_id = format!("dec_{}_{}", &id[4..20], index + 1);
        output.push_str(&format!(
            "| {} | {} | tentative | {} | {} | Not recorded | {} | Review before locking | {} |\n",
            decision_id,
            stage_name(capture.binding.stage),
            escape_table(&decision.statement),
            escape_table(&decision.rationale),
            evidence_basis,
            downstream,
        ));
    }
    output.push_str(&format!("\n<!-- qiongli:capture {id} end -->\n"));
    output
}

fn prepare_document(previous: &str, heading: &str) -> String {
    let mut output = if previous.is_empty() {
        format!("{heading}\n")
    } else {
        previous.to_string()
    };
    if !output.ends_with('\n') {
        output.push('\n');
    }
    if !output.ends_with("\n\n") {
        output.push('\n');
    }
    output
}

fn escape_markdown(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '\\' | '*' | '_' | '{' | '}' | '[' | ']' | '<' | '>' | '#' | '|' | '`'
        ) {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

fn escape_table(value: &str) -> String {
    escape_markdown(value)
}

const fn source_name(source: CaptureSource) -> &'static str {
    match source {
        CaptureSource::Codex => "Codex",
        CaptureSource::ClaudeCode => "Claude Code",
        CaptureSource::ChatGpt => "ChatGPT",
        CaptureSource::Cli => "CLI",
        CaptureSource::Manual => "Manual",
        CaptureSource::Repository => "Repository",
        CaptureSource::PortableFile => "Portable file",
    }
}

const fn area_name(area: CaptureArea) -> &'static str {
    match area {
        CaptureArea::ResearchQuestion => "research-question",
        CaptureArea::Thesis => "thesis",
        CaptureArea::Literature => "literature",
        CaptureArea::Method => "method",
        CaptureArea::Evidence => "evidence",
        CaptureArea::Analysis => "analysis",
        CaptureArea::Manuscript => "manuscript",
        CaptureArea::Scope => "scope",
    }
}

const fn locator_name(kind: EvidenceLocatorKind) -> &'static str {
    match kind {
        EvidenceLocatorKind::Doi => "DOI",
        EvidenceLocatorKind::CitationKey => "citation key",
        EvidenceLocatorKind::HttpsUrl => "HTTPS URL",
        EvidenceLocatorKind::ArtifactAnchor => "artifact anchor",
    }
}

const fn stage_name(stage: ProjectStage) -> &'static str {
    match stage {
        ProjectStage::Idea => "idea",
        ProjectStage::Framing => "framing",
        ProjectStage::Literature => "literature",
        ProjectStage::Design => "design",
        ProjectStage::Analysis => "analysis",
        ProjectStage::Writing => "writing",
        ProjectStage::Review => "review",
        ProjectStage::Submission => "submission",
    }
}

pub(crate) fn read_consolidation_receipt(
    root: &std::path::Path,
    capture_id: &CaptureId,
) -> Result<Option<(CaptureConsolidationReceiptV1, Vec<u8>)>, ProjectError> {
    let Some(bytes) = read_consolidation_document(root, capture_id)? else {
        return Ok(None);
    };
    let value = parse_unique_json(&bytes).map_err(|_| ProjectError::InvalidProjectDocument)?;
    let receipt: CaptureConsolidationReceiptV1 =
        serde_json::from_value(value).map_err(|_| ProjectError::InvalidProjectDocument)?;
    receipt.validate()?;
    if &receipt.capture_id != capture_id {
        return Err(ProjectError::CaptureIdentityConflict);
    }
    Ok(Some((receipt, bytes)))
}

fn validate_plan(plan: &VerifiedCaptureConsolidation) -> Result<(), ProjectError> {
    plan.capture.validate()?;
    if plan.preview.schema_version != ACADEMIC_CONSOLIDATION_SCHEMA_VERSION
        || plan.preview.capture_id != plan.capture.capture_id
        || plan.preview.project_id != plan.capture.binding.project_id
        || plan.preview.receipt_entry != consolidation_relative_path(&plan.capture.capture_id)
        || plan.preview.disposition != classify_capture(&plan.capture, false)
        || plan.preview.next_project_revision
            != plan
                .next_manifest
                .as_ref()
                .map(|manifest| manifest.semantic_revision)
        || plan.preview.artifact_deltas
            != plan
                .artifacts
                .iter()
                .map(artifact_delta)
                .collect::<Vec<_>>()
        || plan.preview.approvals_required
            != if plan.preview.outcome == CaptureConsolidationOutcome::Ready {
                vec![
                    "academic-consolidation".to_string(),
                    "filesystem-write".to_string(),
                ]
            } else {
                Vec::new()
            }
    {
        return Err(ProjectError::PlanMismatch);
    }
    let semantics = ConsolidationPlanSemantics {
        schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
        capture_id: &plan.preview.capture_id,
        project_id: &plan.preview.project_id,
        disposition: plan.preview.disposition,
        outcome: plan.preview.outcome,
        expected_library_revision: plan.preview.expected_library_revision,
        expected_project_revision: plan.preview.expected_project_revision,
        next_project_revision: plan.preview.next_project_revision,
        project_stage: plan.preview.project_stage,
        reviewed_at_unix: plan.preview.reviewed_at_unix,
        root_reference_digest: &plan.root_reference_digest,
        observed_manifest_digest: &plan.observed_manifest_digest,
        observed_receipt_digest: plan.observed_receipt_digest.as_deref(),
        capture_document_digest: &plan.capture_document_digest,
        conflicts: &plan.preview.conflicts,
        artifact_deltas: &plan.preview.artifact_deltas,
    };
    if canonical_digest(&semantics)? != plan.preview.plan_digest {
        return Err(ProjectError::PlanMismatch);
    }
    Ok(())
}

fn revalidate_apply_state(
    plan: &VerifiedCaptureConsolidation,
    entry: &RegisteredProjectV1,
) -> Result<(), ProjectError> {
    let root = project_root_from_string(&entry.root_path)?;
    if root != plan.root
        || sha256_bytes(project_root_string(&root)?.as_bytes()) != plan.root_reference_digest
    {
        return Err(ProjectError::RevisionConflict);
    }
    validate_existing_project_root(&root)?;
    let (manifest, digest) = read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
    validate_registered_manifest(entry, &manifest, &plan.preview.project_id)?;
    if digest != plan.observed_manifest_digest
        || manifest.semantic_revision != plan.preview.expected_project_revision
        || manifest.stage != plan.preview.project_stage
    {
        return Err(ProjectError::RevisionConflict);
    }
    let (capture, capture_digest) = read_capture_document(&root, &plan.preview.capture_id)?
        .ok_or(ProjectError::CaptureNotFound)?;
    if capture != plan.capture || capture_digest != plan.capture_document_digest {
        return Err(ProjectError::RevisionConflict);
    }
    if read_consolidation_receipt(&root, &plan.preview.capture_id)?.is_some() {
        return Err(ProjectError::ConsolidationAlreadyApplied);
    }
    for artifact in &plan.artifacts {
        let observed = read_semantic_artifact(&root, artifact.artifact.relative_path())?;
        if observed.as_ref().map(|(_, digest)| digest) != artifact.previous_digest.as_ref() {
            return Err(ProjectError::RevisionConflict);
        }
    }
    Ok(())
}

fn build_receipt(
    plan: &VerifiedCaptureConsolidation,
) -> Result<CaptureConsolidationReceiptV1, ProjectError> {
    Ok(CaptureConsolidationReceiptV1 {
        schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
        document_kind: CONSOLIDATION_DOCUMENT_KIND.to_string(),
        capture_id: plan.preview.capture_id.clone(),
        project_id: plan.preview.project_id.clone(),
        source_capture_digest: plan.capture_document_digest.clone(),
        plan_digest: plan.preview.plan_digest.clone(),
        disposition: plan.preview.disposition,
        from_project_revision: plan.preview.expected_project_revision,
        to_project_revision: plan
            .preview
            .next_project_revision
            .ok_or(ProjectError::PlanMismatch)?,
        project_stage: plan.preview.project_stage,
        consolidated_at_unix: plan.preview.reviewed_at_unix,
        artifacts: plan
            .artifacts
            .iter()
            .map(|artifact| ConsolidatedArtifactV1 {
                artifact: artifact.artifact,
                relative_path: artifact.artifact.relative_path().to_string(),
                digest: sha256_bytes(&artifact.next_bytes),
            })
            .collect(),
        acknowledgement: String::new(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AcknowledgementSemantics<'a> {
    schema_version: u32,
    capture_id: &'a CaptureId,
    project_id: &'a ProjectId,
    source_capture_digest: &'a str,
    plan_digest: &'a str,
    from_project_revision: u64,
    to_project_revision: u64,
    consolidated_at_unix: u64,
    artifacts: &'a [ConsolidatedArtifactV1],
}

fn acknowledgement(receipt: &CaptureConsolidationReceiptV1) -> Result<String, ProjectError> {
    let semantics = AcknowledgementSemantics {
        schema_version: receipt.schema_version,
        capture_id: &receipt.capture_id,
        project_id: &receipt.project_id,
        source_capture_digest: &receipt.source_capture_digest,
        plan_digest: &receipt.plan_digest,
        from_project_revision: receipt.from_project_revision,
        to_project_revision: receipt.to_project_revision,
        consolidated_at_unix: receipt.consolidated_at_unix,
        artifacts: &receipt.artifacts,
    };
    Ok(format!("ack_{}", canonical_digest(&semantics)?))
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidProjectDocument)?;
    Ok(sha256_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::resolve_config_root;

    use crate::{
        ApprovedCaptureIntake, ApprovedProjectMutation, CaptureDelivery, ContradictionV1,
        DecisionCandidateV1, EvidenceReferenceV1, ProjectBindingV1, ProjectKind,
        ProjectRegistrationOptions, ResearchCaptureDraftV1, SemanticChangeV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        base: PathBuf,
        project_root: PathBuf,
        service: ProjectStateService,
        project_id: ProjectId,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }

    fn fixture() -> Fixture {
        let base = std::env::temp_dir().join(format!(
            "qiongli-consolidation-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir(&base).unwrap();
        let base = fs::canonicalize(base).unwrap();
        let home = base.join("home");
        let project_root = base.join("paper");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&project_root).unwrap();
        fs::create_dir(project_root.join("context")).unwrap();
        fs::write(
            project_root.join(RESEARCH_STATE_PATH),
            "# Existing state\n\nUnmanaged research note.\n",
        )
        .unwrap();
        fs::write(
            project_root.join(DECISION_LOG_PATH),
            "# Existing decisions\n\nUnmanaged decision note.\n",
        )
        .unwrap();
        let service = ProjectStateService::new(resolve_config_root(None, &home).unwrap());
        let project_id = ProjectId::parse("prj_abcdef0123456789abcdef0123456789").unwrap();
        let register = service
            .preview_register(
                &project_root,
                ProjectRegistrationOptions::new("Consolidation paper", ProjectKind::Article)
                    .with_project_id(project_id.clone())
                    .with_stage(ProjectStage::Literature),
                100,
            )
            .unwrap();
        service
            .apply(
                &register,
                &ApprovedProjectMutation::new(register.preview().plan_digest.clone(), true),
                100,
            )
            .unwrap();
        Fixture {
            base,
            project_root,
            service,
            project_id,
        }
    }

    fn draft(project_id: ProjectId, policy: CapturePolicy) -> ResearchCaptureDraftV1 {
        ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                project_id,
                1,
                ProjectStage::Literature,
                "Reconcile the measurement literature",
                policy,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix: 110,
            summary: "Validity and reliability should remain distinct constructs.".to_string(),
            changes: vec![SemanticChangeV1 {
                area: CaptureArea::Literature,
                summary: "Separate the validity and reliability evidence streams.".to_string(),
            }],
            decisions: vec![DecisionCandidateV1 {
                relation: DecisionRelation::Candidate,
                statement: "Organize the review around construct validity.".to_string(),
                rationale: "The distinction explains disagreement across source clusters."
                    .to_string(),
                target: None,
            }],
            evidence: vec![EvidenceReferenceV1 {
                locator_kind: EvidenceLocatorKind::Doi,
                locator: "10.1000/consolidation".to_string(),
                relevance: "Defines the construct-validity distinction.".to_string(),
                limitation: Some("Conceptual evidence only.".to_string()),
            }],
            contradictions: Vec::new(),
            next_actions: vec!["Test the distinction against empirical papers.".to_string()],
        }
    }

    fn intake(fixture: &Fixture, draft: ResearchCaptureDraftV1) -> ResearchCaptureV1 {
        let capture = draft.into_capture().unwrap();
        let intake = fixture.service.preview_capture(capture.clone()).unwrap();
        fixture
            .service
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                115,
            )
            .unwrap();
        capture
    }

    #[test]
    fn reviewed_capture_updates_only_previewed_artifacts_and_records_receipt() {
        let fixture = fixture();
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        let prior_state = fs::read(fixture.project_root.join(RESEARCH_STATE_PATH)).unwrap();
        let prior_decisions = fs::read(fixture.project_root.join(DECISION_LOG_PATH)).unwrap();
        let plan = fixture
            .service
            .preview_capture_consolidation(&fixture.project_id, &capture.capture_id, 120)
            .unwrap();

        assert_eq!(plan.preview().outcome, CaptureConsolidationOutcome::Ready);
        assert_eq!(plan.preview().expected_project_revision, 1);
        assert_eq!(plan.preview().next_project_revision, Some(2));
        assert_eq!(plan.preview().artifact_deltas.len(), 2);
        assert_eq!(
            plan.preview().artifact_deltas[0].previous_bytes,
            prior_state.len()
        );
        assert_eq!(
            plan.preview().artifact_deltas[1].previous_bytes,
            prior_decisions.len()
        );
        assert_eq!(
            plan.preview().approvals_required,
            ["academic-consolidation", "filesystem-write"]
        );
        let debug = format!("{plan:?}");
        assert!(!debug.contains(&fixture.project_root.to_string_lossy().to_string()));
        assert!(!debug.contains(&capture.summary));
        assert_eq!(
            fixture.service.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(
                    plan.preview().plan_digest.clone(),
                    true,
                    false,
                ),
            ),
            Err(ProjectError::ApprovalRequired)
        );
        assert_eq!(
            fixture.service.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new("wrong-plan", true, true),
            ),
            Err(ProjectError::PlanMismatch)
        );

        let commit = fixture
            .service
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview().plan_digest.clone(), true, true),
            )
            .unwrap();
        assert_eq!(commit.semantic_revision, 2);
        assert_eq!(commit.acknowledgement.len(), 68);
        assert_eq!(commit.artifacts_updated.len(), 2);
        assert!(commit.index_rebuild_required);
        let state = fs::read_to_string(fixture.project_root.join(RESEARCH_STATE_PATH)).unwrap();
        assert!(state.starts_with("# Existing state\n\nUnmanaged research note.\n"));
        assert!(state.contains(capture.capture_id.as_str()));
        assert!(state.contains("Conceptual evidence only"));
        let decisions = fs::read_to_string(fixture.project_root.join(DECISION_LOG_PATH)).unwrap();
        assert!(decisions.starts_with("# Existing decisions\n\nUnmanaged decision note.\n"));
        assert!(decisions.contains("| tentative |"));
        assert!(fixture.project_root.join(&commit.receipt_entry).is_file());
        let snapshot = fixture.service.snapshot().unwrap();
        assert_eq!(snapshot.projects[0].semantic_revision, 2);
        let inbox = fixture.service.capture_inbox(&fixture.project_id).unwrap();
        assert_eq!(inbox.applied_count, 1);
        assert_eq!(inbox.entries[0].state, crate::CaptureInboxState::Applied);

        let export_root = fixture.base.join("portable-consolidated");
        let export = fixture
            .service
            .preview_export(&fixture.project_id, &export_root)
            .unwrap();
        fixture
            .service
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                125,
            )
            .unwrap();
        assert!(
            export_root
                .join("project")
                .join(&commit.receipt_entry)
                .is_file()
        );
        assert!(
            fs::read_to_string(export_root.join("project").join(RESEARCH_STATE_PATH))
                .unwrap()
                .contains(capture.capture_id.as_str())
        );
        assert!(!export_root.join("project/.qiongli").exists());

        let imported_home = fixture.base.join("imported-home");
        fs::create_dir(&imported_home).unwrap();
        let imported_service =
            ProjectStateService::new(resolve_config_root(None, &imported_home).unwrap());
        let imported_root = fixture.base.join("imported-paper");
        let import = imported_service
            .preview_import(&export_root, &imported_root)
            .unwrap();
        imported_service
            .apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                126,
            )
            .unwrap();
        let imported_inbox = imported_service.capture_inbox(&fixture.project_id).unwrap();
        assert_eq!(imported_inbox.project_revision, 2);
        assert_eq!(imported_inbox.applied_count, 1);
        assert_eq!(
            imported_inbox.entries[0].state,
            crate::CaptureInboxState::Applied
        );

        let replay = fixture
            .service
            .preview_capture_consolidation(&fixture.project_id, &capture.capture_id, 130)
            .unwrap();
        assert_eq!(
            replay.preview().outcome,
            CaptureConsolidationOutcome::AlreadyConsolidated
        );
        assert!(replay.preview().artifact_deltas.is_empty());
        assert_eq!(
            fixture.service.apply_capture_consolidation(
                &replay,
                &ApprovedCaptureConsolidation::new(
                    replay.preview().plan_digest.clone(),
                    true,
                    true,
                ),
            ),
            Err(ProjectError::ConsolidationAlreadyApplied)
        );
    }

    #[test]
    fn unsafe_semantic_transitions_are_conflicts_without_artifact_writes() {
        let fixture = fixture();
        let original_state = fs::read(fixture.project_root.join(RESEARCH_STATE_PATH)).unwrap();

        let mut scope = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        scope.changes[0].area = CaptureArea::Scope;
        let scope = intake(&fixture, scope);

        let mut locked = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        locked.captured_at_unix += 1;
        locked.decisions[0].relation = DecisionRelation::Refinement;
        locked.decisions[0].target = Some("dec_existing".to_string());
        let locked = intake(&fixture, locked);

        let mut unsupported = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        unsupported.captured_at_unix += 2;
        unsupported.evidence.clear();
        let unsupported = intake(&fixture, unsupported);

        let mut contradictory = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        contradictory.captured_at_unix += 3;
        contradictory.contradictions.push(ContradictionV1 {
            statement: "Reliability determines validity.".to_string(),
            conflicts_with: "The constructs are distinct.".to_string(),
            consequence: "The organizing distinction is unresolved.".to_string(),
        });
        let contradictory = intake(&fixture, contradictory);

        let history_only = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::HistoryOnly),
        );

        let cases = [
            (scope, CaptureConsolidationConflictKind::ScopeBoundaryChange),
            (
                locked,
                CaptureConsolidationConflictKind::LockedDecisionGuard,
            ),
            (
                unsupported,
                CaptureConsolidationConflictKind::UnsupportedEvidence,
            ),
            (
                contradictory,
                CaptureConsolidationConflictKind::ContradictionRequiresResolution,
            ),
            (
                history_only,
                CaptureConsolidationConflictKind::HistoryOnlyPolicy,
            ),
        ];
        for (capture, expected) in cases {
            let plan = fixture
                .service
                .preview_capture_consolidation(&fixture.project_id, &capture.capture_id, 120)
                .unwrap();
            assert_eq!(
                plan.preview().outcome,
                CaptureConsolidationOutcome::Conflicted
            );
            assert!(plan.preview().artifact_deltas.is_empty());
            assert!(
                plan.preview()
                    .conflicts
                    .iter()
                    .any(|item| item.kind == expected)
            );
            assert_eq!(
                fixture.service.apply_capture_consolidation(
                    &plan,
                    &ApprovedCaptureConsolidation::new(
                        plan.preview().plan_digest.clone(),
                        true,
                        true,
                    ),
                ),
                Err(ProjectError::ConsolidationConflict)
            );
        }
        assert_eq!(
            fs::read(fixture.project_root.join(RESEARCH_STATE_PATH)).unwrap(),
            original_state
        );
        assert_eq!(
            fixture.service.snapshot().unwrap().projects[0].semantic_revision,
            1
        );
    }

    #[test]
    fn consolidation_revalidates_library_capture_and_artifact_revisions() {
        let fixture = fixture();
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        let plan = fixture
            .service
            .preview_capture_consolidation(&fixture.project_id, &capture.capture_id, 120)
            .unwrap();
        fs::write(
            fixture.project_root.join(RESEARCH_STATE_PATH),
            "# Changed after preview\n",
        )
        .unwrap();
        assert_eq!(
            fixture.service.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview().plan_digest.clone(), true, true,),
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert!(
            !fixture
                .project_root
                .join(&plan.preview().receipt_entry)
                .exists()
        );
        assert_eq!(
            fixture.service.snapshot().unwrap().projects[0].semantic_revision,
            1
        );
    }

    #[test]
    fn stale_capture_is_previewed_as_a_conflict_after_project_refresh() {
        let fixture = fixture();
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        fs::write(
            fixture.project_root.join(RESEARCH_STATE_PATH),
            "# Independently revised state\n",
        )
        .unwrap();
        let refresh = fixture
            .service
            .preview_refresh(&fixture.project_id, 118)
            .unwrap();
        fixture
            .service
            .apply(
                &refresh,
                &ApprovedProjectMutation::new(refresh.preview().plan_digest.clone(), true),
                118,
            )
            .unwrap();

        let plan = fixture
            .service
            .preview_capture_consolidation(&fixture.project_id, &capture.capture_id, 120)
            .unwrap();
        assert_eq!(
            plan.preview().outcome,
            CaptureConsolidationOutcome::Conflicted
        );
        assert!(plan.preview().conflicts.iter().any(|conflict| {
            conflict.kind == CaptureConsolidationConflictKind::StaleProjectRevision
        }));
        assert!(plan.preview().artifact_deltas.is_empty());
        assert_eq!(
            fixture.service.snapshot().unwrap().projects[0].semantic_revision,
            2
        );
    }

    #[test]
    fn recovery_marker_blocks_project_reads_without_hiding_evidence() {
        let fixture = fixture();
        let _capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        let runtime = fixture.project_root.join(".qiongli");
        fs::create_dir_all(runtime.join("consolidation-transaction")).unwrap();
        assert_eq!(
            fixture
                .service
                .resolve_project_root(&fixture.project_id)
                .err(),
            Some(ProjectError::RecoveryRequired)
        );
        assert!(runtime.join("consolidation-transaction").is_dir());
    }

    #[test]
    fn ambiguous_commit_state_preserves_transaction_evidence() {
        let fixture = fixture();
        let state_path = fixture.project_root.join(RESEARCH_STATE_PATH);
        let original = fs::read(&state_path).unwrap();
        let replacement = b"# Applied but not reconciled\n".to_vec();
        let transaction = ProjectFileTransaction::apply(
            &fixture.project_root,
            &[ProjectFileUpdate {
                relative_path: RESEARCH_STATE_PATH.to_string(),
                expected_digest: Some(sha256_bytes(&original)),
                next_bytes: replacement.clone(),
            }],
        )
        .unwrap();
        transaction.preserve_for_recovery();

        assert_eq!(fs::read(state_path).unwrap(), replacement);
        assert_eq!(
            fixture
                .service
                .resolve_project_root(&fixture.project_id)
                .err(),
            Some(ProjectError::RecoveryRequired)
        );
        assert!(
            fixture
                .project_root
                .join(".qiongli/consolidation-transaction/journal.json")
                .is_file()
        );
    }

    #[test]
    fn library_reconciliation_requires_the_exact_revision_and_entry() {
        let fixture = fixture();
        let document = fixture.service.store.load().unwrap();
        let entry = document.projects[0].clone();
        assert!(library_observation_matches(
            &document,
            document.revision,
            &entry
        ));
        assert!(!library_observation_matches(
            &document,
            document.revision + 1,
            &entry
        ));
        let mut drifted = entry.clone();
        drifted.semantic_revision += 1;
        assert!(!library_observation_matches(
            &document,
            document.revision,
            &drifted
        ));
    }

    #[cfg(unix)]
    #[test]
    fn transaction_rolls_back_an_earlier_artifact_when_a_later_target_is_unsafe() {
        use std::os::unix::fs::symlink;

        let fixture = fixture();
        let state_path = fixture.project_root.join(RESEARCH_STATE_PATH);
        let original = fs::read(&state_path).unwrap();
        let unsafe_destination = fixture.base.join("unsafe-consolidations");
        fs::create_dir(&unsafe_destination).unwrap();
        symlink(
            &unsafe_destination,
            fixture.project_root.join("context/consolidations"),
        )
        .unwrap();
        let capture_id = CaptureId::parse(format!("cap_{}", "1".repeat(64))).unwrap();
        let updates = vec![
            ProjectFileUpdate {
                relative_path: RESEARCH_STATE_PATH.to_string(),
                expected_digest: Some(sha256_bytes(&original)),
                next_bytes: b"# Transactional replacement\n".to_vec(),
            },
            ProjectFileUpdate {
                relative_path: consolidation_relative_path(&capture_id),
                expected_digest: None,
                next_bytes: b"{}".to_vec(),
            },
        ];
        assert!(ProjectFileTransaction::apply(&fixture.project_root, &updates).is_err());
        assert_eq!(fs::read(state_path).unwrap(), original);
        assert!(
            !fixture
                .project_root
                .join(".qiongli/consolidation-transaction")
                .exists()
        );
    }
}
