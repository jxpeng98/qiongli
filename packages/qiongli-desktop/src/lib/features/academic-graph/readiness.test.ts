import { describe, expect, it } from 'vitest';

import type {
  AcademicGraphQueryResult,
  AcademicGraphReadiness
} from '@qiongli/app-api';

import {
  academicGraphReadinessStatus,
  canRenderAcademicGraph,
  effectiveAcademicGraphReadiness
} from './readiness';

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const projectionId = `grp_${'a'.repeat(64)}`;

describe('academic graph visualization readiness', () => {
  it('turns a bounded result into an explicit truncated state', () => {
    const result = queryResult({ nodesTruncated: true, matchedNodeCount: 2 });
    expect(effectiveAcademicGraphReadiness(readiness('visualizable'), result))
      .toBe('bounded-truncated');
    expect(academicGraphReadinessStatus('bounded-truncated')).toBe('attention');
  });

  it('does not render a misleading graph for empty or unrecognized projects', () => {
    const result = queryResult();
    expect(canRenderAcademicGraph(readiness('empty-project'), result)).toBe(false);
    expect(canRenderAcademicGraph(readiness('no-recognized-artifacts'), result)).toBe(false);
    expect(canRenderAcademicGraph(readiness('nodes-without-edges'), result)).toBe(true);
  });
});

function readiness(
  state: AcademicGraphReadiness['state']
): AcademicGraphReadiness {
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-readiness',
    projectionId,
    projectId,
    state,
    reasonCode: `academic-graph-${state}`,
    remediation: state === 'visualizable'
      ? 'none'
      : state === 'empty-project'
        ? 'add-canonical-artifacts'
        : state === 'no-recognized-artifacts'
          ? 'repair-graph-artifacts'
          : state === 'nodes-without-edges'
            ? 'add-semantic-relations'
            : state === 'bounded-truncated' ? 'narrow-query' : 'enrich-graph',
    recognizedSourceCount: 1,
    presentSourceCount: 1,
    missingSourceCount: 0,
    invalidSourceCount: 0,
    unsupportedSourceCount: 0,
    nodeCount: 1,
    semanticNodeCount: 0,
    connectedNodeCount: 0,
    isolatedNodeCount: 1,
    relationCount: 0,
    layerCounts: [{ layer: 'portfolio', nodeCount: 1 }],
    nodeTypeCounts: [{ nodeType: 'project', nodeCount: 1 }],
    relationCounts: [],
    sources: [{
      sourceKind: 'project-manifest',
      artifactPath: 'context/project_manifest.json',
      state: 'present',
      nodeCount: 1,
      edgeCount: 0,
      diagnosticCount: 0
    }]
  };
}

function queryResult(
  override: Partial<AcademicGraphQueryResult> = {}
): AcademicGraphQueryResult {
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-query-result',
    indexId: `gix_${'b'.repeat(64)}`,
    projectionId,
    projectId,
    projectRevision: 1,
    matchedNodeCount: 1,
    matchedEdgeCount: 0,
    nodesTruncated: false,
    edgesTruncated: false,
    nodes: [{
      nodeId: `nod_${'1'.repeat(64)}`,
      nodeType: 'project',
      identityScope: 'project',
      canonicalId: projectId,
      label: 'Fixture project',
      layers: ['portfolio'],
      artifactPath: 'context/project_manifest.json',
      sourceAnchor: 'project'
    }],
    edges: [],
    ...override
  };
}
