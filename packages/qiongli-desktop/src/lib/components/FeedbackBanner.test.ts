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

  it('automatically dismisses transient success notices', async () => {
    vi.useFakeTimers();
    try {
      const onDismiss = vi.fn();
      render(FeedbackBanner, {
        notice: {
          tone: 'success',
          title: 'Operation completed',
          detail: 'qiongli-plugin-verified'
        },
        onDismiss
      });

      await vi.advanceTimersByTimeAsync(4_999);
      expect(onDismiss).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(1);
      expect(onDismiss).toHaveBeenCalledOnce();
    } finally {
      vi.useRealTimers();
    }
  });

  it('automatically dismisses errors after an extended reading interval', async () => {
    vi.useFakeTimers();
    try {
      const onDismiss = vi.fn();
      render(FeedbackBanner, {
        notice: {
          tone: 'danger',
          title: 'Operation failed',
          detail: 'revision-conflict'
        },
        onDismiss
      });

      await vi.advanceTimersByTimeAsync(11_999);
      expect(onDismiss).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(1);
      expect(onDismiss).toHaveBeenCalledOnce();
    } finally {
      vi.useRealTimers();
    }
  });

  it('does not become persistent when the pointer rests over the banner', async () => {
    vi.useFakeTimers();
    try {
      const onDismiss = vi.fn();
      render(FeedbackBanner, {
        notice: {
          tone: 'info',
          title: 'Location selected',
          detail: 'Ready for preview'
        },
        onDismiss
      });

      const banner = screen.getByRole('status');
      await vi.advanceTimersByTimeAsync(2_000);
      await fireEvent.mouseEnter(banner);
      await vi.advanceTimersByTimeAsync(2_999);
      expect(onDismiss).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(1);
      expect(onDismiss).toHaveBeenCalledOnce();
    } finally {
      vi.useRealTimers();
    }
  });

  it('pauses automatic dismissal only while keyboard focus is inside', async () => {
    vi.useFakeTimers();
    try {
      const onDismiss = vi.fn();
      render(FeedbackBanner, {
        notice: {
          tone: 'info',
          title: 'Location selected',
          detail: 'Ready for preview'
        },
        onDismiss
      });

      const dismiss = screen.getByRole('button', { name: 'Dismiss message' });
      await vi.advanceTimersByTimeAsync(2_000);
      dismiss.focus();
      await fireEvent.focusIn(dismiss);
      await vi.advanceTimersByTimeAsync(10_000);
      expect(onDismiss).not.toHaveBeenCalled();

      dismiss.blur();
      await fireEvent.focusOut(dismiss);
      await vi.advanceTimersByTimeAsync(2_999);
      expect(onDismiss).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(1);
      expect(onDismiss).toHaveBeenCalledOnce();
    } finally {
      vi.useRealTimers();
    }
  });
});
