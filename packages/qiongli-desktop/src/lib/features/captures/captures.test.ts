import { describe, expect, it } from 'vitest';
import type { CaptureInboxEntry, CaptureSourceCoverage } from '@qiongli/app-api';

import { canReviewCapture, captureStatus, coverageStatus } from '.';

const entry = {
  captureId: `cap_${'a'.repeat(64)}`,
  state: 'pending-review',
  disposition: 'refinement',
  source: 'codex',
  delivery: 'portable',
  capturedAtUnix: 1,
  baseRevision: 1,
  boundStage: 'writing',
  task: 'Review a bounded capture',
  capturePolicy: 'review-required',
  summary: 'A bounded academic refinement.',
  changeCount: 1,
  decisionCount: 0,
  evidenceCount: 1,
  contradictionCount: 0,
  nextActionCount: 1,
  historyEntry: 'history/captures/capture.json'
} satisfies CaptureInboxEntry;

describe('Capture Inbox presentation model', () => {
  it.each([
    ['pending-review', 'attention'],
    ['stale', 'drifted'],
    ['conflicted', 'conflict'],
    ['applied', 'ready']
  ] as const)('maps %s to the shared %s status', (state, status) => {
    expect(captureStatus({ ...entry, state })).toBe(status);
  });

  it('keeps unresolved captures reviewable and closes applied captures', () => {
    expect(canReviewCapture(entry)).toBe(true);
    expect(canReviewCapture({ ...entry, state: 'conflicted' })).toBe(true);
    expect(canReviewCapture({ ...entry, state: 'applied' })).toBe(false);
  });

  it('maps truthful coverage states without treating unknown as ready', () => {
    const source = {
      source: 'codex',
      state: 'unknown',
      delivery: 'unknown',
      captureCount: 0,
      pendingReviewCount: 0,
      currentCount: 0,
      staleCount: 0,
      conflictedCount: 0,
      unboundCount: 0,
      latestCaptureId: null,
      lastCapturedAtUnix: null
    } satisfies CaptureSourceCoverage;

    expect(coverageStatus(source)).toBe('missing');
    expect(coverageStatus({ ...source, state: 'current' })).toBe('ready');
    expect(coverageStatus({ ...source, state: 'unbound' })).toBe('blocked');
    expect(coverageStatus({ ...source, state: 'conflicted' })).toBe('conflict');
  });
});
