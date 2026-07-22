use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    AcademicGraphEdgeV1, AcademicGraphLayer, AcademicGraphNodeType, AcademicGraphNodeV1,
    AcademicGraphRelation, AcademicGraphService, AcademicGraphSnapshotV1, ProjectError, ProjectId,
    ProjectStateService,
};

pub const ACADEMIC_GRAPH_INDEX_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_INDEX_DOCUMENT_KIND: &str = "qiongli-academic-graph-index";
pub const ACADEMIC_GRAPH_QUERY_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_QUERY_DOCUMENT_KIND: &str = "qiongli-academic-graph-query-result";
pub const ACADEMIC_GRAPH_PATH_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_PATH_DOCUMENT_KIND: &str = "qiongli-academic-graph-explanatory-path";
pub const MAX_ACADEMIC_GRAPH_PATH_HOPS: usize = 12;
const MAX_QUERY_TEXT_BYTES: usize = 256;
const MAX_QUERY_NODES: usize = 256;
const MAX_QUERY_EDGES: usize = 512;
type AcademicGraphPathEdge = (usize, AcademicGraphPathTraversal);
type ReconstructedAcademicGraphPath = (Vec<String>, Vec<AcademicGraphPathEdge>);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphDirection {
    Incoming,
    Outgoing,
    Both,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphQueryV1 {
    pub expected_projection_id: String,
    pub focus_node_id: Option<String>,
    pub direction: AcademicGraphDirection,
    pub node_types: Vec<AcademicGraphNodeType>,
    pub relations: Vec<AcademicGraphRelation>,
    pub layers: Vec<AcademicGraphLayer>,
    pub canonical_id: Option<String>,
    pub text: Option<String>,
    pub max_nodes: usize,
    pub max_edges: usize,
}

impl AcademicGraphQueryV1 {
    #[must_use]
    pub fn new(expected_projection_id: impl Into<String>) -> Self {
        Self {
            expected_projection_id: expected_projection_id.into(),
            focus_node_id: None,
            direction: AcademicGraphDirection::Both,
            node_types: Vec::new(),
            relations: Vec::new(),
            layers: Vec::new(),
            canonical_id: None,
            text: None,
            max_nodes: 100,
            max_edges: 200,
        }
    }

    #[must_use]
    pub fn with_focus(
        mut self,
        node_id: impl Into<String>,
        direction: AcademicGraphDirection,
    ) -> Self {
        self.focus_node_id = Some(node_id.into());
        self.direction = direction;
        self
    }

    #[must_use]
    pub fn with_node_types(mut self, mut node_types: Vec<AcademicGraphNodeType>) -> Self {
        node_types.sort_unstable();
        node_types.dedup();
        self.node_types = node_types;
        self
    }

    #[must_use]
    pub fn with_relations(mut self, mut relations: Vec<AcademicGraphRelation>) -> Self {
        relations.sort_unstable();
        relations.dedup();
        self.relations = relations;
        self
    }

    #[must_use]
    pub fn with_layers(mut self, mut layers: Vec<AcademicGraphLayer>) -> Self {
        layers.sort_unstable();
        layers.dedup();
        self.layers = layers;
        self
    }

    #[must_use]
    pub fn with_canonical_id(mut self, canonical_id: impl Into<String>) -> Self {
        self.canonical_id = Some(canonical_id.into());
        self
    }

    #[must_use]
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    #[must_use]
    pub const fn with_limits(mut self, max_nodes: usize, max_edges: usize) -> Self {
        self.max_nodes = max_nodes;
        self.max_edges = max_edges;
        self
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_hashed_id(&self.expected_projection_id, "grp_")
            || self
                .focus_node_id
                .as_deref()
                .is_some_and(|node_id| !valid_hashed_id(node_id, "nod_"))
            || self.node_types.len() > 15
            || !sorted_unique(&self.node_types)
            || self.relations.len() > 25
            || !sorted_unique(&self.relations)
            || self.layers.len() > 6
            || !sorted_unique(&self.layers)
            || self
                .canonical_id
                .as_deref()
                .is_some_and(|value| !valid_query_text(value, MAX_QUERY_TEXT_BYTES))
            || self
                .text
                .as_deref()
                .is_some_and(|value| !valid_query_text(value, MAX_QUERY_TEXT_BYTES))
            || !(1..=MAX_QUERY_NODES).contains(&self.max_nodes)
            || !(1..=MAX_QUERY_EDGES).contains(&self.max_edges)
        {
            return Err(ProjectError::InvalidGraphQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphQueryResultV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub index_id: String,
    pub projection_id: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub matched_node_count: usize,
    pub matched_edge_count: usize,
    pub nodes_truncated: bool,
    pub edges_truncated: bool,
    pub nodes: Vec<AcademicGraphNodeV1>,
    pub edges: Vec<AcademicGraphEdgeV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphPathStatus {
    Found,
    NotFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphPathTraversal {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphPathQueryV1 {
    pub expected_projection_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub max_hops: usize,
}

impl AcademicGraphPathQueryV1 {
    #[must_use]
    pub fn new(
        expected_projection_id: impl Into<String>,
        source_node_id: impl Into<String>,
        target_node_id: impl Into<String>,
    ) -> Self {
        Self {
            expected_projection_id: expected_projection_id.into(),
            source_node_id: source_node_id.into(),
            target_node_id: target_node_id.into(),
            max_hops: 6,
        }
    }

    #[must_use]
    pub const fn with_max_hops(mut self, max_hops: usize) -> Self {
        self.max_hops = max_hops;
        self
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_hashed_id(&self.expected_projection_id, "grp_")
            || !valid_hashed_id(&self.source_node_id, "nod_")
            || !valid_hashed_id(&self.target_node_id, "nod_")
            || !(1..=MAX_ACADEMIC_GRAPH_PATH_HOPS).contains(&self.max_hops)
        {
            return Err(ProjectError::InvalidGraphQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPathStepV1 {
    pub sequence: usize,
    pub from_node_id: String,
    pub edge_id: String,
    pub to_node_id: String,
    pub traversal: AcademicGraphPathTraversal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicGraphPathResultV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub index_id: String,
    pub projection_id: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub source_node_id: String,
    pub target_node_id: String,
    pub max_hops: usize,
    pub status: AcademicGraphPathStatus,
    pub hop_count: usize,
    pub nodes: Vec<AcademicGraphNodeV1>,
    pub edges: Vec<AcademicGraphEdgeV1>,
    pub steps: Vec<AcademicGraphPathStepV1>,
}

#[derive(Clone)]
pub struct AcademicGraphIndexV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub index_id: String,
    pub projection_id: String,
    pub projection_digest: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub project_semantic_digest: String,
    pub node_count: usize,
    pub edge_count: usize,
    snapshot: AcademicGraphSnapshotV1,
    node_positions: BTreeMap<String, usize>,
    incoming: BTreeMap<String, Vec<usize>>,
    outgoing: BTreeMap<String, Vec<usize>>,
}

impl AcademicGraphIndexV1 {
    pub(crate) fn from_snapshot(snapshot: AcademicGraphSnapshotV1) -> Result<Self, ProjectError> {
        let node_positions = snapshot
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (node.node_id.clone(), position))
            .collect::<BTreeMap<_, _>>();
        let mut incoming = BTreeMap::<String, Vec<usize>>::new();
        let mut outgoing = BTreeMap::<String, Vec<usize>>::new();
        for (position, edge) in snapshot.edges.iter().enumerate() {
            if !node_positions.contains_key(&edge.source_node_id)
                || !node_positions.contains_key(&edge.target_node_id)
            {
                return Err(ProjectError::InvalidGraphDocument);
            }
            outgoing
                .entry(edge.source_node_id.clone())
                .or_default()
                .push(position);
            incoming
                .entry(edge.target_node_id.clone())
                .or_default()
                .push(position);
        }
        let index_id = index_id(&snapshot)?;
        Ok(Self {
            schema_version: ACADEMIC_GRAPH_INDEX_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_INDEX_DOCUMENT_KIND.to_string(),
            index_id,
            projection_id: snapshot.projection_id.clone(),
            projection_digest: snapshot.projection_digest.clone(),
            project_id: snapshot.project_id.clone(),
            project_revision: snapshot.project_revision,
            project_semantic_digest: snapshot.project_semantic_digest.clone(),
            node_count: snapshot.node_count,
            edge_count: snapshot.edge_count,
            snapshot,
            node_positions,
            incoming,
            outgoing,
        })
    }

    pub fn query(
        &self,
        query: &AcademicGraphQueryV1,
    ) -> Result<AcademicGraphQueryResultV1, ProjectError> {
        query.validate()?;
        if query.expected_projection_id != self.projection_id {
            return Err(ProjectError::RevisionConflict);
        }

        let edge_positions = self.edge_positions(query)?;
        let mut candidate_nodes = if query.focus_node_id.is_some() || !query.relations.is_empty() {
            let mut candidates = BTreeSet::new();
            if let Some(focus) = &query.focus_node_id {
                candidates.insert(focus.clone());
            }
            for position in &edge_positions {
                let edge = &self.snapshot.edges[*position];
                candidates.insert(edge.source_node_id.clone());
                candidates.insert(edge.target_node_id.clone());
            }
            candidates
        } else {
            self.node_positions.keys().cloned().collect()
        };

        let normalized_text = query.text.as_ref().map(|value| value.to_lowercase());
        candidate_nodes.retain(|node_id| {
            let node = &self.snapshot.nodes[self.node_positions[node_id]];
            (query.node_types.is_empty() || query.node_types.contains(&node.node_type))
                && layer_matches(&node.layers, &query.layers)
                && query
                    .canonical_id
                    .as_ref()
                    .is_none_or(|value| value == &node.canonical_id)
                && normalized_text.as_ref().is_none_or(|value| {
                    node.label.to_lowercase().contains(value)
                        || node.canonical_id.to_lowercase().contains(value)
                })
        });

        let matched_node_count = candidate_nodes.len();
        let selected_node_ids = candidate_nodes
            .into_iter()
            .take(query.max_nodes)
            .collect::<BTreeSet<_>>();
        let nodes = selected_node_ids
            .iter()
            .map(|node_id| self.snapshot.nodes[self.node_positions[node_id]].clone())
            .collect::<Vec<_>>();

        let matching_edges = edge_positions
            .into_iter()
            .map(|position| &self.snapshot.edges[position])
            .filter(|edge| {
                selected_node_ids.contains(&edge.source_node_id)
                    && selected_node_ids.contains(&edge.target_node_id)
            })
            .collect::<Vec<_>>();
        let matched_edge_count = matching_edges.len();
        let edges = matching_edges
            .into_iter()
            .take(query.max_edges)
            .cloned()
            .collect::<Vec<_>>();

        Ok(AcademicGraphQueryResultV1 {
            schema_version: ACADEMIC_GRAPH_QUERY_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_QUERY_DOCUMENT_KIND.to_string(),
            index_id: self.index_id.clone(),
            projection_id: self.projection_id.clone(),
            project_id: self.project_id.clone(),
            project_revision: self.project_revision,
            matched_node_count,
            matched_edge_count,
            nodes_truncated: matched_node_count > nodes.len(),
            edges_truncated: matched_edge_count > edges.len(),
            nodes,
            edges,
        })
    }

    pub fn explanatory_path(
        &self,
        query: &AcademicGraphPathQueryV1,
    ) -> Result<AcademicGraphPathResultV1, ProjectError> {
        query.validate()?;
        if query.expected_projection_id != self.projection_id {
            return Err(ProjectError::RevisionConflict);
        }
        if !self.node_positions.contains_key(&query.source_node_id)
            || !self.node_positions.contains_key(&query.target_node_id)
        {
            return Err(ProjectError::GraphEntityNotFound);
        }
        if query.source_node_id == query.target_node_id {
            return Ok(self.path_result(
                query,
                AcademicGraphPathStatus::Found,
                vec![query.source_node_id.clone()],
                Vec::new(),
            ));
        }

        let mut queue = VecDeque::from([(query.source_node_id.clone(), 0_usize)]);
        let mut visited = BTreeSet::from([query.source_node_id.clone()]);
        let mut predecessors =
            BTreeMap::<String, (String, usize, AcademicGraphPathTraversal)>::new();

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth >= query.max_hops {
                continue;
            }
            for (edge_position, neighbor_id, traversal) in self.adjacent(&node_id) {
                if !visited.insert(neighbor_id.clone()) {
                    continue;
                }
                predecessors.insert(
                    neighbor_id.clone(),
                    (node_id.clone(), edge_position, traversal),
                );
                if neighbor_id == query.target_node_id {
                    let (node_ids, edge_positions) = reconstruct_path(
                        &query.source_node_id,
                        &query.target_node_id,
                        &predecessors,
                    )?;
                    return Ok(self.path_result(
                        query,
                        AcademicGraphPathStatus::Found,
                        node_ids,
                        edge_positions,
                    ));
                }
                queue.push_back((neighbor_id, depth + 1));
            }
        }

        Ok(self.path_result(
            query,
            AcademicGraphPathStatus::NotFound,
            Vec::new(),
            Vec::new(),
        ))
    }

    fn adjacent(&self, node_id: &str) -> Vec<(usize, String, AcademicGraphPathTraversal)> {
        let mut adjacent = Vec::new();
        for position in self.outgoing.get(node_id).into_iter().flatten() {
            let edge = &self.snapshot.edges[*position];
            adjacent.push((
                *position,
                edge.target_node_id.clone(),
                AcademicGraphPathTraversal::Forward,
            ));
        }
        for position in self.incoming.get(node_id).into_iter().flatten() {
            let edge = &self.snapshot.edges[*position];
            adjacent.push((
                *position,
                edge.source_node_id.clone(),
                AcademicGraphPathTraversal::Reverse,
            ));
        }
        adjacent.sort_by(|left, right| {
            self.snapshot.edges[left.0]
                .edge_id
                .cmp(&self.snapshot.edges[right.0].edge_id)
                .then_with(|| left.1.cmp(&right.1))
        });
        adjacent.dedup_by_key(|entry| entry.0);
        adjacent
    }

    fn path_result(
        &self,
        query: &AcademicGraphPathQueryV1,
        status: AcademicGraphPathStatus,
        node_ids: Vec<String>,
        edge_positions: Vec<AcademicGraphPathEdge>,
    ) -> AcademicGraphPathResultV1 {
        let nodes = node_ids
            .iter()
            .map(|node_id| self.snapshot.nodes[self.node_positions[node_id]].clone())
            .collect::<Vec<_>>();
        let edges = edge_positions
            .iter()
            .map(|(position, _)| self.snapshot.edges[*position].clone())
            .collect::<Vec<_>>();
        let steps = edge_positions
            .iter()
            .enumerate()
            .map(
                |(position, (edge_position, traversal))| AcademicGraphPathStepV1 {
                    sequence: position + 1,
                    from_node_id: node_ids[position].clone(),
                    edge_id: self.snapshot.edges[*edge_position].edge_id.clone(),
                    to_node_id: node_ids[position + 1].clone(),
                    traversal: *traversal,
                },
            )
            .collect::<Vec<_>>();
        AcademicGraphPathResultV1 {
            schema_version: ACADEMIC_GRAPH_PATH_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_PATH_DOCUMENT_KIND.to_string(),
            index_id: self.index_id.clone(),
            projection_id: self.projection_id.clone(),
            project_id: self.project_id.clone(),
            project_revision: self.project_revision,
            source_node_id: query.source_node_id.clone(),
            target_node_id: query.target_node_id.clone(),
            max_hops: query.max_hops,
            status,
            hop_count: edges.len(),
            nodes,
            edges,
            steps,
        }
    }

    fn edge_positions(&self, query: &AcademicGraphQueryV1) -> Result<Vec<usize>, ProjectError> {
        let positions: Vec<usize> = if let Some(focus) = &query.focus_node_id {
            if !self.node_positions.contains_key(focus) {
                return Err(ProjectError::InvalidGraphQuery);
            }
            let mut positions = BTreeSet::new();
            if matches!(
                query.direction,
                AcademicGraphDirection::Incoming | AcademicGraphDirection::Both
            ) {
                positions.extend(self.incoming.get(focus).into_iter().flatten().copied());
            }
            if matches!(
                query.direction,
                AcademicGraphDirection::Outgoing | AcademicGraphDirection::Both
            ) {
                positions.extend(self.outgoing.get(focus).into_iter().flatten().copied());
            }
            positions.into_iter().collect()
        } else {
            (0..self.snapshot.edges.len()).collect()
        };
        Ok(positions
            .into_iter()
            .filter(|position| {
                let edge = &self.snapshot.edges[*position];
                (query.relations.is_empty() || query.relations.contains(&edge.relation))
                    && layer_matches(&edge.layers, &query.layers)
            })
            .collect())
    }
}

fn reconstruct_path(
    source_node_id: &str,
    target_node_id: &str,
    predecessors: &BTreeMap<String, (String, usize, AcademicGraphPathTraversal)>,
) -> Result<ReconstructedAcademicGraphPath, ProjectError> {
    let mut node_ids = vec![target_node_id.to_string()];
    let mut edge_positions = Vec::new();
    let mut current = target_node_id;
    while current != source_node_id {
        let (parent, edge_position, traversal) = predecessors
            .get(current)
            .ok_or(ProjectError::InvalidGraphDocument)?;
        node_ids.push(parent.clone());
        edge_positions.push((*edge_position, *traversal));
        current = parent;
    }
    node_ids.reverse();
    edge_positions.reverse();
    Ok((node_ids, edge_positions))
}

#[derive(Clone)]
pub struct AcademicGraphIndexService {
    graph: AcademicGraphService,
}

impl AcademicGraphIndexService {
    #[must_use]
    pub fn new(projects: ProjectStateService) -> Self {
        Self {
            graph: AcademicGraphService::new(projects),
        }
    }

    pub fn rebuild(&self, project_id: &ProjectId) -> Result<AcademicGraphIndexV1, ProjectError> {
        AcademicGraphIndexV1::from_snapshot(self.graph.rebuild(project_id)?)
    }
}

fn index_id(snapshot: &AcademicGraphSnapshotV1) -> Result<String, ProjectError> {
    #[derive(Serialize)]
    struct Identity<'a> {
        projection_id: &'a str,
        projection_digest: &'a str,
        project_semantic_digest: &'a str,
        node_ids: Vec<&'a str>,
        edge_ids: Vec<&'a str>,
    }
    let bytes = serde_json_canonicalizer::to_vec(&Identity {
        projection_id: &snapshot.projection_id,
        projection_digest: &snapshot.projection_digest,
        project_semantic_digest: &snapshot.project_semantic_digest,
        node_ids: snapshot
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect(),
        edge_ids: snapshot
            .edges
            .iter()
            .map(|edge| edge.edge_id.as_str())
            .collect(),
    })
    .map_err(|_| ProjectError::InvalidGraphDocument)?;
    let mut digest = Sha256::new();
    digest.update(b"qiongli-academic-graph-index-v1\0");
    digest.update(bytes);
    Ok(format!("gix_{:x}", digest.finalize()))
}

fn layer_matches(values: &[AcademicGraphLayer], filter: &[AcademicGraphLayer]) -> bool {
    filter.is_empty() || filter.iter().any(|layer| values.contains(layer))
}

fn sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_hashed_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn valid_query_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcademicGraphConfidence, AcademicGraphEdgeStatus, AcademicGraphIdentityScope,
        AcademicInferenceStrength, ProjectLifecycle, ProjectStage,
    };

    fn fixture_snapshot() -> AcademicGraphSnapshotV1 {
        let project_id = ProjectId::parse("prj_00000000000000000000000000000000").unwrap();
        let claim = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Claim,
            AcademicGraphIdentityScope::Project,
            "CLM-001",
            "Exposure is associated with returns",
            vec![
                AcademicGraphLayer::Argument,
                AcademicGraphLayer::Manuscript,
                AcademicGraphLayer::Combined,
            ],
            "manuscript/claims_evidence_map.md",
            "claim:CLM-001",
        )
        .unwrap();
        let paper = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Paper,
            AcademicGraphIdentityScope::Global,
            "citekey:Smith2024",
            "Smith2024",
            vec![
                AcademicGraphLayer::Literature,
                AcademicGraphLayer::Manuscript,
                AcademicGraphLayer::Combined,
            ],
            "manuscript/claims_evidence_map.md",
            "paper:Smith2024",
        )
        .unwrap();
        let edge = AcademicGraphEdgeV1::new(
            &project_id,
            &claim.node_id,
            AcademicGraphRelation::Cites,
            &paper.node_id,
            vec![
                AcademicGraphLayer::Argument,
                AcademicGraphLayer::Manuscript,
                AcademicGraphLayer::Combined,
            ],
            "The manuscript map records this citation.",
            "manuscript/claims_evidence_map.md",
            "claim-citation:CLM-001:Smith2024",
            "Citation presence does not establish direct support.",
            AcademicInferenceStrength::ReasonableInference,
            AcademicGraphConfidence::Medium,
            AcademicGraphEdgeStatus::Proposed,
            None,
        )
        .unwrap();
        let mut nodes = vec![claim, paper];
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        AcademicGraphSnapshotV1 {
            schema_version: 1,
            document_kind: "qiongli-academic-graph".to_string(),
            projection_id: "grp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            projection_digest: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string(),
            project_id,
            project_revision: 7,
            project_stage: ProjectStage::Writing,
            project_lifecycle: ProjectLifecycle::Active,
            project_manifest_digest:
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
            project_semantic_digest:
                "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string(),
            graph_source_digest: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .to_string(),
            source_count: 0,
            present_source_count: 0,
            node_count: nodes.len(),
            edge_count: 1,
            diagnostic_count: 0,
            sources: Vec::new(),
            nodes,
            edges: vec![edge],
            diagnostics: Vec::new(),
        }
    }

    fn path_fixture_snapshot() -> AcademicGraphSnapshotV1 {
        let project_id = ProjectId::parse("prj_00000000000000000000000000000000").unwrap();
        let source = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::ResearchQuestion,
            AcademicGraphIdentityScope::Project,
            "RQ-001",
            "How should the argument be explained?",
            vec![AcademicGraphLayer::Argument],
            "context/research_state.md",
            "question:RQ-001",
        )
        .unwrap();
        let branch_a = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Evidence,
            AcademicGraphIdentityScope::Project,
            "EVD-001",
            "Evidence route",
            vec![AcademicGraphLayer::Argument],
            "evidence/claim-evidence-ledger.csv",
            "evidence:EVD-001",
        )
        .unwrap();
        let branch_b = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::Concept,
            AcademicGraphIdentityScope::Project,
            "CON-001",
            "Concept route",
            vec![AcademicGraphLayer::Argument],
            "context/research_state.md",
            "concept:CON-001",
        )
        .unwrap();
        let target = AcademicGraphNodeV1::new(
            &project_id,
            AcademicGraphNodeType::ManuscriptSection,
            AcademicGraphIdentityScope::Project,
            "section:discussion",
            "Discussion",
            vec![AcademicGraphLayer::Manuscript],
            "manuscript/claims_evidence_map.md",
            "section:discussion",
        )
        .unwrap();
        let edge = |from: &AcademicGraphNodeV1,
                    relation: AcademicGraphRelation,
                    to: &AcademicGraphNodeV1,
                    anchor: &str| {
            AcademicGraphEdgeV1::new(
                &project_id,
                &from.node_id,
                relation,
                &to.node_id,
                vec![AcademicGraphLayer::Combined],
                "The canonical graph records this explanatory step.",
                "graph/semantic_links.jsonl",
                anchor,
                "This step describes graph structure, not new empirical support.",
                AcademicInferenceStrength::ReasonableInference,
                AcademicGraphConfidence::Medium,
                AcademicGraphEdgeStatus::Reviewed,
                None,
            )
            .unwrap()
        };
        let mut edges = vec![
            edge(
                &source,
                AcademicGraphRelation::Supports,
                &branch_a,
                "path:source-a",
            ),
            edge(
                &branch_a,
                AcademicGraphRelation::Informs,
                &target,
                "path:a-target",
            ),
            edge(
                &source,
                AcademicGraphRelation::Motivates,
                &branch_b,
                "path:source-b",
            ),
            edge(
                &branch_b,
                AcademicGraphRelation::AppearsInSection,
                &target,
                "path:b-target",
            ),
        ];
        edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
        let mut nodes = vec![source, branch_a, branch_b, target];
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        AcademicGraphSnapshotV1 {
            schema_version: 1,
            document_kind: "qiongli-academic-graph".to_string(),
            projection_id: "grp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            projection_digest: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string(),
            project_id,
            project_revision: 7,
            project_stage: ProjectStage::Writing,
            project_lifecycle: ProjectLifecycle::Active,
            project_manifest_digest:
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
            project_semantic_digest:
                "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string(),
            graph_source_digest: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .to_string(),
            source_count: 0,
            present_source_count: 0,
            node_count: nodes.len(),
            edge_count: edges.len(),
            diagnostic_count: 0,
            sources: Vec::new(),
            nodes,
            edges,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn index_is_deterministic_revision_bound_and_focus_queryable() {
        let snapshot = fixture_snapshot();
        let first = AcademicGraphIndexV1::from_snapshot(snapshot.clone()).unwrap();
        let second = AcademicGraphIndexV1::from_snapshot(snapshot).unwrap();
        assert_eq!(first.index_id, second.index_id);
        assert_eq!(first.node_count, 2);
        assert_eq!(first.edge_count, 1);

        let claim = first
            .snapshot
            .nodes
            .iter()
            .find(|node| node.node_type == AcademicGraphNodeType::Claim)
            .unwrap();
        let query = AcademicGraphQueryV1::new(first.projection_id.clone())
            .with_focus(claim.node_id.clone(), AcademicGraphDirection::Outgoing)
            .with_relations(vec![AcademicGraphRelation::Cites])
            .with_layers(vec![AcademicGraphLayer::Manuscript]);
        let result = first.query(&query).unwrap();
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.edges[0].relation, AcademicGraphRelation::Cites);

        let stale = AcademicGraphQueryV1::new(
            "grp_ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        assert_eq!(first.query(&stale), Err(ProjectError::RevisionConflict));
    }

    #[test]
    fn filters_are_bounded_sorted_and_report_truncation() {
        let index = AcademicGraphIndexV1::from_snapshot(fixture_snapshot()).unwrap();
        let text_query = AcademicGraphQueryV1::new(index.projection_id.clone())
            .with_text("smith")
            .with_node_types(vec![AcademicGraphNodeType::Paper])
            .with_limits(1, 1);
        let result = index.query(&text_query).unwrap();
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].canonical_id, "citekey:Smith2024");

        let truncated = index
            .query(&AcademicGraphQueryV1::new(index.projection_id.clone()).with_limits(1, 1))
            .unwrap();
        assert!(truncated.nodes_truncated);

        let invalid = AcademicGraphQueryV1 {
            node_types: vec![AcademicGraphNodeType::Claim, AcademicGraphNodeType::Paper],
            ..AcademicGraphQueryV1::new(index.projection_id.clone())
        };
        assert_eq!(index.query(&invalid), Err(ProjectError::InvalidGraphQuery));
    }

    #[test]
    fn explanatory_path_is_shortest_projection_bound_and_deterministic() {
        let index = AcademicGraphIndexV1::from_snapshot(path_fixture_snapshot()).unwrap();
        let source = index
            .snapshot
            .nodes
            .iter()
            .find(|node| node.canonical_id == "RQ-001")
            .unwrap();
        let target = index
            .snapshot
            .nodes
            .iter()
            .find(|node| node.canonical_id == "section:discussion")
            .unwrap();
        let query = AcademicGraphPathQueryV1::new(
            index.projection_id.clone(),
            source.node_id.clone(),
            target.node_id.clone(),
        );

        let first = index.explanatory_path(&query).unwrap();
        let second = index.explanatory_path(&query).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.status, AcademicGraphPathStatus::Found);
        assert_eq!(first.hop_count, 2);
        assert_eq!(first.nodes.first().unwrap().node_id, source.node_id);
        assert_eq!(first.nodes.last().unwrap().node_id, target.node_id);
        assert_eq!(first.steps.len(), 2);
        assert_eq!(first.steps[0].sequence, 1);
        assert_eq!(first.steps[1].sequence, 2);

        let first_edges = index
            .snapshot
            .edges
            .iter()
            .filter(|edge| edge.source_node_id == source.node_id)
            .map(|edge| edge.edge_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(first.edges[0].edge_id, *first_edges.iter().min().unwrap());

        let stale = AcademicGraphPathQueryV1::new(
            "grp_ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            source.node_id.clone(),
            target.node_id.clone(),
        );
        assert_eq!(
            index.explanatory_path(&stale),
            Err(ProjectError::RevisionConflict)
        );
    }

    #[test]
    fn explanatory_path_reports_reverse_zero_hop_missing_and_bounded_results() {
        let index = AcademicGraphIndexV1::from_snapshot(path_fixture_snapshot()).unwrap();
        let source = index
            .snapshot
            .nodes
            .iter()
            .find(|node| node.canonical_id == "RQ-001")
            .unwrap();
        let target = index
            .snapshot
            .nodes
            .iter()
            .find(|node| node.canonical_id == "section:discussion")
            .unwrap();

        let reverse = index
            .explanatory_path(&AcademicGraphPathQueryV1::new(
                index.projection_id.clone(),
                target.node_id.clone(),
                source.node_id.clone(),
            ))
            .unwrap();
        assert_eq!(reverse.hop_count, 2);
        assert!(
            reverse
                .steps
                .iter()
                .all(|step| step.traversal == AcademicGraphPathTraversal::Reverse)
        );

        let zero_hop = index
            .explanatory_path(&AcademicGraphPathQueryV1::new(
                index.projection_id.clone(),
                source.node_id.clone(),
                source.node_id.clone(),
            ))
            .unwrap();
        assert_eq!(zero_hop.status, AcademicGraphPathStatus::Found);
        assert_eq!(zero_hop.hop_count, 0);
        assert_eq!(zero_hop.nodes.len(), 1);

        let bounded = index
            .explanatory_path(
                &AcademicGraphPathQueryV1::new(
                    index.projection_id.clone(),
                    source.node_id.clone(),
                    target.node_id.clone(),
                )
                .with_max_hops(1),
            )
            .unwrap();
        assert_eq!(bounded.status, AcademicGraphPathStatus::NotFound);
        assert!(bounded.nodes.is_empty());
        assert!(bounded.edges.is_empty());
        assert!(bounded.steps.is_empty());

        let unknown = AcademicGraphPathQueryV1::new(
            index.projection_id.clone(),
            format!("nod_{}", "f".repeat(64)),
            target.node_id.clone(),
        );
        assert_eq!(
            index.explanatory_path(&unknown),
            Err(ProjectError::GraphEntityNotFound)
        );
        let invalid = AcademicGraphPathQueryV1::new(
            index.projection_id.clone(),
            source.node_id.clone(),
            target.node_id.clone(),
        )
        .with_max_hops(MAX_ACADEMIC_GRAPH_PATH_HOPS + 1);
        assert_eq!(
            index.explanatory_path(&invalid),
            Err(ProjectError::InvalidGraphQuery)
        );
    }
}
