import { describe, expect, it, vi } from 'vitest';
import type {
  ArtifactChangeSnapshot,
  AppEvent,
  CaptureInboxEntry,
  CaptureSourceCoverage
} from '@qiongli/app-api';

import {
  artifactChangeStatus,
  canReviewCapture,
  captureStatus,
  coverageStatus,
  loadCapturePresentationState
} from '.';

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

  it('keeps unattributed artifact drift visible instead of reporting it current', () => {
    const changes = {
      state: 'unattributed'
    } as ArtifactChangeSnapshot;
    expect(artifactChangeStatus(changes)).toBe('attention');
    expect(artifactChangeStatus({ ...changes, state: 'current' })).toBe('ready');
  });

  it('reports ready only after all three matching native views load', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const execute = vi.fn()
      .mockResolvedValueOnce({ type: 'capture-inbox', inbox: { projectId, projectRevision: 7 } } as AppEvent)
      .mockResolvedValueOnce({ type: 'capture-coverage', coverage: { projectId, projectRevision: 7 } } as AppEvent)
      .mockResolvedValueOnce({ type: 'artifact-changes', changes: { projectId, projectRevision: 7 } } as AppEvent);

    await expect(loadCapturePresentationState(projectId, 7, execute)).resolves.toBe(true);
    expect(execute).toHaveBeenCalledTimes(3);
  });

  it('rejects partial success instead of leaving the presentation loading forever', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const execute = vi.fn()
      .mockResolvedValueOnce({ type: 'capture-inbox', inbox: { projectId, projectRevision: 7 } } as AppEvent)
      .mockResolvedValueOnce(null);

    await expect(loadCapturePresentationState(projectId, 7, execute)).resolves.toBe(false);
    expect(execute).toHaveBeenCalledTimes(2);
  });

  it('rejects a native view for a different project', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const execute = vi.fn().mockResolvedValueOnce({
      type: 'capture-inbox',
      inbox: { projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4052' }
    } as AppEvent);

    await expect(loadCapturePresentationState(projectId, 7, execute)).resolves.toBe(false);
    expect(execute).toHaveBeenCalledOnce();
  });

  it('rejects native views read from different project revisions', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const execute = vi.fn()
      .mockResolvedValueOnce({ type: 'capture-inbox', inbox: { projectId, projectRevision: 7 } } as AppEvent)
      .mockResolvedValueOnce({ type: 'capture-coverage', coverage: { projectId, projectRevision: 8 } } as AppEvent);

    await expect(loadCapturePresentationState(projectId, 7, execute)).resolves.toBe(false);
    expect(execute).toHaveBeenCalledTimes(2);
  });

  it('rejects a coherent view set from an older requested revision', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const execute = vi.fn()
      .mockResolvedValueOnce({ type: 'capture-inbox', inbox: { projectId, projectRevision: 7 } } as AppEvent);

    await expect(loadCapturePresentationState(projectId, 8, execute)).resolves.toBe(false);
    expect(execute).toHaveBeenCalledOnce();
  });
});
