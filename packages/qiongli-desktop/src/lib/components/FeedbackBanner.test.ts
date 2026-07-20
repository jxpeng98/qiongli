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
    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss message' }));
    expect(onDismiss).toHaveBeenCalledOnce();
  });
});
