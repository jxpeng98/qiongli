import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { CaptureConsolidationPreview, OperationPreview } from '@qiongli/app-api';

import ConfirmationDialog from './ConfirmationDialog.svelte';

const blockedPreview = {
  token: '00000000000000000000000000000001',
  kind: 'activation',
  title: 'Qiongli plugin preview',
  summary: 'Review the selected Qiongli content before applying it.',
  displayTarget: null,
  planDigestSha256: null,
  approvalsRequired: [],
  canConfirm: false,
  blockedReason: 'source-build-read-only'
} satisfies OperationPreview;

const conflictedConsolidation = {
  schemaVersion: 1,
  planDigest: '1'.repeat(64),
  captureId: `cap_${'a'.repeat(64)}`,
  projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
  disposition: 'contradiction',
  outcome: 'conflicted',
  expectedLibraryRevision: 7,
  expectedProjectRevision: 12,
  nextProjectRevision: null,
  projectStage: 'writing',
  reviewedAtUnix: 12,
  conflicts: [{
    kind: 'contradiction-requires-resolution',
    artifact: null,
    resolution: 'resolve-contradiction-before-consolidation'
  }],
  artifactDeltas: [],
  receiptEntry: 'history/consolidations/capture.json',
  approvalsRequired: []
} satisfies CaptureConsolidationPreview;

describe('ConfirmationDialog', () => {
  it('names the source authority block and disables confirmation', () => {
    render(ConfirmationDialog, {
      preview: blockedPreview,
      busy: false,
      onConfirm: vi.fn(),
      onCancel: vi.fn()
    });

    expect(screen.getByRole('dialog')).toHaveAccessibleName('Qiongli plugin preview');
    expect(screen.getByRole('alert')).toHaveTextContent('source-build-read-only');
    expect(screen.getByRole('button', { name: 'Confirm changes' })).toBeDisabled();
  });

  it('exposes a keyboard-addressable cancel action', async () => {
    const onCancel = vi.fn();
    render(ConfirmationDialog, {
      preview: blockedPreview,
      busy: false,
      onConfirm: vi.fn(),
      onCancel
    });

    const cancel = screen.getByRole('button', { name: 'Cancel' });
    cancel.focus();
    expect(cancel).toHaveFocus();
    await fireEvent.click(cancel);
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('shows academic conflicts inside the confirmation boundary', () => {
    render(ConfirmationDialog, {
      preview: {
        ...blockedPreview,
        kind: 'capture-consolidation',
        title: 'Consolidate reviewed capture',
        blockedReason: 'academic-review-conflict'
      },
      consolidation: conflictedConsolidation,
      busy: false,
      onConfirm: vi.fn(),
      onCancel: vi.fn()
    });

    expect(screen.getByRole('region', { name: 'Academic consolidation review' }))
      .toHaveTextContent('contradiction-requires-resolution');
    expect(screen.getByRole('alert')).toHaveTextContent('academic-review-conflict');
    expect(screen.getByRole('button', { name: 'Confirm changes' })).toBeDisabled();
  });
});
