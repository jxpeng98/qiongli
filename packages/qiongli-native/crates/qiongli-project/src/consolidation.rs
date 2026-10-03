use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::capture::{
    CaptureArea, CaptureDisposition, CaptureId, CapturePolicy, CaptureSource, DecisionRelation,
    EvidenceLocatorKind, ResearchCaptureV1, classify_capture,
};
use crate::json::parse_unique_json;
use crate::model::{
    ArticleProjectManifestV1, MAX_SEMANTIC_REVISION, ProjectId, ProjectLifecycle, ProjectStage,
    RegisteredProjectV1, valid_lower_hex,
};
use crate::paper_note::valid_note_path;
use crate::retrieval_manifest::RETRIEVAL_MANIFEST_PATH;
use crate::service::ProjectStateService;
use crate::source_packet::valid_packet_path;
use crate::stage_summary::valid_summary_path;
use crate::storage::{
    ProjectFileTransaction, ProjectFileUpdate, consolidation_relative_path,
    encode_project_document, project_root_from_string, project_root_string, read_capture_document,
    read_consolidation_document, read_manifest, read_project_source, read_semantic_artifact,
    semantic_digest, semantic_digest_with_overrides, sha256_bytes, validate_existing_project_root,
};
use crate::{
    PaperNoteDraftV1, ProjectError, RetrievalManifestDraftV1, SourcePacketDraftV1,
    StageSummaryDraftV1,
};

pub const ACADEMIC_CONSOLIDATION_SCHEMA_VERSION: u32 = 1;
const CONSOLIDATION_DOCUMENT_KIND: &str = "qiongli-capture-consolidation";
const RESEARCH_STATE_PATH: &str = "context/research_state.md";
const DECISION_LOG_PATH: &str = "context/decision_log.md";
const PROJECT_MANIFEST_PATH: &str = "context/project_manifest.json";
const STAGE_HANDOFF_PATH: &str = "context/stage_handoff.md";
const MAX_CONSOLIDATED_ARTIFACTS: usize = 7;

/// Optional reviewed documents composed by the existing consolidation owner.
#[derive(Clone, Copy, Default)]
pub struct CaptureConsolidationDrafts<'a> {
    pub stage_handoff: Option<&'a str>,
    pub stage_summary: Option<&'a StageSummaryDraftV1>,
    pub paper_note: Option<&'a PaperNoteDraftV1>,
    pub source_packet: Option<&'a SourcePacketDraftV1>,
    pub retrieval_manifest: Option<&'a RetrievalManifestDraftV1>,
}

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
    StageHandoff,
    StageSummary,
    PaperNote,
    SourcePacket,
    RetrievalManifest,
}

impl ConsolidationArtifact {
    const fn relative_path(self) -> Option<&'static str> {
        match self {
            Self::ResearchState => Some(RESEARCH_STATE_PATH),
            Self::DecisionLog => Some(DECISION_LOG_PATH),
            Self::StageHandoff => Some(STAGE_HANDOFF_PATH),
            Self::RetrievalManifest => Some(RETRIEVAL_MANIFEST_PATH),
            Self::StageSummary | Self::PaperNote | Self::SourcePacket => None,
        }
    }

    fn accepts_path(self, path: &str) -> bool {
        match self {
            Self::StageSummary => valid_summary_path(path),
            Self::PaperNote => valid_note_path(path),
            Self::SourcePacket => valid_packet_path(path),
            _ => self.relative_path() == Some(path),
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
            if !artifact.artifact.accepts_path(&artifact.relative_path)
                || !valid_lower_hex(&artifact.digest, 64)
                || (artifact.artifact == ConsolidationArtifact::SourcePacket
                    && !artifact
                        .relative_path
                        .ends_with(&format!("/{}.json", artifact.digest)))
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
    relative_path: String,
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
    stage_summary: Option<StageSummaryDraftV1>,
    paper_note: Option<PaperNoteDraftV1>,
    source_packet: Option<SourcePacketDraftV1>,
    retrieval_manifest: Option<RetrievalManifestDraftV1>,
}

impl VerifiedCaptureConsolidation {
    #[must_use]
    pub const fn preview(&self) -> &CaptureConsolidationPreviewV1 {
        &self.preview
    }

    /// Exact proposed file bytes for review; omitted when no handoff is planned.
    #[must_use]
    pub fn stage_handoff_content(&self) -> Option<&str> {
        self.artifact_content(ConsolidationArtifact::StageHandoff)
    }

    #[must_use]
    pub fn stage_summary_content(&self) -> Option<&str> {
        self.artifact_content(ConsolidationArtifact::StageSummary)
    }

    #[must_use]
    pub fn paper_note_content(&self) -> Option<&str> {
        self.artifact_content(ConsolidationArtifact::PaperNote)
    }

    #[must_use]
    pub fn source_packet_content(&self) -> Option<&str> {
        self.artifact_content(ConsolidationArtifact::SourcePacket)
    }

    #[must_use]
    pub fn retrieval_manifest_content(&self) -> Option<&str> {
        self.artifact_content(ConsolidationArtifact::RetrievalManifest)
    }

    /// Include the exact history-table edit whenever a summary is planned.
    #[must_use]
    pub fn summary_research_state_content(&self) -> Option<&str> {
        self.stage_summary.as_ref()?;
        self.artifact_content(ConsolidationArtifact::ResearchState)
    }

    fn artifact_content(&self, target: ConsolidationArtifact) -> Option<&str> {
        self.artifacts
            .iter()
            .find(|artifact| artifact.artifact == target)
            .and_then(|artifact| std::str::from_utf8(&artifact.next_bytes).ok())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_summary: Option<&'a StageSummaryDraftV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paper_note: Option<&'a PaperNoteDraftV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_packet: Option<&'a SourcePacketDraftV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retrieval_manifest: Option<&'a RetrievalManifestDraftV1>,
}

impl ProjectStateService {
    pub fn preview_capture_consolidation(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
        reviewed_at_unix: u64,
    ) -> Result<VerifiedCaptureConsolidation, ProjectError> {
        self.preview_capture_consolidation_with_handoff(
            project_id,
            capture_id,
            reviewed_at_unix,
            None,
        )
    }

    /// Append an explicitly reviewed handoff alongside the capture's normal artifacts.
    /// Markdown is retained verbatim; academic completeness remains a review obligation.
    pub fn preview_capture_consolidation_with_handoff(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
        reviewed_at_unix: u64,
        stage_handoff: Option<&str>,
    ) -> Result<VerifiedCaptureConsolidation, ProjectError> {
        self.preview_capture_consolidation_with_summary(
            project_id,
            capture_id,
            reviewed_at_unix,
            stage_handoff,
            None,
        )
    }

    /// Save one immutable summary and its continuity links with the existing owner.
    pub fn preview_capture_consolidation_with_summary(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
        reviewed_at_unix: u64,
        stage_handoff: Option<&str>,
        stage_summary: Option<&StageSummaryDraftV1>,
    ) -> Result<VerifiedCaptureConsolidation, ProjectError> {
        self.preview_capture_consolidation_with_paper_note(
            project_id,
            capture_id,
            reviewed_at_unix,
            stage_handoff,
            stage_summary,
            None,
        )
    }

    /// Create or append one source-bound note through the same reviewed transaction.
    pub fn preview_capture_consolidation_with_paper_note(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
        reviewed_at_unix: u64,
        stage_handoff: Option<&str>,
        stage_summary: Option<&StageSummaryDraftV1>,
        paper_note: Option<&PaperNoteDraftV1>,
    ) -> Result<VerifiedCaptureConsolidation, ProjectError> {
        self.preview_capture_consolidation_with_drafts(
            project_id,
            capture_id,
            reviewed_at_unix,
            CaptureConsolidationDrafts {
                stage_handoff,
                stage_summary,
                paper_note,
                source_packet: None,
                retrieval_manifest: None,
            },
        )
    }

    pub fn preview_capture_consolidation_with_drafts(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
        reviewed_at_unix: u64,
        drafts: CaptureConsolidationDrafts<'_>,
    ) -> Result<VerifiedCaptureConsolidation, ProjectError> {
        let CaptureConsolidationDrafts {
            stage_handoff,
            stage_summary,
            paper_note,
            source_packet,
            retrieval_manifest,
        } = drafts;
        if let Some(packet) = source_packet {
            packet.validate()?;
        }
        if let Some(manifest) = retrieval_manifest {
            manifest.validate()?;
        }
        if let Some(note) = paper_note {
            note.validate()?;
        }
        if let Some(summary) = stage_summary {
            summary.validate()?;
        }
        if let Some(handoff) = stage_handoff {
            if handoff.len() > 4 * 1024 * 1024 {
                return Err(ProjectError::DocumentTooLarge);
            }
            if handoff.trim().is_empty()
                || handoff.contains('\0')
                || handoff.contains("<!-- qiongli:")
            {
                return Err(ProjectError::InvalidProjectDocument);
            }
        }
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
        if semantic_digest(&root)? != manifest.semantic_digest {
            return Err(ProjectError::RevisionConflict);
        }
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

        if let Some(summary) = stage_summary {
            summary.revalidate_sources(&root)?;
        }
        if let Some(note) = paper_note {
            note.revalidate_sources(&root)?;
        }
        if let Some(manifest) = retrieval_manifest {
            manifest.revalidate_sources(&root)?;
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
                match plan_artifacts(&root, &capture, drafts, reviewed_at_unix) {
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
                    .filter(|artifact| {
                        crate::storage::SEMANTIC_ARTIFACTS
                            .contains(&artifact.relative_path.as_str())
                    })
                    .map(|artifact| ProjectFileUpdate {
                        relative_path: artifact.relative_path.clone(),
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
            stage_summary,
            paper_note,
            source_packet,
            retrieval_manifest,
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
            stage_summary: stage_summary.cloned(),
            paper_note: paper_note.cloned(),
            source_packet: source_packet.cloned(),
            retrieval_manifest: retrieval_manifest.cloned(),
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
                relative_path: artifact.relative_path.clone(),
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
    drafts: CaptureConsolidationDrafts<'_>,
    reviewed_at_unix: u64,
) -> Result<Vec<PlannedArtifact>, ArtifactPlanError> {
    let CaptureConsolidationDrafts {
        stage_handoff,
        stage_summary,
        paper_note,
        source_packet,
        retrieval_manifest,
    } = drafts;
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
    if let Some(summary) = stage_summary {
        let state = std::str::from_utf8(&artifacts[0].next_bytes)
            .map_err(|_| ProjectError::InvalidProjectDocument)?;
        artifacts[0].next_bytes = summary
            .append_history(state, stage_name(capture.binding.stage), reviewed_at_unix)?
            .into_bytes();
        let relative_path = summary.relative_path();
        if read_project_source(root, &relative_path)?.is_some() {
            return Err(ProjectError::ConsolidationConflict.into());
        }
        artifacts.push(PlannedArtifact {
            artifact: ConsolidationArtifact::StageSummary,
            relative_path,
            previous_digest: None,
            previous_bytes: 0,
            next_bytes: summary
                .render(
                    stage_name(capture.binding.stage),
                    capture.binding.base_revision,
                    reviewed_at_unix,
                )
                .into_bytes(),
        });
    }
    if let Some(note) = paper_note {
        let relative_path = note.relative_path();
        let previous = read_project_source(root, &relative_path)?;
        if previous.as_ref().map(|(_, digest)| digest) != note.previous_sha256.as_ref() {
            return Err(ProjectError::RevisionConflict.into());
        }
        let bytes = previous
            .as_ref()
            .map_or(&[][..], |(bytes, _)| bytes.as_slice());
        let text = std::str::from_utf8(bytes).map_err(|_| ProjectError::InvalidProjectDocument)?;
        if text.contains('\0')
            || text.contains(&format!(
                "<!-- qiongli:capture {} begin -->",
                capture.capture_id.as_str()
            ))
        {
            return Err(ProjectError::ConsolidationConflict.into());
        }
        artifacts.push(PlannedArtifact {
            artifact: ConsolidationArtifact::PaperNote,
            relative_path,
            previous_digest: note.previous_sha256.clone(),
            previous_bytes: bytes.len(),
            next_bytes: note
                .render(text, &capture.capture_id, reviewed_at_unix)
                .into_bytes(),
        });
    }
    if let Some(packet) = source_packet {
        let relative_path = packet.relative_path();
        if read_project_source(root, &relative_path)?.is_some() {
            return Err(ProjectError::ConsolidationConflict.into());
        }
        artifacts.push(PlannedArtifact {
            artifact: ConsolidationArtifact::SourcePacket,
            relative_path,
            previous_digest: None,
            previous_bytes: 0,
            next_bytes: packet.content.as_bytes().to_vec(),
        });
    }
    if let Some(manifest) = retrieval_manifest {
        let previous = read_project_source(root, RETRIEVAL_MANIFEST_PATH)?;
        if previous.as_ref().map(|(_, digest)| digest) != manifest.previous_sha256.as_ref() {
            return Err(ProjectError::RevisionConflict.into());
        }
        let previous_text = previous
            .as_ref()
            .map(|(bytes, _)| std::str::from_utf8(bytes))
            .transpose()
            .map_err(|_| ProjectError::InvalidProjectDocument)?;
        artifacts.push(PlannedArtifact {
            artifact: ConsolidationArtifact::RetrievalManifest,
            relative_path: RETRIEVAL_MANIFEST_PATH.to_string(),
            previous_digest: manifest.previous_sha256.clone(),
            previous_bytes: previous.as_ref().map_or(0, |(bytes, _)| bytes.len()),
            next_bytes: manifest.render(previous_text)?.into_bytes(),
        });
    }
    let handoff = stage_summary
        .map(|summary| format!("{}{}", stage_handoff.unwrap_or(""), summary.handoff_link()));
    if let Some(handoff) = handoff.as_deref().or(stage_handoff) {
        artifacts.push(plan_artifact(
            root,
            capture,
            ConsolidationArtifact::StageHandoff,
            |previous, capture| {
                let mut output = prepare_document(previous, "# Stage Handoff");
                let id = capture.capture_id.as_str();
                output.push_str(&format!("<!-- qiongli:capture {id} begin -->\n"));
                output.push_str(&format!("## Reviewed handoff `{id}`\n\n"));
                output.push_str(handoff);
                output.push_str(&format!("\n\n<!-- qiongli:capture {id} end -->\n"));
                output
            },
        )?);
    }
    if artifacts
        .iter()
        .any(|artifact| artifact.next_bytes.len() > 4 * 1024 * 1024)
    {
        return Err(ProjectError::DocumentTooLarge.into());
    }
    Ok(artifacts)
}

fn plan_artifact(
    root: &std::path::Path,
    capture: &ResearchCaptureV1,
    artifact: ConsolidationArtifact,
    render: impl FnOnce(&str, &ResearchCaptureV1) -> String,
) -> Result<PlannedArtifact, ArtifactPlanError> {
    let relative_path = artifact
        .relative_path()
        .ok_or(ProjectError::InvalidProjectDocument)?;
    let observed = read_semantic_artifact(root, relative_path)?;
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
        relative_path: relative_path.to_string(),
        previous_digest,
        previous_bytes,
        next_bytes,
    })
}

fn artifact_delta(artifact: &PlannedArtifact) -> ConsolidationArtifactDeltaV1 {
    ConsolidationArtifactDeltaV1 {
        artifact: artifact.artifact,
        relative_path: artifact.relative_path.clone(),
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
        stage_summary: plan.stage_summary.as_ref(),
        paper_note: plan.paper_note.as_ref(),
        source_packet: plan.source_packet.as_ref(),
        retrieval_manifest: plan.retrieval_manifest.as_ref(),
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
    if semantic_digest(&root)? != manifest.semantic_digest {
        return Err(ProjectError::RevisionConflict);
    }
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
    if let Some(summary) = &plan.stage_summary {
        summary.revalidate_sources(&root)?;
    }
    if let Some(note) = &plan.paper_note {
        note.revalidate_sources(&root)?;
    }
    if let Some(manifest) = &plan.retrieval_manifest {
        manifest.revalidate_sources(&root)?;
    }
    for artifact in &plan.artifacts {
        if !artifact.artifact.accepts_path(&artifact.relative_path) {
            return Err(ProjectError::PlanMismatch);
        }
        let observed = read_project_source(&root, &artifact.relative_path)?;
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
                relative_path: artifact.relative_path.clone(),
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

    const HANDOFF: &str = "### Decision Summary
DEC-001 / CLM-001: retain @example and evidence/claim-evidence-ledger.csv#EV-001.
### Evidence Dependencies
Abstract-only conceptual evidence; no full-text or causal inference.
### Unresolved Questions
The denominator remains unresolved.
### Recommended Next Tasks
Inspect the source table before dependent analysis.
";

    #[test]
    fn stage_handoff_preserves_history_and_resumes_from_current_disk_revision() {
        let fixture = fixture();
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        let plan = fixture
            .service
            .preview_capture_consolidation_with_handoff(
                &fixture.project_id,
                &capture.capture_id,
                120,
                Some(HANDOFF),
            )
            .unwrap();
        let expected = plan.stage_handoff_content().unwrap().to_owned();
        assert!(!fixture.project_root.join(STAGE_HANDOFF_PATH).exists());
        assert!(!format!("{plan:?}").contains("denominator"));
        assert_eq!(plan.preview().artifact_deltas.len(), 3);
        for (filesystem, academic) in [(false, true), (true, false)] {
            assert_eq!(
                fixture.service.apply_capture_consolidation(
                    &plan,
                    &ApprovedCaptureConsolidation::new(
                        plan.preview().plan_digest.clone(),
                        filesystem,
                        academic
                    )
                ),
                Err(ProjectError::ApprovalRequired)
            );
        }
        let changed = fixture
            .service
            .preview_capture_consolidation_with_handoff(
                &fixture.project_id,
                &capture.capture_id,
                120,
                Some("Changed draft"),
            )
            .unwrap();
        assert_eq!(
            fixture.service.apply_capture_consolidation(
                &changed,
                &ApprovedCaptureConsolidation::new(plan.preview().plan_digest.clone(), true, true)
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
        assert_eq!(commit.artifacts_updated.len(), 3);
        assert_eq!(
            fs::read_to_string(fixture.project_root.join(STAGE_HANDOFF_PATH)).unwrap(),
            expected
        );
        let restarted = ProjectStateService::new(
            resolve_config_root(None, &fixture.base.join("home")).unwrap(),
        );
        let graph = crate::AcademicGraphService::new(restarted.clone());
        let view = graph
            .read_registered_artifact(&fixture.project_id, 2, STAGE_HANDOFF_PATH, None, 16_384)
            .unwrap();
        assert_eq!(view.content, expected);
        assert_eq!(view.content_digest, sha256_bytes(expected.as_bytes()));
        assert_eq!(
            graph
                .read_registered_artifact(&fixture.project_id, 1, STAGE_HANDOFF_PATH, None, 16_384)
                .unwrap_err(),
            ProjectError::RevisionConflict
        );
        assert_eq!(
            restarted
                .capture_inbox(&fixture.project_id)
                .unwrap()
                .applied_count,
            1
        );
        assert_eq!(
            restarted.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview().plan_digest.clone(), true, true)
            ),
            Err(ProjectError::RevisionConflict)
        );

        let mut next = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        next.binding.base_revision = 2;
        next.captured_at_unix = 130;
        let next = next.into_capture().unwrap();
        let intake = restarted.preview_capture(next.clone()).unwrap();
        restarted
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                135,
            )
            .unwrap();
        let plan = restarted
            .preview_capture_consolidation_with_handoff(
                &fixture.project_id,
                &next.capture_id,
                140,
                Some("### Decision Summary\nDEC-001 remains tentative; EV-001 is unchanged."),
            )
            .unwrap();
        assert!(plan.stage_handoff_content().unwrap().starts_with(&expected));
        restarted
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview().plan_digest.clone(), true, true),
            )
            .unwrap();
        assert!(
            fs::read_to_string(fixture.project_root.join(STAGE_HANDOFF_PATH))
                .unwrap()
                .starts_with(&expected)
        );
    }

    #[test]
    fn stage_handoff_conflicts_preserve_all_existing_bytes_and_receipts() {
        // Include an input that consolidation does not write: source drift must
        // not be silently incorporated into the next manifest.
        for path in [
            STAGE_HANDOFF_PATH,
            RESEARCH_STATE_PATH,
            "context/boundary_review.md",
        ] {
            let fixture = fixture();
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            let plan = fixture
                .service
                .preview_capture_consolidation_with_handoff(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    Some(HANDOFF),
                )
                .unwrap();
            fs::write(fixture.project_root.join(path), "# Independently changed\n").unwrap();
            let paths = [
                RESEARCH_STATE_PATH,
                DECISION_LOG_PATH,
                STAGE_HANDOFF_PATH,
                PROJECT_MANIFEST_PATH,
                "context/boundary_review.md",
            ];
            let before = paths.map(|path| fs::read(fixture.project_root.join(path)).ok());
            assert_eq!(
                fixture.service.apply_capture_consolidation(
                    &plan,
                    &ApprovedCaptureConsolidation::new(
                        plan.preview().plan_digest.clone(),
                        true,
                        true
                    )
                ),
                Err(ProjectError::RevisionConflict)
            );
            assert_eq!(
                fixture
                    .service
                    .preview_capture_consolidation_with_handoff(
                        &fixture.project_id,
                        &capture.capture_id,
                        120,
                        Some(HANDOFF)
                    )
                    .unwrap_err(),
                ProjectError::RevisionConflict
            );
            assert_eq!(
                paths.map(|path| fs::read(fixture.project_root.join(path)).ok()),
                before
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
    }

    #[test]
    fn stage_handoff_input_is_bounded_utf8_data_not_a_path_or_lineage_override() {
        let fixture = fixture();
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        for text in [" ", "bad\0text", "<!-- qiongli:capture forged begin -->"] {
            assert_eq!(
                fixture
                    .service
                    .preview_capture_consolidation_with_handoff(
                        &fixture.project_id,
                        &capture.capture_id,
                        120,
                        Some(text)
                    )
                    .unwrap_err(),
                ProjectError::InvalidProjectDocument
            );
        }
        assert_eq!(
            fixture
                .service
                .preview_capture_consolidation_with_handoff(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    Some(&"x".repeat(4 * 1024 * 1024 + 1))
                )
                .unwrap_err(),
            ProjectError::DocumentTooLarge
        );
        let file = fixture.base.join("handoff.md");
        fs::write(&file, HANDOFF).unwrap();
        assert_eq!(crate::read_stage_handoff_file(&file).unwrap(), HANDOFF);
        assert!(crate::read_stage_handoff_file(std::path::Path::new("handoff.md")).is_err());
        assert!(crate::read_stage_handoff_file(&fixture.base).is_err());
        fs::write(&file, [0xff]).unwrap();
        assert_eq!(
            crate::read_stage_handoff_file(&file),
            Err(ProjectError::InvalidProjectDocument)
        );
        fs::write(&file, vec![b'x'; 4 * 1024 * 1024 + 1]).unwrap();
        assert_eq!(
            crate::read_stage_handoff_file(&file),
            Err(ProjectError::DocumentTooLarge)
        );
        #[cfg(unix)]
        {
            let link = fixture.base.join("handoff-link.md");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            assert!(crate::read_stage_handoff_file(&link).is_err());
        }
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

    #[test]
    fn retrieval_manifest_rechecks_sources_and_history_before_any_write() {
        for mutation in [
            "packet",
            "fulltext",
            "missing",
            "destination",
            "old-manifest",
            "academic",
            "filesystem",
            "symlink",
        ] {
            let fixture = fixture();
            let packet = packet_draft();
            let packet_path = fixture.project_root.join(packet.relative_path());
            fs::create_dir_all(packet_path.parent().unwrap()).unwrap();
            fs::write(&packet_path, &packet.content).unwrap();
            let text_path = fixture.project_root.join("sources/body.txt");
            fs::write(&text_path, "Actual saved excerpt").unwrap();
            let mut retrieval = crate::retrieval_manifest::test_draft();
            retrieval.attempts[0].source_packet = Some(crate::StageSummarySourceV1 {
                relative_path: packet.relative_path(),
                sha256: sha256_bytes(packet.content.as_bytes()),
            });
            retrieval.attempts[0].fulltext_path = "sources/body.txt".into();
            retrieval.attempts[0].fulltext_sha256 = Some(sha256_bytes(b"Actual saved excerpt"));
            let target = fixture.project_root.join(RETRIEVAL_MANIFEST_PATH);
            if mutation == "old-manifest" {
                let previous = retrieval.render(None).unwrap();
                fs::write(&target, &previous).unwrap();
                retrieval.previous_sha256 = Some(sha256_bytes(previous.as_bytes()));
            }
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            let plan = fixture
                .service
                .preview_capture_consolidation_with_drafts(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    CaptureConsolidationDrafts {
                        retrieval_manifest: Some(&retrieval),
                        ..Default::default()
                    },
                )
                .unwrap();
            match mutation {
                "packet" => fs::write(&packet_path, "changed packet").unwrap(),
                "fulltext" => fs::write(&text_path, "changed excerpt").unwrap(),
                "missing" => fs::remove_file(&packet_path).unwrap(),
                "destination" | "old-manifest" => {
                    fs::write(&target, "User file; do not replace").unwrap()
                }
                "symlink" => {
                    #[cfg(unix)]
                    {
                        fs::remove_file(&packet_path).unwrap();
                        std::os::unix::fs::symlink(&text_path, &packet_path).unwrap();
                    }
                    #[cfg(not(unix))]
                    {
                        continue;
                    }
                }
                _ => {}
            }
            let mut paths = plan
                .artifacts
                .iter()
                .map(|a| a.relative_path.clone())
                .collect::<Vec<_>>();
            paths.extend([
                PROJECT_MANIFEST_PATH.into(),
                plan.preview.receipt_entry.clone(),
                packet.relative_path(),
                "sources/body.txt".into(),
            ]);
            let before = paths
                .iter()
                .map(|p| fs::read(fixture.project_root.join(p)).ok())
                .collect::<Vec<_>>();
            assert!(
                fixture
                    .service
                    .apply_capture_consolidation(
                        &plan,
                        &ApprovedCaptureConsolidation::new(
                            plan.preview.plan_digest.clone(),
                            mutation != "filesystem",
                            mutation != "academic"
                        )
                    )
                    .is_err(),
                "{mutation}"
            );
            assert_eq!(
                before,
                paths
                    .iter()
                    .map(|p| fs::read(fixture.project_root.join(p)).ok())
                    .collect::<Vec<_>>(),
                "{mutation}"
            );
            assert!(
                !fixture
                    .project_root
                    .join(".qiongli/consolidation-transaction")
                    .exists()
            );
            assert_eq!(
                fixture.service.snapshot().unwrap().projects[0].semantic_revision,
                1
            );
        }
    }

    #[test]
    fn retrieval_manifest_is_receipted_and_can_bind_a_later_note_after_restart() {
        let fixture = fixture();
        let retrieval = crate::retrieval_manifest::test_draft();
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        let plan = fixture
            .service
            .preview_capture_consolidation_with_drafts(
                &fixture.project_id,
                &capture.capture_id,
                120,
                CaptureConsolidationDrafts {
                    retrieval_manifest: Some(&retrieval),
                    ..Default::default()
                },
            )
            .unwrap();
        let content = plan.retrieval_manifest_content().unwrap().to_string();
        fixture
            .service
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview.plan_digest.clone(), true, true),
            )
            .unwrap();
        let (receipt, _) = read_consolidation_receipt(&fixture.project_root, &capture.capture_id)
            .unwrap()
            .unwrap();
        let artifact = receipt
            .artifacts
            .iter()
            .find(|a| a.artifact == ConsolidationArtifact::RetrievalManifest)
            .unwrap();
        assert_eq!(artifact.relative_path, RETRIEVAL_MANIFEST_PATH);
        assert_eq!(artifact.digest, sha256_bytes(content.as_bytes()));
        let mut forged = receipt;
        forged
            .artifacts
            .iter_mut()
            .find(|a| a.artifact == ConsolidationArtifact::RetrievalManifest)
            .unwrap()
            .relative_path = "other.csv".into();
        forged.acknowledgement = acknowledgement(&forged).unwrap();
        assert_eq!(forged.validate(), Err(ProjectError::InvalidProjectDocument));
        let restarted = ProjectStateService::new(
            resolve_config_root(None, &fixture.base.join("home")).unwrap(),
        );
        let mut next_capture = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        next_capture.binding.base_revision = 2;
        let capture = intake(&fixture, next_capture);
        let note = PaperNoteDraftV1 {
            schema_version: 1,
            citekey: "Smith2024".into(),
            previous_sha256: None,
            sources: vec![crate::StageSummarySourceV1 {
                relative_path: RETRIEVAL_MANIFEST_PATH.into(),
                sha256: sha256_bytes(content.as_bytes()),
            }],
            markdown: "CLM-001 still needs evidence: native retrieval failed before HTTP.".into(),
        };
        let plan = restarted
            .preview_capture_consolidation_with_paper_note(
                &fixture.project_id,
                &capture.capture_id,
                130,
                None,
                None,
                Some(&note),
            )
            .unwrap();
        assert!(plan.retrieval_manifest_content().is_none());
        restarted
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview.plan_digest.clone(), true, true),
            )
            .unwrap();
        assert_eq!(
            fs::read_to_string(fixture.project_root.join(RETRIEVAL_MANIFEST_PATH)).unwrap(),
            content
        );
        assert!(
            fs::read_to_string(fixture.project_root.join(note.relative_path()))
                .unwrap()
                .contains(RETRIEVAL_MANIFEST_PATH)
        );
    }

    #[test]
    fn retrieval_manifest_draft_envelope_is_strict_and_bounded() {
        let fixture = fixture();
        let path = fixture.base.join("retrieval.json");
        let draft = crate::retrieval_manifest::test_draft();
        let json = serde_json::to_string(&draft).unwrap();
        fs::write(&path, &json).unwrap();
        assert!(RetrievalManifestDraftV1::read_file(&path).is_ok());
        for value in [
            json.replacen(
                "\"schemaVersion\":1",
                "\"schemaVersion\":1,\"schemaVersion\":1",
                1,
            ),
            json.replacen(
                "\"schemaVersion\":1",
                "\"schemaVersion\":1,\"unknown\":true",
                1,
            ),
            json.replacen("\"recordId\":", "\"extra\":true,\"recordId\":", 1),
            " ".repeat(4 * 1024 * 1024 + 1),
        ] {
            fs::write(&path, value).unwrap();
            assert!(RetrievalManifestDraftV1::read_file(&path).is_err());
        }
        assert!(
            RetrievalManifestDraftV1::read_file(std::path::Path::new("relative.json")).is_err()
        );
        #[cfg(unix)]
        {
            let link = fixture.base.join("linked-draft.json");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(RetrievalManifestDraftV1::read_file(&link).is_err());
        }
    }

    fn packet_draft() -> SourcePacketDraftV1 {
        SourcePacketDraftV1 {
            schema_version: 1,
            citekey: "Smith2024".into(),
            content: " {\"source_url\":\"https://example.org/paper.pdf\",\"identity_status\":\"not_checked\",\"segments\":[{\"anchor\":\"page=2;segment=3\",\"text\":\"原文\"}]}\r\n".into(),
        }
    }

    #[test]
    fn source_packets_preserve_versions_and_bind_later_notes_after_restart() {
        let fixture = fixture();
        let mut packet = packet_draft();
        let mut saved = Vec::new();
        for revision in 1..=2 {
            let mut candidate = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
            candidate.binding.base_revision = revision;
            candidate.captured_at_unix = 100 + revision * 10;
            let capture = intake(&fixture, candidate);
            let plan = fixture
                .service
                .preview_capture_consolidation_with_drafts(
                    &fixture.project_id,
                    &capture.capture_id,
                    110 + revision * 10,
                    CaptureConsolidationDrafts {
                        source_packet: Some(&packet),
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(plan.source_packet_content(), Some(packet.content.as_str()));
            assert!(!fixture.project_root.join(packet.relative_path()).exists());
            assert!(!format!("{plan:?}").contains(&packet.content));
            fixture
                .service
                .apply_capture_consolidation(
                    &plan,
                    &ApprovedCaptureConsolidation::new(
                        plan.preview.plan_digest.clone(),
                        true,
                        true,
                    ),
                )
                .unwrap();
            saved.push((packet.relative_path(), packet.content.clone()));
            for (path, content) in &saved {
                assert_eq!(
                    fs::read(fixture.project_root.join(path)).unwrap(),
                    content.as_bytes()
                );
            }
            let (receipt, _) =
                read_consolidation_receipt(&fixture.project_root, &capture.capture_id)
                    .unwrap()
                    .unwrap();
            let artifact = receipt
                .artifacts
                .iter()
                .find(|a| a.artifact == ConsolidationArtifact::SourcePacket)
                .unwrap();
            assert_eq!(artifact.relative_path, packet.relative_path());
            assert_eq!(artifact.digest, sha256_bytes(packet.content.as_bytes()));
            let mut forged = receipt;
            forged
                .artifacts
                .iter_mut()
                .find(|a| a.artifact == ConsolidationArtifact::SourcePacket)
                .unwrap()
                .digest = "0".repeat(64);
            forged.acknowledgement = acknowledgement(&forged).unwrap();
            assert_eq!(forged.validate(), Err(ProjectError::InvalidProjectDocument));
            packet.content.push('\n');
        }
        assert_ne!(saved[0].0, saved[1].0);
        let restarted = ProjectStateService::new(
            resolve_config_root(None, &fixture.base.join("home")).unwrap(),
        );
        assert_eq!(
            restarted.snapshot().unwrap().projects[0].semantic_revision,
            3
        );
        let mut candidate = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        candidate.binding.base_revision = 3;
        candidate.captured_at_unix = 140;
        let capture = intake(&fixture, candidate);
        packet.content.clone_from(&saved[1].1);
        assert_eq!(
            restarted
                .preview_capture_consolidation_with_drafts(
                    &fixture.project_id,
                    &capture.capture_id,
                    150,
                    CaptureConsolidationDrafts {
                        source_packet: Some(&packet),
                        ..Default::default()
                    },
                )
                .unwrap_err(),
            ProjectError::ConsolidationConflict
        );
        let note = PaperNoteDraftV1 {
            schema_version: 1,
            citekey: packet.citekey.clone(),
            previous_sha256: None,
            sources: vec![crate::StageSummarySourceV1 {
                relative_path: packet.relative_path(),
                sha256: sha256_bytes(packet.content.as_bytes()),
            }],
            markdown: "CLM-001 remains partial; source anchor page=2;segment=3.".into(),
        };
        let plan = restarted
            .preview_capture_consolidation_with_paper_note(
                &fixture.project_id,
                &capture.capture_id,
                150,
                None,
                None,
                Some(&note),
            )
            .unwrap();
        restarted
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview.plan_digest.clone(), true, true),
            )
            .unwrap();
        assert!(
            fs::read_to_string(fixture.project_root.join(note.relative_path()))
                .unwrap()
                .contains(&packet.relative_path())
        );
    }

    #[test]
    fn source_packet_refusals_preserve_project_bytes() {
        for mutation in [
            "destination",
            "draft",
            "academic",
            "filesystem",
            "semantic",
            "symlink",
        ] {
            let fixture = fixture();
            let mut packet = packet_draft();
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            let preview = |packet: &SourcePacketDraftV1| {
                fixture.service.preview_capture_consolidation_with_drafts(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    CaptureConsolidationDrafts {
                        source_packet: Some(packet),
                        ..Default::default()
                    },
                )
            };
            let mut plan = preview(&packet).unwrap();
            let digest = plan.preview.plan_digest.clone();
            match mutation {
                "destination" => {
                    let path = fixture.project_root.join(packet.relative_path());
                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    fs::write(path, &packet.content).unwrap();
                    assert!(preview(&packet).is_err());
                }
                "draft" => {
                    packet.content.push('\n');
                    plan = preview(&packet).unwrap();
                }
                "semantic" => {
                    fs::write(fixture.project_root.join(RESEARCH_STATE_PATH), "Human edit").unwrap()
                }
                "symlink" => {
                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(
                            &fixture.base,
                            fixture.project_root.join("sources"),
                        )
                        .unwrap();
                        assert!(preview(&packet).is_err());
                    }
                    #[cfg(not(unix))]
                    {
                        continue;
                    }
                }
                _ => {}
            }
            let mut paths: Vec<_> = plan
                .artifacts
                .iter()
                .map(|a| a.relative_path.clone())
                .collect();
            paths.extend([
                PROJECT_MANIFEST_PATH.into(),
                plan.preview.receipt_entry.clone(),
            ]);
            let before: Vec<_> = paths
                .iter()
                .map(|p| fs::read(fixture.project_root.join(p)).ok())
                .collect();
            let library = fixture.service.snapshot().unwrap();
            assert!(
                fixture
                    .service
                    .apply_capture_consolidation(
                        &plan,
                        &ApprovedCaptureConsolidation::new(
                            digest,
                            mutation != "filesystem",
                            mutation != "academic"
                        ),
                    )
                    .is_err(),
                "{mutation}"
            );
            assert_eq!(
                paths
                    .iter()
                    .map(|p| fs::read(fixture.project_root.join(p)).ok())
                    .collect::<Vec<_>>(),
                before,
                "{mutation}"
            );
            assert_eq!(fixture.service.snapshot().unwrap(), library);
        }
    }

    #[test]
    fn source_packet_drafts_are_strict_bounded_and_path_safe() {
        let fixture = fixture();
        let valid = packet_draft();
        let path = fixture.base.join("packet-draft.json");
        fs::write(&path, serde_json::to_vec(&valid).unwrap()).unwrap();
        assert!(SourcePacketDraftV1::read_file(&path).is_ok());
        for body in [
            "{}",
            "[]",
            "null",
            "\"text\"",
            "{\"x\":1,\"x\":2}",
            "[{\"x\":1,\"x\":2}]",
            "{} {}",
            "{\u{0}}",
        ] {
            let mut bad = valid.clone();
            bad.content = body.into();
            assert!(bad.validate().is_err(), "{body}");
        }
        for key in [
            "",
            "../escape",
            "a/b",
            "a\\b",
            "CON",
            "COM1",
            "A:B",
            "a.b",
            "a\n",
        ] {
            let mut bad = valid.clone();
            bad.citekey = key.into();
            assert!(bad.validate().is_err(), "{key}");
            assert!(!valid_packet_path(&bad.relative_path()));
        }
        let mut bad = valid.clone();
        bad.content = " ".repeat(4 * 1024 * 1024 + 1);
        assert_eq!(bad.validate(), Err(ProjectError::DocumentTooLarge));
        for body in [
            "{\"schemaVersion\":1,\"schemaVersion\":1}",
            "{\"schemaVersion\":1,\"citekey\":\"Smith2024\",\"content\":\"[1]\",\"unknown\":true}",
        ] {
            fs::write(&path, body).unwrap();
            assert!(SourcePacketDraftV1::read_file(&path).is_err());
        }
        fs::write(&path, " ".repeat(4 * 1024 * 1024 + 1)).unwrap();
        assert_eq!(
            SourcePacketDraftV1::read_file(&path).err(),
            Some(ProjectError::DocumentTooLarge)
        );
        assert!(SourcePacketDraftV1::read_file(std::path::Path::new("relative.json")).is_err());
        #[cfg(unix)]
        {
            let link = fixture.base.join("linked-packet.json");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(SourcePacketDraftV1::read_file(&link).is_err());
        }
    }

    fn note_draft(fixture: &Fixture) -> PaperNoteDraftV1 {
        PaperNoteDraftV1 {
            schema_version: 1,
            citekey: "Smith2024".into(),
            previous_sha256: None,
            sources: summary_draft(fixture).sources,
            markdown:
                "# @Smith2024\n\nCLM-001: source R1; abstract only, denominator unresolved.\n"
                    .into(),
        }
    }

    #[test]
    fn paper_note_create_append_and_restart_preserve_exact_prior_bytes() {
        let fixture = fixture();
        let mut note = note_draft(&fixture);
        let mut previous = String::new();
        for revision in 1..=2 {
            let mut candidate = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
            candidate.binding.base_revision = revision;
            candidate.captured_at_unix = 100 + revision * 10;
            let capture = intake(&fixture, candidate);
            let plan = fixture
                .service
                .preview_capture_consolidation_with_paper_note(
                    &fixture.project_id,
                    &capture.capture_id,
                    110 + revision * 10,
                    None,
                    None,
                    Some(&note),
                )
                .unwrap();
            let content = plan.paper_note_content().unwrap().to_string();
            assert!(content.starts_with(&previous));
            assert!(content.contains(&note.markdown));
            assert!(content.contains(&note.sources[0].sha256));
            let debug = format!("{plan:?}");
            assert!(!debug.contains(&note.markdown));
            assert!(!debug.contains(&fixture.project_root.to_string_lossy().to_string()));
            fixture
                .service
                .apply_capture_consolidation(
                    &plan,
                    &ApprovedCaptureConsolidation::new(
                        plan.preview.plan_digest.clone(),
                        true,
                        true,
                    ),
                )
                .unwrap();
            assert_eq!(
                fs::read_to_string(fixture.project_root.join(note.relative_path())).unwrap(),
                content
            );
            let restarted = ProjectStateService::new(
                resolve_config_root(None, &fixture.base.join("home")).unwrap(),
            );
            assert_eq!(
                restarted.snapshot().unwrap().projects[0].semantic_revision,
                revision + 1
            );
            let (receipt, _) =
                read_consolidation_receipt(&fixture.project_root, &capture.capture_id)
                    .unwrap()
                    .unwrap();
            let artifact = receipt
                .artifacts
                .iter()
                .find(|a| a.artifact == ConsolidationArtifact::PaperNote)
                .unwrap();
            assert_eq!(artifact.digest, sha256_bytes(content.as_bytes()));
            assert_eq!(artifact.relative_path, note.relative_path());
            let mut forged = receipt;
            forged
                .artifacts
                .iter_mut()
                .find(|a| a.artifact == ConsolidationArtifact::PaperNote)
                .unwrap()
                .relative_path = "context/research_state.md".into();
            forged.acknowledgement = acknowledgement(&forged).unwrap();
            assert_eq!(forged.validate(), Err(ProjectError::InvalidProjectDocument));
            note.previous_sha256 = Some(sha256_bytes(content.as_bytes()));
            note.markdown =
                "## Correction\nCLM-001 retains its source anchor; still no causal conclusion."
                    .into();
            previous = content;
        }
    }

    #[test]
    fn paper_note_rejects_changed_inputs_or_missing_approval_without_writes() {
        for mutation in [
            "source",
            "missing-source",
            "note",
            "draft",
            "academic",
            "filesystem",
            "digest",
            "library",
            "semantic",
        ] {
            let fixture = fixture();
            let mut note = note_draft(&fixture);
            fs::create_dir(fixture.project_root.join("notes")).unwrap();
            fs::write(
                fixture.project_root.join(note.relative_path()),
                "Human note without final newline",
            )
            .unwrap();
            note.previous_sha256 = Some(sha256_bytes(b"Human note without final newline"));
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            let mut plan = fixture
                .service
                .preview_capture_consolidation_with_paper_note(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    None,
                    None,
                    Some(&note),
                )
                .unwrap();
            let digest = plan.preview.plan_digest.clone();
            match mutation {
                "source" => fs::write(
                    fixture.project_root.join("sources/current.md"),
                    "Changed source",
                )
                .unwrap(),
                "missing-source" => {
                    fs::remove_file(fixture.project_root.join("sources/current.md")).unwrap()
                }
                "note" => fs::write(
                    fixture.project_root.join(note.relative_path()),
                    "New human text",
                )
                .unwrap(),
                "semantic" => fs::write(
                    fixture.project_root.join(RESEARCH_STATE_PATH),
                    "Changed registered state",
                )
                .unwrap(),
                "draft" => {
                    note.markdown.push_str("Changed reviewed addition.");
                    plan = fixture
                        .service
                        .preview_capture_consolidation_with_paper_note(
                            &fixture.project_id,
                            &capture.capture_id,
                            120,
                            None,
                            None,
                            Some(&note),
                        )
                        .unwrap();
                }
                "library" => {
                    let archive = fixture
                        .service
                        .preview_archive(&fixture.project_id)
                        .unwrap();
                    fixture
                        .service
                        .apply(
                            &archive,
                            &ApprovedProjectMutation::new(
                                archive.preview().plan_digest.clone(),
                                true,
                            ),
                            121,
                        )
                        .unwrap();
                    assert_ne!(
                        fixture.service.snapshot().unwrap().revision,
                        plan.preview.expected_library_revision
                    );
                }
                _ => {}
            }
            let paths = [
                RESEARCH_STATE_PATH,
                DECISION_LOG_PATH,
                PROJECT_MANIFEST_PATH,
                "notes/Smith2024.md",
                "sources/current.md",
            ];
            let before = paths.map(|path| fs::read(fixture.project_root.join(path)).ok());
            let library = fixture.service.snapshot().unwrap();
            let result = fixture.service.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(
                    if mutation == "digest" {
                        "0".repeat(64)
                    } else {
                        digest
                    },
                    mutation != "filesystem",
                    mutation != "academic",
                ),
            );
            assert!(result.is_err(), "{mutation}");
            assert_eq!(
                paths.map(|path| fs::read(fixture.project_root.join(path)).ok()),
                before,
                "{mutation}"
            );
            assert_eq!(fixture.service.snapshot().unwrap(), library);
            assert!(
                !fixture
                    .project_root
                    .join(&plan.preview.receipt_entry)
                    .exists()
            );
        }
    }

    #[test]
    fn paper_note_draft_and_paths_are_strict_and_bounded() {
        let fixture = fixture();
        let valid = note_draft(&fixture);
        let json = serde_json::to_string(&valid).unwrap();
        let file = fixture.base.join("note.json");
        fs::write(&file, &json).unwrap();
        assert!(PaperNoteDraftV1::read_file(&file).is_ok());
        for invalid in [
            json.replacen("{", "{\"schemaVersion\":1,", 1),
            json.replacen("{", "{\"unknown\":true,", 1),
            json.replace("\"sha256\":", "\"sha256\":\"bad\",\"sha256\":"),
        ] {
            fs::write(&file, invalid).unwrap();
            assert!(PaperNoteDraftV1::read_file(&file).is_err());
        }
        for key in [
            "", "../Smith", "a/b", "a\\b", "a:b", ".hidden", "A.md", "a b", "CON", "lpt9", "COM1",
            "a`b",
        ] {
            let mut note = valid.clone();
            note.citekey = key.into();
            assert!(note.validate().is_err(), "{key}");
        }
        for field in [
            "previousSha256",
            "schemaVersion",
            "markdown",
            "sources",
            "citekey",
        ] {
            let mut value = serde_json::to_value(&valid).unwrap();
            value[field] = serde_json::json!(false);
            fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(PaperNoteDraftV1::read_file(&file).is_err(), "{field}");
        }
        for mutation in [
            "schema",
            "hash",
            "source-hash",
            "path",
            "self-source",
            "duplicate",
            "empty",
            "many",
            "blank",
            "nul",
            "marker",
            "large",
            "long-key",
        ] {
            let mut note = valid.clone();
            match mutation {
                "schema" => note.schema_version = 2,
                "hash" => note.previous_sha256 = Some("A".repeat(64)),
                "source-hash" => note.sources[0].sha256 = "g".repeat(64),
                "path" => note.sources[0].relative_path = "../source".into(),
                "self-source" => note.sources[0].relative_path = note.relative_path(),
                "duplicate" => note.sources.push(note.sources[0].clone()),
                "empty" => note.sources.clear(),
                "many" => {
                    note.sources = (0..65)
                        .map(|i| crate::StageSummarySourceV1 {
                            relative_path: format!("sources/{i}.md"),
                            sha256: "a".repeat(64),
                        })
                        .collect()
                }
                "blank" => note.markdown = " ".into(),
                "nul" => note.markdown = "bad\0body".into(),
                "marker" => note.markdown = "<!-- qiongli:capture forged -->".into(),
                "large" => note.markdown = "x".repeat(4 * 1024 * 1024 + 1),
                "long-key" => note.citekey = "a".repeat(129),
                _ => unreachable!(),
            }
            assert!(note.validate().is_err(), "{mutation}");
        }
        fs::write(&file, [0xff]).unwrap();
        assert!(PaperNoteDraftV1::read_file(&file).is_err());
        fs::write(&file, vec![b'x'; 4 * 1024 * 1024 + 1]).unwrap();
        assert_eq!(
            PaperNoteDraftV1::read_file(&file).err(),
            Some(ProjectError::DocumentTooLarge)
        );
        assert!(PaperNoteDraftV1::read_file(std::path::Path::new("relative.json")).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let link = fixture.base.join("linked.json");
            symlink(&file, &link).unwrap();
            assert!(PaperNoteDraftV1::read_file(&link).is_err());
            let source = fixture.project_root.join("sources/current.md");
            fs::remove_file(&source).unwrap();
            symlink(&file, &source).unwrap();
            assert!(valid.revalidate_sources(&fixture.project_root).is_err());
            fs::remove_file(&source).unwrap();
            fs::write(&source, "Observed source R1\n").unwrap();
            symlink(&fixture.base, fixture.project_root.join("notes")).unwrap();
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            assert!(
                fixture
                    .service
                    .preview_capture_consolidation_with_paper_note(
                        &fixture.project_id,
                        &capture.capture_id,
                        120,
                        None,
                        None,
                        Some(&valid)
                    )
                    .is_err()
            );
        }
    }

    fn summary_draft(fixture: &Fixture) -> StageSummaryDraftV1 {
        fs::create_dir_all(fixture.project_root.join("sources")).unwrap();
        fs::write(
            fixture.project_root.join("sources/current.md"),
            "Observed source R1\n",
        )
        .unwrap();
        StageSummaryDraftV1 {
            schema_version: 1,
            summary_id: "STG-B-001".to_string(),
            status: crate::StageSummaryStatus::Partial,
            previous_summary: None,
            sources: vec![crate::StageSummarySourceV1 {
                relative_path: "sources/current.md".to_string(),
                sha256: sha256_bytes(b"Observed source R1\n"),
            }],
            markdown: "# Stage Summary\n\nQ-C1 and DEC-001 remain tentative. The sample denominator is unresolved.\n".to_string(),
        }
    }

    #[test]
    fn stage_summary_versions_preserve_history_and_resume_with_source_changes() {
        let fixture = fixture();
        let mut summary = summary_draft(&fixture);
        let capture = intake(
            &fixture,
            draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
        );
        let plan = fixture
            .service
            .preview_capture_consolidation_with_summary(
                &fixture.project_id,
                &capture.capture_id,
                120,
                Some(HANDOFF),
                Some(&summary),
            )
            .unwrap();
        let first_summary = plan.stage_summary_content().unwrap().to_string();
        let first_handoff = plan.stage_handoff_content().unwrap().to_string();
        assert_eq!(plan.preview.artifact_deltas.len(), 4);
        assert!(
            plan.summary_research_state_content()
                .unwrap()
                .contains("| Summary ID | Stage | Date | Document | Previous summary | Status |")
        );
        assert!(first_handoff.contains("[STG-B-001](stage_summaries/STG-B-001.md)"));
        fixture
            .service
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview.plan_digest.clone(), true, true),
            )
            .unwrap();
        let first_state =
            fs::read_to_string(fixture.project_root.join(RESEARCH_STATE_PATH)).unwrap();
        let restarted = ProjectStateService::new(
            resolve_config_root(None, &fixture.base.join("home")).unwrap(),
        );
        assert_eq!(
            restarted.snapshot().unwrap().projects[0].semantic_revision,
            2
        );
        assert_eq!(
            fs::read_to_string(fixture.project_root.join(summary.relative_path())).unwrap(),
            first_summary
        );
        let (receipt, _) = read_consolidation_receipt(&fixture.project_root, &capture.capture_id)
            .unwrap()
            .unwrap();
        assert_eq!(receipt.artifacts.len(), 4);
        for artifact in &receipt.artifacts {
            assert_eq!(
                sha256_bytes(
                    &fs::read(fixture.project_root.join(&artifact.relative_path)).unwrap()
                ),
                artifact.digest
            );
        }
        let mut forged = receipt;
        forged
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.artifact == ConsolidationArtifact::StageSummary)
            .unwrap()
            .relative_path = "context/stage_summaries/../outside.md".to_string();
        forged.acknowledgement = acknowledgement(&forged).unwrap();
        assert_eq!(forged.validate(), Err(ProjectError::InvalidProjectDocument));

        let predecessor = crate::StageSummarySourceV1 {
            relative_path: summary.relative_path(),
            sha256: sha256_bytes(first_summary.as_bytes()),
        };
        summary.summary_id = "STG-B-002".to_string();
        summary.previous_summary = Some(predecessor.clone());
        summary.status = crate::StageSummaryStatus::Correction;
        summary.markdown = "# Source-expanded summary\n\nQ-C1/DEC-001 require model-sensitive qualification; the denominator remains unresolved.\n".to_string();
        fs::write(
            fixture.project_root.join("sources/current.md"),
            "Observed source R2\n",
        )
        .unwrap();
        summary.sources[0].sha256 = sha256_bytes(b"Observed source R2\n");
        assert_eq!(
            restarted.snapshot().unwrap().projects[0].semantic_revision,
            2
        );
        let mut next = draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired);
        next.binding.base_revision = 2;
        next.captured_at_unix = 130;
        let next = next.into_capture().unwrap();
        let intake = restarted.preview_capture(next.clone()).unwrap();
        restarted
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                135,
            )
            .unwrap();
        let plan = restarted
            .preview_capture_consolidation_with_summary(
                &fixture.project_id,
                &next.capture_id,
                140,
                None,
                Some(&summary),
            )
            .unwrap();
        assert!(
            plan.stage_handoff_content()
                .unwrap()
                .starts_with(&first_handoff)
        );
        let second_row = plan
            .summary_research_state_content()
            .unwrap()
            .lines()
            .find(|line| line.starts_with("| STG-B-002 |"))
            .unwrap();
        assert!(second_row.contains("[STG-B-001](stage_summaries/STG-B-001.md)"));
        assert_eq!(
            plan.summary_research_state_content()
                .unwrap()
                .replacen(&format!("{second_row}\n"), "", 1)
                .get(..first_state.len()),
            Some(first_state.as_str())
        );
        let predecessor_path = fixture.project_root.join(&predecessor.relative_path);
        fs::write(&predecessor_path, "Externally changed prior summary").unwrap();
        assert_eq!(
            restarted.apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview.plan_digest.clone(), true, true)
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert!(
            restarted
                .preview_capture_consolidation_with_summary(
                    &fixture.project_id,
                    &next.capture_id,
                    140,
                    None,
                    Some(&summary)
                )
                .is_err()
        );
        assert_eq!(
            fs::read_to_string(fixture.project_root.join(RESEARCH_STATE_PATH)).unwrap(),
            first_state
        );
        assert!(!fixture.project_root.join(summary.relative_path()).exists());
        // Restore only this deliberately modified isolated test input.
        fs::write(&predecessor_path, &first_summary).unwrap();
        let expected_summary = plan.stage_summary_content().unwrap().to_string();
        restarted
            .apply_capture_consolidation(
                &plan,
                &ApprovedCaptureConsolidation::new(plan.preview.plan_digest.clone(), true, true),
            )
            .unwrap();
        assert_eq!(fs::read_to_string(predecessor_path).unwrap(), first_summary);
        assert_eq!(
            fs::read_to_string(fixture.project_root.join(summary.relative_path())).unwrap(),
            expected_summary
        );
        assert_eq!(
            restarted.snapshot().unwrap().projects[0].semantic_revision,
            3
        );
        assert_eq!(
            restarted.snapshot().unwrap().projects[0].stage,
            ProjectStage::Literature
        );
    }

    #[test]
    fn stage_summary_rejects_drift_reuse_and_missing_approval_without_writes() {
        for mutation in [
            "source",
            "destination",
            "draft",
            "filesystem",
            "academic",
            "digest",
        ] {
            let fixture = fixture();
            let mut summary = summary_draft(&fixture);
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            let mut plan = fixture
                .service
                .preview_capture_consolidation_with_summary(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    None,
                    Some(&summary),
                )
                .unwrap();
            let original_digest = plan.preview.plan_digest.clone();
            match mutation {
                "source" => fs::write(
                    fixture.project_root.join("sources/current.md"),
                    "Changed source",
                )
                .unwrap(),
                "destination" => {
                    fs::create_dir_all(fixture.project_root.join("context/stage_summaries"))
                        .unwrap();
                    fs::write(
                        fixture.project_root.join(summary.relative_path()),
                        plan.stage_summary_content().unwrap(),
                    )
                    .unwrap();
                }
                "draft" => {
                    summary.markdown.push_str("Changed candidate");
                    plan = fixture
                        .service
                        .preview_capture_consolidation_with_summary(
                            &fixture.project_id,
                            &capture.capture_id,
                            120,
                            None,
                            Some(&summary),
                        )
                        .unwrap();
                }
                _ => {}
            }
            let paths = [
                RESEARCH_STATE_PATH,
                DECISION_LOG_PATH,
                STAGE_HANDOFF_PATH,
                PROJECT_MANIFEST_PATH,
                "sources/current.md",
                "context/stage_summaries/STG-B-001.md",
            ];
            let before = paths.map(|path| fs::read(fixture.project_root.join(path)).ok());
            let approval = ApprovedCaptureConsolidation::new(
                if mutation == "digest" {
                    "0".repeat(64)
                } else {
                    original_digest
                },
                mutation != "filesystem",
                mutation != "academic",
            );
            assert!(
                fixture
                    .service
                    .apply_capture_consolidation(&plan, &approval)
                    .is_err(),
                "{mutation}"
            );
            assert_eq!(
                paths.map(|path| fs::read(fixture.project_root.join(path)).ok()),
                before,
                "{mutation}"
            );
            assert_eq!(
                fixture.service.snapshot().unwrap().projects[0].semantic_revision,
                1
            );
            assert!(
                !fixture
                    .project_root
                    .join(&plan.preview.receipt_entry)
                    .exists()
            );
            if matches!(mutation, "source" | "destination") {
                assert!(
                    fixture
                        .service
                        .preview_capture_consolidation_with_summary(
                            &fixture.project_id,
                            &capture.capture_id,
                            120,
                            None,
                            Some(&summary)
                        )
                        .is_err()
                );
            }
        }
    }

    #[test]
    fn all_artifacts_nine_file_transaction_rolls_back_documents_and_history() {
        for existing_note in [false, true] {
            let fixture = fixture();
            let summary = summary_draft(&fixture);
            let mut note = note_draft(&fixture);
            let packet = packet_draft();
            let mut retrieval = crate::retrieval_manifest::test_draft();
            if existing_note {
                fs::create_dir(fixture.project_root.join("notes")).unwrap();
                let prior = b"Human note\r\nCLM-001 and its old source anchor.";
                fs::write(fixture.project_root.join(note.relative_path()), prior).unwrap();
                note.previous_sha256 = Some(sha256_bytes(prior));
                let csv = retrieval.render(None).unwrap();
                fs::write(fixture.project_root.join(RETRIEVAL_MANIFEST_PATH), &csv).unwrap();
                retrieval.previous_sha256 = Some(sha256_bytes(csv.as_bytes()));
            }
            let capture = intake(
                &fixture,
                draft(fixture.project_id.clone(), CapturePolicy::ReviewRequired),
            );
            let plan = fixture
                .service
                .preview_capture_consolidation_with_drafts(
                    &fixture.project_id,
                    &capture.capture_id,
                    120,
                    CaptureConsolidationDrafts {
                        stage_handoff: None,
                        stage_summary: Some(&summary),
                        paper_note: Some(&note),
                        source_packet: Some(&packet),
                        retrieval_manifest: Some(&retrieval),
                    },
                )
                .unwrap();
            let mut receipt = build_receipt(&plan).unwrap();
            receipt.acknowledgement = acknowledgement(&receipt).unwrap();
            let mut updates: Vec<_> = plan
                .artifacts
                .iter()
                .map(|artifact| ProjectFileUpdate {
                    relative_path: artifact.relative_path.clone(),
                    expected_digest: artifact.previous_digest.clone(),
                    next_bytes: artifact.next_bytes.clone(),
                })
                .collect();
            updates.push(ProjectFileUpdate {
                relative_path: plan.preview.receipt_entry.clone(),
                expected_digest: None,
                next_bytes: encode_project_document(&receipt).unwrap(),
            });
            updates.push(ProjectFileUpdate {
                relative_path: PROJECT_MANIFEST_PATH.to_string(),
                expected_digest: Some(plan.observed_manifest_digest.clone()),
                next_bytes: encode_project_document(plan.next_manifest.as_ref().unwrap()).unwrap(),
            });
            assert_eq!(updates.len(), 9);
            let before: Vec<_> = updates
                .iter()
                .map(|update| fs::read(fixture.project_root.join(&update.relative_path)).ok())
                .collect();
            let transaction =
                ProjectFileTransaction::apply(&fixture.project_root, &updates).unwrap();
            assert!(fixture.project_root.join(summary.relative_path()).exists());
            transaction.rollback().unwrap();
            assert_eq!(
                updates
                    .iter()
                    .map(|update| fs::read(fixture.project_root.join(&update.relative_path)).ok())
                    .collect::<Vec<_>>(),
                before
            );
            assert!(
                !fixture
                    .project_root
                    .join(".qiongli/consolidation-transaction")
                    .exists()
            );
            assert_eq!(
                fixture.service.snapshot().unwrap().projects[0].semantic_revision,
                1
            );
        }
    }
}
