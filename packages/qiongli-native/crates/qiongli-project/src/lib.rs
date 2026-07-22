//! Portable article-project identity and native Research Library authority.

mod academic_graph;
mod academic_graph_compare;
mod academic_graph_extract;
mod academic_graph_index;
mod academic_graph_portfolio;
mod artifact_changes;
mod capture;
mod capture_coverage;
mod capture_inbox;
mod consolidation;
mod error;
mod json;
mod migration;
mod model;
mod portable;
mod repository_inbox;
mod service;
mod storage;

pub use academic_graph::{
    ACADEMIC_GRAPH_DOCUMENT_KIND, ACADEMIC_GRAPH_SCHEMA_VERSION, AcademicGraphArtifactTarget,
    AcademicGraphConfidence, AcademicGraphDiagnosticCode, AcademicGraphDiagnosticV1,
    AcademicGraphEdgeStatus, AcademicGraphEdgeV1, AcademicGraphEntityKind,
    AcademicGraphIdentityScope, AcademicGraphLayer, AcademicGraphNodeType, AcademicGraphNodeV1,
    AcademicGraphRelation, AcademicGraphService, AcademicGraphSnapshotV1, AcademicGraphSourceKind,
    AcademicGraphSourceRefV1, AcademicInferenceStrength,
};
pub use academic_graph_compare::{
    ACADEMIC_GRAPH_COMPARISON_DOCUMENT_KIND, ACADEMIC_GRAPH_COMPARISON_SCHEMA_VERSION,
    AcademicGraphChangeKind, AcademicGraphComparisonService, AcademicGraphEdgeChangeV1,
    AcademicGraphNodeChangeV1, AcademicGraphRevisionAction, AcademicGraphRevisionComparisonV1,
    AcademicGraphRiskDeltaV1, AcademicGraphRiskSignalsV1, AcademicGraphSourceChangeV1,
};
pub use academic_graph_index::{
    ACADEMIC_GRAPH_INDEX_DOCUMENT_KIND, ACADEMIC_GRAPH_INDEX_SCHEMA_VERSION,
    ACADEMIC_GRAPH_PATH_DOCUMENT_KIND, ACADEMIC_GRAPH_PATH_SCHEMA_VERSION,
    ACADEMIC_GRAPH_QUERY_DOCUMENT_KIND, ACADEMIC_GRAPH_QUERY_SCHEMA_VERSION,
    AcademicGraphDirection, AcademicGraphIndexService, AcademicGraphIndexV1,
    AcademicGraphPathQueryV1, AcademicGraphPathResultV1, AcademicGraphPathStatus,
    AcademicGraphPathStepV1, AcademicGraphPathTraversal, AcademicGraphQueryResultV1,
    AcademicGraphQueryV1, MAX_ACADEMIC_GRAPH_PATH_HOPS,
};
pub use academic_graph_portfolio::{
    ACADEMIC_GRAPH_PORTFOLIO_DOCUMENT_KIND, ACADEMIC_GRAPH_PORTFOLIO_SCHEMA_VERSION,
    AcademicGraphPortfolioEdgeOriginV1, AcademicGraphPortfolioEdgeV1, AcademicGraphPortfolioNodeV1,
    AcademicGraphPortfolioOccurrenceV1, AcademicGraphPortfolioProjectV1,
    AcademicGraphPortfolioService, AcademicGraphPortfolioSnapshotV1,
};
pub use artifact_changes::{
    ARTIFACT_CHANGE_SCHEMA_VERSION, ArtifactChangeDetection, ArtifactChangeEffect,
    ArtifactChangeReason, ArtifactChangeSnapshotV1, ArtifactChangeState, RegisteredArtifact,
    RegisteredArtifactChangeV1, RegisteredArtifactObservationV1,
};
pub use capture::{
    ApprovedCaptureIntake, CAPTURE_ID_PREFIX, CAPTURE_INTAKE_SCHEMA_VERSION, CaptureArea,
    CaptureDelivery, CaptureDisposition, CaptureId, CaptureIntakeCommitV1, CaptureIntakeEffect,
    CaptureIntakePreviewV1, CapturePolicy, CaptureSource, ContradictionV1, DecisionCandidateV1,
    DecisionRelation, EvidenceLocatorKind, EvidenceReferenceV1, PROJECT_BINDING_DOCUMENT_KIND,
    PROJECT_BINDING_SCHEMA_VERSION, ProjectBindingV1, RESEARCH_CAPTURE_DOCUMENT_KIND,
    RESEARCH_CAPTURE_SCHEMA_VERSION, ResearchCaptureDraftV1, ResearchCaptureV1, SemanticChangeV1,
    VerifiedCaptureIntake, read_portable_capture_packet,
};
pub use capture_coverage::{
    CAPTURE_COVERAGE_SCHEMA_VERSION, CaptureCoverageDelivery, CaptureCoverageSnapshotV1,
    CaptureCoverageState, CaptureSourceCoverageV1,
};
pub use capture_inbox::{
    CAPTURE_INBOX_SCHEMA_VERSION, CaptureInboxEntryV1, CaptureInboxSnapshotV1, CaptureInboxState,
};
pub use consolidation::{
    ACADEMIC_CONSOLIDATION_SCHEMA_VERSION, ApprovedCaptureConsolidation,
    CaptureConsolidationCommitV1, CaptureConsolidationConflictKind, CaptureConsolidationConflictV1,
    CaptureConsolidationOutcome, CaptureConsolidationPreviewV1, CaptureConsolidationReceiptV1,
    ConsolidatedArtifactV1, ConsolidationArtifact, ConsolidationArtifactDeltaV1,
    ConsolidationArtifactEffect, VerifiedCaptureConsolidation,
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
pub use repository_inbox::{
    REPOSITORY_CAPTURE_INBOX_SCHEMA_VERSION, RepositoryCaptureInboxEntryV1,
    RepositoryCaptureInboxSnapshotV1, RepositoryCaptureInboxState,
    RepositoryCaptureIntakePreviewV1, VerifiedRepositoryCaptureIntake,
};
pub use service::{
    ApprovedProjectMutation, ProjectMutationCommitV1, ProjectRegistrationOptions,
    ProjectStateService, RegisteredProjectRoot, VerifiedProjectMutation,
};
