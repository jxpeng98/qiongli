import type { FeatureDescriptor } from '../types';
import type {
  ArtifactChangeSnapshot,
  AppEvent,
  AppIntent,
  CaptureAssignmentPage,
  CaptureAssignmentView,
  CaptureDeliveryPage,
  CaptureDeliveryView,
  CaptureInboxEntry,
  CaptureResolutionPage,
  CaptureResolutionPreview,
  CaptureResolutionSelection,
  CaptureResolutionView,
  CaptureSourceCoverage,
  StatusCode
} from '@qiongli/app-api';

export const capturesFeature: FeatureDescriptor = {
  id: 'captures',
  label: 'Research Captures',
  route: '/captures',
  stage: 'r4b'
};

export type CaptureWorkspaceMode = 'inbox' | 'outbox' | 'conflicts' | 'coverage';

export const captureWorkspaceModes: readonly CaptureWorkspaceMode[] = [
  'inbox',
  'outbox',
  'conflicts',
  'coverage'
];

export function captureStatus(entry: CaptureInboxEntry): StatusCode {
  switch (entry.state) {
    case 'pending-review': return 'attention';
    case 'stale': return 'drifted';
    case 'conflicted': return 'conflict';
    case 'applied': return 'ready';
  }
}

export function canReviewCapture(entry: CaptureInboxEntry): boolean {
  return entry.state !== 'applied';
}

export function coverageStatus(source: CaptureSourceCoverage): StatusCode {
  switch (source.state) {
    case 'pending-review': return 'attention';
    case 'current': return 'ready';
    case 'stale': return 'drifted';
    case 'conflicted': return 'conflict';
    case 'unbound': return 'blocked';
    case 'unknown': return 'missing';
  }
}

export function artifactChangeStatus(changes: ArtifactChangeSnapshot): StatusCode {
  return changes.state === 'current' ? 'ready' : 'attention';
}

export function deliveryStatus(delivery: CaptureDeliveryView): StatusCode {
  switch (delivery.state) {
    case 'queued':
    case 'delivering':
    case 'delivered':
      return 'attention';
    case 'acknowledged':
      return 'ready';
    case 'retry-required':
      return 'drifted';
    case 'conflicted':
      return 'conflict';
    case 'cancelled':
      return 'missing';
  }
}

export function deliveryNeedsAttention(delivery: CaptureDeliveryView): boolean {
  return !['acknowledged', 'cancelled'].includes(delivery.state);
}

export function assignmentStatus(assignment: CaptureAssignmentView): StatusCode {
  if (assignment.state === 'pending') return 'attention';
  if (assignment.outcome === 'rejected') return 'missing';
  return assignment.canResolve ? 'conflict' : 'ready';
}

export function prioritizeDeliveries(entries: readonly CaptureDeliveryView[]): CaptureDeliveryView[] {
  return entries
    .map((entry, index) => ({ entry, index }))
    .sort((left, right) =>
      Number(deliveryNeedsAttention(right.entry)) - Number(deliveryNeedsAttention(left.entry))
      || right.entry.updatedAtUnix - left.entry.updatedAtUnix
      || left.index - right.index
    )
    .map(({ entry }) => entry);
}

export function prioritizeAssignments(
  entries: readonly CaptureAssignmentView[]
): CaptureAssignmentView[] {
  return entries
    .map((entry, index) => ({ entry, index }))
    .sort((left, right) =>
      Number(right.entry.state === 'pending' || right.entry.canResolve)
        - Number(left.entry.state === 'pending' || left.entry.canResolve)
      || right.entry.createdAtUnix - left.entry.createdAtUnix
      || left.index - right.index
    )
    .map(({ entry }) => entry);
}

export function completeResolutionSelections(
  preview: CaptureResolutionPreview,
  selections: ReadonlyMap<string, CaptureResolutionSelection['disposition']>
): CaptureResolutionSelection[] | null {
  if (selections.size !== preview.items.length) return null;
  const complete = preview.items.map((item) => {
    const disposition = selections.get(item.itemId);
    return disposition && item.allowedDispositions.includes(disposition)
      ? { itemId: item.itemId, disposition }
      : null;
  });
  return complete.every((selection) => selection !== null)
    ? complete as CaptureResolutionSelection[]
    : null;
}

type CaptureLoadIntent = Extract<AppIntent, {
  action: 'load-capture-inbox' | 'load-capture-coverage' | 'load-artifact-changes';
}>;

type DeliveryLoadIntent = Extract<AppIntent, { action: 'load-capture-deliveries' }>;
type ConflictLoadIntent = Extract<AppIntent, {
  action: 'load-capture-assignments' | 'load-capture-resolutions';
}>;

export interface CaptureConflictState {
  assignments: CaptureAssignmentPage;
  resolutions: CaptureResolutionPage;
}

/**
 * Loads the three Rust-owned Capture Inbox views as one presentation unit.
 * A partial or mismatched response is never reported as a completed load.
 */
export async function loadCapturePresentationState(
  projectId: string,
  projectRevision: number,
  execute: (intent: CaptureLoadIntent) => Promise<AppEvent | null>
): Promise<boolean> {
  try {
    const inbox = await execute({ action: 'load-capture-inbox', projectId });
    if (
      inbox?.type !== 'capture-inbox'
      || inbox.inbox.projectId !== projectId
      || inbox.inbox.projectRevision !== projectRevision
    ) return false;

    const coverage = await execute({ action: 'load-capture-coverage', projectId });
    if (
      coverage?.type !== 'capture-coverage'
      || coverage.coverage.projectId !== projectId
      || coverage.coverage.projectRevision !== projectRevision
    ) return false;

    const changes = await execute({ action: 'load-artifact-changes', projectId });
    return changes?.type === 'artifact-changes'
      && changes.changes.projectId === projectId
      && changes.changes.projectRevision === projectRevision;
  } catch {
    return false;
  }
}

export async function loadCaptureDeliveryPage(
  projectId: string,
  cursor: CaptureDeliveryPage['nextCursor'],
  execute: (intent: DeliveryLoadIntent) => Promise<AppEvent | null>
): Promise<CaptureDeliveryPage | null> {
  try {
    const event = await execute({
      action: 'load-capture-deliveries',
      request: {
        projectId,
        limit: 128,
        ...(cursor ? { cursor } : {})
      }
    });
    return event?.type === 'capture-deliveries' && event.page.projectId === projectId
      ? event.page
      : null;
  } catch {
    return null;
  }
}

export async function loadCaptureConflictState(
  projectId: string,
  execute: (intent: ConflictLoadIntent) => Promise<AppEvent | null>
): Promise<CaptureConflictState | null> {
  try {
    const assignments = await execute({
      action: 'load-capture-assignments',
      request: { projectId, limit: 128 }
    });
    if (assignments?.type !== 'capture-assignments'
      || assignments.page.projectId !== projectId) return null;

    const resolutions = await execute({
      action: 'load-capture-resolutions',
      request: { projectId, limit: 64 }
    });
    return resolutions?.type === 'capture-resolutions'
      && resolutions.page.projectId === projectId
      ? { assignments: assignments.page, resolutions: resolutions.page }
      : null;
  } catch {
    return null;
  }
}

export function mergeDeliveryPages(
  current: CaptureDeliveryPage,
  next: CaptureDeliveryPage
): CaptureDeliveryPage | null {
  if (current.projectId !== next.projectId || current.snapshotId !== next.snapshotId) return null;
  const ids = new Set(current.entries.map((entry) => entry.envelopeId));
  if (next.entries.some((entry) => ids.has(entry.envelopeId))) return null;
  return { ...next, entries: [...current.entries, ...next.entries] };
}

export function mergeAssignmentPages(
  current: CaptureAssignmentPage,
  next: CaptureAssignmentPage
): CaptureAssignmentPage | null {
  if (current.projectId !== next.projectId || current.snapshotId !== next.snapshotId) return null;
  const ids = new Set(current.entries.map((entry) => entry.intentId));
  if (next.entries.some((entry) => ids.has(entry.intentId))) return null;
  return { ...next, entries: [...current.entries, ...next.entries] };
}

export function mergeResolutionPages(
  current: CaptureResolutionPage,
  next: CaptureResolutionPage
): CaptureResolutionPage | null {
  if (current.projectId !== next.projectId || current.snapshotId !== next.snapshotId) return null;
  const ids = new Set(current.entries.map((entry) => entry.receiptId));
  if (next.entries.some((entry) => ids.has(entry.receiptId))) return null;
  return { ...next, entries: [...current.entries, ...next.entries] };
}

export function resolutionForAssignment(
  assignment: CaptureAssignmentView,
  resolutions: readonly CaptureResolutionView[]
): CaptureResolutionView | null {
  if (!assignment.receiptId) return null;
  return resolutions.find((resolution) =>
    resolution.assignmentReceiptId === assignment.receiptId
  ) ?? null;
}
