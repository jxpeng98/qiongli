import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import ProjectArtifactViewer from './ProjectArtifactViewer.svelte';

describe('ProjectArtifactViewer', () => {
  it('moves focus inside, supports Escape, and restores the invoking control', async () => {
    const trigger = document.createElement('button');
    trigger.textContent = 'Preview artifact';
    document.body.append(trigger);
    trigger.focus();
    const onClose = vi.fn();

    render(ProjectArtifactViewer, {
      artifact: {
        schemaVersion: 1,
        documentKind: 'qiongli-project-artifact-view',
        projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
        projectRevision: 12,
        projectionId: null,
        entityKind: null,
        entityId: null,
        artifactPath: 'context/research_state.md',
        sourceAnchor: 'claim:C1',
        format: 'markdown',
        contentDigest: '7'.repeat(64),
        sourceSizeBytes: 12,
        content: 'claim:C1\n',
        contentSizeBytes: 9,
        startLine: 4,
        endLine: 5,
        anchorLine: 4,
        anchorMatched: true,
        truncatedBefore: true,
        truncatedAfter: false
      },
      onClose,
      returnFocusTarget: trigger
    });

    const close = screen.getByRole('button', { name: 'Close source preview' });
    await waitFor(() => expect(close).toHaveFocus());
    expect(screen.getByRole('textbox', { name: 'Bounded project artifact content' }))
      .toHaveAttribute('tabindex', '0');

    await fireEvent.keyDown(close, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledOnce();
    await waitFor(() => expect(trigger).toHaveFocus());
    trigger.remove();
  });

  it('uses the explicit return target when focus is lost during an async preview', async () => {
    const trigger = document.createElement('button');
    trigger.textContent = 'Preview artifact';
    document.body.append(trigger);
    const onClose = vi.fn();

    render(ProjectArtifactViewer, {
      artifact: {
        schemaVersion: 1,
        documentKind: 'qiongli-project-artifact-view',
        projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
        projectRevision: 12,
        projectionId: null,
        entityKind: null,
        entityId: null,
        artifactPath: 'context/research_state.md',
        sourceAnchor: null,
        format: 'markdown',
        contentDigest: '7'.repeat(64),
        sourceSizeBytes: 9,
        content: 'claim:C1\n',
        contentSizeBytes: 9,
        startLine: 1,
        endLine: 2,
        anchorLine: null,
        anchorMatched: false,
        truncatedBefore: false,
        truncatedAfter: false
      },
      onClose,
      returnFocusTarget: trigger
    });

    const close = screen.getByRole('button', { name: 'Close source preview' });
    await waitFor(() => expect(close).toHaveFocus());
    await fireEvent.keyDown(close, { key: 'Escape' });

    await waitFor(() => expect(trigger).toHaveFocus());
    trigger.remove();
  });
});
