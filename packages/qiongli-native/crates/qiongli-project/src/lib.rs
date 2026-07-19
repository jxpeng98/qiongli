//! Portable article-project identity and native Research Library authority.

mod capture;
mod error;
mod json;
mod migration;
mod model;
mod portable;
mod service;
mod storage;

pub use capture::{
    CAPTURE_ID_PREFIX, CaptureArea, CaptureDelivery, CaptureId, CapturePolicy, CaptureSource,
    ContradictionV1, DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind,
    EvidenceReferenceV1, PROJECT_BINDING_DOCUMENT_KIND, PROJECT_BINDING_SCHEMA_VERSION,
    ProjectBindingV1, RESEARCH_CAPTURE_DOCUMENT_KIND, RESEARCH_CAPTURE_SCHEMA_VERSION,
    ResearchCaptureDraftV1, ResearchCaptureV1, SemanticChangeV1,
};
pub use error::ProjectError;
pub use migration::{
    PROJECT_MIGRATION_DOCUMENT_KIND, PROJECT_MIGRATION_SCHEMA_VERSION, ProjectMigrationCommitV1,
    ProjectMigrationPreviewV1, VerifiedProjectMigration,
};
pub use model::{
    ARTICLE_PROJECT_DOCUMENT_KIND, ARTICLE_PROJECT_SCHEMA_VERSION, ArticleProjectManifestV1,
    ArticleProjectSummaryV1, LibraryHealth, MissingContinuityArtifact, ProjectHealth, ProjectId,
    ProjectKind, ProjectLifecycle, ProjectMutationEffect, ProjectMutationKind,
    ProjectMutationPreviewV1, ProjectNextAction, ProjectOverviewV1, ProjectStage,
    RESEARCH_LIBRARY_SCHEMA_VERSION, ResearchLibrarySnapshotV1,
};
pub use portable::{
    PORTABLE_PROJECT_DOCUMENT_KIND, PORTABLE_PROJECT_SCHEMA_VERSION, PortableProjectCommitV1,
    PortableProjectOperation, PortableProjectPreviewV1, VerifiedPortableProjectOperation,
};
pub use service::{
    ApprovedProjectMutation, ProjectMutationCommitV1, ProjectRegistrationOptions,
    ProjectStateService, RegisteredProjectRoot, VerifiedProjectMutation,
};
