use std::fmt::{self, Debug, Formatter};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::ProjectError;
use crate::model::{
    MAX_SEMANTIC_REVISION, ProjectId, ProjectLifecycle, ProjectStage, valid_lower_hex,
};
use crate::service::ProjectStateService;
use crate::storage::{
    capture_history_relative_path, lock_capture_history, project_root_from_string,
    project_root_string, read_capture_document, read_manifest, validate_existing_project_root,
    write_capture_document,
};

pub const RESEARCH_CAPTURE_SCHEMA_VERSION: u32 = 1;
pub const RESEARCH_CAPTURE_DOCUMENT_KIND: &str = "qiongli-research-capture";
pub const PROJECT_BINDING_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_BINDING_DOCUMENT_KIND: &str = "qiongli-project-binding";
pub const CAPTURE_ID_PREFIX: &str = "cap_";
pub const CAPTURE_INTAKE_SCHEMA_VERSION: u32 = 1;

pub(crate) const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_TASK_BYTES: usize = 300;
const MAX_SUMMARY_BYTES: usize = 2_000;
const MAX_ITEM_TEXT_BYTES: usize = 1_000;
const MAX_LOCATOR_BYTES: usize = 500;
const MAX_ITEMS_PER_FIELD: usize = 16;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaptureId(String);

impl CaptureId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectError> {
        let value = value.into();
        if value.len() != CAPTURE_ID_PREFIX.len() + 64
            || !value.starts_with(CAPTURE_ID_PREFIX)
            || !valid_lower_hex(&value[CAPTURE_ID_PREFIX.len()..], 64)
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(Self(value))
    }

    fn from_digest(digest: &[u8]) -> Self {
        let mut value = String::with_capacity(CAPTURE_ID_PREFIX.len() + digest.len() * 2);
        value.push_str(CAPTURE_ID_PREFIX);
        for byte in digest {
            use std::fmt::Write as _;
            let _ = write!(value, "{byte:02x}");
        }
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), ProjectError> {
        Self::parse(self.0.clone()).map(|_| ())
    }
}

impl Debug for CaptureId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("CaptureId").field(&self.0).finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapturePolicy {
    ReviewRequired,
    HistoryOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindingV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub base_revision: u64,
    pub stage: ProjectStage,
    pub task: String,
    pub capture_policy: CapturePolicy,
}

impl ProjectBindingV1 {
    pub fn new(
        project_id: ProjectId,
        base_revision: u64,
        stage: ProjectStage,
        task: impl Into<String>,
        capture_policy: CapturePolicy,
    ) -> Result<Self, ProjectError> {
        let binding = Self {
            schema_version: PROJECT_BINDING_SCHEMA_VERSION,
            document_kind: PROJECT_BINDING_DOCUMENT_KIND.to_string(),
            project_id,
            base_revision,
            stage,
            task: task.into(),
            capture_policy,
        };
        binding.validate()?;
        Ok(binding)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != PROJECT_BINDING_SCHEMA_VERSION
            || self.document_kind != PROJECT_BINDING_DOCUMENT_KIND
            || self.base_revision == 0
            || self.base_revision > MAX_SEMANTIC_REVISION
            || !valid_text(&self.task, MAX_TASK_BYTES)
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        self.project_id
            .validate()
            .map_err(|_| ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureSource {
    Codex,
    ClaudeCode,
    ChatGpt,
    Cli,
    Manual,
    Repository,
    PortableFile,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureDelivery {
    Connected,
    RepositoryBacked,
    Portable,
    Manual,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureArea {
    ResearchQuestion,
    Thesis,
    Literature,
    Method,
    Evidence,
    Analysis,
    Manuscript,
    Scope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticChangeV1 {
    pub area: CaptureArea,
    pub summary: String,
}

impl SemanticChangeV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        valid_text(&self.summary, MAX_ITEM_TEXT_BYTES)
            .then_some(())
            .ok_or(ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionRelation {
    Candidate,
    Refinement,
    Challenge,
    Supersession,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionCandidateV1 {
    pub relation: DecisionRelation,
    pub statement: String,
    pub rationale: String,
    pub target: Option<String>,
}

impl DecisionCandidateV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_text(&self.statement, MAX_ITEM_TEXT_BYTES)
            || !valid_text(&self.rationale, MAX_ITEM_TEXT_BYTES)
            || self
                .target
                .as_deref()
                .is_some_and(|value| !valid_text(value, MAX_ITEM_TEXT_BYTES))
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        if self.relation == DecisionRelation::Candidate && self.target.is_some() {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        if self.relation != DecisionRelation::Candidate && self.target.is_none() {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceLocatorKind {
    Doi,
    CitationKey,
    HttpsUrl,
    ArtifactAnchor,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReferenceV1 {
    pub locator_kind: EvidenceLocatorKind,
    pub locator: String,
    pub relevance: String,
    pub limitation: Option<String>,
}

impl EvidenceReferenceV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_text(&self.locator, MAX_LOCATOR_BYTES)
            || !valid_text(&self.relevance, MAX_ITEM_TEXT_BYTES)
            || self
                .limitation
                .as_deref()
                .is_some_and(|value| !valid_text(value, MAX_ITEM_TEXT_BYTES))
            || !valid_locator(self.locator_kind, &self.locator)
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContradictionV1 {
    pub statement: String,
    pub conflicts_with: String,
    pub consequence: String,
}

impl ContradictionV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        [
            self.statement.as_str(),
            self.conflicts_with.as_str(),
            self.consequence.as_str(),
        ]
        .into_iter()
        .all(|value| valid_text(value, MAX_ITEM_TEXT_BYTES))
        .then_some(())
        .ok_or(ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchCaptureDraftV1 {
    pub binding: ProjectBindingV1,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub captured_at_unix: u64,
    pub summary: String,
    pub changes: Vec<SemanticChangeV1>,
    pub decisions: Vec<DecisionCandidateV1>,
    pub evidence: Vec<EvidenceReferenceV1>,
    pub contradictions: Vec<ContradictionV1>,
    pub next_actions: Vec<String>,
}

impl ResearchCaptureDraftV1 {
    pub fn validate(&self) -> Result<(), ProjectError> {
        self.binding.validate()?;
        if self.captured_at_unix > MAX_SEMANTIC_REVISION
            || !valid_text(&self.summary, MAX_SUMMARY_BYTES)
            || !valid_collection(&self.changes, SemanticChangeV1::validate)
            || !valid_collection(&self.decisions, DecisionCandidateV1::validate)
            || !valid_collection(&self.evidence, EvidenceReferenceV1::validate)
            || !valid_collection(&self.contradictions, ContradictionV1::validate)
            || self.next_actions.len() > MAX_ITEMS_PER_FIELD
            || self
                .next_actions
                .iter()
                .any(|value| !valid_text(value, MAX_ITEM_TEXT_BYTES))
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidCaptureDocument)?;
        if bytes.len() > MAX_CAPTURE_BYTES {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(())
    }

    pub fn into_capture(self) -> Result<ResearchCaptureV1, ProjectError> {
        self.validate()?;
        let capture_id = capture_id(&self)?;
        let capture = ResearchCaptureV1 {
            schema_version: RESEARCH_CAPTURE_SCHEMA_VERSION,
            document_kind: RESEARCH_CAPTURE_DOCUMENT_KIND.to_string(),
            capture_id,
            binding: self.binding,
            source: self.source,
            delivery: self.delivery,
            captured_at_unix: self.captured_at_unix,
            summary: self.summary,
            changes: self.changes,
            decisions: self.decisions,
            evidence: self.evidence,
            contradictions: self.contradictions,
            next_actions: self.next_actions,
        };
        capture.validate()?;
        Ok(capture)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchCaptureV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub capture_id: CaptureId,
    pub binding: ProjectBindingV1,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub captured_at_unix: u64,
    pub summary: String,
    pub changes: Vec<SemanticChangeV1>,
    pub decisions: Vec<DecisionCandidateV1>,
    pub evidence: Vec<EvidenceReferenceV1>,
    pub contradictions: Vec<ContradictionV1>,
    pub next_actions: Vec<String>,
}

impl ResearchCaptureV1 {
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != RESEARCH_CAPTURE_SCHEMA_VERSION
            || self.document_kind != RESEARCH_CAPTURE_DOCUMENT_KIND
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        self.capture_id.validate()?;
        let draft = ResearchCaptureDraftV1 {
            binding: self.binding.clone(),
            source: self.source,
            delivery: self.delivery,
            captured_at_unix: self.captured_at_unix,
            summary: self.summary.clone(),
            changes: self.changes.clone(),
            decisions: self.decisions.clone(),
            evidence: self.evidence.clone(),
            contradictions: self.contradictions.clone(),
            next_actions: self.next_actions.clone(),
        };
        draft.validate()?;
        if capture_id(&draft)? != self.capture_id {
            return Err(ProjectError::CaptureIdentityConflict);
        }
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidCaptureDocument)?;
        (bytes.len() <= MAX_CAPTURE_BYTES)
            .then_some(())
            .ok_or(ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureDisposition {
    Duplicate,
    Refinement,
    Contradiction,
    Supersession,
    UnresolvedCandidate,
    UnsupportedGap,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureIntakeEffect {
    AppendPendingHistory,
    NoChange,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureIntakePreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub capture_id: CaptureId,
    pub project_id: ProjectId,
    pub disposition: CaptureDisposition,
    pub effect: CaptureIntakeEffect,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub change_count: usize,
    pub decision_count: usize,
    pub evidence_count: usize,
    pub contradiction_count: usize,
    pub next_action_count: usize,
    pub history_entry: String,
    pub approvals_required: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedCaptureIntake {
    preview: CaptureIntakePreviewV1,
    capture: ResearchCaptureV1,
    root: PathBuf,
    root_reference_digest: String,
    observed_manifest_digest: String,
    capture_document_digest: String,
}

impl VerifiedCaptureIntake {
    #[must_use]
    pub const fn preview(&self) -> &CaptureIntakePreviewV1 {
        &self.preview
    }
}

impl Debug for VerifiedCaptureIntake {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedCaptureIntake")
            .field("preview", &self.preview)
            .field("capture", &"<bounded-research-capture>")
            .field("root", &"<registered-project-root>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedCaptureIntake {
    expected_plan_digest: String,
    filesystem_write: bool,
}

impl ApprovedCaptureIntake {
    #[must_use]
    pub fn new(expected_plan_digest: impl Into<String>, filesystem_write: bool) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            filesystem_write,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureIntakeCommitV1 {
    pub schema_version: u32,
    pub capture_id: CaptureId,
    pub project_id: ProjectId,
    pub disposition: CaptureDisposition,
    pub base_revision: u64,
    pub accepted_at_unix: u64,
    pub history_entry: String,
    pub acknowledgement: String,
}

#[derive(Serialize)]
struct CaptureIntakePlanSemantics<'a> {
    schema_version: u32,
    capture_id: &'a CaptureId,
    project_id: &'a ProjectId,
    disposition: CaptureDisposition,
    effect: CaptureIntakeEffect,
    source: CaptureSource,
    delivery: CaptureDelivery,
    expected_library_revision: u64,
    expected_project_revision: u64,
    root_reference_digest: &'a str,
    observed_manifest_digest: &'a str,
    capture_document_digest: &'a str,
}

#[derive(Serialize)]
struct CaptureAcknowledgementSemantics<'a> {
    schema_version: u32,
    plan_digest: &'a str,
    capture_id: &'a CaptureId,
    project_id: &'a ProjectId,
    base_revision: u64,
    accepted_at_unix: u64,
    capture_document_digest: &'a str,
}

impl ProjectStateService {
    pub fn preview_capture(
        &self,
        capture: ResearchCaptureV1,
    ) -> Result<VerifiedCaptureIntake, ProjectError> {
        capture.validate()?;
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| entry.project_id == capture.binding.project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        let (manifest, observed_manifest_digest) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        validate_capture_binding(&capture, entry, &manifest)?;

        let duplicate = match read_capture_document(&root, &capture.capture_id)? {
            Some((existing, _)) if existing == capture => true,
            Some(_) => return Err(ProjectError::CaptureIdentityConflict),
            None => false,
        };
        let disposition = classify_capture(&capture, duplicate);
        let effect = if duplicate {
            CaptureIntakeEffect::NoChange
        } else {
            CaptureIntakeEffect::AppendPendingHistory
        };
        let root_reference_digest = sha256(project_root_string(&root)?.as_bytes());
        let capture_document_digest = canonical_digest(&capture)?;
        let semantics = CaptureIntakePlanSemantics {
            schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
            capture_id: &capture.capture_id,
            project_id: &capture.binding.project_id,
            disposition,
            effect,
            source: capture.source,
            delivery: capture.delivery,
            expected_library_revision: library.revision,
            expected_project_revision: manifest.semantic_revision,
            root_reference_digest: &root_reference_digest,
            observed_manifest_digest: &observed_manifest_digest,
            capture_document_digest: &capture_document_digest,
        };
        let preview = CaptureIntakePreviewV1 {
            schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
            plan_digest: canonical_digest(&semantics)?,
            capture_id: capture.capture_id.clone(),
            project_id: capture.binding.project_id.clone(),
            disposition,
            effect,
            source: capture.source,
            delivery: capture.delivery,
            expected_library_revision: library.revision,
            expected_project_revision: manifest.semantic_revision,
            change_count: capture.changes.len(),
            decision_count: capture.decisions.len(),
            evidence_count: capture.evidence.len(),
            contradiction_count: capture.contradictions.len(),
            next_action_count: capture.next_actions.len(),
            history_entry: capture_history_relative_path(&capture.capture_id),
            approvals_required: if duplicate {
                Vec::new()
            } else {
                vec!["filesystem-write".to_string()]
            },
        };
        Ok(VerifiedCaptureIntake {
            preview,
            capture,
            root,
            root_reference_digest,
            observed_manifest_digest,
            capture_document_digest,
        })
    }

    pub fn apply_capture(
        &self,
        plan: &VerifiedCaptureIntake,
        approval: &ApprovedCaptureIntake,
        now_unix: u64,
    ) -> Result<CaptureIntakeCommitV1, ProjectError> {
        validate_intake_plan(plan)?;
        if plan.preview.effect == CaptureIntakeEffect::NoChange {
            return Err(ProjectError::CaptureAlreadyApplied);
        }
        if !approval.filesystem_write {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview.plan_digest {
            return Err(ProjectError::PlanMismatch);
        }
        if now_unix > MAX_SEMANTIC_REVISION {
            return Err(ProjectError::InvalidCaptureDocument);
        }

        let library = self.store.lock(plan.preview.expected_library_revision)?;
        let entry = library
            .document
            .projects
            .iter()
            .find(|entry| entry.project_id == plan.preview.project_id)
            .ok_or(ProjectError::RevisionConflict)?;
        let root = project_root_from_string(&entry.root_path)?;
        if root != plan.root
            || sha256(project_root_string(&root)?.as_bytes()) != plan.root_reference_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        validate_existing_project_root(&root)?;
        let (manifest, manifest_digest) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest_digest != plan.observed_manifest_digest {
            return Err(ProjectError::RevisionConflict);
        }
        validate_capture_binding(&plan.capture, entry, &manifest)?;

        let history_lock = lock_capture_history(&root)?;
        if read_capture_document(&root, &plan.capture.capture_id)?.is_some() {
            return Err(ProjectError::CaptureAlreadyApplied);
        }
        let committed_digest = write_capture_document(&root, &plan.capture, &history_lock)?;
        if committed_digest != plan.capture_document_digest {
            return Err(ProjectError::RecoveryRequired);
        }

        let acknowledgement = CaptureAcknowledgementSemantics {
            schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
            plan_digest: &plan.preview.plan_digest,
            capture_id: &plan.capture.capture_id,
            project_id: &plan.capture.binding.project_id,
            base_revision: plan.capture.binding.base_revision,
            accepted_at_unix: now_unix,
            capture_document_digest: &plan.capture_document_digest,
        };
        Ok(CaptureIntakeCommitV1 {
            schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
            capture_id: plan.capture.capture_id.clone(),
            project_id: plan.capture.binding.project_id.clone(),
            disposition: plan.preview.disposition,
            base_revision: plan.capture.binding.base_revision,
            accepted_at_unix: now_unix,
            history_entry: plan.preview.history_entry.clone(),
            acknowledgement: format!("ack_{}", canonical_digest(&acknowledgement)?),
        })
    }

    pub fn read_capture(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<Option<ResearchCaptureV1>, ProjectError> {
        let root = self.resolve_project_root(project_id)?;
        read_capture_document(root.path(), capture_id)
            .map(|capture| capture.map(|(capture, _)| capture))
    }
}

fn validate_capture_binding(
    capture: &ResearchCaptureV1,
    entry: &crate::model::RegisteredProjectV1,
    manifest: &crate::ArticleProjectManifestV1,
) -> Result<(), ProjectError> {
    if entry.lifecycle != ProjectLifecycle::Active
        || manifest.lifecycle != ProjectLifecycle::Active
        || entry.project_id != manifest.project_id
        || entry.semantic_revision != manifest.semantic_revision
        || entry.semantic_digest != manifest.semantic_digest
        || capture.binding.project_id != manifest.project_id
        || capture.binding.base_revision != manifest.semantic_revision
        || capture.binding.stage != manifest.stage
    {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

fn classify_capture(capture: &ResearchCaptureV1, duplicate: bool) -> CaptureDisposition {
    if duplicate {
        return CaptureDisposition::Duplicate;
    }
    if capture.evidence.is_empty()
        && (!capture.changes.is_empty()
            || !capture.decisions.is_empty()
            || !capture.contradictions.is_empty())
    {
        return CaptureDisposition::UnsupportedGap;
    }
    if capture
        .decisions
        .iter()
        .any(|decision| decision.relation == DecisionRelation::Supersession)
    {
        return CaptureDisposition::Supersession;
    }
    if !capture.contradictions.is_empty()
        || capture
            .decisions
            .iter()
            .any(|decision| decision.relation == DecisionRelation::Challenge)
    {
        return CaptureDisposition::Contradiction;
    }
    if !capture.changes.is_empty()
        || !capture.evidence.is_empty()
        || capture
            .decisions
            .iter()
            .any(|decision| decision.relation == DecisionRelation::Refinement)
    {
        return CaptureDisposition::Refinement;
    }
    CaptureDisposition::UnresolvedCandidate
}

fn validate_intake_plan(plan: &VerifiedCaptureIntake) -> Result<(), ProjectError> {
    plan.capture.validate()?;
    let duplicate = plan.preview.effect == CaptureIntakeEffect::NoChange;
    let expected_effect = if duplicate {
        CaptureIntakeEffect::NoChange
    } else {
        CaptureIntakeEffect::AppendPendingHistory
    };
    if plan.preview.schema_version != CAPTURE_INTAKE_SCHEMA_VERSION
        || plan.preview.capture_id != plan.capture.capture_id
        || plan.preview.project_id != plan.capture.binding.project_id
        || plan.preview.disposition != classify_capture(&plan.capture, duplicate)
        || plan.preview.effect != expected_effect
        || plan.preview.source != plan.capture.source
        || plan.preview.delivery != plan.capture.delivery
        || plan.preview.expected_project_revision != plan.capture.binding.base_revision
        || plan.preview.change_count != plan.capture.changes.len()
        || plan.preview.decision_count != plan.capture.decisions.len()
        || plan.preview.evidence_count != plan.capture.evidence.len()
        || plan.preview.contradiction_count != plan.capture.contradictions.len()
        || plan.preview.next_action_count != plan.capture.next_actions.len()
        || plan.preview.history_entry != capture_history_relative_path(&plan.capture.capture_id)
        || plan.preview.approvals_required
            != if duplicate {
                Vec::<String>::new()
            } else {
                vec!["filesystem-write".to_string()]
            }
        || canonical_digest(&plan.capture)? != plan.capture_document_digest
    {
        return Err(ProjectError::PlanMismatch);
    }
    let semantics = CaptureIntakePlanSemantics {
        schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
        capture_id: &plan.capture.capture_id,
        project_id: &plan.capture.binding.project_id,
        disposition: plan.preview.disposition,
        effect: plan.preview.effect,
        source: plan.capture.source,
        delivery: plan.capture.delivery,
        expected_library_revision: plan.preview.expected_library_revision,
        expected_project_revision: plan.preview.expected_project_revision,
        root_reference_digest: &plan.root_reference_digest,
        observed_manifest_digest: &plan.observed_manifest_digest,
        capture_document_digest: &plan.capture_document_digest,
    };
    if canonical_digest(&semantics)? != plan.preview.plan_digest {
        return Err(ProjectError::PlanMismatch);
    }
    Ok(())
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidCaptureDocument)?;
    Ok(sha256(&bytes))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn capture_id(draft: &ResearchCaptureDraftV1) -> Result<CaptureId, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(draft)
        .map_err(|_| ProjectError::InvalidCaptureDocument)?;
    Ok(CaptureId::from_digest(&Sha256::digest(bytes)))
}

fn valid_collection<T>(values: &[T], validate: impl Fn(&T) -> Result<(), ProjectError>) -> bool {
    values.len() <= MAX_ITEMS_PER_FIELD && values.iter().all(|value| validate(value).is_ok())
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_locator(kind: EvidenceLocatorKind, value: &str) -> bool {
    match kind {
        EvidenceLocatorKind::Doi => {
            value.starts_with("10.")
                && value.contains('/')
                && value.bytes().all(|byte| !byte.is_ascii_whitespace())
        }
        EvidenceLocatorKind::CitationKey => value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        }),
        EvidenceLocatorKind::HttpsUrl => Url::parse(value).is_ok_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        }),
        EvidenceLocatorKind::ArtifactAnchor => valid_artifact_anchor(value),
    }
}

fn valid_artifact_anchor(value: &str) -> bool {
    let path = value.split_once('#').map_or(value, |(path, _)| path);
    if path.is_empty()
        || path.starts_with(['/', '\\', '~'])
        || path.contains(['\\', ':'])
        || path.ends_with('/')
    {
        return false;
    }
    Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(value) if !value.is_empty()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::resolve_config_root;

    use crate::{
        ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions, ProjectStateService,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn valid_draft() -> ResearchCaptureDraftV1 {
        ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap(),
                3,
                ProjectStage::Literature,
                "Reconcile the methods literature",
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix: 1_721_337_600,
            summary: "The measurement literature separates construct validity from reliability."
                .to_string(),
            changes: vec![SemanticChangeV1 {
                area: CaptureArea::Literature,
                summary: "Split the measurement cluster into validity and reliability streams."
                    .to_string(),
            }],
            decisions: vec![DecisionCandidateV1 {
                relation: DecisionRelation::Candidate,
                statement: "Use construct validity as the organizing distinction.".to_string(),
                rationale: "It explains the disagreement between the two source clusters."
                    .to_string(),
                target: None,
            }],
            evidence: vec![EvidenceReferenceV1 {
                locator_kind: EvidenceLocatorKind::Doi,
                locator: "10.1000/example".to_string(),
                relevance: "Defines the construct-validity distinction.".to_string(),
                limitation: Some("Conceptual rather than empirical evidence.".to_string()),
            }],
            contradictions: Vec::new(),
            next_actions: vec![
                "Check whether the distinction survives the empirical papers.".to_string(),
            ],
        }
    }

    fn project_fixture() -> (PathBuf, ProjectStateService, ProjectId) {
        let root = std::env::temp_dir().join(format!(
            "qiongli-capture-service-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let home = root.join("home");
        let projects = root.join("projects");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&projects).unwrap();
        let service = ProjectStateService::new(resolve_config_root(None, &home).unwrap());
        let project_id = ProjectId::parse("prj_abcdef0123456789abcdef0123456789").unwrap();
        let create = service
            .preview_create(
                projects.join("capture-project"),
                ProjectRegistrationOptions::new("Capture project", ProjectKind::Article)
                    .with_project_id(project_id.clone())
                    .with_stage(ProjectStage::Literature),
                100,
            )
            .unwrap();
        service
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                100,
            )
            .unwrap();
        (root, service, project_id)
    }

    fn project_capture(project_id: ProjectId, base_revision: u64) -> ResearchCaptureV1 {
        let mut draft = valid_draft();
        draft.binding = ProjectBindingV1::new(
            project_id,
            base_revision,
            ProjectStage::Literature,
            "Reconcile the methods literature",
            CapturePolicy::ReviewRequired,
        )
        .unwrap();
        draft.into_capture().unwrap()
    }

    #[test]
    fn content_addressed_capture_round_trips_without_session_or_path_fields() {
        let capture = valid_draft().into_capture().unwrap();
        let bytes = serde_json_canonicalizer::to_vec(&capture).unwrap();
        let decoded: ResearchCaptureV1 = serde_json::from_slice(&bytes).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, capture);
        assert_eq!(
            capture.capture_id.as_str().len(),
            CAPTURE_ID_PREFIX.len() + 64
        );

        let text = String::from_utf8(bytes).unwrap();
        for forbidden in ["session", "transcript", "root_path", "paper_body"] {
            assert!(!text.contains(forbidden));
        }
    }

    #[test]
    fn semantic_change_changes_capture_identity() {
        let first = valid_draft().into_capture().unwrap();
        let mut changed = valid_draft();
        changed
            .summary
            .push_str(" Reliability remains a secondary axis.");
        let changed = changed.into_capture().unwrap();
        assert_ne!(first.capture_id, changed.capture_id);
    }

    #[test]
    fn unknown_raw_payload_and_host_path_fields_fail_closed() {
        let capture = valid_draft().into_capture().unwrap();
        let mut value = serde_json::to_value(capture).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("transcript".to_string(), serde_json::json!(["raw prompt"]));
        assert!(serde_json::from_value::<ResearchCaptureV1>(value).is_err());

        let mut binding = serde_json::to_value(valid_draft().binding).unwrap();
        binding.as_object_mut().unwrap().insert(
            "root_path".to_string(),
            serde_json::json!("/Users/example/paper"),
        );
        assert!(serde_json::from_value::<ProjectBindingV1>(binding).is_err());
    }

    #[test]
    fn binding_and_evidence_locators_are_bounded_and_portable() {
        assert!(
            ProjectBindingV1::new(
                ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap(),
                0,
                ProjectStage::Idea,
                "Task",
                CapturePolicy::ReviewRequired,
            )
            .is_err()
        );
        assert!(valid_locator(
            EvidenceLocatorKind::ArtifactAnchor,
            "literature/paper_notes.md#measurement"
        ));
        assert!(!valid_locator(
            EvidenceLocatorKind::ArtifactAnchor,
            "/Users/example/private.md"
        ));
        assert!(!valid_locator(
            EvidenceLocatorKind::ArtifactAnchor,
            "C:\\Users\\example\\private.md"
        ));
        assert!(valid_locator(
            EvidenceLocatorKind::HttpsUrl,
            "https://example.org/paper"
        ));
        assert!(!valid_locator(
            EvidenceLocatorKind::HttpsUrl,
            "file:///Users/example/paper.pdf"
        ));
    }

    #[test]
    fn capture_identity_is_revalidated_after_deserialization() {
        let capture = valid_draft().into_capture().unwrap();
        let mut value = serde_json::to_value(capture).unwrap();
        value["summary"] = serde_json::json!("Tampered summary");
        let decoded: ResearchCaptureV1 = serde_json::from_value(value).unwrap();
        assert_eq!(
            decoded.validate(),
            Err(ProjectError::CaptureIdentityConflict)
        );
    }

    #[test]
    fn preview_apply_and_reopen_append_one_pending_capture() {
        let (root, service, project_id) = project_fixture();
        let capture = project_capture(project_id.clone(), 1);
        let plan = service.preview_capture(capture.clone()).unwrap();
        assert_eq!(plan.preview().disposition, CaptureDisposition::Refinement);
        assert_eq!(
            plan.preview().effect,
            CaptureIntakeEffect::AppendPendingHistory
        );
        assert_eq!(plan.preview().expected_project_revision, 1);
        assert_eq!(plan.preview().approvals_required, ["filesystem-write"]);
        assert!(
            plan.preview()
                .history_entry
                .starts_with("context/captures/cap_")
        );

        let debug = format!("{plan:?}");
        assert!(!debug.contains(&root.to_string_lossy().to_string()));
        assert!(!debug.contains(&capture.summary));
        assert_eq!(
            service.apply_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().plan_digest.clone(), false),
                120,
            ),
            Err(ProjectError::ApprovalRequired)
        );
        assert_eq!(
            service.apply_capture(&plan, &ApprovedCaptureIntake::new("wrong-plan", true), 120,),
            Err(ProjectError::PlanMismatch)
        );

        let commit = service
            .apply_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().plan_digest.clone(), true),
                120,
            )
            .unwrap();
        assert_eq!(commit.capture_id, capture.capture_id);
        assert_eq!(commit.base_revision, 1);
        assert_eq!(commit.acknowledgement.len(), 68);

        let reopened = service
            .read_capture(&project_id, &capture.capture_id)
            .unwrap()
            .unwrap();
        assert_eq!(reopened, capture);
        let snapshot = service.snapshot().unwrap();
        assert_eq!(snapshot.projects[0].semantic_revision, 1);
    }

    #[test]
    fn replay_is_classified_and_rejected_without_a_second_write() {
        let (_root, service, project_id) = project_fixture();
        let capture = project_capture(project_id, 1);
        let first = service.preview_capture(capture.clone()).unwrap();
        service
            .apply_capture(
                &first,
                &ApprovedCaptureIntake::new(first.preview().plan_digest.clone(), true),
                120,
            )
            .unwrap();

        let replay = service.preview_capture(capture).unwrap();
        assert_eq!(replay.preview().disposition, CaptureDisposition::Duplicate);
        assert_eq!(replay.preview().effect, CaptureIntakeEffect::NoChange);
        assert!(replay.preview().approvals_required.is_empty());
        assert_eq!(
            service.apply_capture(
                &replay,
                &ApprovedCaptureIntake::new(replay.preview().plan_digest.clone(), true),
                130,
            ),
            Err(ProjectError::CaptureAlreadyApplied)
        );
    }

    #[test]
    fn semantic_dispositions_are_deterministic_and_conservative() {
        let mut unsupported = valid_draft();
        unsupported.evidence.clear();
        let unsupported = unsupported.into_capture().unwrap();
        assert_eq!(
            classify_capture(&unsupported, false),
            CaptureDisposition::UnsupportedGap
        );

        let mut supersession = valid_draft();
        supersession.decisions[0].relation = DecisionRelation::Supersession;
        supersession.decisions[0].target = Some("dec_measurement_model".to_string());
        let supersession = supersession.into_capture().unwrap();
        assert_eq!(
            classify_capture(&supersession, false),
            CaptureDisposition::Supersession
        );

        let mut contradiction = valid_draft();
        contradiction.contradictions.push(ContradictionV1 {
            statement: "Reliability determines construct validity.".to_string(),
            conflicts_with: "Validity and reliability are distinct dimensions.".to_string(),
            consequence: "The organizing distinction remains unresolved.".to_string(),
        });
        let contradiction = contradiction.into_capture().unwrap();
        assert_eq!(
            classify_capture(&contradiction, false),
            CaptureDisposition::Contradiction
        );

        let mut unresolved = valid_draft();
        unresolved.changes.clear();
        unresolved.decisions.clear();
        unresolved.evidence.clear();
        unresolved.contradictions.clear();
        let unresolved = unresolved.into_capture().unwrap();
        assert_eq!(
            classify_capture(&unresolved, false),
            CaptureDisposition::UnresolvedCandidate
        );
    }

    #[test]
    fn stale_project_or_library_revisions_fail_before_capture_write() {
        let (root, service, project_id) = project_fixture();
        let stale_binding = project_capture(project_id.clone(), 2);
        assert!(matches!(
            service.preview_capture(stale_binding),
            Err(ProjectError::RevisionConflict)
        ));

        let capture = project_capture(project_id, 1);
        let plan = service.preview_capture(capture.clone()).unwrap();
        let second_id = ProjectId::parse("prj_11111111111111111111111111111111").unwrap();
        let second = service
            .preview_create(
                root.join("projects/second-project"),
                ProjectRegistrationOptions::new("Second project", ProjectKind::Article)
                    .with_project_id(second_id),
                130,
            )
            .unwrap();
        service
            .apply(
                &second,
                &ApprovedProjectMutation::new(second.preview().plan_digest.clone(), true),
                130,
            )
            .unwrap();
        assert_eq!(
            service.apply_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().plan_digest.clone(), true),
                140,
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert!(
            service
                .read_capture(&capture.binding.project_id, &capture.capture_id)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn portable_export_carries_capture_history_but_not_runtime_lock() {
        let (root, service, project_id) = project_fixture();
        let capture = project_capture(project_id.clone(), 1);
        let intake = service.preview_capture(capture.clone()).unwrap();
        service
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                120,
            )
            .unwrap();

        let export_root = root.join("portable-capture");
        let export = service.preview_export(&project_id, &export_root).unwrap();
        service
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                130,
            )
            .unwrap();
        assert!(
            export_root
                .join("project/context/captures")
                .join(format!("{}.json", capture.capture_id.as_str()))
                .is_file()
        );
        assert!(!export_root.join("project/.qiongli").exists());
    }
}
