//! Portable article-project identity and native Research Library authority.

mod academic_graph;
mod academic_graph_compare;
mod academic_graph_extract;
mod academic_graph_index;
mod academic_graph_portfolio;
mod artifact_changes;
mod capture;
mod capture_assignment_service;
mod capture_coverage;
mod capture_delivery;
mod capture_delivery_service;
mod capture_delivery_storage;
mod capture_inbox;
mod capture_resolution;
mod capture_resolution_service;
mod capture_resolution_storage;
mod consolidation;
mod error;
mod incremental_portfolio;
mod json;
mod migration;
mod model;
mod portable;
mod portfolio_catalog;
mod portfolio_catalog_storage;
mod portfolio_query;
mod repository_inbox;
mod runtime_state;
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
pub use capture_assignment_service::{
    ApprovedCaptureAssignment, CAPTURE_ASSIGNMENT_SERVICE_SCHEMA_VERSION,
    CaptureAssignmentBindingEffect, CaptureAssignmentCommitV1, CaptureAssignmentDecision,
    CaptureAssignmentPreviewOutcome, CaptureAssignmentPreviewV1, CaptureAssignmentStatusState,
    CaptureAssignmentStatusV1, VerifiedCaptureAssignment,
};
pub use capture_coverage::{
    CAPTURE_COVERAGE_SCHEMA_VERSION, CaptureCoverageDelivery, CaptureCoverageSnapshotV1,
    CaptureCoverageState, CaptureSourceCoverageV1,
};
pub use capture_delivery::{
    CAPTURE_DELIVERY_ACKNOWLEDGEMENT_DOCUMENT_KIND,
    CAPTURE_DELIVERY_ACKNOWLEDGEMENT_SCHEMA_VERSION, CAPTURE_DELIVERY_ENVELOPE_DOCUMENT_KIND,
    CAPTURE_DELIVERY_ENVELOPE_SCHEMA_VERSION, CAPTURE_DELIVERY_RECORD_DOCUMENT_KIND,
    CAPTURE_DELIVERY_RECORD_SCHEMA_VERSION, CaptureDeliveryAcknowledgementV1,
    CaptureDeliveryDestinationV1, CaptureDeliveryEnvelopeV1, CaptureDeliveryReason,
    CaptureDeliveryRecordV1, CaptureDeliveryState, CaptureDeliveryTransitionV1,
    DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX, DELIVERY_ENVELOPE_ID_PREFIX, DeliveryAcknowledgementId,
    DeliveryEnvelopeId,
};
pub use capture_delivery_service::{
    CAPTURE_DELIVERY_SERVICE_SCHEMA_VERSION, CaptureDeliveryAcknowledgementRequestV1,
    CaptureDeliveryAcknowledgementSummaryV1, CaptureDeliveryDestinationSummaryV1,
    CaptureDeliveryRetryCause, CaptureDeliveryStatusV1,
};
pub use capture_inbox::{
    CAPTURE_INBOX_SCHEMA_VERSION, CaptureInboxEntryV1, CaptureInboxSnapshotV1, CaptureInboxState,
};
pub use capture_resolution::{
    CAPTURE_ASSIGNMENT_INTENT_DOCUMENT_KIND, CAPTURE_ASSIGNMENT_INTENT_ID_PREFIX,
    CAPTURE_ASSIGNMENT_RECEIPT_DOCUMENT_KIND, CAPTURE_ASSIGNMENT_RECEIPT_ID_PREFIX,
    CAPTURE_RESOLUTION_ITEM_ID_PREFIX, CAPTURE_RESOLUTION_PLAN_DOCUMENT_KIND,
    CAPTURE_RESOLUTION_RECEIPT_DOCUMENT_KIND, CAPTURE_RESOLUTION_RECEIPT_ID_PREFIX,
    CAPTURE_RESOLUTION_SCHEMA_VERSION, CaptureAssignmentIntentBodyV1, CaptureAssignmentIntentId,
    CaptureAssignmentIntentV1, CaptureAssignmentOutcome, CaptureAssignmentReceiptBodyV1,
    CaptureAssignmentReceiptId, CaptureAssignmentReceiptV1, CaptureAssignmentResultV1,
    CaptureResolutionArtifact, CaptureResolutionArtifactObservationV1,
    CaptureResolutionCounterpartState, CaptureResolutionDecisionV1, CaptureResolutionDisposition,
    CaptureResolutionItemId, CaptureResolutionItemKind, CaptureResolutionItemV1,
    CaptureResolutionPlanBodyV1, CaptureResolutionPlanInputV1, CaptureResolutionPlanV1,
    CaptureResolutionReceiptBodyV1, CaptureResolutionReceiptId, CaptureResolutionReceiptV1,
    CaptureResolutionResultV1, CaptureResolutionSelectionV1,
    capture_resolution_allowed_dispositions,
};
pub use capture_resolution_service::{
    ApprovedCaptureResolution, CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION,
    CaptureResolutionCommitV1, CaptureResolutionItemContentV1, CaptureResolutionItemPreviewV1,
    CaptureResolutionPreviewV1, CaptureResolutionSelectionSetV1, VerifiedCaptureResolution,
};
pub use consolidation::{
    ACADEMIC_CONSOLIDATION_SCHEMA_VERSION, ApprovedCaptureConsolidation,
    CaptureConsolidationCommitV1, CaptureConsolidationConflictKind, CaptureConsolidationConflictV1,
    CaptureConsolidationOutcome, CaptureConsolidationPreviewV1, CaptureConsolidationReceiptV1,
    ConsolidatedArtifactV1, ConsolidationArtifact, ConsolidationArtifactDeltaV1,
    ConsolidationArtifactEffect, VerifiedCaptureConsolidation,
};
pub use error::ProjectError;
pub use incremental_portfolio::{
    INCREMENTAL_PORTFOLIO_SCHEMA_VERSION, INCREMENTAL_PORTFOLIO_SNAPSHOT_DOCUMENT_KIND,
    IncrementalPortfolioService, IncrementalPortfolioSnapshotV1,
    PORTFOLIO_RECONCILIATION_DOCUMENT_KIND, PortfolioReconciliationMode, PortfolioReconciliationV1,
};
pub use migration::{
    PROJECT_MIGRATION_DOCUMENT_KIND, PROJECT_MIGRATION_SCHEMA_VERSION,
    ProjectMigrationArtifactCategory, ProjectMigrationArtifactReconciliationV1,
    ProjectMigrationArtifactState, ProjectMigrationCommitV1, ProjectMigrationDoctorStatus,
    ProjectMigrationDoctorV1, ProjectMigrationMarkerState, ProjectMigrationPreviewV1,
    ProjectMigrationReconciliationStatus, ProjectMigrationReconciliationV1,
    ProjectMigrationRecoveryPreviewV1, ProjectMigrationRegistrationState,
    ProjectMigrationRollbackCommitV1, ProjectMigrationRollbackPreviewV1, VerifiedProjectMigration,
    VerifiedProjectMigrationRecovery, VerifiedProjectMigrationRollback,
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
pub use portfolio_catalog::{
    PORTFOLIO_CATALOG_MANIFEST_DOCUMENT_KIND, PORTFOLIO_CATALOG_SCHEMA_VERSION,
    PORTFOLIO_CATALOG_SNAPSHOT_DOCUMENT_KIND, PORTFOLIO_CONTRIBUTION_DOCUMENT_KIND,
    PORTFOLIO_CONTRIBUTION_SCHEMA_VERSION, PortfolioCatalogManifestV1, PortfolioCatalogSnapshotV1,
    PortfolioContributionRefV1, PortfolioContributionV1,
};
pub use portfolio_query::{
    PORTFOLIO_QUERY_DOCUMENT_KIND, PORTFOLIO_QUERY_RESULT_DOCUMENT_KIND,
    PORTFOLIO_QUERY_SCHEMA_VERSION, PortfolioEvidenceSignal, PortfolioLineageKind,
    PortfolioLineageRecordV1, PortfolioQueryCursorV1, PortfolioQueryEdgeV1,
    PortfolioQueryFiltersV1, PortfolioQueryLimitsV1, PortfolioQueryNodeV1, PortfolioQueryProjectV1,
    PortfolioQueryResultV1, PortfolioQueryService, PortfolioQueryV1,
    PortfolioSharedIdentityFilterV1,
};
pub use repository_inbox::{
    ApprovedRepositoryCaptureDelivery, REPOSITORY_CAPTURE_DELIVERY_SCHEMA_VERSION,
    REPOSITORY_CAPTURE_INBOX_SCHEMA_VERSION, RepositoryCaptureDeliveryCommitV1,
    RepositoryCaptureDeliveryPreviewV1, RepositoryCaptureInboxEntryV1,
    RepositoryCaptureInboxSnapshotV1, RepositoryCaptureInboxState,
    RepositoryCaptureIntakePreviewV1, VerifiedRepositoryCaptureDelivery,
    VerifiedRepositoryCaptureIntake,
};
pub use runtime_state::{
    PROJECT_RUNTIME_CHECKPOINT_SCHEMA_VERSION, ProjectRuntimeCheckpointCommitV1,
    ProjectRuntimeCheckpointDocument, ProjectRuntimeCheckpointEntry,
};
pub use service::{
    ApprovedProjectMutation, ProjectMutationCommitV1, ProjectRegistrationOptions,
    ProjectStateService, RegisteredProjectRoot, VerifiedProjectMutation,
};
