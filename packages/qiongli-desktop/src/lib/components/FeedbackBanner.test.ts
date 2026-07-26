import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import FeedbackBanner from './FeedbackBanner.svelte';

describe('FeedbackBanner', () => {
  it('keeps the operation result beside the active page and allows dismissal', async () => {
    const onDismiss = vi.fn();
    render(FeedbackBanner, {
      notice: {
        tone: 'success',
        title: 'Operation completed',
        detail: 'qiongli-plugin-verified'
      },
      onDismiss
    });

    expect(screen.getByText('Operation completed')).toBeVisible();
    expect(screen.getByText('qiongli-plugin-verified')).toBeVisible();
    expect(screen.getByRole('status')).toHaveAttribute('aria-atomic', 'true');
    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss message' }));
    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it('announces danger notices assertively without relying on color', () => {
    render(FeedbackBanner, {
      notice: {
        tone: 'danger',
        title: 'Operation failed',
        detail: 'revision-conflict'
      },
      onDismiss: vi.fn()
    });

    expect(screen.getByRole('alert')).toHaveAttribute('aria-live', 'assertive');
    expect(screen.getByRole('alert')).toHaveAttribute('aria-atomic', 'true');
  });
});
