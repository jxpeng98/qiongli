//! Portable article-project identity and native Research Library authority.

mod error;
mod json;
mod model;
mod portable;
mod service;
mod storage;

pub use error::ProjectError;
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
