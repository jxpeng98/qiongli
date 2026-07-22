import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import AcademicGraphRiskOverlay from './AcademicGraphRiskOverlay.svelte';
import type { AcademicGraphRiskOverlay as RiskOverlay } from './risk';

describe('AcademicGraphRiskOverlay', () => {
  it('renders a semantic risk oracle and inspects by opaque graph identity', async () => {
    const onInspect = vi.fn();
    const edgeId = `edg_${'1'.repeat(64)}`;
    render(AcademicGraphRiskOverlay, {
      overlay: {
        schemaVersion: 1,
        projectionId: `grp_${'a'.repeat(64)}`,
        indexId: `gix_${'b'.repeat(64)}`,
        riskCount: 1,
        highRiskCount: 1,
        affectedNodeCount: 2,
        affectedEdgeCount: 1,
        entries: [{
          riskId: `edge:${edgeId}:contradiction`,
          kind: 'contradiction',
          severity: 'high',
          entity: { kind: 'edge', id: edgeId },
          sourceLabel: 'Observed evidence',
          relation: 'contradicts',
          targetLabel: 'Central claim',
          rationale: 'The registered outcome conflicts with the claim.',
          evidenceLimit: 'One bounded sample.',
          artifactPath: 'evidence/claim-evidence-ledger.csv',
          sourceAnchor: 'EVD-001',
          affectedNodeIds: [`nod_${'1'.repeat(64)}`, `nod_${'2'.repeat(64)}`]
        }]
      } satisfies RiskOverlay,
      onInspect
    });

    expect(screen.getByRole('heading', { name: 'Risks requiring scholarly review' })).toBeVisible();
    expect(screen.getByText('Observed evidence')).toBeVisible();
    expect(screen.getByText('The registered outcome conflicts with the claim.')).toBeVisible();
    expect(screen.getByText('evidence/claim-evidence-ledger.csv')).toBeVisible();

    await fireEvent.click(screen.getByRole('button', { name: 'Inspect source-bound record' }));
    expect(onInspect).toHaveBeenCalledWith({ kind: 'edge', id: edgeId });
  });

  it('announces when the exact bounded result has no explicit risk', () => {
    render(AcademicGraphRiskOverlay, {
      overlay: {
        schemaVersion: 1,
        projectionId: `grp_${'a'.repeat(64)}`,
        indexId: `gix_${'b'.repeat(64)}`,
        riskCount: 0,
        highRiskCount: 0,
        affectedNodeCount: 0,
        affectedEdgeCount: 0,
        entries: []
      } satisfies RiskOverlay,
      onInspect: vi.fn()
    });

    expect(screen.getByText('No flagged risks')).toBeVisible();
    expect(screen.getByText(/contains no explicit contradiction/)).toBeVisible();
  });
});
