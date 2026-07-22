import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import AcademicGraphInspector from './AcademicGraphInspector.svelte';
import type { AcademicGraphInspection } from './inspection';

const edgeId = `edg_${'4'.repeat(64)}`;

describe('AcademicGraphInspector', () => {
  it('renders source-bound relation details and opens by opaque entity identity', async () => {
    const onOpen = vi.fn().mockResolvedValue(true);
    render(AcademicGraphInspector, {
      inspection: relationInspection(),
      onOpen
    });

    expect(screen.getByRole('heading', { name: 'Reviewed evidence → Central claim' }))
      .toBeVisible();
    expect(screen.getByText('E1 directly supports C1.')).toBeVisible();
    expect(screen.getByText('link:support-C1')).toBeVisible();
    expect(screen.getByText('Discussion')).toBeVisible();

    await fireEvent.click(screen.getByRole('button', { name: 'Open source artifact' }));
    expect(onOpen).toHaveBeenCalledWith({ kind: 'edge', id: edgeId });
    await waitFor(() => expect(screen.getByText(
      'The exact registered project artifact was opened.'
    )).toBeVisible());
  });

  it('reports a bounded failure without exposing a host path', async () => {
    const onOpen = vi.fn().mockResolvedValue(false);
    render(AcademicGraphInspector, {
      inspection: relationInspection(),
      onOpen
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Open source artifact' }));
    await waitFor(() => expect(screen.getByText(
      'The artifact could not be opened. Refresh the graph and try again.'
    )).toBeVisible());
    expect(document.body.textContent).not.toContain('/private/');
  });
});

function relationInspection(): AcademicGraphInspection {
  return {
    entity: { kind: 'edge', id: edgeId },
    title: 'Reviewed evidence → Central claim',
    canonicalId: null,
    nodeType: null,
    relation: 'supports',
    layers: ['argument'],
    artifactPath: 'graph/semantic_links.jsonl',
    sourceAnchor: 'link:support-C1',
    rationale: 'E1 directly supports C1.',
    evidenceLimit: 'One reviewed source; no causal claim.',
    inferenceStrength: 'direct_evidence',
    confidence: 'medium',
    status: 'reviewed',
    createdFromCapture: null,
    manuscriptLocations: [{
      nodeId: `nod_${'3'.repeat(64)}`,
      label: 'Discussion',
      canonicalId: 'section:discussion',
      sourceAnchor: 'section:discussion'
    }]
  };
}
