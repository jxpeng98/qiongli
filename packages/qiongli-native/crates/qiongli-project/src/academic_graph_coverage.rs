use serde::Serialize;

pub const ACADEMIC_GRAPH_COVERAGE_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_COVERAGE_DOCUMENT_KIND: &str = "qiongli-academic-graph-source-coverage";

pub const ACADEMIC_GRAPH_REGISTERED_ARTIFACT_PATHS: [&str; 8] = [
    "context/research_state.md",
    "context/decision_log.md",
    "context/stage_handoff.md",
    "context/boundary_review.md",
    "context/idea_funnel.md",
    "literature/literature_map.md",
    "evidence/claim-evidence-ledger.csv",
    "manuscript/claims_evidence_map.md",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphPortableAuthorityV1 {
    ProjectManifest,
    CanonicalArtifact,
    ExplicitSemanticLinks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphContributionV1 {
    ProjectIdentity,
    StructuredEntitiesAndRelations,
    StructuralArtifactOnly,
    ExplicitSemanticRecords,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphStableIdentityPolicyV1 {
    ProjectId,
    CanonicalArtifactPath,
    StructuredStableId,
    ExplicitRecordId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphSourceAnchorPolicyV1 {
    JsonPointer,
    Document,
    StructuredFieldOrRow,
    ExplicitRecordAnchor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphDiagnosticPolicyV1 {
    ProjectIdentityValidation,
    StructuredIdentityAndRelationValidation,
    StructuralOnlyNoInference,
    ExplicitRecordValidation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphExtractorV1 {
    ProjectManifest,
    ResearchState,
    DecisionLog,
    StageHandoffStructural,
    BoundaryReview,
    IdeaFunnel,
    LiteratureMap,
    ClaimEvidenceLedger,
    ManuscriptClaimMap,
    SemanticLinks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphSurfaceVisibilityV1 {
    pub app: bool,
    pub cli: bool,
    pub full_mcp: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphSourceCoverageV1 {
    pub artifact_path: &'static str,
    pub portable_authority: AcademicGraphPortableAuthorityV1,
    pub graph_contribution: AcademicGraphContributionV1,
    pub stable_identity: AcademicGraphStableIdentityPolicyV1,
    pub source_anchor: AcademicGraphSourceAnchorPolicyV1,
    pub diagnostic_policy: AcademicGraphDiagnosticPolicyV1,
    pub diagnostic_reason: &'static str,
    pub extractor: AcademicGraphExtractorV1,
    pub visibility: AcademicGraphSurfaceVisibilityV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphCoverageRegistryV1 {
    pub schema_version: u32,
    pub document_kind: &'static str,
    pub sources: &'static [AcademicGraphSourceCoverageV1],
}

const ALL_READ_SURFACES: AcademicGraphSurfaceVisibilityV1 = AcademicGraphSurfaceVisibilityV1 {
    app: true,
    cli: true,
    full_mcp: true,
};
const MANIFEST_DIAGNOSTIC_REASON: &str = "invalid-project-identity-or-manifest-binding";
const STRUCTURED_DIAGNOSTIC_REASON: &str = "invalid-structured-identity-anchor-or-relation";
const STRUCTURAL_DIAGNOSTIC_REASON: &str = "semantic-inference-disabled-for-structural-source";
const EXPLICIT_DIAGNOSTIC_REASON: &str = "invalid-explicit-identity-anchor-or-relation";

pub const ACADEMIC_GRAPH_SOURCE_COVERAGE_V1: [AcademicGraphSourceCoverageV1; 10] = [
    AcademicGraphSourceCoverageV1 {
        artifact_path: "context/project_manifest.json",
        portable_authority: AcademicGraphPortableAuthorityV1::ProjectManifest,
        graph_contribution: AcademicGraphContributionV1::ProjectIdentity,
        stable_identity: AcademicGraphStableIdentityPolicyV1::ProjectId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::JsonPointer,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::ProjectIdentityValidation,
        diagnostic_reason: MANIFEST_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::ProjectManifest,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "context/research_state.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::ResearchState,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "context/decision_log.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::DecisionLog,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "context/stage_handoff.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuralArtifactOnly,
        stable_identity: AcademicGraphStableIdentityPolicyV1::CanonicalArtifactPath,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::Document,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuralOnlyNoInference,
        diagnostic_reason: STRUCTURAL_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::StageHandoffStructural,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "context/boundary_review.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::BoundaryReview,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "context/idea_funnel.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::IdeaFunnel,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "literature/literature_map.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::LiteratureMap,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "evidence/claim-evidence-ledger.csv",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::ClaimEvidenceLedger,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "manuscript/claims_evidence_map.md",
        portable_authority: AcademicGraphPortableAuthorityV1::CanonicalArtifact,
        graph_contribution: AcademicGraphContributionV1::StructuredEntitiesAndRelations,
        stable_identity: AcademicGraphStableIdentityPolicyV1::StructuredStableId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::StructuredFieldOrRow,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::StructuredIdentityAndRelationValidation,
        diagnostic_reason: STRUCTURED_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::ManuscriptClaimMap,
        visibility: ALL_READ_SURFACES,
    },
    AcademicGraphSourceCoverageV1 {
        artifact_path: "graph/semantic_links.jsonl",
        portable_authority: AcademicGraphPortableAuthorityV1::ExplicitSemanticLinks,
        graph_contribution: AcademicGraphContributionV1::ExplicitSemanticRecords,
        stable_identity: AcademicGraphStableIdentityPolicyV1::ExplicitRecordId,
        source_anchor: AcademicGraphSourceAnchorPolicyV1::ExplicitRecordAnchor,
        diagnostic_policy: AcademicGraphDiagnosticPolicyV1::ExplicitRecordValidation,
        diagnostic_reason: EXPLICIT_DIAGNOSTIC_REASON,
        extractor: AcademicGraphExtractorV1::SemanticLinks,
        visibility: ALL_READ_SURFACES,
    },
];

pub const ACADEMIC_GRAPH_COVERAGE_REGISTRY_V1: AcademicGraphCoverageRegistryV1 =
    AcademicGraphCoverageRegistryV1 {
        schema_version: ACADEMIC_GRAPH_COVERAGE_SCHEMA_VERSION,
        document_kind: ACADEMIC_GRAPH_COVERAGE_DOCUMENT_KIND,
        sources: &ACADEMIC_GRAPH_SOURCE_COVERAGE_V1,
    };

#[must_use]
pub fn academic_graph_source_coverage(
    artifact_path: &str,
) -> Option<&'static AcademicGraphSourceCoverageV1> {
    ACADEMIC_GRAPH_COVERAGE_REGISTRY_V1
        .sources
        .iter()
        .find(|source| source.artifact_path == artifact_path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn registry_covers_the_manifest_eight_artifacts_and_semantic_links_once() {
        assert_eq!(ACADEMIC_GRAPH_COVERAGE_REGISTRY_V1.schema_version, 1);
        assert_eq!(ACADEMIC_GRAPH_COVERAGE_REGISTRY_V1.sources.len(), 10);
        let paths = ACADEMIC_GRAPH_COVERAGE_REGISTRY_V1
            .sources
            .iter()
            .map(|source| source.artifact_path)
            .collect::<BTreeSet<_>>();
        assert_eq!(paths.len(), 10);
        assert!(paths.contains("context/project_manifest.json"));
        assert!(paths.contains("graph/semantic_links.jsonl"));
        assert!(
            ACADEMIC_GRAPH_REGISTERED_ARTIFACT_PATHS
                .iter()
                .all(|path| paths.contains(path))
        );
        assert!(
            ACADEMIC_GRAPH_COVERAGE_REGISTRY_V1
                .sources
                .iter()
                .all(|source| source.visibility.app
                    && source.visibility.cli
                    && source.visibility.full_mcp
                    && !source.diagnostic_reason.is_empty())
        );
    }

    #[test]
    fn every_registered_artifact_has_one_specific_extractor() {
        for path in ACADEMIC_GRAPH_REGISTERED_ARTIFACT_PATHS {
            let coverage = academic_graph_source_coverage(path).expect("registered source");
            assert_eq!(
                coverage.portable_authority,
                AcademicGraphPortableAuthorityV1::CanonicalArtifact
            );
            assert!(!matches!(
                coverage.extractor,
                AcademicGraphExtractorV1::ProjectManifest | AcademicGraphExtractorV1::SemanticLinks
            ));
        }
    }

    #[test]
    fn stage_handoff_is_explicitly_structural_only() {
        let coverage = academic_graph_source_coverage("context/stage_handoff.md").unwrap();
        assert_eq!(
            coverage.graph_contribution,
            AcademicGraphContributionV1::StructuralArtifactOnly
        );
        assert_eq!(
            coverage.stable_identity,
            AcademicGraphStableIdentityPolicyV1::CanonicalArtifactPath
        );
        assert_eq!(
            coverage.diagnostic_policy,
            AcademicGraphDiagnosticPolicyV1::StructuralOnlyNoInference
        );
        assert_eq!(
            coverage.extractor,
            AcademicGraphExtractorV1::StageHandoffStructural
        );
    }
}
