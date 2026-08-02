import { describe, expect, it } from 'vitest';

import type { AcademicGraphQueryResult } from '@qiongli/app-api';

import { buildAcademicGraphRiskOverlay } from './risk';

describe('academic graph risk overlay', () => {
  it('classifies and orders source-bound risks without changing graph facts', () => {
    const result = fixture();
    const before = structuredClone(result);

    const overlay = buildAcademicGraphRiskOverlay(result);

    expect(overlay).toMatchObject({
      schemaVersion: 1,
      projectionId: result.projectionId,
      indexId: result.indexId,
      riskCount: 5,
      highRiskCount: 4,
      affectedNodeCount: 3,
      affectedEdgeCount: 2
    });
    expect(overlay.entries.map((entry) => entry.kind)).toEqual([
      'contradiction',
      'gap',
      'gap',
      'rejected-relation',
      'low-confidence'
    ]);
    expect(overlay.entries[0]).toMatchObject({
      sourceLabel: 'Evidence',
      relation: 'contradicts',
      targetLabel: 'Claim',
      artifactPath: 'evidence/claim-evidence-ledger.csv',
      sourceAnchor: 'EVD-001'
    });
    expect(result).toStrictEqual(before);
  });

  it('returns a stable empty overlay for a projection with no risks', () => {
    const result = fixture();
    result.nodes = result.nodes.filter((node) => node.nodeType !== 'gap');
    result.edges = [];
    result.matchedNodeCount = result.nodes.length;
    result.matchedEdgeCount = 0;

    expect(buildAcademicGraphRiskOverlay(result)).toMatchObject({
      riskCount: 0,
      highRiskCount: 0,
      affectedNodeCount: 0,
      affectedEdgeCount: 0,
      entries: []
    });
  });
});

function fixture(): AcademicGraphQueryResult {
  const claimId = `nod_${'1'.repeat(64)}`;
  const evidenceId = `nod_${'2'.repeat(64)}`;
  const gapId = `nod_${'3'.repeat(64)}`;
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-query-result',
    indexId: `gix_${'a'.repeat(64)}`,
    projectionId: `grp_${'b'.repeat(64)}`,
    projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
    projectRevision: 8,
    matchedNodeCount: 3,
    matchedEdgeCount: 2,
    nodesTruncated: false,
    edgesTruncated: false,
    nodes: [
      { nodeId: claimId, nodeType: 'claim', identityScope: 'project', canonicalId: 'CLM-001', label: 'Claim', layers: ['argument'], artifactPath: 'manuscript/claims_evidence_map.md', sourceAnchor: 'CLM-001' },
      { nodeId: evidenceId, nodeType: 'evidence', identityScope: 'project', canonicalId: 'EVD-001', label: 'Evidence', layers: ['argument'], artifactPath: 'evidence/claim-evidence-ledger.csv', sourceAnchor: 'EVD-001' },
      { nodeId: gapId, nodeType: 'gap', identityScope: 'project', canonicalId: 'GAP-001', label: 'Replication gap', layers: ['argument'], artifactPath: 'context/research_state.md', sourceAnchor: 'GAP-001' }
    ],
    edges: [
      { edgeId: `edg_${'1'.repeat(64)}`, sourceNodeId: evidenceId, relation: 'contradicts', targetNodeId: claimId, layers: ['argument'], rationale: 'The observed direction conflicts with the claim.', artifactPath: 'evidence/claim-evidence-ledger.csv', sourceAnchor: 'EVD-001', evidenceLimit: 'One bounded sample.', inferenceStrength: 'reasonable_inference', confidence: 'medium', status: 'reviewed', createdFromCapture: null },
      { edgeId: `edg_${'2'.repeat(64)}`, sourceNodeId: gapId, relation: 'weakens', targetNodeId: claimId, layers: ['argument'], rationale: 'The claim lacks replication.', artifactPath: 'context/research_state.md', sourceAnchor: 'GAP-001', evidenceLimit: 'No replication is registered.', inferenceStrength: 'unsupported_gap', confidence: 'low', status: 'rejected', createdFromCapture: null }
    ]
  };
}
