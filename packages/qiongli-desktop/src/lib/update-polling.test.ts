import { afterEach, describe, expect, it, vi } from 'vitest';

import { createUpdatePollingController, type UpdatePollResult } from './update-polling';

afterEach(() => {
  vi.useRealTimers();
});

describe('createUpdatePollingController', () => {
  it('maintains one polling loop while an update remains busy', async () => {
    vi.useFakeTimers();
    const poll = vi.fn<() => Promise<UpdatePollResult>>()
      .mockResolvedValueOnce('busy')
      .mockResolvedValueOnce('settled');
    const controller = createUpdatePollingController({ poll, intervalMs: 100 });

    controller.sync(true);
    controller.sync(true);
    expect(vi.getTimerCount()).toBe(1);

    await vi.advanceTimersByTimeAsync(100);
    expect(poll).toHaveBeenCalledTimes(1);
    expect(vi.getTimerCount()).toBe(1);

    await vi.advanceTimersByTimeAsync(100);
    expect(poll).toHaveBeenCalledTimes(2);
    expect(vi.getTimerCount()).toBe(0);
  });

  it('clears its pending timer when the route is destroyed', async () => {
    vi.useFakeTimers();
    const poll = vi.fn<() => Promise<UpdatePollResult>>().mockResolvedValue('busy');
    const controller = createUpdatePollingController({ poll, intervalMs: 100 });

    controller.sync(true);
    controller.destroy();
    expect(vi.getTimerCount()).toBe(0);

    await vi.advanceTimersByTimeAsync(1_000);
    expect(poll).not.toHaveBeenCalled();
  });

  it('backs off, pauses after bounded failures, and supports an explicit retry', async () => {
    vi.useFakeTimers();
    const pauseChanges = vi.fn();
    const poll = vi.fn<() => Promise<UpdatePollResult>>()
      .mockResolvedValueOnce('failed')
      .mockResolvedValueOnce('failed')
      .mockResolvedValueOnce('failed')
      .mockResolvedValueOnce('settled');
    const controller = createUpdatePollingController({
      poll,
      intervalMs: 100,
      maxConsecutiveFailures: 3,
      onPauseChange: pauseChanges
    });

    controller.sync(true);
    await vi.advanceTimersByTimeAsync(100);
    await vi.advanceTimersByTimeAsync(200);
    await vi.advanceTimersByTimeAsync(400);

    expect(poll).toHaveBeenCalledTimes(3);
    expect(pauseChanges).toHaveBeenLastCalledWith(true);
    expect(vi.getTimerCount()).toBe(0);

    controller.retry();
    expect(pauseChanges).toHaveBeenLastCalledWith(false);
    await vi.advanceTimersByTimeAsync(100);

    expect(poll).toHaveBeenCalledTimes(4);
    expect(vi.getTimerCount()).toBe(0);
  });
});
