use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    AcademicGraphConfidence, AcademicGraphEdgeStatus, AcademicGraphIdentityScope,
    AcademicGraphNodeType, AcademicGraphRelation, AcademicGraphService, AcademicGraphSnapshotV1,
    AcademicInferenceStrength, ProjectError, ProjectHealth, ProjectId, ProjectLifecycle,
    ProjectStateService, ResearchLibrarySnapshotV1,
};

pub const ACADEMIC_GRAPH_PORTFOLIO_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_PORTFOLIO_DOCUMENT_KIND: &str = "qiongli-academic-graph-portfolio";
const MAX_PORTFOLIO_NODES: usize = 16_384;
const MAX_PORTFOLIO_EDGES: usize = 32_768;
const MAX_PORTFOLIO_OCCURRENCES: usize = 65_536;
const MAX_PORTFOLIO_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPortfolioProjectV1 {
    pub project_id: ProjectId,
    pub display_name: String,
    pub lifecycle: ProjectLifecycle,
    pub health: ProjectHealth,
    pub included: bool,
    pub project_revision: Option<u64>,
    pub projection_id: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPortfolioOccurrenceV1 {
    pub project_id: ProjectId,
    pub projection_id: String,
    pub graph_node_id: String,
    pub label: String,
    pub artifact_path: String,
    pub source_anchor: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPortfolioNodeV1 {
    pub node_id: String,
    pub node_type: AcademicGraphNodeType,
    pub identity_scope: AcademicGraphIdentityScope,
    pub canonical_id: String,
    pub label: String,
    pub project_ids: Vec<ProjectId>,
    pub occurrences: Vec<AcademicGraphPortfolioOccurrenceV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPortfolioEdgeOriginV1 {
    pub project_id: ProjectId,
    pub projection_id: String,
    pub graph_edge_id: Option<String>,
    pub artifact_path: String,
    pub source_anchor: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPortfolioEdgeV1 {
    pub edge_id: String,
    pub source_node_id: String,
    pub relation: AcademicGraphRelation,
    pub target_node_id: String,
    pub shared_canonical_id: Option<String>,
    pub rationale: String,
    pub evidence_limit: String,
    pub inference_strength: AcademicInferenceStrength,
    pub confidence: AcademicGraphConfidence,
    pub status: AcademicGraphEdgeStatus,
    pub origins: Vec<AcademicGraphPortfolioEdgeOriginV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPortfolioSnapshotV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub portfolio_id: String,
    pub library_revision: u64,
    pub project_count: usize,
    pub included_project_count: usize,
    pub skipped_project_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub projects: Vec<AcademicGraphPortfolioProjectV1>,
    pub nodes: Vec<AcademicGraphPortfolioNodeV1>,
    pub edges: Vec<AcademicGraphPortfolioEdgeV1>,
}

#[derive(Clone)]
pub struct AcademicGraphPortfolioService {
    projects: ProjectStateService,
}

impl AcademicGraphPortfolioService {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn rebuild(&self) -> Result<AcademicGraphPortfolioSnapshotV1, ProjectError> {
        let library = self.projects.snapshot()?;
        let graph_service = AcademicGraphService::new(self.projects.clone());
        let mut graphs = Vec::new();
        let mut ready_ids = library
            .projects
            .iter()
            .filter(|project| portfolio_project_is_included(project))
            .map(|project| project.project_id.clone())
            .collect::<Vec<_>>();
        ready_ids.sort_unstable();
        for project_id in ready_ids {
            graphs.push(graph_service.rebuild(&project_id)?);
        }
        let confirmed = self.projects.snapshot()?;
        if confirmed != library {
            return Err(ProjectError::RevisionConflict);
        }
        build_portfolio(&library, &graphs)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct GlobalIdentity {
    node_type: AcademicGraphNodeType,
    canonical_id: String,
}

#[derive(Serialize)]
struct PortfolioIdentity<'a> {
    schema_version: u32,
    library_revision: u64,
    projects: &'a [AcademicGraphPortfolioProjectV1],
    nodes: &'a [AcademicGraphPortfolioNodeV1],
    edges: &'a [AcademicGraphPortfolioEdgeV1],
}

pub(crate) fn build_portfolio(
    library: &ResearchLibrarySnapshotV1,
    graphs: &[AcademicGraphSnapshotV1],
) -> Result<AcademicGraphPortfolioSnapshotV1, ProjectError> {
    let graph_by_project = graphs
        .iter()
        .map(|graph| (graph.project_id.clone(), graph))
        .collect::<BTreeMap<_, _>>();
    if graph_by_project.len() != graphs.len() {
        return Err(ProjectError::InvalidGraphDocument);
    }

    let mut summaries = library.projects.iter().collect::<Vec<_>>();
    summaries.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    let projects = summaries
        .iter()
        .map(|summary| {
            let graph = graph_by_project.get(&summary.project_id).copied();
            if portfolio_project_is_included(summary) != graph.is_some()
                || graph.is_some_and(|graph| graph.project_revision != summary.semantic_revision)
            {
                return Err(ProjectError::RevisionConflict);
            }
            Ok(AcademicGraphPortfolioProjectV1 {
                project_id: summary.project_id.clone(),
                display_name: summary.display_name.clone(),
                lifecycle: summary.lifecycle,
                health: summary.health,
                included: graph.is_some(),
                project_revision: graph.map(|graph| graph.project_revision),
                projection_id: graph.map(|graph| graph.projection_id.clone()),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let included_ids = graphs
        .iter()
        .map(|graph| graph.project_id.clone())
        .collect::<BTreeSet<_>>();
    let mut nodes = BTreeMap::<String, AcademicGraphPortfolioNodeV1>::new();
    let mut project_node_ids = BTreeMap::<ProjectId, String>::new();
    let mut graph_nodes = BTreeMap::new();
    let mut global_occurrences =
        BTreeMap::<GlobalIdentity, Vec<AcademicGraphPortfolioOccurrenceV1>>::new();

    for graph in graphs {
        let graph_node_by_id = graph
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let project_node = graph.nodes.iter().find(|node| {
            node.node_type == AcademicGraphNodeType::Project
                && node.identity_scope == AcademicGraphIdentityScope::Project
                && node.canonical_id == graph.project_id.as_str()
        });
        let project_node = project_node.ok_or(ProjectError::InvalidGraphDocument)?;
        let portfolio_node_id = portfolio_node_id(
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Project,
            graph.project_id.as_str(),
        )?;
        let project_occurrence = occurrence(graph, project_node);
        nodes.insert(
            portfolio_node_id.clone(),
            AcademicGraphPortfolioNodeV1 {
                node_id: portfolio_node_id.clone(),
                node_type: AcademicGraphNodeType::Project,
                identity_scope: AcademicGraphIdentityScope::Project,
                canonical_id: graph.project_id.as_str().to_string(),
                label: project_node.label.clone(),
                project_ids: vec![graph.project_id.clone()],
                occurrences: vec![project_occurrence],
            },
        );
        project_node_ids.insert(graph.project_id.clone(), portfolio_node_id);

        for node in graph.nodes.iter().filter(|node| {
            node.identity_scope == AcademicGraphIdentityScope::Global
                && matches!(
                    node.node_type,
                    AcademicGraphNodeType::Paper
                        | AcademicGraphNodeType::Concept
                        | AcademicGraphNodeType::Method
                )
        }) {
            global_occurrences
                .entry(GlobalIdentity {
                    node_type: node.node_type,
                    canonical_id: node.canonical_id.clone(),
                })
                .or_default()
                .push(occurrence(graph, node));
        }
        graph_nodes.insert(graph.project_id.clone(), graph_node_by_id);
    }

    let mut edges = BTreeMap::<String, AcademicGraphPortfolioEdgeV1>::new();
    let mut occurrence_count = nodes.len();
    for (identity, mut occurrences) in global_occurrences {
        occurrences.sort_unstable();
        occurrences.dedup();
        let project_ids = occurrences
            .iter()
            .map(|occurrence| occurrence.project_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if project_ids.len() < 2 {
            continue;
        }
        occurrence_count = occurrence_count
            .checked_add(occurrences.len())
            .ok_or(ProjectError::InvalidGraphDocument)?;
        if occurrence_count > MAX_PORTFOLIO_OCCURRENCES {
            return Err(ProjectError::InvalidGraphDocument);
        }
        let shared_node_id = portfolio_node_id(
            identity.node_type,
            AcademicGraphIdentityScope::Global,
            &identity.canonical_id,
        )?;
        let label = occurrences
            .iter()
            .map(|occurrence| occurrence.label.as_str())
            .min()
            .ok_or(ProjectError::InvalidGraphDocument)?
            .to_string();
        nodes.insert(
            shared_node_id.clone(),
            AcademicGraphPortfolioNodeV1 {
                node_id: shared_node_id.clone(),
                node_type: identity.node_type,
                identity_scope: AcademicGraphIdentityScope::Global,
                canonical_id: identity.canonical_id.clone(),
                label,
                project_ids: project_ids.clone(),
                occurrences: occurrences.clone(),
            },
        );

        for project_id in project_ids {
            let source_node_id = project_node_ids
                .get(&project_id)
                .ok_or(ProjectError::InvalidGraphDocument)?;
            let relation = match identity.node_type {
                AcademicGraphNodeType::Paper => AcademicGraphRelation::SharesSource,
                AcademicGraphNodeType::Concept => AcademicGraphRelation::SharesConcept,
                AcademicGraphNodeType::Method => AcademicGraphRelation::UsesMethod,
                _ => return Err(ProjectError::InvalidGraphDocument),
            };
            let origins = occurrences
                .iter()
                .filter(|occurrence| occurrence.project_id == project_id)
                .map(|occurrence| AcademicGraphPortfolioEdgeOriginV1 {
                    project_id: occurrence.project_id.clone(),
                    projection_id: occurrence.projection_id.clone(),
                    graph_edge_id: None,
                    artifact_path: occurrence.artifact_path.clone(),
                    source_anchor: occurrence.source_anchor.clone(),
                })
                .collect::<Vec<_>>();
            let edge_id = portfolio_edge_id(
                source_node_id,
                relation,
                &shared_node_id,
                Some(&identity.canonical_id),
                &origins,
            )?;
            edges.insert(
                edge_id.clone(),
                AcademicGraphPortfolioEdgeV1 {
                    edge_id,
                    source_node_id: source_node_id.clone(),
                    relation,
                    target_node_id: shared_node_id.clone(),
                    shared_canonical_id: Some(identity.canonical_id.clone()),
                    rationale: shared_rationale(identity.node_type),
                    evidence_limit: "This edge records an exact reused identifier across registered project projections; it does not imply shared conclusions, authorship, or evidence strength."
                        .to_string(),
                    inference_strength: AcademicInferenceStrength::DirectEvidence,
                    confidence: AcademicGraphConfidence::High,
                    status: AcademicGraphEdgeStatus::Observed,
                    origins,
                },
            );
        }
    }

    for graph in graphs {
        let node_by_id = graph_nodes
            .get(&graph.project_id)
            .ok_or(ProjectError::InvalidGraphDocument)?;
        for edge in graph.edges.iter().filter(|edge| {
            matches!(
                edge.relation,
                AcademicGraphRelation::ForkedFrom | AcademicGraphRelation::ExtendsProject
            )
        }) {
            let source = node_by_id
                .get(edge.source_node_id.as_str())
                .ok_or(ProjectError::InvalidGraphDocument)?;
            let target = node_by_id
                .get(edge.target_node_id.as_str())
                .ok_or(ProjectError::InvalidGraphDocument)?;
            if source.node_type != AcademicGraphNodeType::Project
                || target.node_type != AcademicGraphNodeType::Project
            {
                return Err(ProjectError::InvalidGraphDocument);
            }
            let source_project = ProjectId::parse(source.canonical_id.clone())?;
            let target_project = ProjectId::parse(target.canonical_id.clone())?;
            if !included_ids.contains(&source_project) || !included_ids.contains(&target_project) {
                continue;
            }
            let source_node_id = project_node_ids
                .get(&source_project)
                .ok_or(ProjectError::InvalidGraphDocument)?;
            let target_node_id = project_node_ids
                .get(&target_project)
                .ok_or(ProjectError::InvalidGraphDocument)?;
            let origins = vec![AcademicGraphPortfolioEdgeOriginV1 {
                project_id: graph.project_id.clone(),
                projection_id: graph.projection_id.clone(),
                graph_edge_id: Some(edge.edge_id.clone()),
                artifact_path: edge.artifact_path.clone(),
                source_anchor: edge.source_anchor.clone(),
            }];
            let edge_id = portfolio_edge_id(
                source_node_id,
                edge.relation,
                target_node_id,
                None,
                &origins,
            )?;
            edges.insert(
                edge_id.clone(),
                AcademicGraphPortfolioEdgeV1 {
                    edge_id,
                    source_node_id: source_node_id.clone(),
                    relation: edge.relation,
                    target_node_id: target_node_id.clone(),
                    shared_canonical_id: None,
                    rationale: edge.rationale.clone(),
                    evidence_limit: edge.evidence_limit.clone(),
                    inference_strength: edge.inference_strength,
                    confidence: edge.confidence,
                    status: edge.status,
                    origins,
                },
            );
        }
    }

    if nodes.len() > MAX_PORTFOLIO_NODES || edges.len() > MAX_PORTFOLIO_EDGES {
        return Err(ProjectError::InvalidGraphDocument);
    }
    let nodes = nodes.into_values().collect::<Vec<_>>();
    let edges = edges.into_values().collect::<Vec<_>>();
    let identity = PortfolioIdentity {
        schema_version: ACADEMIC_GRAPH_PORTFOLIO_SCHEMA_VERSION,
        library_revision: library.revision,
        projects: &projects,
        nodes: &nodes,
        edges: &edges,
    };
    let bytes = serde_json_canonicalizer::to_vec(&identity)
        .map_err(|_| ProjectError::InvalidGraphDocument)?;
    let portfolio_id = format!("gpf_{:x}", Sha256::digest(&bytes));
    let snapshot = AcademicGraphPortfolioSnapshotV1 {
        schema_version: ACADEMIC_GRAPH_PORTFOLIO_SCHEMA_VERSION,
        document_kind: ACADEMIC_GRAPH_PORTFOLIO_DOCUMENT_KIND.to_string(),
        portfolio_id,
        library_revision: library.revision,
        project_count: projects.len(),
        included_project_count: projects.iter().filter(|project| project.included).count(),
        skipped_project_count: projects.iter().filter(|project| !project.included).count(),
        node_count: nodes.len(),
        edge_count: edges.len(),
        projects,
        nodes,
        edges,
    };
    let snapshot_bytes = serde_json_canonicalizer::to_vec(&snapshot)
        .map_err(|_| ProjectError::InvalidGraphDocument)?;
    if snapshot_bytes.len() > MAX_PORTFOLIO_BYTES {
        return Err(ProjectError::InvalidGraphDocument);
    }
    Ok(snapshot)
}

pub(crate) fn portfolio_project_is_included(project: &crate::ArticleProjectSummaryV1) -> bool {
    project.health == ProjectHealth::Ready && project.lifecycle == ProjectLifecycle::Active
}

fn occurrence(
    graph: &AcademicGraphSnapshotV1,
    node: &crate::AcademicGraphNodeV1,
) -> AcademicGraphPortfolioOccurrenceV1 {
    AcademicGraphPortfolioOccurrenceV1 {
        project_id: graph.project_id.clone(),
        projection_id: graph.projection_id.clone(),
        graph_node_id: node.node_id.clone(),
        label: node.label.clone(),
        artifact_path: node.artifact_path.clone(),
        source_anchor: node.source_anchor.clone(),
    }
}

fn shared_rationale(node_type: AcademicGraphNodeType) -> String {
    match node_type {
        AcademicGraphNodeType::Paper => {
            "The registered project projections contain the same exact global paper identifier."
        }
        AcademicGraphNodeType::Concept => {
            "The registered project projections contain the same exact global concept identifier."
        }
        AcademicGraphNodeType::Method => {
            "The registered project projections contain the same exact global method identifier."
        }
        _ => "The registered project projections contain the same exact global identifier.",
    }
    .to_string()
}

fn portfolio_node_id(
    node_type: AcademicGraphNodeType,
    identity_scope: AcademicGraphIdentityScope,
    canonical_id: &str,
) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(&(node_type, identity_scope, canonical_id))
        .map_err(|_| ProjectError::InvalidGraphDocument)?;
    Ok(format!("pnd_{:x}", Sha256::digest(bytes)))
}

fn portfolio_edge_id(
    source_node_id: &str,
    relation: AcademicGraphRelation,
    target_node_id: &str,
    shared_canonical_id: Option<&str>,
    origins: &[AcademicGraphPortfolioEdgeOriginV1],
) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(&(
        source_node_id,
        relation,
        target_node_id,
        shared_canonical_id,
        origins,
    ))
    .map_err(|_| ProjectError::InvalidGraphDocument)?;
    Ok(format!("ped_{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcademicGraphEdgeV1, AcademicGraphLayer, AcademicGraphNodeV1, AcademicGraphSourceKind,
        AcademicGraphSourceRefV1, ArticleProjectSummaryV1, LibraryHealth, ProjectKind,
        ProjectNextAction, ProjectOverviewV1, ProjectStage,
    };

    fn project_id(value: char) -> ProjectId {
        ProjectId::parse(format!("prj_{}", value.to_string().repeat(32)))
            .expect("project id is valid")
    }

    fn summary(project_id: ProjectId, name: &str) -> ArticleProjectSummaryV1 {
        ArticleProjectSummaryV1 {
            project_id,
            display_name: name.to_string(),
            project_kind: ProjectKind::Article,
            stage: ProjectStage::Writing,
            lifecycle: ProjectLifecycle::Active,
            semantic_revision: 1,
            registered_at_unix: 1,
            last_opened_at_unix: None,
            academically_updated_at_unix: 1,
            health: ProjectHealth::Ready,
            next_action: ProjectNextAction::Open,
            root_label: name.to_lowercase().replace(' ', "-"),
            overview: ProjectOverviewV1 {
                focal_question: None,
                thesis: None,
                evidence_position: None,
                unresolved_risk_count: 0,
                claim_evidence_coverage_percent: None,
                next_priorities: Vec::new(),
            },
        }
    }

    fn graph(
        project_id: ProjectId,
        name: &str,
        parent: Option<ProjectId>,
    ) -> AcademicGraphSnapshotV1 {
        let project = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Project,
            project_id.as_str(),
            name,
            vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
            "context/project_manifest.json",
            "#/project_id",
        )
        .expect("project node is valid");
        let paper = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Paper,
            AcademicGraphIdentityScope::Global,
            "doi:10.1000/shared",
            if name == "Project A" {
                "Shared source"
            } else {
                "A differently formatted shared source"
            },
            vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
            "literature/literature_map.md",
            "paper:shared",
        )
        .expect("paper node is valid");
        let mut nodes = vec![project.clone(), paper];
        let mut edges = Vec::new();
        if let Some(parent) = parent {
            let parent_node = AcademicGraphNodeV1::new(
                &project_id,
                AcademicGraphNodeType::Project,
                AcademicGraphIdentityScope::Global,
                parent.as_str(),
                "Parent project",
                vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
                "graph/semantic_links.jsonl",
                "line:1",
            )
            .expect("external project node is valid");
            edges.push(
                AcademicGraphEdgeV1::new(
                    &project_id,
                    &project.node_id,
                    AcademicGraphRelation::ForkedFrom,
                    &parent_node.node_id,
                    vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
                    "The reviewed semantic link records explicit project ancestry.",
                    "graph/semantic_links.jsonl",
                    "line:1",
                    "Lineage does not imply identical conclusions.",
                    AcademicInferenceStrength::DirectEvidence,
                    AcademicGraphConfidence::High,
                    AcademicGraphEdgeStatus::Reviewed,
                    None,
                )
                .expect("lineage edge is valid"),
            );
            nodes.push(parent_node);
        }
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
        let projection_digest = if name == "Project A" {
            "a".repeat(64)
        } else {
            "b".repeat(64)
        };
        AcademicGraphSnapshotV1 {
            schema_version: 1,
            document_kind: "qiongli-academic-graph".to_string(),
            projection_id: format!("grp_{projection_digest}"),
            projection_digest,
            project_id,
            project_revision: 1,
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
    fn portfolio_merges_only_exact_global_identity_and_preserves_lineage() {
        let first_id = project_id('1');
        let second_id = project_id('2');
        let library = ResearchLibrarySnapshotV1 {
            schema_version: 1,
            revision: 2,
            health: LibraryHealth::Ready,
            projects: vec![
                summary(first_id.clone(), "Project A"),
                summary(second_id.clone(), "Project B"),
            ],
        };
        let graphs = vec![
            graph(first_id.clone(), "Project A", None),
            graph(second_id.clone(), "Project B", Some(first_id.clone())),
        ];

        let portfolio = build_portfolio(&library, &graphs).expect("portfolio builds");
        assert_eq!(portfolio.project_count, 2);
        assert_eq!(portfolio.included_project_count, 2);
        let paper = portfolio
            .nodes
            .iter()
            .find(|node| node.node_type == AcademicGraphNodeType::Paper)
            .expect("one exact shared paper hub exists");
        assert_eq!(paper.canonical_id, "doi:10.1000/shared");
        assert_eq!(paper.project_ids, vec![first_id, second_id]);
        assert_eq!(paper.occurrences.len(), 2);
        assert_eq!(
            portfolio
                .edges
                .iter()
                .filter(|edge| edge.relation == AcademicGraphRelation::SharesSource)
                .count(),
            2
        );
        assert_eq!(
            portfolio
                .edges
                .iter()
                .filter(|edge| edge.relation == AcademicGraphRelation::ForkedFrom)
                .count(),
            1
        );
    }

    #[test]
    fn portfolio_identity_is_independent_of_input_graph_order() {
        let first_id = project_id('1');
        let second_id = project_id('2');
        let library = ResearchLibrarySnapshotV1 {
            schema_version: 1,
            revision: 2,
            health: LibraryHealth::Ready,
            projects: vec![
                summary(first_id.clone(), "Project A"),
                summary(second_id.clone(), "Project B"),
            ],
        };
        let first = graph(first_id, "Project A", None);
        let second = graph(second_id, "Project B", None);
        let forward =
            build_portfolio(&library, &[first.clone(), second.clone()]).expect("portfolio builds");
        let reverse = build_portfolio(&library, &[second, first]).expect("portfolio builds");
        assert_eq!(forward, reverse);
    }

    #[test]
    fn large_portfolio_fixture_remains_deterministic_and_exactly_bounded() {
        let project_ids = (1_u128..=64)
            .map(|value| ProjectId::parse(format!("prj_{value:032x}")).unwrap())
            .collect::<Vec<_>>();
        let library = ResearchLibrarySnapshotV1 {
            schema_version: 1,
            revision: 64,
            health: LibraryHealth::Ready,
            projects: project_ids
                .iter()
                .enumerate()
                .map(|(position, project_id)| {
                    summary(
                        project_id.clone(),
                        &format!("Portfolio project {position:02}"),
                    )
                })
                .collect(),
        };
        let graphs = project_ids
            .iter()
            .enumerate()
            .map(|(position, project_id)| {
                graph(
                    project_id.clone(),
                    &format!("Portfolio project {position:02}"),
                    None,
                )
            })
            .collect::<Vec<_>>();

        let forward = build_portfolio(&library, &graphs).expect("large portfolio builds");
        let reverse = build_portfolio(&library, &graphs.iter().cloned().rev().collect::<Vec<_>>())
            .expect("reordered large portfolio builds");
        assert_eq!(forward, reverse);
        assert_eq!(forward.project_count, 64);
        assert_eq!(forward.included_project_count, 64);
        assert_eq!(forward.skipped_project_count, 0);
        assert_eq!(forward.node_count, 65);
        assert_eq!(forward.edge_count, 64);
        let shared = forward
            .nodes
            .iter()
            .find(|node| node.node_type == AcademicGraphNodeType::Paper)
            .expect("one shared exact paper identity exists");
        assert_eq!(shared.occurrences.len(), 64);
        assert_eq!(shared.project_ids.len(), 64);
    }
}
