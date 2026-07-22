use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    AcademicGraphConfidence, AcademicGraphEdgeStatus, AcademicGraphEdgeV1, AcademicGraphNodeType,
    AcademicGraphNodeV1, AcademicGraphRelation, AcademicGraphSnapshotV1, AcademicGraphSourceRefV1,
    AcademicInferenceStrength, ProjectError, ProjectId,
};

pub const ACADEMIC_GRAPH_COMPARISON_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_COMPARISON_DOCUMENT_KIND: &str =
    "qiongli-academic-graph-revision-comparison";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphChangeKind {
    Added,
    Removed,
    Modified,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphRevisionAction {
    InspectNewContradictions,
    FillNewGaps,
    VerifyLowConfidenceEvidence,
    ReviewRejectedRelations,
    ReconnectRemovedEvidence,
    InspectModifiedRelations,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphSourceChangeV1 {
    pub change_kind: AcademicGraphChangeKind,
    pub artifact_path: String,
    pub before: Option<AcademicGraphSourceRefV1>,
    pub after: Option<AcademicGraphSourceRefV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphNodeChangeV1 {
    pub change_kind: AcademicGraphChangeKind,
    pub node_id: String,
    pub before: Option<AcademicGraphNodeV1>,
    pub after: Option<AcademicGraphNodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphEdgeChangeV1 {
    pub change_kind: AcademicGraphChangeKind,
    pub edge_id: String,
    pub before: Option<AcademicGraphEdgeV1>,
    pub after: Option<AcademicGraphEdgeV1>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphRiskSignalsV1 {
    pub contradiction_count: usize,
    pub gap_count: usize,
    pub rejected_relation_count: usize,
    pub low_confidence_count: usize,
    pub total_signal_count: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphRiskDeltaV1 {
    pub contradiction_count: i64,
    pub gap_count: i64,
    pub rejected_relation_count: i64,
    pub low_confidence_count: i64,
    pub total_signal_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphRevisionComparisonV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub comparison_id: String,
    pub project_id: ProjectId,
    pub before_project_revision: u64,
    pub after_project_revision: u64,
    pub before_projection_id: String,
    pub after_projection_id: String,
    pub source_change_count: usize,
    pub node_change_count: usize,
    pub edge_change_count: usize,
    pub has_changes: bool,
    pub before_risks: AcademicGraphRiskSignalsV1,
    pub after_risks: AcademicGraphRiskSignalsV1,
    pub risk_delta: AcademicGraphRiskDeltaV1,
    pub source_changes: Vec<AcademicGraphSourceChangeV1>,
    pub node_changes: Vec<AcademicGraphNodeChangeV1>,
    pub edge_changes: Vec<AcademicGraphEdgeChangeV1>,
    pub next_actions: Vec<AcademicGraphRevisionAction>,
}

#[derive(Serialize)]
struct ComparisonIdentity<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
    before_project_revision: u64,
    after_project_revision: u64,
    before_projection_id: &'a str,
    after_projection_id: &'a str,
    source_changes: &'a [AcademicGraphSourceChangeV1],
    node_changes: &'a [AcademicGraphNodeChangeV1],
    edge_changes: &'a [AcademicGraphEdgeChangeV1],
}

pub struct AcademicGraphComparisonService;

impl AcademicGraphComparisonService {
    pub fn compare(
        before: &AcademicGraphSnapshotV1,
        after: &AcademicGraphSnapshotV1,
    ) -> Result<AcademicGraphRevisionComparisonV1, ProjectError> {
        if before.project_id != after.project_id
            || before.project_revision > after.project_revision
            || before.projection_id != format!("grp_{}", before.projection_digest)
            || after.projection_id != format!("grp_{}", after.projection_digest)
        {
            return Err(ProjectError::InvalidGraphQuery);
        }

        let source_changes = compare_sources(&before.sources, &after.sources);
        let node_changes = compare_nodes(&before.nodes, &after.nodes);
        let edge_changes = compare_edges(&before.edges, &after.edges);
        let before_risks = graph_risks(before);
        let after_risks = graph_risks(after);
        let risk_delta = AcademicGraphRiskDeltaV1 {
            contradiction_count: delta(
                before_risks.contradiction_count,
                after_risks.contradiction_count,
            ),
            gap_count: delta(before_risks.gap_count, after_risks.gap_count),
            rejected_relation_count: delta(
                before_risks.rejected_relation_count,
                after_risks.rejected_relation_count,
            ),
            low_confidence_count: delta(
                before_risks.low_confidence_count,
                after_risks.low_confidence_count,
            ),
            total_signal_count: delta(
                before_risks.total_signal_count,
                after_risks.total_signal_count,
            ),
        };
        let next_actions = comparison_actions(&node_changes, &edge_changes, &risk_delta);
        let identity = ComparisonIdentity {
            schema_version: ACADEMIC_GRAPH_COMPARISON_SCHEMA_VERSION,
            project_id: &after.project_id,
            before_project_revision: before.project_revision,
            after_project_revision: after.project_revision,
            before_projection_id: &before.projection_id,
            after_projection_id: &after.projection_id,
            source_changes: &source_changes,
            node_changes: &node_changes,
            edge_changes: &edge_changes,
        };
        let bytes = serde_json_canonicalizer::to_vec(&identity)
            .map_err(|_| ProjectError::InvalidGraphDocument)?;
        let comparison_id = format!("gcp_{:x}", Sha256::digest(bytes));
        let has_changes =
            !source_changes.is_empty() || !node_changes.is_empty() || !edge_changes.is_empty();

        Ok(AcademicGraphRevisionComparisonV1 {
            schema_version: ACADEMIC_GRAPH_COMPARISON_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_COMPARISON_DOCUMENT_KIND.to_string(),
            comparison_id,
            project_id: after.project_id.clone(),
            before_project_revision: before.project_revision,
            after_project_revision: after.project_revision,
            before_projection_id: before.projection_id.clone(),
            after_projection_id: after.projection_id.clone(),
            source_change_count: source_changes.len(),
            node_change_count: node_changes.len(),
            edge_change_count: edge_changes.len(),
            has_changes,
            before_risks,
            after_risks,
            risk_delta,
            source_changes,
            node_changes,
            edge_changes,
            next_actions,
        })
    }
}

fn compare_sources(
    before: &[AcademicGraphSourceRefV1],
    after: &[AcademicGraphSourceRefV1],
) -> Vec<AcademicGraphSourceChangeV1> {
    let before = before
        .iter()
        .map(|source| (source.artifact_path.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let after = after
        .iter()
        .map(|source| (source.artifact_path.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    merge_changes(
        &before,
        &after,
        |change_kind, artifact_path, before, after| AcademicGraphSourceChangeV1 {
            change_kind,
            artifact_path: artifact_path.to_string(),
            before: before.cloned(),
            after: after.cloned(),
        },
    )
}

fn compare_nodes(
    before: &[AcademicGraphNodeV1],
    after: &[AcademicGraphNodeV1],
) -> Vec<AcademicGraphNodeChangeV1> {
    let before = before
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let after = after
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    merge_changes(&before, &after, |change_kind, node_id, before, after| {
        AcademicGraphNodeChangeV1 {
            change_kind,
            node_id: node_id.to_string(),
            before: before.cloned(),
            after: after.cloned(),
        }
    })
}

fn compare_edges(
    before: &[AcademicGraphEdgeV1],
    after: &[AcademicGraphEdgeV1],
) -> Vec<AcademicGraphEdgeChangeV1> {
    let before = before
        .iter()
        .map(|edge| (edge.edge_id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    let after = after
        .iter()
        .map(|edge| (edge.edge_id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    merge_changes(&before, &after, |change_kind, edge_id, before, after| {
        AcademicGraphEdgeChangeV1 {
            change_kind,
            edge_id: edge_id.to_string(),
            before: before.cloned(),
            after: after.cloned(),
        }
    })
}

fn merge_changes<T, U, F>(
    before: &BTreeMap<&str, &T>,
    after: &BTreeMap<&str, &T>,
    mut build: F,
) -> Vec<U>
where
    T: Eq,
    F: FnMut(AcademicGraphChangeKind, &str, Option<&T>, Option<&T>) -> U,
{
    let mut changes = Vec::new();
    for (id, before_value) in before {
        match after.get(id) {
            Some(after_value) if *before_value != *after_value => changes.push(build(
                AcademicGraphChangeKind::Modified,
                id,
                Some(*before_value),
                Some(*after_value),
            )),
            None => changes.push(build(
                AcademicGraphChangeKind::Removed,
                id,
                Some(*before_value),
                None,
            )),
            Some(_) => {}
        }
    }
    for (id, after_value) in after {
        if !before.contains_key(id) {
            changes.push(build(
                AcademicGraphChangeKind::Added,
                id,
                None,
                Some(*after_value),
            ));
        }
    }
    changes
}

fn graph_risks(graph: &AcademicGraphSnapshotV1) -> AcademicGraphRiskSignalsV1 {
    let contradiction_count = graph
        .edges
        .iter()
        .filter(|edge| edge.relation == AcademicGraphRelation::Contradicts)
        .count();
    let gap_count = graph
        .nodes
        .iter()
        .filter(|node| node.node_type == AcademicGraphNodeType::Gap)
        .count()
        + graph
            .edges
            .iter()
            .filter(|edge| edge.inference_strength == AcademicInferenceStrength::UnsupportedGap)
            .count();
    let rejected_relation_count = graph
        .edges
        .iter()
        .filter(|edge| edge.status == AcademicGraphEdgeStatus::Rejected)
        .count();
    let low_confidence_count = graph
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.confidence,
                AcademicGraphConfidence::Low | AcademicGraphConfidence::Unknown
            )
        })
        .count();
    AcademicGraphRiskSignalsV1 {
        contradiction_count,
        gap_count,
        rejected_relation_count,
        low_confidence_count,
        total_signal_count: contradiction_count
            + gap_count
            + rejected_relation_count
            + low_confidence_count,
    }
}

fn comparison_actions(
    node_changes: &[AcademicGraphNodeChangeV1],
    edge_changes: &[AcademicGraphEdgeChangeV1],
    risk_delta: &AcademicGraphRiskDeltaV1,
) -> Vec<AcademicGraphRevisionAction> {
    let mut actions = Vec::new();
    if risk_delta.contradiction_count > 0 {
        actions.push(AcademicGraphRevisionAction::InspectNewContradictions);
    }
    if risk_delta.gap_count > 0 {
        actions.push(AcademicGraphRevisionAction::FillNewGaps);
    }
    if risk_delta.low_confidence_count > 0 {
        actions.push(AcademicGraphRevisionAction::VerifyLowConfidenceEvidence);
    }
    if risk_delta.rejected_relation_count > 0 {
        actions.push(AcademicGraphRevisionAction::ReviewRejectedRelations);
    }
    if node_changes.iter().any(|change| {
        change.change_kind == AcademicGraphChangeKind::Removed
            && change
                .before
                .as_ref()
                .is_some_and(|node| node.node_type == AcademicGraphNodeType::Evidence)
    }) {
        actions.push(AcademicGraphRevisionAction::ReconnectRemovedEvidence);
    }
    if edge_changes
        .iter()
        .any(|change| change.change_kind == AcademicGraphChangeKind::Modified)
    {
        actions.push(AcademicGraphRevisionAction::InspectModifiedRelations);
    }
    actions
}

fn delta(before: usize, after: usize) -> i64 {
    i64::try_from(after).unwrap_or(i64::MAX) - i64::try_from(before).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcademicGraphIdentityScope, AcademicGraphLayer, AcademicGraphSourceKind, ProjectLifecycle,
        ProjectStage,
    };

    fn fixture(project_revision: u64, risky: bool) -> AcademicGraphSnapshotV1 {
        let project_id =
            ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051").expect("project id is valid");
        let project = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Project,
            project_id.as_str(),
            "Revision comparison fixture",
            vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
            "context/project_manifest.json",
            "#/project_id",
        )
        .expect("project node is valid");
        let gap = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Gap,
            AcademicGraphIdentityScope::Project,
            "GAP-001",
            "Missing evidence",
            vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
            "evidence/claim-evidence-ledger.csv",
            "row:GAP-001",
        )
        .expect("gap node is valid");
        let edge = AcademicGraphEdgeV1::new(
            &project_id,
            &project.node_id,
            AcademicGraphRelation::Contradicts,
            &gap.node_id,
            vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
            "The reviewed record identifies an explicit contradiction.",
            "evidence/claim-evidence-ledger.csv",
            "row:GAP-001",
            "The contradiction still requires source verification.",
            AcademicInferenceStrength::ReasonableInference,
            AcademicGraphConfidence::Low,
            AcademicGraphEdgeStatus::Rejected,
            None,
        )
        .expect("edge is valid");
        let nodes = if risky {
            vec![project, gap]
        } else {
            vec![project]
        };
        let edges = if risky { vec![edge] } else { Vec::new() };
        let projection_digest = format!("{:064x}", project_revision);
        AcademicGraphSnapshotV1 {
            schema_version: 1,
            document_kind: "qiongli-academic-graph".to_string(),
            projection_id: format!("grp_{projection_digest}"),
            projection_digest,
            project_id,
            project_revision,
            project_stage: ProjectStage::Writing,
            project_lifecycle: ProjectLifecycle::Active,
            project_manifest_digest: "1".repeat(64),
            project_semantic_digest: "2".repeat(64),
            graph_source_digest: "3".repeat(64),
            source_count: 1,
            present_source_count: 1,
            node_count: nodes.len(),
            edge_count: edges.len(),
            diagnostic_count: 0,
            sources: vec![AcademicGraphSourceRefV1 {
                source_kind: AcademicGraphSourceKind::ProjectManifest,
                artifact_path: "context/project_manifest.json".to_string(),
                present: true,
                content_digest: Some("1".repeat(64)),
                size_bytes: 120,
            }],
            nodes,
            edges,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn comparison_is_deterministic_and_reports_risk_actions() {
        let before = fixture(1, false);
        let after = fixture(2, true);
        let first =
            AcademicGraphComparisonService::compare(&before, &after).expect("comparison succeeds");
        let second =
            AcademicGraphComparisonService::compare(&before, &after).expect("comparison succeeds");

        assert_eq!(first, second);
        assert!(first.has_changes);
        assert_eq!(first.node_change_count, 1);
        assert_eq!(first.edge_change_count, 1);
        assert_eq!(first.risk_delta.total_signal_count, 4);
        assert_eq!(
            first.next_actions,
            vec![
                AcademicGraphRevisionAction::InspectNewContradictions,
                AcademicGraphRevisionAction::FillNewGaps,
                AcademicGraphRevisionAction::VerifyLowConfidenceEvidence,
                AcademicGraphRevisionAction::ReviewRejectedRelations,
            ]
        );
    }

    #[test]
    fn comparison_rejects_cross_project_or_reverse_revisions() {
        let after = fixture(2, true);
        let reverse = AcademicGraphComparisonService::compare(&after, &fixture(1, false));
        assert_eq!(reverse, Err(ProjectError::InvalidGraphQuery));

        let mut other = fixture(3, true);
        other.project_id =
            ProjectId::parse("prj_118f4d5a3b2c71008a9b0c1d2e3f4051").expect("project id is valid");
        assert_eq!(
            AcademicGraphComparisonService::compare(&after, &other),
            Err(ProjectError::InvalidGraphQuery)
        );
    }
}
