export type UpdatePollResult = 'busy' | 'settled' | 'failed';

export interface UpdatePollingController {
  sync(shouldPoll: boolean): void;
  retry(): void;
  destroy(): void;
}

export interface UpdatePollingOptions {
  poll: () => Promise<UpdatePollResult>;
  onPauseChange?: (paused: boolean) => void;
  intervalMs?: number;
  maxConsecutiveFailures?: number;
}

/**
 * Owns the single update-status polling loop for the About route.
 *
 * A busy update starts one loop, repeated syncs are idempotent, and transient
 * failures back off before the loop pauses with an explicit retry path.
 */
export function createUpdatePollingController({
  poll,
  onPauseChange,
  intervalMs = 350,
  maxConsecutiveFailures = 3
}: UpdatePollingOptions): UpdatePollingController {
  let desired = false;
  let destroyed = false;
  let inFlight = false;
  let paused = false;
  let failures = 0;
  let generation = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;

  function setPaused(next: boolean): void {
    if (paused === next) return;
    paused = next;
    onPauseChange?.(next);
  }

  function clearPendingTimer(): void {
    if (timer === null) return;
    clearTimeout(timer);
    timer = null;
  }

  function schedule(delayMs = intervalMs): void {
    if (destroyed || !desired || paused || inFlight || timer !== null) return;
    timer = setTimeout(() => {
      timer = null;
      void runPoll();
    }, delayMs);
  }

  async function runPoll(): Promise<void> {
    if (destroyed || !desired || paused || inFlight) return;
    inFlight = true;
    const runGeneration = generation;
    let result: UpdatePollResult;
    try {
      result = await poll();
    } catch {
      result = 'failed';
    }
    inFlight = false;

    if (destroyed) return;
    if (runGeneration !== generation) {
      schedule();
      return;
    }

    if (result === 'settled') {
      desired = false;
      failures = 0;
      return;
    }

    if (result === 'busy') {
      failures = 0;
      schedule();
      return;
    }

    failures += 1;
    if (failures >= maxConsecutiveFailures) {
      setPaused(true);
      return;
    }
    schedule(intervalMs * (2 ** failures));
  }

  return {
    sync(shouldPoll: boolean): void {
      if (destroyed || shouldPoll === desired) return;
      generation += 1;
      desired = shouldPoll;
      failures = 0;
      clearPendingTimer();
      setPaused(false);
      if (shouldPoll) schedule();
    },

    retry(): void {
      if (destroyed || !desired || !paused) return;
      generation += 1;
      failures = 0;
      setPaused(false);
      schedule();
    },

    destroy(): void {
      if (destroyed) return;
      destroyed = true;
      desired = false;
      generation += 1;
      clearPendingTimer();
    }
  };
}
