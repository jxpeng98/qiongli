use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::ProjectError;

pub const ARTICLE_PROJECT_SCHEMA_VERSION: u32 = 1;
pub const ARTICLE_PROJECT_DOCUMENT_KIND: &str = "qiongli-article-project";
pub const RESEARCH_LIBRARY_SCHEMA_VERSION: u32 = 1;
pub(crate) const RESEARCH_LIBRARY_DOCUMENT_KIND: &str = "qiongli-research-library";
pub(crate) const MAX_LIBRARY_PROJECTS: usize = 512;
pub(crate) const MAX_REGISTRATION_TOMBSTONES: usize = 1_024;
pub(crate) const MAX_DISPLAY_NAME_BYTES: usize = 160;
pub(crate) const MAX_OVERVIEW_TEXT_BYTES: usize = 500;
pub(crate) const MAX_PRIORITIES: usize = 8;
pub(crate) const MAX_SEMANTIC_REVISION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectError> {
        let value = value.into();
        if value.len() != 36
            || !value.starts_with("prj_")
            || !value[4..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        Ok(Self(value))
    }

    pub(crate) fn from_random_bytes(bytes: &[u8; 16]) -> Self {
        let mut value = String::with_capacity(36);
        value.push_str("prj_");
        for byte in bytes {
            use std::fmt::Write as _;
            let _ = write!(value, "{byte:02x}");
        }
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        Self::parse(self.0.clone()).map(|_| ())
    }
}

impl Debug for ProjectId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProjectId(")?;
        formatter.write_str(&self.0)?;
        formatter.write_str(")")
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectKind {
    Article,
    Review,
    DissertationArticle,
    Manuscript,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectStage {
    Idea,
    Framing,
    Literature,
    Design,
    Analysis,
    Writing,
    Review,
    Submission,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectLifecycle {
    Active,
    Archived,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArticleProjectManifestV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub display_name: String,
    pub project_kind: ProjectKind,
    pub stage: ProjectStage,
    pub lifecycle: ProjectLifecycle,
    pub semantic_revision: u64,
    pub semantic_digest: String,
    pub created_at_unix: u64,
    pub academically_updated_at_unix: u64,
}

impl ArticleProjectManifestV1 {
    pub(crate) fn new(
        project_id: ProjectId,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
        semantic_digest: String,
        now_unix: u64,
    ) -> Result<Self, ProjectError> {
        let manifest = Self {
            schema_version: ARTICLE_PROJECT_SCHEMA_VERSION,
            document_kind: ARTICLE_PROJECT_DOCUMENT_KIND.to_string(),
            project_id,
            display_name,
            project_kind,
            stage,
            lifecycle: ProjectLifecycle::Active,
            semantic_revision: 1,
            semantic_digest,
            created_at_unix: now_unix,
            academically_updated_at_unix: now_unix,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != ARTICLE_PROJECT_SCHEMA_VERSION
            || self.document_kind != ARTICLE_PROJECT_DOCUMENT_KIND
            || !valid_display_name(&self.display_name)
            || self.semantic_revision == 0
            || self.semantic_revision > MAX_SEMANTIC_REVISION
            || !valid_lower_hex(&self.semantic_digest, 64)
            || self.created_at_unix > MAX_SEMANTIC_REVISION
            || self.academically_updated_at_unix < self.created_at_unix
            || self.academically_updated_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        self.project_id.validate()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryHealth {
    Ready,
    Empty,
    RecoveryRequired,
    InspectionBlocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectHealth {
    Ready,
    MissingRoot,
    MissingManifest,
    ManifestConflict,
    RevisionDrift,
    InspectionBlocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectNextAction {
    Open,
    Refresh,
    Relocate,
    RepairManifest,
    InspectPermissions,
    Restore,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverviewV1 {
    pub focal_question: Option<String>,
    pub thesis: Option<String>,
    pub evidence_position: Option<String>,
    pub unresolved_risk_count: u32,
    pub claim_evidence_coverage_percent: Option<u8>,
    pub next_priorities: Vec<String>,
}

impl ProjectOverviewV1 {
    pub(crate) fn empty() -> Self {
        Self {
            focal_question: None,
            thesis: None,
            evidence_position: None,
            unresolved_risk_count: 0,
            claim_evidence_coverage_percent: None,
            next_priorities: Vec::new(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self
            .focal_question
            .iter()
            .chain(self.thesis.iter())
            .chain(self.evidence_position.iter())
            .chain(self.next_priorities.iter())
            .any(|value| !valid_overview_text(value))
            || self.next_priorities.len() > MAX_PRIORITIES
            || self
                .claim_evidence_coverage_percent
                .is_some_and(|value| value > 100)
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleProjectSummaryV1 {
    pub project_id: ProjectId,
    pub display_name: String,
    pub project_kind: ProjectKind,
    pub stage: ProjectStage,
    pub lifecycle: ProjectLifecycle,
    pub semantic_revision: u64,
    pub registered_at_unix: u64,
    pub last_opened_at_unix: Option<u64>,
    pub academically_updated_at_unix: u64,
    pub health: ProjectHealth,
    pub next_action: ProjectNextAction,
    pub root_label: String,
    pub overview: ProjectOverviewV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchLibrarySnapshotV1 {
    pub schema_version: u32,
    pub revision: u64,
    pub health: LibraryHealth,
    pub projects: Vec<ArticleProjectSummaryV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingContinuityArtifact {
    ResearchState,
    DecisionLog,
    StageHandoff,
    LiteratureMap,
    ClaimEvidenceLedger,
    ManuscriptClaimMap,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMutationKind {
    Register,
    Create,
    RepairManifest,
    Archive,
    Restore,
    Refresh,
    Unregister,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMutationEffect {
    CreateManifestAndRegister,
    RegisterExistingManifest,
    CreateProject,
    RebuildPortableManifest,
    UpdateLifecycle,
    UpdateSemanticRevision,
    RemoveLibraryEntry,
    NoChange,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMutationPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub operation: ProjectMutationKind,
    pub effect: ProjectMutationEffect,
    pub project_id: ProjectId,
    pub display_name: String,
    pub project_kind: ProjectKind,
    pub stage: ProjectStage,
    pub expected_library_revision: u64,
    pub expected_project_revision: Option<u64>,
    pub root_label: String,
    pub manifest_action: String,
    pub missing_continuity_artifacts: Vec<MissingContinuityArtifact>,
    pub approvals_required: Vec<String>,
}

pub(crate) fn valid_display_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_DISPLAY_NAME_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

pub(crate) fn valid_overview_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_OVERVIEW_TEXT_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

pub(crate) fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResearchLibraryDocumentV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub revision: u64,
    pub projects: Vec<RegisteredProjectV1>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub registration_recovery_floor_revision: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registration_tombstones: Vec<ProjectRegistrationTombstoneV1>,
}

impl ResearchLibraryDocumentV1 {
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: RESEARCH_LIBRARY_SCHEMA_VERSION,
            document_kind: RESEARCH_LIBRARY_DOCUMENT_KIND.to_string(),
            revision: 0,
            projects: Vec::new(),
            registration_recovery_floor_revision: 0,
            registration_tombstones: Vec::new(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != RESEARCH_LIBRARY_SCHEMA_VERSION
            || self.document_kind != RESEARCH_LIBRARY_DOCUMENT_KIND
            || self.revision > MAX_SEMANTIC_REVISION
            || self.projects.len() > MAX_LIBRARY_PROJECTS
            || self.registration_tombstones.len() > MAX_REGISTRATION_TOMBSTONES
            || self.registration_recovery_floor_revision > self.revision
        {
            return Err(ProjectError::InvalidLibraryDocument);
        }
        let mut previous: Option<&ProjectId> = None;
        for project in &self.projects {
            project.validate()?;
            if previous.is_some_and(|value| value >= &project.project_id) {
                return Err(ProjectError::InvalidLibraryDocument);
            }
            previous = Some(&project.project_id);
        }
        let mut previous_tombstone: Option<&ProjectRegistrationTombstoneV1> = None;
        for tombstone in &self.registration_tombstones {
            tombstone.validate()?;
            if tombstone.unregistered_at_library_revision
                <= self.registration_recovery_floor_revision
                || tombstone.unregistered_at_library_revision > self.revision
            {
                return Err(ProjectError::InvalidLibraryDocument);
            }
            if previous_tombstone.is_some_and(|previous| {
                previous.identity_kind > tombstone.identity_kind
                    || (previous.identity_kind == tombstone.identity_kind
                        && previous.identity_value >= tombstone.identity_value)
            }) {
                return Err(ProjectError::InvalidLibraryDocument);
            }
            previous_tombstone = Some(tombstone);
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectRegistrationTombstoneV1 {
    pub identity_kind: ProjectRegistrationTombstoneIdentityKindV1,
    pub identity_value: String,
    pub unregistered_at_library_revision: u64,
}

impl Debug for ProjectRegistrationTombstoneV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProjectRegistrationTombstoneV1")
            .field("identity_kind", &self.identity_kind)
            .field("identity_value", &"<registration-identity>")
            .field(
                "unregistered_at_library_revision",
                &self.unregistered_at_library_revision,
            )
            .finish()
    }
}

impl ProjectRegistrationTombstoneV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        let valid_identity = match self.identity_kind {
            ProjectRegistrationTombstoneIdentityKindV1::ProjectId => {
                ProjectId::parse(self.identity_value.clone()).is_ok()
            }
            ProjectRegistrationTombstoneIdentityKindV1::RootReferenceDigest => {
                valid_lower_hex(&self.identity_value, 64)
            }
        };
        if !valid_identity || self.unregistered_at_library_revision == 0 {
            return Err(ProjectError::InvalidLibraryDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProjectRegistrationTombstoneIdentityKindV1 {
    ProjectId,
    RootReferenceDigest,
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegisteredProjectV1 {
    pub project_id: ProjectId,
    pub display_name: String,
    pub project_kind: ProjectKind,
    pub stage: ProjectStage,
    pub lifecycle: ProjectLifecycle,
    pub semantic_revision: u64,
    pub semantic_digest: String,
    pub root_path: String,
    pub registered_at_unix: u64,
    pub last_opened_at_unix: Option<u64>,
    pub academically_updated_at_unix: u64,
}

impl Debug for RegisteredProjectV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RegisteredProjectV1")
            .field("project_id", &self.project_id)
            .field("display_name", &self.display_name)
            .field("project_kind", &self.project_kind)
            .field("stage", &self.stage)
            .field("lifecycle", &self.lifecycle)
            .field("semantic_revision", &self.semantic_revision)
            .field("root_path", &"<registered-project-root>")
            .finish()
    }
}

impl RegisteredProjectV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        self.project_id.validate()?;
        if !valid_display_name(&self.display_name)
            || self.semantic_revision == 0
            || self.semantic_revision > MAX_SEMANTIC_REVISION
            || !valid_lower_hex(&self.semantic_digest, 64)
            || self.root_path.is_empty()
            || self.root_path.len() > 4096
            || self.root_path.chars().any(char::is_control)
            || self.registered_at_unix > MAX_SEMANTIC_REVISION
            || self
                .last_opened_at_unix
                .is_some_and(|value| value > MAX_SEMANTIC_REVISION)
            || self.academically_updated_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidLibraryDocument);
        }
        Ok(())
    }
}
