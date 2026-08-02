import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import type {
  AcademicGraphPathResult,
  AcademicGraphSnapshot
} from '@qiongli/app-api';
import { describe, expect, it, vi } from 'vitest';

import AcademicGraphPathFinder from './AcademicGraphPathFinder.svelte';

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const paperId = `nod_${'1'.repeat(64)}`;
const claimId = `nod_${'2'.repeat(64)}`;
const edgeId = `edg_${'3'.repeat(64)}`;

describe('AcademicGraphPathFinder', () => {
  it('queries by exact endpoint identities and a bounded hop count', async () => {
    const onQuery = vi.fn().mockResolvedValue(true);
    render(AcademicGraphPathFinder, { graph: graph(), result: null, onQuery });

    await fireEvent.click(await screen.findByRole('button', { name: 'Find path' }));

    await waitFor(() => expect(onQuery).toHaveBeenCalledWith({
      sourceNodeId: paperId,
      targetNodeId: claimId,
      maxHops: 6
    }));
  });

  it('explains ordered reverse traversal without reversing the scholarly relation', async () => {
    render(AcademicGraphPathFinder, {
      graph: graph(),
      result: pathResult(),
      onQuery: vi.fn().mockResolvedValue(true)
    });

    expect(await screen.findByText('Shortest explanation: 1 relation step.')).toBeVisible();
    expect(screen.getByText('Cites · reverse traversal')).toBeVisible();
    expect(screen.getByText('The claim cites this paper.')).toBeVisible();
    await fireEvent.click(screen.getByText('Evidence limit'));
    expect(screen.getByText('Citation presence does not establish support.')).toBeVisible();
  });

  it('reports a bounded one-hop miss without claiming the graph is disconnected', async () => {
    const missing: AcademicGraphPathResult = {
      ...pathResult(),
      maxHops: 1,
      status: 'not-found',
      hopCount: 0,
      nodes: [],
      edges: [],
      steps: []
    };
    render(AcademicGraphPathFinder, {
      graph: graph(),
      result: missing,
      onQuery: vi.fn().mockResolvedValue(true)
    });

    await fireEvent.change(screen.getByRole('combobox', { name: 'Maximum hops' }), {
      target: { value: '1' }
    });

    expect(await screen.findByText('No direct connection was found within 1 hop.')).toBeVisible();
  });
});

function graph(): AcademicGraphSnapshot {
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph',
    projectionId: `grp_${'a'.repeat(64)}`,
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
    nodeCount: 2,
    edgeCount: 1,
    diagnosticCount: 0,
    sources: [{
      sourceKind: 'project-manifest',
      artifactPath: 'context/project_manifest.json',
      present: true,
      contentDigest: 'c'.repeat(64),
      sizeBytes: 512
    }],
    nodes: [
      {
        nodeId: paperId,
        nodeType: 'paper',
        identityScope: 'global',
        canonicalId: 'citekey:Smith2024',
        label: 'Smith 2024',
        layers: ['literature'],
        artifactPath: 'literature/literature_map.md',
        sourceAnchor: 'Smith2024'
      },
      {
        nodeId: claimId,
        nodeType: 'claim',
        identityScope: 'project',
        canonicalId: 'claim:C1',
        label: 'Central claim',
        layers: ['argument'],
        artifactPath: 'manuscript/claims_evidence_map.md',
        sourceAnchor: 'claim:C1'
      }
    ],
    edges: [edge()],
    diagnostics: []
  };
}

function edge(): AcademicGraphSnapshot['edges'][number] {
  return {
    edgeId,
    sourceNodeId: claimId,
    relation: 'cites',
    targetNodeId: paperId,
    layers: ['combined'],
    rationale: 'The claim cites this paper.',
    artifactPath: 'manuscript/claims_evidence_map.md',
    sourceAnchor: 'claim:C1',
    evidenceLimit: 'Citation presence does not establish support.',
    inferenceStrength: 'reasonable_inference',
    confidence: 'medium',
    status: 'reviewed',
    createdFromCapture: null
  };
}

function pathResult(): AcademicGraphPathResult {
  const snapshot = graph();
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-explanatory-path',
    indexId: `gix_${'f'.repeat(64)}`,
    projectionId: snapshot.projectionId,
    projectId,
    projectRevision: 12,
    sourceNodeId: paperId,
    targetNodeId: claimId,
    maxHops: 6,
    status: 'found',
    hopCount: 1,
    nodes: snapshot.nodes,
    edges: snapshot.edges,
    steps: [{
      sequence: 1,
      fromNodeId: paperId,
      edgeId,
      toNodeId: claimId,
      traversal: 'reverse'
    }]
  };
}
