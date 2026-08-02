import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import type { AcademicGraphRevisionComparison } from '@qiongli/app-api';

import AcademicGraphRevisionComparisonView from './AcademicGraphRevisionComparison.svelte';

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const nodeId = `nod_${'1'.repeat(64)}`;

const comparison: AcademicGraphRevisionComparison = {
  schemaVersion: 1,
  documentKind: 'qiongli-academic-graph-revision-comparison',
  comparisonId: `gcp_${'2'.repeat(64)}`,
  projectId,
  beforeProjectRevision: 11,
  afterProjectRevision: 12,
  beforeProjectionId: `grp_${'3'.repeat(64)}`,
  afterProjectionId: `grp_${'4'.repeat(64)}`,
  sourceChangeCount: 0,
  nodeChangeCount: 1,
  edgeChangeCount: 0,
  hasChanges: true,
  beforeRisks: {
    contradictionCount: 0,
    gapCount: 0,
    rejectedRelationCount: 0,
    lowConfidenceCount: 0,
    totalSignalCount: 0
  },
  afterRisks: {
    contradictionCount: 0,
    gapCount: 1,
    rejectedRelationCount: 0,
    lowConfidenceCount: 0,
    totalSignalCount: 1
  },
  riskDelta: {
    contradictionCount: 0,
    gapCount: 1,
    rejectedRelationCount: 0,
    lowConfidenceCount: 0,
    totalSignalCount: 1
  },
  sourceChanges: [],
  nodeChanges: [{
    changeKind: 'added',
    nodeId,
    before: null,
    after: {
      nodeId,
      nodeType: 'gap',
      identityScope: 'project',
      canonicalId: 'GAP-001',
      label: 'Independent replication gap',
      layers: ['argument', 'combined'],
      artifactPath: 'context/research_state.md',
      sourceAnchor: 'GAP-001'
    }
  }],
  edgeChanges: [],
  nextActions: ['fill-new-gaps']
};

describe('AcademicGraphRevisionComparison', () => {
  it('renders revision-bound changes and inspects only a current record', async () => {
    const onInspect = vi.fn();
    render(AcademicGraphRevisionComparisonView, { comparison, onInspect });

    expect(screen.getByRole('heading', { name: 'What changed between projections' })).toBeVisible();
    expect(screen.getByText('Revision 11')).toBeVisible();
    expect(screen.getByText('Revision 12')).toBeVisible();
    expect(screen.getByText('Resolve or explicitly bound newly introduced evidence gaps.')).toBeVisible();

    await fireEvent.click(screen.getByText('Inspect exact changed records'));
    await fireEvent.click(screen.getByRole('button', { name: 'Inspect current record' }));
    expect(onInspect).toHaveBeenCalledWith({ kind: 'node', id: nodeId });
  });

  it('announces that no comparison exists before a second rebuild', () => {
    render(AcademicGraphRevisionComparisonView, { comparison: null, onInspect: vi.fn() });
    expect(screen.getByText(/establish a before-and-after comparison/)).toBeVisible();
  });
});
