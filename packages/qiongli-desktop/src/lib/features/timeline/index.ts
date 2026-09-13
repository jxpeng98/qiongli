import type {
  AppEvent,
  AppIntent,
  SemanticTimelineRequest,
  SemanticTimelineResult
} from '@qiongli/app-api';

import type { FeatureDescriptor } from '../types';

export const timelineFeature: FeatureDescriptor = {
  id: 'timeline',
  label: 'Semantic Timeline',
  route: '/timeline',
  stage: 'r5c'
};

export type TimelineMode =
  | 'portfolio-activity'
  | 'project-activity'
  | 'revision-history'
  | 'merge-resolution-history';

export interface TimelineSelection {
  mode: TimelineMode;
  projectId: string | null;
}

export interface TimelineWorkspace {
  requestId: string;
  queryId: string;
  catalogId: string;
  portfolioId: string;
  timelineDigest: string;
  projectId: string | null;
  view: SemanticTimelineResult['view'];
  matchedEventCount: number;
  events: SemanticTimelineResult['events'];
  nextCursor: SemanticTimelineResult['nextCursor'];
  truncated: boolean;
}

export type TimelineRelatedIdentityKind =
  | 'project'
  | 'projection'
  | 'graph-node'
  | 'graph-edge'
  | 'capture'
  | 'delivery'
  | 'delivery-acknowledgement'
  | 'assignment-intent'
  | 'assignment-receipt'
  | 'resolution-item'
  | 'resolution-receipt'
  | 'portfolio'
  | 'unknown';

type TimelineIntent = Extract<AppIntent, { action: 'load-semantic-timeline' }>;

export const timelinePageLimit = 64;
export const timelineMaxBytes = 512 * 1_024;

export function timelineSelectionKey(
  catalogId: string,
  selection: TimelineSelection
): string {
  return JSON.stringify([
    catalogId,
    selection.mode,
    selection.mode === 'portfolio-activity' ? null : selection.projectId
  ]);
}

export function timelineRequest(
  catalogId: string,
  selection: TimelineSelection,
  cursor: SemanticTimelineResult['nextCursor'] = null
): SemanticTimelineRequest | null {
  if (selection.mode === 'project-activity' && !selection.projectId) return null;
  const view = selection.mode === 'portfolio-activity'
    || selection.mode === 'project-activity'
    ? 'activity'
    : selection.mode;
  const projectId = selection.mode === 'portfolio-activity'
    ? null
    : selection.projectId;
  return {
    catalogId,
    ...(projectId ? { projectId } : {}),
    view,
    limit: timelinePageLimit,
    maxBytes: timelineMaxBytes,
    ...(cursor ? { cursor } : {})
  };
}

export async function loadTimelinePage(
  catalogId: string,
  selection: TimelineSelection,
  cursor: SemanticTimelineResult['nextCursor'],
  execute: (intent: TimelineIntent) => Promise<AppEvent | null>
): Promise<SemanticTimelineResult | null> {
  const request = timelineRequest(catalogId, selection, cursor);
  if (!request) return null;
  try {
    const event = await execute({
      action: 'load-semantic-timeline',
      request
    });
    if (
      event?.type !== 'semantic-timeline'
      || !timelineResultMatchesSelection(event.result, catalogId, selection)
      || (cursor && event.result.queryId !== cursor.queryId)
      || !validTimelinePage(event.result)
      || (!cursor
        && !event.result.nextCursor
        && event.result.events.length !== event.result.matchedEventCount)
    ) return null;
    return event.result;
  } catch {
    return null;
  }
}

export function timelineWorkspaceFromResult(
  result: SemanticTimelineResult
): TimelineWorkspace | null {
  return validTimelinePage(result)
    && (result.nextCursor !== null || result.events.length === result.matchedEventCount)
    ? { ...result }
    : null;
}

export function mergeTimelineWorkspace(
  current: TimelineWorkspace,
  next: SemanticTimelineResult
): TimelineWorkspace | null {
  if (
    current.requestId !== next.requestId
    || current.queryId !== next.queryId
    || current.catalogId !== next.catalogId
    || current.portfolioId !== next.portfolioId
    || current.timelineDigest !== next.timelineDigest
    || current.projectId !== next.projectId
    || current.view !== next.view
    || current.matchedEventCount !== next.matchedEventCount
    || !validTimelinePage(next)
  ) return null;

  const identities = new Set(current.events.map((event) => event.eventId));
  if (next.events.some((event) => identities.has(event.eventId))) return null;
  const last = current.events.at(-1);
  const first = next.events[0];
  if (last && first && compareTimelineEvents(last, first) >= 0) return null;

  const events = [...current.events, ...next.events];
  if (
    events.length > next.matchedEventCount
    || (!next.nextCursor && events.length !== next.matchedEventCount)
  ) return null;
  return { ...next, events };
}

export function timelineWorkspaceIsEmpty(workspace: TimelineWorkspace): boolean {
  return workspace.matchedEventCount === 0 && workspace.events.length === 0;
}

export function timelineRelatedIdentityKind(value: string): TimelineRelatedIdentityKind {
  if (/^prj_[0-9a-f]{32}$/.test(value)) return 'project';
  if (/^grp_[0-9a-f]{64}$/.test(value)) return 'projection';
  if (/^nod_[0-9a-f]{64}$/.test(value)) return 'graph-node';
  if (/^edg_[0-9a-f]{64}$/.test(value)) return 'graph-edge';
  if (/^cap_[0-9a-f]{64}$/.test(value)) return 'capture';
  if (/^env_[0-9a-f]{64}$/.test(value)) return 'delivery';
  if (/^dack_[0-9a-f]{64}$/.test(value)) return 'delivery-acknowledgement';
  if (/^cai_[0-9a-f]{64}$/.test(value)) return 'assignment-intent';
  if (/^car_[0-9a-f]{64}$/.test(value)) return 'assignment-receipt';
  if (/^cri_[0-9a-f]{64}$/.test(value)) return 'resolution-item';
  if (/^crr_[0-9a-f]{64}$/.test(value)) return 'resolution-receipt';
  if (/^gpf_[0-9a-f]{64}$/.test(value)) return 'portfolio';
  return 'unknown';
}

export function timelineIdentityHref(
  value: string,
  projectId: string | null = null
): string | null {
  const kind = timelineRelatedIdentityKind(value);
  if (kind === 'project') {
    return `/academic-graph?project=${encodeURIComponent(value)}`;
  }
  if (projectId && (kind === 'graph-node' || kind === 'graph-edge')) {
    return `/academic-graph?project=${encodeURIComponent(projectId)}&entity=${encodeURIComponent(value)}`;
  }
  if (projectId && kind === 'capture') {
    return `/captures?project=${encodeURIComponent(projectId)}&capture=${encodeURIComponent(value)}`;
  }
  return null;
}

function timelineResultMatchesSelection(
  result: SemanticTimelineResult,
  catalogId: string,
  selection: TimelineSelection
): boolean {
  const request = timelineRequest(catalogId, selection);
  return request !== null
    && result.catalogId === catalogId
    && result.view === request.view
    && result.projectId === (request.projectId ?? null);
}

function validTimelinePage(result: SemanticTimelineResult): boolean {
  if (result.events.length > result.matchedEventCount) return false;
  const identities = new Set<string>();
  for (let index = 0; index < result.events.length; index += 1) {
    const event = result.events[index]!;
    if (identities.has(event.eventId)) return false;
    identities.add(event.eventId);
    const previous = result.events[index - 1];
    if (previous && compareTimelineEvents(previous, event) >= 0) return false;
  }
  return true;
}

function compareTimelineEvents(
  left: SemanticTimelineResult['events'][number],
  right: SemanticTimelineResult['events'][number]
): number {
  return left.occurredAtUnix - right.occurredAtUnix
    || left.eventId.localeCompare(right.eventId);
}
