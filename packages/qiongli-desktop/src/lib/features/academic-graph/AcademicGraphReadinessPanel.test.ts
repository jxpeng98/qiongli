import { render, screen, within } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import type {
  AcademicGraphQueryResult,
  AcademicGraphReadiness
} from '@qiongli/app-api';

import AcademicGraphReadinessPanel from './AcademicGraphReadinessPanel.svelte';

describe('AcademicGraphReadinessPanel', () => {
  it('states exact bounded-result limits instead of presenting a partial graph as complete', () => {
    render(AcademicGraphReadinessPanel, {
      readiness: readinessFixture(),
      result: queryFixture()
    });

    expect(screen.getByRole('heading', { name: 'Showing a bounded graph view' }))
      .toBeVisible();
    expect(screen.getByText(
      'The projection is valid, but the current query omits additional matching records.'
    )).toBeVisible();
    expect(screen.getByText(
      'Narrow the filters to inspect records outside the current display bound.'
    )).toBeVisible();
    expect(screen.getByText('Bounded')).toBeVisible();
    const relationFact = screen.getByText('Semantic relations').closest('div');
    expect(relationFact).not.toBeNull();
    expect(within(relationFact!).getByText('4')).toBeVisible();
  });

  it('opens source evidence and gives a concrete recovery step for unrecognized content', () => {
    const readiness = readinessFixture();
    readiness.state = 'no-recognized-artifacts';
    readiness.reasonCode = 'academic-graph-no-recognized-artifacts';
    readiness.remediation = 'repair-graph-artifacts';
    readiness.semanticNodeCount = 0;
    readiness.relationCount = 0;
    readiness.relationCounts = [];
    const result = queryFixture();
    result.nodesTruncated = false;
    result.nodes = [];
    result.edges = [];

    render(AcademicGraphReadinessPanel, { readiness, result });

    expect(screen.getByRole('heading', { name: 'Artifacts need graph-ready structure' }))
      .toBeVisible();
    expect(screen.getByText(
      'Use Run in client to add the supported stable-ID structure, then rebuild the graph.'
    )).toBeVisible();
    expect(screen.getByRole('group')).toHaveAttribute('open');
  });

  it('shows native stale-source state and the persisted rebuild action', () => {
    const readiness = readinessFixture();
    readiness.state = 'stale';
    readiness.reasonCode = 'academic-graph-sources-stale';
    readiness.remediation = 'rebuild-graph';
    readiness.staleSourceCount = 1;
    readiness.sources[1].freshness = 'stale';
    readiness.lastSuccessfulBuild.projectionId = `grp_${'b'.repeat(64)}`;
    const result = queryFixture();
    result.nodesTruncated = false;
    result.edgesTruncated = false;

    render(AcademicGraphReadinessPanel, { readiness, result });

    expect(screen.getByRole('heading', { name: 'Graph sources changed' })).toBeVisible();
    expect(screen.getByText(
      'Rebuild the portfolio graph before relying on cross-project topology.'
    )).toBeVisible();
    expect(screen.getByText('Rebuild needed')).toBeVisible();
    expect(screen.getByRole('group')).toHaveAttribute('open');
  });
});

function readinessFixture(): AcademicGraphReadiness {
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-readiness',
    projectionId: `grp_${'a'.repeat(64)}`,
    projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
    projectRevision: 1,
    graphSourceDigest: 'c'.repeat(64),
    lastSuccessfulBuild: {
      projectRevision: 1,
      projectionId: `grp_${'a'.repeat(64)}`,
      graphSourceDigest: 'c'.repeat(64)
    },
    state: 'visualizable',
    reasonCode: 'academic-graph-visualizable',
    remediation: 'none',
    recognizedSourceCount: 2,
    presentSourceCount: 2,
    missingSourceCount: 0,
    invalidSourceCount: 0,
    unsupportedSourceCount: 0,
    staleSourceCount: 0,
    nodeCount: 6,
    semanticNodeCount: 5,
    connectedNodeCount: 5,
    isolatedNodeCount: 1,
    relationCount: 6,
    layerCounts: [
      { layer: 'argument', nodeCount: 3 },
      { layer: 'portfolio', nodeCount: 1 }
    ],
    nodeTypeCounts: [
      { nodeType: 'claim', nodeCount: 3 },
      { nodeType: 'project', nodeCount: 1 }
    ],
    relationCounts: [
      { relation: 'contains', edgeCount: 2 },
      { relation: 'supports', edgeCount: 4 }
    ],
    sources: [
      {
        sourceKind: 'project-manifest',
        artifactPath: 'context/project_manifest.json',
        state: 'present',
        freshness: 'fresh',
        nodeCount: 1,
        edgeCount: 0,
        diagnosticCount: 0
      },
      {
        sourceKind: 'registered-artifact',
        artifactPath: 'context/research_state.md',
        state: 'present',
        freshness: 'fresh',
        nodeCount: 5,
        edgeCount: 6,
        diagnosticCount: 0
      }
    ]
  };
}

function queryFixture(): AcademicGraphQueryResult {
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-query-result',
    indexId: `gix_${'b'.repeat(64)}`,
    projectionId: `grp_${'a'.repeat(64)}`,
    projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
    projectRevision: 12,
    matchedNodeCount: 6,
    matchedEdgeCount: 6,
    nodesTruncated: true,
    edgesTruncated: false,
    nodes: [],
    edges: []
  };
}
