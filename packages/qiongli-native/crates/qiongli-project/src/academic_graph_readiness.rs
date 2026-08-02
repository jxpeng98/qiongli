use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    AcademicGraphDiagnosticCode, AcademicGraphLayer, AcademicGraphNodeType, AcademicGraphRelation,
    AcademicGraphSnapshotV1, AcademicGraphSourceKind, ProjectId,
};

pub const ACADEMIC_GRAPH_READINESS_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_READINESS_DOCUMENT_KIND: &str = "qiongli-academic-graph-readiness";
pub const ACADEMIC_GRAPH_PROJECTION_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_PROJECTION_DOCUMENT_KIND: &str = "qiongli-academic-graph-projection";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphReadinessState {
    Stale,
    EmptyProject,
    NoRecognizedArtifacts,
    NodesWithoutEdges,
    Sparse,
    Visualizable,
    BoundedTruncated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphReadinessRemediation {
    RebuildGraph,
    AddCanonicalArtifacts,
    RepairGraphArtifacts,
    AddSemanticRelations,
    EnrichGraph,
    NarrowQuery,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphReadinessSourceState {
    Missing,
    Present,
    Invalid,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphSourceFreshness {
    Fresh,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphBuildBindingV1 {
    pub project_revision: u64,
    pub projection_id: String,
    pub graph_source_digest: String,
}

impl AcademicGraphBuildBindingV1 {
    #[must_use]
    pub fn from_graph(graph: &AcademicGraphSnapshotV1) -> Self {
        Self {
            project_revision: graph.project_revision,
            projection_id: graph.projection_id.clone(),
            graph_source_digest: graph.graph_source_digest.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphReadinessSourceV1 {
    pub source_kind: AcademicGraphSourceKind,
    pub artifact_path: String,
    pub state: AcademicGraphReadinessSourceState,
    pub freshness: AcademicGraphSourceFreshness,
    pub node_count: usize,
    pub edge_count: usize,
    pub diagnostic_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphLayerCountV1 {
    pub layer: AcademicGraphLayer,
    pub node_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphNodeTypeCountV1 {
    pub node_type: AcademicGraphNodeType,
    pub node_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphRelationCountV1 {
    pub relation: AcademicGraphRelation,
    pub edge_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphReadinessV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub projection_id: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub graph_source_digest: String,
    pub last_successful_build: AcademicGraphBuildBindingV1,
    pub state: AcademicGraphReadinessState,
    pub reason_code: String,
    pub remediation: AcademicGraphReadinessRemediation,
    pub recognized_source_count: usize,
    pub present_source_count: usize,
    pub missing_source_count: usize,
    pub invalid_source_count: usize,
    pub unsupported_source_count: usize,
    pub stale_source_count: usize,
    pub node_count: usize,
    pub semantic_node_count: usize,
    pub connected_node_count: usize,
    pub isolated_node_count: usize,
    pub relation_count: usize,
    pub layer_counts: Vec<AcademicGraphLayerCountV1>,
    pub node_type_counts: Vec<AcademicGraphNodeTypeCountV1>,
    pub relation_counts: Vec<AcademicGraphRelationCountV1>,
    pub sources: Vec<AcademicGraphReadinessSourceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphProjectionV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub graph: AcademicGraphSnapshotV1,
    pub readiness: AcademicGraphReadinessV1,
}

impl AcademicGraphReadinessV1 {
    #[must_use]
    pub fn from_graph(graph: &AcademicGraphSnapshotV1) -> Self {
        Self::from_graph_and_last_successful(graph, None)
    }

    #[must_use]
    pub fn from_graph_and_last_successful(
        graph: &AcademicGraphSnapshotV1,
        last_successful: Option<&AcademicGraphSnapshotV1>,
    ) -> Self {
        let last_successful = last_successful.unwrap_or(graph);
        let node_counts = count_by_artifact(graph.nodes.iter().map(|node| &node.artifact_path));
        let edge_counts = count_by_artifact(graph.edges.iter().map(|edge| &edge.artifact_path));
        let diagnostic_counts = count_by_artifact(
            graph
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.artifact_path),
        );
        let unsupported_paths = graph
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == AcademicGraphDiagnosticCode::UnsupportedRelation
            })
            .map(|diagnostic| diagnostic.artifact_path.as_str())
            .collect::<BTreeSet<_>>();
        let invalid_paths = graph
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code != AcademicGraphDiagnosticCode::UnsupportedRelation
            })
            .map(|diagnostic| diagnostic.artifact_path.as_str())
            .collect::<BTreeSet<_>>();

        let sources = graph
            .sources
            .iter()
            .map(|source| {
                let freshness = last_successful
                    .sources
                    .iter()
                    .find(|previous| previous.artifact_path == source.artifact_path)
                    .filter(|previous| *previous == source)
                    .map_or(AcademicGraphSourceFreshness::Stale, |_| {
                        AcademicGraphSourceFreshness::Fresh
                    });
                let state = if !source.present {
                    AcademicGraphReadinessSourceState::Missing
                } else if unsupported_paths.contains(source.artifact_path.as_str()) {
                    AcademicGraphReadinessSourceState::Unsupported
                } else if invalid_paths.contains(source.artifact_path.as_str()) {
                    AcademicGraphReadinessSourceState::Invalid
                } else {
                    AcademicGraphReadinessSourceState::Present
                };
                AcademicGraphReadinessSourceV1 {
                    source_kind: source.source_kind,
                    artifact_path: source.artifact_path.clone(),
                    state,
                    freshness,
                    node_count: node_counts
                        .get(source.artifact_path.as_str())
                        .copied()
                        .unwrap_or_default(),
                    edge_count: edge_counts
                        .get(source.artifact_path.as_str())
                        .copied()
                        .unwrap_or_default(),
                    diagnostic_count: diagnostic_counts
                        .get(source.artifact_path.as_str())
                        .copied()
                        .unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        let present_non_manifest = graph
            .sources
            .iter()
            .filter(|source| {
                source.present && source.source_kind != AcademicGraphSourceKind::ProjectManifest
            })
            .count();
        let semantic_node_count = graph
            .nodes
            .iter()
            .filter(|node| node.node_type != AcademicGraphNodeType::Project)
            .count();
        let connected_node_ids = graph
            .edges
            .iter()
            .flat_map(|edge| [&edge.source_node_id, &edge.target_node_id])
            .collect::<BTreeSet<_>>();
        let connected_node_count = graph
            .nodes
            .iter()
            .filter(|node| connected_node_ids.contains(&node.node_id))
            .count();

        let invalid_source_count = sources
            .iter()
            .filter(|source| source.state == AcademicGraphReadinessSourceState::Invalid)
            .count();
        let unsupported_source_count = sources
            .iter()
            .filter(|source| source.state == AcademicGraphReadinessSourceState::Unsupported)
            .count();
        let stale_source_count = sources
            .iter()
            .filter(|source| source.freshness == AcademicGraphSourceFreshness::Stale)
            .count();
        let build_is_stale = graph.project_revision != last_successful.project_revision
            || graph.projection_id != last_successful.projection_id
            || graph.graph_source_digest != last_successful.graph_source_digest
            || stale_source_count > 0;
        let (state, remediation, reason_code) = if build_is_stale {
            (
                AcademicGraphReadinessState::Stale,
                AcademicGraphReadinessRemediation::RebuildGraph,
                "academic-graph-sources-stale",
            )
        } else {
            classify(
                present_non_manifest,
                graph.node_count,
                semantic_node_count,
                graph.edge_count,
                invalid_source_count,
                unsupported_source_count,
            )
        };

        Self {
            schema_version: ACADEMIC_GRAPH_READINESS_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_READINESS_DOCUMENT_KIND.to_owned(),
            projection_id: graph.projection_id.clone(),
            project_id: graph.project_id.clone(),
            project_revision: graph.project_revision,
            graph_source_digest: graph.graph_source_digest.clone(),
            last_successful_build: AcademicGraphBuildBindingV1::from_graph(last_successful),
            state,
            reason_code: reason_code.to_owned(),
            remediation,
            recognized_source_count: graph.source_count,
            present_source_count: graph.present_source_count,
            missing_source_count: graph
                .source_count
                .saturating_sub(graph.present_source_count),
            invalid_source_count,
            unsupported_source_count,
            stale_source_count,
            node_count: graph.node_count,
            semantic_node_count,
            connected_node_count,
            isolated_node_count: graph.node_count.saturating_sub(connected_node_count),
            relation_count: graph.edge_count,
            layer_counts: counts_by_layer(graph),
            node_type_counts: counts_by_node_type(graph),
            relation_counts: counts_by_relation(graph),
            sources,
        }
    }
}

fn classify(
    present_non_manifest: usize,
    node_count: usize,
    semantic_node_count: usize,
    edge_count: usize,
    invalid_source_count: usize,
    unsupported_source_count: usize,
) -> (
    AcademicGraphReadinessState,
    AcademicGraphReadinessRemediation,
    &'static str,
) {
    if present_non_manifest == 0 {
        return (
            AcademicGraphReadinessState::EmptyProject,
            AcademicGraphReadinessRemediation::AddCanonicalArtifacts,
            "academic-graph-empty-project",
        );
    }
    if semantic_node_count == 0 {
        return (
            AcademicGraphReadinessState::NoRecognizedArtifacts,
            AcademicGraphReadinessRemediation::RepairGraphArtifacts,
            "academic-graph-no-recognized-artifacts",
        );
    }
    if edge_count == 0 {
        return (
            AcademicGraphReadinessState::NodesWithoutEdges,
            AcademicGraphReadinessRemediation::AddSemanticRelations,
            "academic-graph-nodes-without-edges",
        );
    }
    if invalid_source_count > 0 || unsupported_source_count > 0 {
        return (
            AcademicGraphReadinessState::Sparse,
            AcademicGraphReadinessRemediation::RepairGraphArtifacts,
            "academic-graph-artifacts-need-repair",
        );
    }
    if semantic_node_count < 3 || edge_count.saturating_add(1) < node_count {
        return (
            AcademicGraphReadinessState::Sparse,
            AcademicGraphReadinessRemediation::EnrichGraph,
            "academic-graph-sparse",
        );
    }
    (
        AcademicGraphReadinessState::Visualizable,
        AcademicGraphReadinessRemediation::None,
        "academic-graph-visualizable",
    )
}

fn count_by_artifact<'a>(values: impl Iterator<Item = &'a String>) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.as_str()).or_default() += 1;
    }
    counts
}

fn counts_by_layer(graph: &AcademicGraphSnapshotV1) -> Vec<AcademicGraphLayerCountV1> {
    let mut counts = BTreeMap::new();
    for layer in graph
        .nodes
        .iter()
        .flat_map(|node| node.layers.iter().copied())
    {
        *counts.entry(layer).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(layer, node_count)| AcademicGraphLayerCountV1 { layer, node_count })
        .collect()
}

fn counts_by_node_type(graph: &AcademicGraphSnapshotV1) -> Vec<AcademicGraphNodeTypeCountV1> {
    let mut counts = BTreeMap::new();
    for node_type in graph.nodes.iter().map(|node| node.node_type) {
        *counts.entry(node_type).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(node_type, node_count)| AcademicGraphNodeTypeCountV1 {
            node_type,
            node_count,
        })
        .collect()
}

fn counts_by_relation(graph: &AcademicGraphSnapshotV1) -> Vec<AcademicGraphRelationCountV1> {
    let mut counts = BTreeMap::new();
    for relation in graph.edges.iter().map(|edge| edge.relation) {
        *counts.entry(relation).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(relation, edge_count)| AcademicGraphRelationCountV1 {
            relation,
            edge_count,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcademicGraphConfidence, AcademicGraphDiagnosticV1, AcademicGraphEdgeStatus,
        AcademicGraphEdgeV1, AcademicGraphIdentityScope, AcademicGraphNodeV1,
        AcademicGraphSourceRefV1, AcademicInferenceStrength, ProjectLifecycle, ProjectStage,
    };

    fn project_id() -> ProjectId {
        ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051").expect("valid project id")
    }

    fn fixture(
        present_artifact: bool,
        semantic_nodes: usize,
        connect_nodes: bool,
        diagnostics: Vec<AcademicGraphDiagnosticV1>,
    ) -> AcademicGraphSnapshotV1 {
        let project_id = project_id();
        let mut nodes = vec![
            AcademicGraphNodeV1::new(
                &project_id,
                AcademicGraphNodeType::Project,
                AcademicGraphIdentityScope::Project,
                project_id.as_str(),
                "Fixture project",
                vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
                "context/project_manifest.json",
                "project",
            )
            .expect("project node"),
        ];
        for index in 0..semantic_nodes {
            nodes.push(
                AcademicGraphNodeV1::new(
                    &project_id,
                    AcademicGraphNodeType::Claim,
                    AcademicGraphIdentityScope::Project,
                    format!("CLM-{index:03}"),
                    format!("Claim {index}"),
                    vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
                    "context/research_state.md",
                    format!("CLM-{index:03}"),
                )
                .expect("claim node"),
            );
        }
        let edges = if connect_nodes {
            nodes
                .windows(2)
                .enumerate()
                .map(|(index, pair)| {
                    AcademicGraphEdgeV1::new(
                        &project_id,
                        pair[0].node_id.clone(),
                        AcademicGraphRelation::Contains,
                        pair[1].node_id.clone(),
                        vec![AcademicGraphLayer::Combined],
                        "Fixture containment",
                        "context/research_state.md",
                        format!("edge-{index}"),
                        "Fixture only",
                        AcademicInferenceStrength::DirectEvidence,
                        AcademicGraphConfidence::High,
                        AcademicGraphEdgeStatus::Observed,
                        None,
                    )
                    .expect("edge")
                })
                .collect()
        } else {
            Vec::new()
        };
        let sources = vec![
            AcademicGraphSourceRefV1 {
                source_kind: AcademicGraphSourceKind::ProjectManifest,
                artifact_path: "context/project_manifest.json".to_owned(),
                present: true,
                content_digest: Some("a".repeat(64)),
                size_bytes: 128,
            },
            AcademicGraphSourceRefV1 {
                source_kind: AcademicGraphSourceKind::RegisteredArtifact,
                artifact_path: "context/research_state.md".to_owned(),
                present: present_artifact,
                content_digest: present_artifact.then(|| "b".repeat(64)),
                size_bytes: if present_artifact { 256 } else { 0 },
            },
        ];
        AcademicGraphSnapshotV1 {
            schema_version: 1,
            document_kind: "qiongli-academic-graph".to_owned(),
            projection_id: format!("grp_{}", "c".repeat(64)),
            projection_digest: "c".repeat(64),
            project_id,
            project_revision: 1,
            project_stage: ProjectStage::Writing,
            project_lifecycle: ProjectLifecycle::Active,
            project_manifest_digest: "d".repeat(64),
            project_semantic_digest: "e".repeat(64),
            graph_source_digest: "f".repeat(64),
            source_count: sources.len(),
            present_source_count: sources.iter().filter(|source| source.present).count(),
            node_count: nodes.len(),
            edge_count: edges.len(),
            diagnostic_count: diagnostics.len(),
            sources,
            nodes,
            edges,
            diagnostics,
        }
    }

    #[test]
    fn classifies_empty_unconnected_sparse_and_visualizable_graphs() {
        let empty = AcademicGraphReadinessV1::from_graph(&fixture(false, 0, false, Vec::new()));
        assert_eq!(empty.state, AcademicGraphReadinessState::EmptyProject);
        assert_eq!(empty.missing_source_count, 1);

        let unrecognized =
            AcademicGraphReadinessV1::from_graph(&fixture(true, 0, false, Vec::new()));
        assert_eq!(
            unrecognized.state,
            AcademicGraphReadinessState::NoRecognizedArtifacts
        );
        assert_eq!(
            unrecognized.remediation,
            AcademicGraphReadinessRemediation::RepairGraphArtifacts
        );

        let no_edges = AcademicGraphReadinessV1::from_graph(&fixture(true, 2, false, Vec::new()));
        assert_eq!(
            no_edges.state,
            AcademicGraphReadinessState::NodesWithoutEdges
        );
        assert_eq!(no_edges.isolated_node_count, 3);

        let sparse = AcademicGraphReadinessV1::from_graph(&fixture(true, 2, true, Vec::new()));
        assert_eq!(sparse.state, AcademicGraphReadinessState::Sparse);

        let visual = AcademicGraphReadinessV1::from_graph(&fixture(true, 3, true, Vec::new()));
        assert_eq!(visual.state, AcademicGraphReadinessState::Visualizable);
        assert_eq!(visual.connected_node_count, 4);
        assert_eq!(visual.relation_count, 3);
    }

    #[test]
    fn reports_unsupported_sources_without_exposing_absolute_paths() {
        let diagnostic = AcademicGraphDiagnosticV1 {
            code: AcademicGraphDiagnosticCode::UnsupportedRelation,
            artifact_path: "context/research_state.md".to_owned(),
            source_anchor: Some("line:1".to_owned()),
            related_id: None,
        };
        let readiness =
            AcademicGraphReadinessV1::from_graph(&fixture(true, 3, true, vec![diagnostic]));

        assert_eq!(readiness.state, AcademicGraphReadinessState::Sparse);
        assert_eq!(readiness.unsupported_source_count, 1);
        assert_eq!(
            readiness.remediation,
            AcademicGraphReadinessRemediation::RepairGraphArtifacts
        );
        assert!(
            readiness
                .sources
                .iter()
                .all(|source| !source.artifact_path.starts_with('/'))
        );
    }

    #[test]
    fn stale_state_is_bound_to_revision_digest_and_last_successful_build() {
        let previous = fixture(true, 3, true, Vec::new());
        let mut current = previous.clone();
        current.project_revision = 2;
        current.projection_id = format!("grp_{}", "1".repeat(64));
        current.projection_digest = "1".repeat(64);
        current.graph_source_digest = "2".repeat(64);
        current.sources[1].content_digest = Some("3".repeat(64));

        let readiness =
            AcademicGraphReadinessV1::from_graph_and_last_successful(&current, Some(&previous));

        assert_eq!(readiness.state, AcademicGraphReadinessState::Stale);
        assert_eq!(
            readiness.remediation,
            AcademicGraphReadinessRemediation::RebuildGraph
        );
        assert_eq!(readiness.project_revision, 2);
        assert_eq!(readiness.graph_source_digest, "2".repeat(64));
        assert_eq!(readiness.last_successful_build.project_revision, 1);
        assert_eq!(readiness.stale_source_count, 1);
        assert_eq!(
            readiness.sources[1].freshness,
            AcademicGraphSourceFreshness::Stale
        );
    }
}
