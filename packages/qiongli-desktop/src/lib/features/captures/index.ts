import type { FeatureDescriptor } from '../types';
import type {
  ArtifactChangeSnapshot,
  AppEvent,
  AppIntent,
  CaptureInboxEntry,
  CaptureSourceCoverage,
  StatusCode
} from '@qiongli/app-api';

export const capturesFeature: FeatureDescriptor = {
  id: 'captures',
  label: 'Research Captures',
  route: '/captures',
  stage: 'r4b'
};

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

type CaptureLoadIntent = Extract<AppIntent, {
  action: 'load-capture-inbox' | 'load-capture-coverage' | 'load-artifact-changes';
}>;

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
