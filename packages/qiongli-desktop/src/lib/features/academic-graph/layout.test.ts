import { describe, expect, it } from 'vitest';

import type {
  AcademicGraphEdge,
  AcademicGraphNode,
  AcademicGraphQueryResult
} from '@qiongli/app-api';

import {
  ACADEMIC_GRAPH_LAYOUT_ALGORITHM,
  buildAcademicGraphLayout,
  buildAcademicGraphViewState
} from './layout';

const projectionId = `grp_${'a'.repeat(64)}`;
const indexId = `gix_${'b'.repeat(64)}`;

describe('academic graph layout contract', () => {
  it('produces identical geometry when validated query records arrive in another order', () => {
    const result = graphResult();
    const reordered = {
      ...result,
      nodes: [...result.nodes].reverse(),
      edges: [...result.edges].reverse()
    };

    expect(buildAcademicGraphLayout(reordered)).toEqual(buildAcademicGraphLayout(result));
  });

  it('places every node inside a labelled semantic band and anchors every edge', () => {
    const layout = buildAcademicGraphLayout(graphResult());

    expect(layout.algorithm).toBe(ACADEMIC_GRAPH_LAYOUT_ALGORITHM);
    expect(layout.layoutKey).toBe(`${projectionId}:${indexId}:${ACADEMIC_GRAPH_LAYOUT_ALGORITHM}`);
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
  });

  it('keeps view state ephemeral and drops identities outside the exact result', () => {
    const layout = buildAcademicGraphLayout(graphResult());
    const selectedNodeId = layout.nodes[0]!.nodeId;

    expect(buildAcademicGraphViewState(layout, {
      selectedNodeId,
      focusNodeId: `nod_${'f'.repeat(64)}`,
      direction: 'outgoing'
    })).toEqual({
      schemaVersion: 1,
      layoutKey: layout.layoutKey,
      viewportMode: 'scroll',
      selectedNodeId,
      focusNodeId: null,
      direction: 'outgoing'
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
