import { describe, expect, it } from 'vitest';

import type { AcademicGraphNode } from '@qiongli/app-api';

import {
  findAcademicGraphMatches,
  moveAcademicGraphFocus,
  pushAcademicGraphFocus
} from './exploration';

describe('Academic Graph exploration state', () => {
  it('ranks exact and prefix matches deterministically without exposing prose search', () => {
    const nodes = [
      node('1', 'Claim provenance', 'CLM-002'),
      node('2', 'Primary claim', 'CLM-001'),
      node('3', 'Unrelated', 'SRC-001')
    ];
    expect(findAcademicGraphMatches([...nodes].reverse(), 'clm')).toEqual([
      expect.objectContaining({ nodeId: nodes[1]!.nodeId, score: 1 }),
      expect.objectContaining({ nodeId: nodes[0]!.nodeId, score: 1 })
    ]);
    expect(findAcademicGraphMatches(nodes, 'CLM-001')[0]).toMatchObject({
      nodeId: nodes[1]!.nodeId,
      score: 0
    });
    expect(findAcademicGraphMatches(nodes, '   ')).toEqual([]);
  });

  it('truncates forward history and preserves deterministic back and forward bounds', () => {
    const first = pushAcademicGraphFocus({ entries: [], index: -1 }, 'node-a');
    const second = pushAcademicGraphFocus(first, 'node-b');
    const back = moveAcademicGraphFocus(second, -1);
    const branched = pushAcademicGraphFocus(back, 'node-c');

    expect(back).toEqual({ entries: ['node-a', 'node-b'], index: 0 });
    expect(branched).toEqual({ entries: ['node-a', 'node-c'], index: 1 });
    expect(moveAcademicGraphFocus(branched, 1)).toEqual(branched);
  });
});

function node(id: string, label: string, canonicalId: string): AcademicGraphNode {
  return {
    nodeId: `nod_${id.repeat(64)}`,
    nodeType: 'claim',
    identityScope: 'project',
    canonicalId,
    label,
    layers: ['argument'],
    artifactPath: 'manuscript/claims_evidence_map.md',
    sourceAnchor: `claim:${canonicalId}`
  };
}
