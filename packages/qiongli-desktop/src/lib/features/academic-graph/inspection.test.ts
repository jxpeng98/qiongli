import type { AcademicGraphSnapshot } from '@qiongli/app-api';
import { describe, expect, it } from 'vitest';

import { buildAcademicGraphInspection } from './inspection';

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const claimId = `nod_${'1'.repeat(64)}`;
const evidenceId = `nod_${'2'.repeat(64)}`;
const sectionId = `nod_${'3'.repeat(64)}`;
const supportId = `edg_${'4'.repeat(64)}`;
const sectionEdgeId = `edg_${'5'.repeat(64)}`;

describe('Academic Graph inspection', () => {
  it('derives node evidence and directly connected manuscript locations', () => {
    const inspection = buildAcademicGraphInspection(graph(), { kind: 'node', id: claimId });

    expect(inspection).toMatchObject({
      entity: { kind: 'node', id: claimId },
      title: 'Central claim',
      canonicalId: 'claim:C1',
      artifactPath: 'context/research_state.md',
      sourceAnchor: 'claim:C1'
    });
    expect(inspection?.manuscriptLocations).toEqual([{
      nodeId: sectionId,
      label: 'Discussion',
      canonicalId: 'section:discussion',
      sourceAnchor: 'section:discussion'
    }]);
  });

  it('derives relation rationale and evidence limits without inventing locations', () => {
    const inspection = buildAcademicGraphInspection(graph(), { kind: 'edge', id: supportId });

    expect(inspection).toMatchObject({
      entity: { kind: 'edge', id: supportId },
      title: 'Reviewed evidence → Central claim',
      relation: 'supports',
      rationale: 'E1 directly supports C1.',
      evidenceLimit: 'One reviewed source; no causal claim.',
      confidence: 'medium',
      status: 'reviewed',
      manuscriptLocations: []
    });
  });

  it('rejects an entity that is absent from the exact projection', () => {
    expect(buildAcademicGraphInspection(graph(), {
      kind: 'node',
      id: `nod_${'9'.repeat(64)}`
    })).toBeNull();
  });
});

function graph(): AcademicGraphSnapshot {
  const nodes: AcademicGraphSnapshot['nodes'] = [
    {
      nodeId: claimId,
      nodeType: 'claim',
      identityScope: 'project',
      canonicalId: 'claim:C1',
      label: 'Central claim',
      layers: ['argument'],
      artifactPath: 'context/research_state.md',
      sourceAnchor: 'claim:C1'
    },
    {
      nodeId: evidenceId,
      nodeType: 'evidence',
      identityScope: 'project',
      canonicalId: 'evidence:E1',
      label: 'Reviewed evidence',
      layers: ['argument'],
      artifactPath: 'evidence/claim-evidence-ledger.csv',
      sourceAnchor: 'row:E1'
    },
    {
      nodeId: sectionId,
      nodeType: 'manuscript-section',
      identityScope: 'project',
      canonicalId: 'section:discussion',
      label: 'Discussion',
      layers: ['manuscript'],
      artifactPath: 'manuscript/claims_evidence_map.md',
      sourceAnchor: 'section:discussion'
    }
  ];
  const edges: AcademicGraphSnapshot['edges'] = [
    {
      edgeId: supportId,
      sourceNodeId: evidenceId,
      relation: 'supports',
      targetNodeId: claimId,
      layers: ['argument'],
      rationale: 'E1 directly supports C1.',
      artifactPath: 'graph/semantic_links.jsonl',
      sourceAnchor: 'link:support-C1',
      evidenceLimit: 'One reviewed source; no causal claim.',
      inferenceStrength: 'direct_evidence',
      confidence: 'medium',
      status: 'reviewed',
      createdFromCapture: null
    },
    {
      edgeId: sectionEdgeId,
      sourceNodeId: claimId,
      relation: 'appears-in-section',
      targetNodeId: sectionId,
      layers: ['argument', 'manuscript'],
      rationale: 'C1 is assigned to the Discussion section.',
      artifactPath: 'manuscript/claims_evidence_map.md',
      sourceAnchor: 'claim:C1',
      evidenceLimit: 'Placement only.',
      inferenceStrength: 'direct_evidence',
      confidence: 'high',
      status: 'observed',
      createdFromCapture: null
    }
  ];
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph',
    projectionId: `grp_${'a'.repeat(64)}`,
    projectionDigest: 'a'.repeat(64),
    projectId,
    projectRevision: 12,
    projectStage: 'writing',
    projectLifecycle: 'active',
    projectManifestDigest: 'b'.repeat(64),
    projectSemanticDigest: 'c'.repeat(64),
    graphSourceDigest: 'd'.repeat(64),
    sourceCount: 1,
    presentSourceCount: 1,
    nodeCount: nodes.length,
    edgeCount: edges.length,
    diagnosticCount: 0,
    sources: [{
      sourceKind: 'project-manifest',
      artifactPath: 'context/project_manifest.json',
      present: true,
      contentDigest: 'b'.repeat(64),
      sizeBytes: 512
    }],
    nodes,
    edges,
    diagnostics: []
  };
}
