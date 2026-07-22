import type {
  AcademicGraphEdge,
  AcademicGraphEntityReference,
  AcademicGraphLayer,
  AcademicGraphNodeType,
  AcademicGraphRelation,
  AcademicGraphSnapshot
} from '@qiongli/app-api';

export interface AcademicGraphManuscriptLocation {
  nodeId: string;
  label: string;
  canonicalId: string;
  sourceAnchor: string;
}

export interface AcademicGraphInspection {
  entity: AcademicGraphEntityReference;
  title: string;
  canonicalId: string | null;
  nodeType: AcademicGraphNodeType | null;
  relation: AcademicGraphRelation | null;
  layers: AcademicGraphLayer[];
  artifactPath: string;
  sourceAnchor: string;
  rationale: string | null;
  evidenceLimit: string | null;
  inferenceStrength: AcademicGraphEdge['inferenceStrength'] | null;
  confidence: AcademicGraphEdge['confidence'] | null;
  status: AcademicGraphEdge['status'] | null;
  createdFromCapture: string | null;
  manuscriptLocations: AcademicGraphManuscriptLocation[];
}

export function buildAcademicGraphInspection(
  graph: AcademicGraphSnapshot,
  entity: AcademicGraphEntityReference | null
): AcademicGraphInspection | null {
  if (!entity) return null;
  const nodeById = new Map(graph.nodes.map((node) => [node.nodeId, node]));
  if (entity.kind === 'node') {
    const node = nodeById.get(entity.id);
    if (!node) return null;
    const relatedNodeIds = new Set([node.nodeId]);
    for (const edge of graph.edges) {
      if (edge.sourceNodeId === node.nodeId) relatedNodeIds.add(edge.targetNodeId);
      if (edge.targetNodeId === node.nodeId) relatedNodeIds.add(edge.sourceNodeId);
    }
    return {
      entity,
      title: node.label,
      canonicalId: node.canonicalId,
      nodeType: node.nodeType,
      relation: null,
      layers: node.layers,
      artifactPath: node.artifactPath,
      sourceAnchor: node.sourceAnchor,
      rationale: null,
      evidenceLimit: null,
      inferenceStrength: null,
      confidence: null,
      status: null,
      createdFromCapture: null,
      manuscriptLocations: manuscriptLocations(nodeById, relatedNodeIds)
    };
  }

  const edge = graph.edges.find((candidate) => candidate.edgeId === entity.id);
  if (!edge) return null;
  const source = nodeById.get(edge.sourceNodeId);
  const target = nodeById.get(edge.targetNodeId);
  return {
    entity,
    title: `${source?.label ?? edge.sourceNodeId} → ${target?.label ?? edge.targetNodeId}`,
    canonicalId: null,
    nodeType: null,
    relation: edge.relation,
    layers: edge.layers,
    artifactPath: edge.artifactPath,
    sourceAnchor: edge.sourceAnchor,
    rationale: edge.rationale,
    evidenceLimit: edge.evidenceLimit,
    inferenceStrength: edge.inferenceStrength,
    confidence: edge.confidence,
    status: edge.status,
    createdFromCapture: edge.createdFromCapture,
    manuscriptLocations: manuscriptLocations(
      nodeById,
      new Set([edge.sourceNodeId, edge.targetNodeId])
    )
  };
}

function manuscriptLocations(
  nodeById: Map<string, AcademicGraphSnapshot['nodes'][number]>,
  candidateIds: Set<string>
): AcademicGraphManuscriptLocation[] {
  return [...candidateIds]
    .map((nodeId) => nodeById.get(nodeId))
    .filter((node): node is AcademicGraphSnapshot['nodes'][number] =>
      node?.nodeType === 'manuscript-section')
    .map((node) => ({
      nodeId: node.nodeId,
      label: node.label,
      canonicalId: node.canonicalId,
      sourceAnchor: node.sourceAnchor
    }))
    .sort((left, right) => left.canonicalId.localeCompare(right.canonicalId));
}
