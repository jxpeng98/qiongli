import { describe, expect, it, vi } from 'vitest';
import type {
  ArtifactChangeSnapshot,
  AppEvent,
  CaptureAssignmentPage,
  CaptureAssignmentView,
  CaptureDeliveryPage,
  CaptureDeliveryView,
  CaptureInboxEntry,
  CaptureResolutionPreview,
  CaptureResolutionView,
  CaptureSourceCoverage
} from '@qiongli/app-api';

import {
  artifactChangeStatus,
  assignmentStatus,
  canReviewCapture,
  captureStatus,
  completeResolutionSelections,
  coverageStatus,
  deliveryNeedsAttention,
  deliveryStatus,
  loadCaptureConflictState,
  loadCaptureDeliveryPage,
  loadCapturePresentationState,
  mergeAssignmentPages,
  mergeDeliveryPages,
  prioritizeAssignments,
  prioritizeDeliveries,
  resolutionForAssignment
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

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const envelopeId = `env_${'a'.repeat(64)}`;
const assignmentReceiptId = `car_${'b'.repeat(64)}`;

const delivery = {
  schemaVersion: 1,
  envelopeId,
  captureId: `cap_${'c'.repeat(64)}`,
  source: 'codex',
  delivery: 'connected',
  destination: { projectId, expectedProjectRevision: 7 },
  state: 'delivered',
  generation: 3,
  attemptCount: 1,
  retryCount: 0,
  createdAtUnix: 10,
  updatedAtUnix: 20,
  lastReason: 'delivery-accepted',
  envelopeSha256: 'd'.repeat(64),
  recordSha256: 'e'.repeat(64),
  acknowledgement: null,
  capabilities: { canRetry: false, canCancel: false, canAcknowledge: true }
} satisfies CaptureDeliveryView;

const assignment = {
  schemaVersion: 1,
  state: 'completed',
  intentId: `cai_${'f'.repeat(64)}`,
  sourceEnvelopeId: envelopeId,
  sourceCaptureId: delivery.captureId,
  targetProjectId: projectId,
  targetProjectRevision: 7,
  outcome: 'assigned',
  receiptId: assignmentReceiptId,
  derivedCaptureId: `cap_${'1'.repeat(64)}`,
  childEnvelopeId: `env_${'2'.repeat(64)}`,
  createdAtUnix: 20,
  decidedAtUnix: 21,
  canResolve: true
} satisfies CaptureAssignmentView;

const resolution = {
  schemaVersion: 1,
  receiptId: `crr_${'3'.repeat(64)}`,
  assignmentReceiptId,
  sourceEnvelopeId: envelopeId,
  sourceCaptureId: delivery.captureId,
  derivedCaptureId: assignment.derivedCaptureId,
  childEnvelopeId: assignment.childEnvelopeId,
  targetProjectId: projectId,
  fromProjectRevision: 7,
  toProjectRevision: 8,
  reviewedAtUnix: 22,
  resolvedAtUnix: 23,
  decisions: [{
    itemId: `cri_${'4'.repeat(64)}`,
    kind: 'semantic-change',
    disposition: 'accept-capture'
  }]
} satisfies CaptureResolutionView;

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

  it('maps native delivery and assignment attention without upgrading unknown outcomes', () => {
    expect(deliveryStatus(delivery)).toBe('attention');
    expect(deliveryNeedsAttention(delivery)).toBe(true);
    expect(deliveryStatus({
      ...delivery,
      state: 'acknowledged',
      acknowledgement: {
        acknowledgementId: `dack_${'5'.repeat(64)}`,
        destinationProjectId: projectId,
        acceptedCaptureId: delivery.captureId,
        expectedProjectRevision: 7,
        resultingProjectRevision: 8,
        acknowledgedAtUnix: 24
      },
      capabilities: { canRetry: false, canCancel: false, canAcknowledge: false }
    })).toBe('ready');
    expect(deliveryStatus({
      ...delivery,
      state: 'conflicted',
      lastReason: 'delivery-destination-conflict',
      capabilities: { canRetry: true, canCancel: true, canAcknowledge: false }
    })).toBe('conflict');
    expect(assignmentStatus(assignment)).toBe('conflict');
    expect(assignmentStatus({ ...assignment, canResolve: false })).toBe('ready');
    expect(assignmentStatus({
      ...assignment,
      outcome: 'rejected',
      derivedCaptureId: null,
      childEnvelopeId: null,
      canResolve: false
    })).toBe('missing');
  });

  it('keeps attention work first while preserving deterministic history order', () => {
    const acknowledged = {
      ...delivery,
      envelopeId: `env_${'6'.repeat(64)}`,
      state: 'acknowledged',
      updatedAtUnix: 30,
      acknowledgement: {
        acknowledgementId: `dack_${'7'.repeat(64)}`,
        destinationProjectId: projectId,
        acceptedCaptureId: delivery.captureId,
        expectedProjectRevision: 7,
        resultingProjectRevision: 8,
        acknowledgedAtUnix: 30
      },
      capabilities: { canRetry: false, canCancel: false, canAcknowledge: false }
    } satisfies CaptureDeliveryView;
    expect(prioritizeDeliveries([acknowledged, delivery]).map((item) => item.envelopeId))
      .toEqual([delivery.envelopeId, acknowledged.envelopeId]);
    expect(prioritizeAssignments([
      { ...assignment, canResolve: false, createdAtUnix: 40 },
      assignment
    ])[0]?.canResolve).toBe(true);
  });

  it('loads a project-bound Outbox request and rejects a foreign result', async () => {
    const page = {
      schemaVersion: 1,
      snapshotId: `dls_${'8'.repeat(64)}`,
      projectId,
      entries: [delivery],
      truncated: false,
      nextCursor: null
    } satisfies CaptureDeliveryPage;
    const execute = vi.fn().mockResolvedValue({ type: 'capture-deliveries', page } as AppEvent);

    await expect(loadCaptureDeliveryPage(projectId, null, execute)).resolves.toEqual(page);
    expect(execute).toHaveBeenCalledWith({
      action: 'load-capture-deliveries',
      request: { projectId, limit: 128 }
    });
    execute.mockResolvedValueOnce({
      type: 'capture-deliveries',
      page: { ...page, projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4052' }
    } as AppEvent);
    await expect(loadCaptureDeliveryPage(projectId, null, execute)).resolves.toBeNull();
  });

  it('loads assignments and resolutions only as one complete project-bound conflict view', async () => {
    const assignments = {
      schemaVersion: 1,
      snapshotId: `als_${'9'.repeat(64)}`,
      projectId,
      entries: [assignment],
      truncated: false,
      nextCursor: null
    } satisfies CaptureAssignmentPage;
    const resolutions = {
      schemaVersion: 1,
      snapshotId: `rls_${'a'.repeat(64)}`,
      projectId,
      entries: [resolution],
      truncated: false,
      nextCursor: null
    };
    const execute = vi.fn()
      .mockResolvedValueOnce({ type: 'capture-assignments', page: assignments } as AppEvent)
      .mockResolvedValueOnce({ type: 'capture-resolutions', page: resolutions } as AppEvent);

    await expect(loadCaptureConflictState(projectId, execute)).resolves.toEqual({
      assignments,
      resolutions
    });
    expect(execute).toHaveBeenCalledTimes(2);
    expect(resolutionForAssignment(assignment, [resolution])).toEqual(resolution);

    execute.mockReset();
    execute.mockResolvedValueOnce({ type: 'capture-assignments', page: assignments } as AppEvent)
      .mockResolvedValueOnce(null);
    await expect(loadCaptureConflictState(projectId, execute)).resolves.toBeNull();
  });

  it('rejects mixed snapshots and duplicate identities while merging pagination', () => {
    const first = {
      schemaVersion: 1,
      snapshotId: `dls_${'b'.repeat(64)}`,
      projectId,
      entries: [delivery],
      truncated: true,
      nextCursor: {
        schemaVersion: 1,
        cursorId: `apc_${'c'.repeat(64)}`,
        kind: 'deliveries',
        snapshotId: `dls_${'b'.repeat(64)}`,
        afterId: delivery.envelopeId
      }
    } satisfies CaptureDeliveryPage;
    const next = {
      ...first,
      entries: [{ ...delivery, envelopeId: `env_${'d'.repeat(64)}` }],
      truncated: false,
      nextCursor: null
    } satisfies CaptureDeliveryPage;
    expect(mergeDeliveryPages(first, next)?.entries).toHaveLength(2);
    expect(mergeDeliveryPages(first, { ...next, snapshotId: `dls_${'e'.repeat(64)}` }))
      .toBeNull();
    expect(mergeDeliveryPages(first, { ...next, entries: [delivery] })).toBeNull();

    const assignmentPage = {
      schemaVersion: 1,
      snapshotId: `als_${'f'.repeat(64)}`,
      projectId,
      entries: [assignment],
      truncated: false,
      nextCursor: null
    } satisfies CaptureAssignmentPage;
    expect(mergeAssignmentPages(assignmentPage, {
      ...assignmentPage,
      entries: [assignment]
    })).toBeNull();
  });

  it('requires one explicitly allowed disposition for every resolution item', () => {
    const itemId = `cri_${'0'.repeat(64)}`;
    const preview = {
      schemaVersion: 1,
      planDigest: '1'.repeat(64),
      assignmentReceiptId,
      sourceEnvelopeId: envelopeId,
      sourceCaptureId: delivery.captureId,
      derivedCaptureId: assignment.derivedCaptureId,
      childEnvelopeId: assignment.childEnvelopeId,
      targetProjectId: projectId,
      expectedLibraryRevision: 7,
      expectedProjectRevision: 7,
      nextProjectRevision: 8,
      reviewedAtUnix: 24,
      items: [{
        itemId,
        kind: 'semantic-change',
        counterpartState: 'absent',
        allowedDispositions: ['accept-capture', 'reject-capture'],
        unavailableDispositions: ['accept-current', 'retain-both'],
        sourceSummary: 'Add reviewed evidence.',
        currentSummary: null,
        explanation: 'The item has no current counterpart.'
      }],
      approvalsRequired: ['academic-review', 'filesystem-write'],
      exactReplay: false
    } satisfies CaptureResolutionPreview;

    expect(completeResolutionSelections(preview, new Map())).toBeNull();
    expect(completeResolutionSelections(
      preview,
      new Map([[itemId, 'accept-current']])
    )).toBeNull();
    expect(completeResolutionSelections(
      preview,
      new Map([[itemId, 'accept-capture']])
    )).toEqual([{ itemId, disposition: 'accept-capture' }]);
  });
});
