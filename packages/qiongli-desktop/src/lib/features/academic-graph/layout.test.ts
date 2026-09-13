import { describe, expect, it } from 'vitest';

import type {
  AcademicGraphEdge,
  AcademicGraphNode,
  AcademicGraphQueryResult
} from '@qiongli/app-api';

import {
  ACADEMIC_GRAPH_LAYOUT_ALGORITHM,
  ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM,
  ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM,
  academicGraphViewportClass,
  buildAcademicGraphLayout,
  buildAcademicGraphViewState
} from './layout';

const projectionId = `grp_${'a'.repeat(64)}`;
const indexId = `gix_${'b'.repeat(64)}`;

describe('academic graph layout contract', () => {
  it('produces identical topology, communities, and geometry when records arrive reordered', () => {
    const result = graphResult();
    const reordered = {
      ...result,
      nodes: [...result.nodes].reverse(),
      edges: [...result.edges].reverse()
    };

    expect(buildAcademicGraphLayout(reordered)).toEqual(buildAcademicGraphLayout(result));
  });

  it('places every node inside a semantic band and exposes deterministic component spines', () => {
    const layout = buildAcademicGraphLayout(graphResult());

    expect(layout.algorithm).toBe(ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM);
    expect(layout.algorithm).toBe(ACADEMIC_GRAPH_LAYOUT_ALGORITHM);
    expect(layout.layoutKey).toBe(
      `${projectionId}:${indexId}:${ACADEMIC_GRAPH_LAYOUT_ALGORITHM}:standard`
    );
    expect(layout.viewportClass).toBe('standard');
    expect(layout.fallbackReason).toBeNull();
    expect(layout.bands.map((band) => band.layer)).toEqual([
      'portfolio',
      'literature',
      'idea-decision',
      'argument',
      'manuscript'
    ]);
    expect(layout.nodes).toHaveLength(5);
    for (const node of layout.nodes) {
      expect(node.x).toBeGreaterThanOrEqual(0);
      expect(node.y).toBeGreaterThanOrEqual(0);
      expect(node.x + node.width).toBeLessThanOrEqual(layout.width);
      expect(node.y + node.height).toBeLessThanOrEqual(layout.height);
    }
    const nodeIds = new Set(layout.nodes.map((node) => node.nodeId));
    expect(layout.edges.every((edge) =>
      nodeIds.has(edge.sourceNodeId) && nodeIds.has(edge.targetNodeId)
    )).toBe(true);
    expect(layout.edges.every((edge) => edge.x1 !== edge.x2 || edge.y1 !== edge.y2)).toBe(true);
    expect(layout.edges.every((edge) =>
      edge.inferenceStrength === 'direct_evidence'
      && edge.confidence === 'high'
      && edge.status === 'reviewed'
    )).toBe(true);
    expect(layout.components).toHaveLength(3);
    expect(layout.clusters).toHaveLength(3);
    expect(layout.clusters?.every((cluster) => cluster.clusterId.startsWith('vcl_'))).toBe(true);
    expect(layout.nodes.filter((node) => node.spine)).toHaveLength(3);
    expect(layout.nodes.every((node) => node.componentId && node.communityId)).toBe(true);
  });

  it('keeps connected entities closer than unrelated components', () => {
    const result = relationshipGraphResult();
    const layout = buildAcademicGraphLayout(result);
    const byId = new Map(layout.nodes.map((node) => [node.nodeId, node]));
    const connectedDistances = result.edges.map((edge) =>
      centerDistance(byId.get(edge.sourceNodeId)!, byId.get(edge.targetNodeId)!));
    const unrelatedDistance = centerDistance(
      byId.get(result.nodes[1]!.nodeId)!,
      byId.get(result.nodes[4]!.nodeId)!
    );

    expect(Math.max(...connectedDistances)).toBeLessThan(unrelatedDistance);
  });

  it('uses multiple stable lanes instead of an unbounded single-layer column', () => {
    const result = denseSingleLayerResult(30);
    const topology = buildAcademicGraphLayout(result);
    const fallback = buildAcademicGraphLayout(result, undefined, { forceFallback: true });

    expect(topology.algorithm).toBe(ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM);
    expect(new Set(topology.nodes.map((node) => node.x)).size).toBe(3);
    expect(topology.height).toBeLessThan(fallback.height / 2);
    expect(fallback.algorithm).toBe(ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM);
    expect(fallback.fallbackReason).toBe('topology-layout-failed');
  });

  it('retains positions for an incremental revision when the viewport class is unchanged', () => {
    const before = buildAcademicGraphLayout(boundedGraphResult(100, 200, 'c'));
    const after = buildAcademicGraphLayout(
      boundedGraphResult(105, 210, 'd'),
      undefined,
      { previousLayout: before }
    );
    const previousById = new Map(before.nodes.map((node) => [node.nodeId, node]));
    const unchanged = after.nodes.filter((node) => previousById.has(node.nodeId));
    const stable = unchanged.filter((node) => {
      const previous = previousById.get(node.nodeId)!;
      return Math.hypot(node.x - previous.x, node.y - previous.y) <= node.width;
    });

    expect(after.algorithm).toBe(ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM);
    expect(stable.length / unchanged.length).toBeGreaterThanOrEqual(0.9);
  });

  it('meets the desktop and hard-bound pure layout budgets', () => {
    const desktop = boundedGraphResult(100, 200, 'e');
    const hardBound = boundedGraphResult(256, 512, 'f');
    buildAcademicGraphLayout(desktop, undefined, { performanceBudgetMs: 10_000 });

    const desktopStartedAt = performance.now();
    const desktopLayout = buildAcademicGraphLayout(
      desktop,
      undefined,
      { performanceBudgetMs: 10_000 }
    );
    const desktopDuration = performance.now() - desktopStartedAt;
    const hardBoundStartedAt = performance.now();
    const hardBoundLayout = buildAcademicGraphLayout(
      hardBound,
      undefined,
      { performanceBudgetMs: 10_000 }
    );
    const hardBoundDuration = performance.now() - hardBoundStartedAt;

    expect(desktopLayout.algorithm).toBe(ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM);
    expect(hardBoundLayout.algorithm).toBe(ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM);
    expect(desktopDuration).toBeLessThan(100);
    expect(hardBoundDuration).toBeLessThan(250);
  });

  it('falls back truthfully when the topology budget is exceeded', () => {
    const timestamps = [0, 101];
    const layout = buildAcademicGraphLayout(graphResult(), undefined, {
      performanceBudgetMs: 100,
      now: () => timestamps.shift() ?? 101
    });

    expect(layout.algorithm).toBe(ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM);
    expect(layout.fallbackReason).toBe('layout-budget-exceeded');
  });

  it('rejects dangling semantic edges before either visual layout runs', () => {
    const result = graphResult();
    result.edges[0] = {
      ...result.edges[0]!,
      targetNodeId: `nod_${'f'.repeat(64)}`
    };

    expect(() => buildAcademicGraphLayout(result)).toThrow('inconsistent topology');
  });

  it('classifies only the supported responsive viewport bands', () => {
    expect(academicGraphViewportClass(375)).toBe('compact');
    expect(academicGraphViewportClass(768)).toBe('standard');
    expect(academicGraphViewportClass(1024)).toBe('standard');
    expect(academicGraphViewportClass(1440)).toBe('wide');
  });

  it('keeps view state ephemeral and drops identities outside the exact result', () => {
    const layout = buildAcademicGraphLayout(graphResult());
    const selectedNodeId = layout.nodes[0]!.nodeId;

    expect(buildAcademicGraphViewState(layout, {
      selectedNodeId,
      selectedEdgeId: `edg_${'f'.repeat(64)}`,
      focusNodeId: `nod_${'f'.repeat(64)}`,
      direction: 'outgoing',
      matchingNodeIds: [selectedNodeId, selectedNodeId, `nod_${'f'.repeat(64)}`],
      collapsedClusterIds: [
        ...(layout.clusters?.[0] ? [layout.clusters[0].clusterId] : []),
        'vcl_unknown'
      ]
    })).toEqual({
      schemaVersion: 1,
      layoutKey: layout.layoutKey,
      viewportMode: 'scroll',
      selectedNodeId,
      selectedEdgeId: null,
      focusNodeId: null,
      direction: 'outgoing',
      matchingNodeIds: [selectedNodeId],
      collapsedClusterIds: layout.clusters?.[0] ? [layout.clusters[0].clusterId] : [],
      hiddenNodeTypes: [],
      hiddenRelationFamilies: []
    });
  });
});

function graphResult(): AcademicGraphQueryResult {
  const nodes: AcademicGraphNode[] = [
    graphNode('1', 'project', 'PROJECT-001', 'Project', ['portfolio']),
    graphNode('2', 'paper', 'PAPER-001', 'Foundational paper', ['literature']),
    graphNode('3', 'decision', 'DEC-001', 'Use a bounded index', ['idea-decision']),
    graphNode('4', 'claim', 'CLM-001', 'Projection is reproducible', ['argument', 'manuscript']),
    graphNode('5', 'manuscript-section', 'SEC-001', 'Methods', ['manuscript'])
  ];
  const edges: AcademicGraphEdge[] = [
    graphEdge('2', nodes[3]!.nodeId, 'appears-in-section', nodes[4]!.nodeId),
    graphEdge('1', nodes[1]!.nodeId, 'supports', nodes[3]!.nodeId)
  ];
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-query-result',
    indexId,
    projectionId,
    projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
    projectRevision: 12,
    matchedNodeCount: nodes.length,
    matchedEdgeCount: edges.length,
    nodesTruncated: false,
    edgesTruncated: false,
    nodes,
    edges
  };
}

function relationshipGraphResult(): AcademicGraphQueryResult {
  const nodes = [
    graphNode('a', 'project', 'PROJECT-A', 'Project A', ['portfolio']),
    graphNode('b', 'paper', 'PAPER-A', 'Paper A', ['literature']),
    graphNode('c', 'claim', 'CLAIM-A', 'Claim A', ['argument']),
    graphNode('d', 'project', 'PROJECT-B', 'Project B', ['portfolio']),
    graphNode('e', 'paper', 'PAPER-B', 'Paper B', ['literature']),
    graphNode('f', 'claim', 'CLAIM-B', 'Claim B', ['argument'])
  ];
  const edges = [
    graphEdge('a', nodes[0]!.nodeId, 'contains', nodes[1]!.nodeId),
    graphEdge('b', nodes[1]!.nodeId, 'supports', nodes[2]!.nodeId),
    graphEdge('c', nodes[3]!.nodeId, 'contains', nodes[4]!.nodeId),
    graphEdge('d', nodes[4]!.nodeId, 'supports', nodes[5]!.nodeId)
  ];
  return graphQueryResult(nodes, edges, '8');
}

function denseSingleLayerResult(nodeCount: number): AcademicGraphQueryResult {
  const nodes = Array.from({ length: nodeCount }, (_, index) =>
    graphNode(
      stableHex(index + 1).slice(-1),
      'claim',
      `CLAIM-${index.toString().padStart(3, '0')}`,
      `Claim ${index + 1}`,
      ['argument']
    )).map((node, index) => ({ ...node, nodeId: stableNodeId(index) }));
  const edges = Array.from({ length: nodeCount - 1 }, (_, index) =>
    graphEdgeRecord(index, nodes[index]!.nodeId, 'supports', nodes[index + 1]!.nodeId));
  return graphQueryResult(nodes, edges, '9');
}

function boundedGraphResult(
  nodeCount: number,
  edgeCount: number,
  digestCharacter: string
): AcademicGraphQueryResult {
  const nodeTypes: AcademicGraphNode['nodeType'][] = [
    'project',
    'paper',
    'idea',
    'claim',
    'evidence',
    'manuscript-section'
  ];
  const layers: AcademicGraphNode['layers'][] = [
    ['portfolio'],
    ['literature'],
    ['idea-decision'],
    ['argument'],
    ['argument'],
    ['manuscript']
  ];
  const nodes = Array.from({ length: nodeCount }, (_, index): AcademicGraphNode => ({
    nodeId: stableNodeId(index),
    nodeType: nodeTypes[index % nodeTypes.length]!,
    identityScope: 'project',
    canonicalId: `ENTITY-${index.toString().padStart(4, '0')}`,
    label: `Entity ${index + 1}`,
    layers: layers[index % layers.length]!,
    artifactPath: 'context/project_manifest.json',
    sourceAnchor: `row:${index + 1}`
  }));
  const relations: AcademicGraphEdge['relation'][] = ['contains', 'supports', 'informs'];
  const edges = Array.from({ length: edgeCount }, (_, index) => {
    const sourceIndex = index % nodeCount;
    let targetIndex = (index * 17 + 7) % nodeCount;
    if (targetIndex === sourceIndex) targetIndex = (targetIndex + 1) % nodeCount;
    return graphEdgeRecord(
      index,
      nodes[sourceIndex]!.nodeId,
      relations[index % relations.length]!,
      nodes[targetIndex]!.nodeId
    );
  });
  return {
    ...graphQueryResult(nodes, edges, digestCharacter),
    matchedNodeCount: nodeCount,
    matchedEdgeCount: edgeCount
  };
}

function graphQueryResult(
  nodes: AcademicGraphNode[],
  edges: AcademicGraphEdge[],
  digestCharacter: string
): AcademicGraphQueryResult {
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-query-result',
    indexId: `gix_${digestCharacter.repeat(64)}`,
    projectionId: `grp_${digestCharacter.repeat(64)}`,
    projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
    projectRevision: 12,
    matchedNodeCount: nodes.length,
    matchedEdgeCount: edges.length,
    nodesTruncated: false,
    edgesTruncated: false,
    nodes,
    edges
  };
}

function stableNodeId(index: number): string {
  return `nod_${stableHex(index)}`;
}

function stableEdgeId(index: number): string {
  return `edg_${stableHex(index)}`;
}

function stableHex(index: number): string {
  return index.toString(16).padStart(64, '0');
}

function graphEdgeRecord(
  index: number,
  sourceNodeId: string,
  relation: AcademicGraphEdge['relation'],
  targetNodeId: string
): AcademicGraphEdge {
  return {
    edgeId: stableEdgeId(index),
    sourceNodeId,
    relation,
    targetNodeId,
    layers: ['argument'],
    rationale: 'Typed fixture relation.',
    artifactPath: 'context/project_manifest.json',
    sourceAnchor: `edge:${index}`,
    evidenceLimit: 'Fixture evidence only.',
    inferenceStrength: 'direct_evidence',
    confidence: 'high',
    status: 'reviewed',
    createdFromCapture: null
  };
}

function centerDistance(
  left: ReturnType<typeof buildAcademicGraphLayout>['nodes'][number],
  right: ReturnType<typeof buildAcademicGraphLayout>['nodes'][number]
): number {
  return Math.hypot(
    left.x + left.width / 2 - right.x - right.width / 2,
    left.y + left.height / 2 - right.y - right.height / 2
  );
}

function graphNode(
  id: string,
  nodeType: AcademicGraphNode['nodeType'],
  canonicalId: string,
  label: string,
  layers: AcademicGraphNode['layers']
): AcademicGraphNode {
  return {
    nodeId: `nod_${id.repeat(64)}`,
    nodeType,
    identityScope: 'project',
    canonicalId,
    label,
    layers,
    artifactPath: 'context/project_manifest.json',
    sourceAnchor: canonicalId
  };
}

function graphEdge(
  id: string,
  sourceNodeId: string,
  relation: AcademicGraphEdge['relation'],
  targetNodeId: string
): AcademicGraphEdge {
  return {
    edgeId: `edg_${id.repeat(64)}`,
    sourceNodeId,
    relation,
    targetNodeId,
    layers: ['argument'],
    rationale: 'Typed fixture relation.',
    artifactPath: 'context/project_manifest.json',
    sourceAnchor: id,
    evidenceLimit: 'Fixture evidence only.',
    inferenceStrength: 'direct_evidence',
    confidence: 'high',
    status: 'reviewed',
    createdFromCapture: null
  };
}
