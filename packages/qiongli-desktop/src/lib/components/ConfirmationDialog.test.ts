import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { OperationPreview } from '@qiongli/app-api';

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
});
