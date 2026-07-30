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
      onPreview: vi.fn().mockResolvedValue(true),
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
      onPreview: vi.fn().mockResolvedValue(false),
      onOpen
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Open source artifact' }));
    await waitFor(() => expect(screen.getByText(
      'The artifact could not be opened. Refresh the graph and try again.'
    )).toBeVisible());
    expect(document.body.textContent).not.toContain('/private/');
  });

  it('renders a bounded source preview inside Qiongli', async () => {
    const onPreview = vi.fn().mockResolvedValue(true);
    render(AcademicGraphInspector, {
      inspection: relationInspection(),
      artifact: {
        schemaVersion: 1,
        documentKind: 'qiongli-project-artifact-view',
        projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
        projectRevision: 12,
        projectionId: `grp_${'a'.repeat(64)}`,
        entityKind: 'edge',
        entityId: edgeId,
        artifactPath: 'graph/semantic_links.jsonl',
        sourceAnchor: 'link:support-C1',
        format: 'json-lines',
        contentDigest: '7'.repeat(64),
        sourceSizeBytes: 57,
        content: '{"anchor":"link:support-C1","relation":"supports"}\n',
        contentSizeBytes: 57,
        startLine: 8,
        endLine: 9,
        anchorLine: 8,
        anchorMatched: true,
        truncatedBefore: true,
        truncatedAfter: true
      },
      onPreview,
      onOpen: vi.fn().mockResolvedValue(true)
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Preview in Qiongli' }));
    expect(onPreview).toHaveBeenCalledWith({ kind: 'edge', id: edgeId });
    expect(await screen.findByRole('heading', { name: 'graph/semantic_links.jsonl' })).toBeVisible();
    expect(screen.getByText('Source anchor found at line 8')).toBeVisible();
    expect(document.body.textContent).not.toContain('/private/');

    const close = screen.getByRole('button', { name: 'Close source preview' });
    await waitFor(() => expect(close).toHaveFocus());
    await fireEvent.keyDown(close, { key: 'Escape' });
    await waitFor(() => expect(
      screen.getByRole('button', { name: 'Preview in Qiongli' })
    ).toHaveFocus());
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
