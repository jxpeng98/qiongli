import { describe, expect, it } from 'vitest';

import type { AppEvent } from '@qiongli/app-api';

import {
  academicGraphQueryMatchesScope,
  buildAcademicGraphQuery,
  loadAcademicGraphPresentationState
} from '.';

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const projectionId = `grp_${'a'.repeat(64)}`;

describe('academic graph presentation contract', () => {
  it('builds bounded revision-bound queries without empty text filters', () => {
    expect(buildAcademicGraphQuery(projectionId, {
      focusNodeId: null,
      direction: 'both',
      maxDepth: 1,
      nodeType: 'claim',
      relation: 'supports',
      layer: 'argument',
      text: '  provenance  '
    })).toEqual({
      expectedProjectionId: projectionId,
      focusNodeId: null,
      direction: 'both',
      maxDepth: 1,
      nodeTypes: ['claim'],
      relations: ['supports'],
      layers: ['argument'],
      canonicalId: null,
      text: 'provenance',
      maxNodes: 100,
      maxEdges: 200
    });
  });

  it('carries a bounded focus depth only when a stable focus identity is present', () => {
    const focusNodeId = `nod_${'b'.repeat(64)}`;
    expect(buildAcademicGraphQuery(projectionId, {
      focusNodeId,
      direction: 'outgoing',
      maxDepth: 3,
      nodeType: null,
      relation: null,
      layer: null,
      text: ''
    })).toMatchObject({
      focusNodeId,
      direction: 'outgoing',
      maxDepth: 3
    });
  });

  it('does not report a partial or revision-mismatched load as ready', async () => {
    const events = [graphEvent(), queryEvent({ projectRevision: 11 })];
    const complete = await loadAcademicGraphPresentationState(
      projectId,
      12,
      async () => events.shift() ?? null
    );
    expect(complete).toBe(false);
  });

  it('loads the snapshot before querying its exact projection', async () => {
    const seen: string[] = [];
    const events = [graphEvent(), queryEvent()];
    const complete = await loadAcademicGraphPresentationState(
      projectId,
      12,
      async (intent) => {
        seen.push(intent.action);
        if (intent.action === 'query-academic-graph') {
          expect(intent.query.expectedProjectionId).toBe(projectionId);
        }
        return events.shift() ?? null;
      }
    );
    expect(complete).toBe(true);
    expect(seen).toEqual(['load-academic-graph', 'query-academic-graph']);
  });

  it('rejects stale query results without accepting the wrong projection scope', () => {
    expect(academicGraphQueryMatchesScope(
      queryEvent({ projectRevision: 11 }),
      projectId,
      12,
      projectionId
    )).toBe(false);
    expect(academicGraphQueryMatchesScope(
      queryEvent(),
      projectId,
      12,
      projectionId
    )).toBe(true);
  });
});

function graphEvent(): AppEvent {
  return {
    type: 'academic-graph',
    comparison: null,
    readiness: {
      schemaVersion: 1,
      documentKind: 'qiongli-academic-graph-readiness',
      projectionId,
      projectId,
      state: 'empty-project',
      reasonCode: 'academic-graph-empty-project',
      remediation: 'add-canonical-artifacts',
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
    },
    graph: {
      schemaVersion: 1,
      documentKind: 'qiongli-academic-graph',
      projectionId,
      projectionDigest: 'b'.repeat(64),
      projectId,
      projectRevision: 12,
      projectStage: 'writing',
      projectLifecycle: 'active',
      projectManifestDigest: 'c'.repeat(64),
      projectSemanticDigest: 'd'.repeat(64),
      graphSourceDigest: 'e'.repeat(64),
      sourceCount: 1,
      presentSourceCount: 1,
      nodeCount: 1,
      edgeCount: 0,
      diagnosticCount: 0,
      sources: [{
        sourceKind: 'project-manifest',
        artifactPath: 'context/project_manifest.json',
        present: true,
        contentDigest: 'c'.repeat(64),
        sizeBytes: 100
      }],
      nodes: [{
        nodeId: `nod_${'1'.repeat(64)}`,
        nodeType: 'project',
        identityScope: 'project',
        canonicalId: projectId,
        label: 'Canonical project',
        layers: ['portfolio'],
        artifactPath: 'context/project_manifest.json',
        sourceAnchor: 'project'
      }],
      edges: [],
      diagnostics: []
    }
  };
}

function queryEvent(
  override: Partial<Extract<AppEvent, { type: 'academic-graph-query' }>['result']> = {}
): AppEvent {
  return {
    type: 'academic-graph-query',
    result: {
      schemaVersion: 1,
      documentKind: 'qiongli-academic-graph-query-result',
      indexId: `gix_${'f'.repeat(64)}`,
      projectionId,
      projectId,
      projectRevision: 12,
      matchedNodeCount: 0,
      matchedEdgeCount: 0,
      nodesTruncated: false,
      edgesTruncated: false,
      nodes: [],
      edges: [],
      ...override
    }
  };
}
