import type {
  AppEvent,
  SemanticTimelineResult
} from '@qiongli/app-api';
import { QiongliAppClient } from '@qiongli/app-api';
import { describe, expect, it, vi } from 'vitest';

import { sourceFixtureTransport } from '$lib/dev-transport';

import {
  loadTimelinePage,
  mergeTimelineWorkspace,
  timelineIdentityHref,
  timelineRelatedIdentityKind,
  timelineRequest,
  timelineSelectionKey,
  timelineWorkspaceFromResult,
  timelineWorkspaceIsEmpty,
  type TimelineSelection
} from '.';

const catalogId = `pca_${'1'.repeat(64)}`;
const portfolioId = `gpf_${'2'.repeat(64)}`;
const projectId = `prj_${'3'.repeat(32)}`;
const queryId = `pty_${'4'.repeat(64)}`;

const projectActivity = {
  mode: 'project-activity',
  projectId
} satisfies TimelineSelection;

function event(id: string, occurredAtUnix: number): SemanticTimelineResult['events'][number] {
  return {
    eventId: `pte_${id.repeat(64)}`,
    kind: 'project-registered',
    occurredAtUnix,
    timestampSource: 'project-registered-at',
    projectIds: [projectId],
    relatedIds: [projectId],
    fromProjectRevision: null,
    toProjectRevision: null,
    lifecycle: null,
    source: null,
    delivery: null,
    deliveryState: null,
    deliveryReason: null,
    deliveryGeneration: null,
    assignmentOutcome: null,
    resolutionItemId: null,
    resolutionItemKind: null,
    resolutionDisposition: null
  };
}

function result(secondPage = false): SemanticTimelineResult {
  return {
    schemaVersion: 1,
    requestId: `ptr_${'5'.repeat(64)}`,
    queryId,
    catalogId,
    portfolioId,
    timelineDigest: `ptl_${'6'.repeat(64)}`,
    projectId,
    view: 'activity',
    matchedEventCount: 2,
    truncated: !secondPage,
    events: [event(secondPage ? '8' : '7', secondPage ? 20 : 10)],
    nextCursor: secondPage ? null : {
      cursorId: `ptc_${'9'.repeat(64)}`,
      queryId,
      afterOccurredAtUnix: 10,
      afterEventId: `pte_${'7'.repeat(64)}`
    }
  };
}

describe('Semantic Timeline presentation model', () => {
  it('maps four experience modes onto the three native views', () => {
    expect(timelineRequest(catalogId, {
      mode: 'portfolio-activity',
      projectId: projectId
    })).toMatchObject({ view: 'activity', catalogId });
    expect(timelineRequest(catalogId, {
      mode: 'portfolio-activity',
      projectId
    })).not.toHaveProperty('projectId');
    expect(timelineRequest(catalogId, projectActivity)).toMatchObject({
      view: 'activity',
      projectId
    });
    expect(timelineRequest(catalogId, {
      mode: 'revision-history',
      projectId: null
    })).toMatchObject({ view: 'revision-history' });
    expect(timelineRequest(catalogId, {
      mode: 'merge-resolution-history',
      projectId
    })).toMatchObject({ view: 'merge-resolution-history', projectId });
    expect(timelineRequest(catalogId, {
      mode: 'project-activity',
      projectId: null
    })).toBeNull();
  });

  it('binds selection identity to catalog, mode, and applicable project scope', () => {
    expect(timelineSelectionKey(catalogId, {
      mode: 'portfolio-activity',
      projectId
    })).toBe(timelineSelectionKey(catalogId, {
      mode: 'portfolio-activity',
      projectId: null
    }));
    expect(timelineSelectionKey(catalogId, projectActivity))
      .not.toBe(timelineSelectionKey(catalogId, {
        ...projectActivity,
        projectId: `prj_${'a'.repeat(32)}`
      }));
  });

  it('loads only the exact catalog, view, scope, and cursor-bound page', async () => {
    const first = result();
    const execute = vi.fn().mockResolvedValue({
      type: 'semantic-timeline',
      result: first
    } as AppEvent);

    await expect(loadTimelinePage(catalogId, projectActivity, null, execute))
      .resolves.toEqual(first);
    expect(execute).toHaveBeenCalledWith({
      action: 'load-semantic-timeline',
      request: {
        catalogId,
        projectId,
        view: 'activity',
        limit: 64,
        maxBytes: 524_288
      }
    });

    execute.mockResolvedValueOnce({
      type: 'semantic-timeline',
      result: { ...first, catalogId: `pca_${'b'.repeat(64)}` }
    } as AppEvent);
    await expect(loadTimelinePage(catalogId, projectActivity, null, execute))
      .resolves.toBeNull();

    execute.mockResolvedValueOnce({
      type: 'semantic-timeline',
      result: { ...first, view: 'revision-history' }
    } as AppEvent);
    await expect(loadTimelinePage(catalogId, projectActivity, null, execute))
      .resolves.toBeNull();

    execute.mockResolvedValueOnce({
      type: 'semantic-timeline',
      result: { ...first, projectId: null }
    } as AppEvent);
    await expect(loadTimelinePage(catalogId, projectActivity, null, execute))
      .resolves.toBeNull();

    execute.mockResolvedValueOnce({
      type: 'semantic-timeline',
      result: { ...result(true), queryId: `pty_${'c'.repeat(64)}` }
    } as AppEvent);
    await expect(loadTimelinePage(
      catalogId,
      projectActivity,
      first.nextCursor,
      execute
    )).resolves.toBeNull();
  });

  it('appends only a complete, ordered page from the same native snapshot', () => {
    const first = timelineWorkspaceFromResult(result());
    expect(first).not.toBeNull();
    const next = result(true);
    expect(mergeTimelineWorkspace(first!, next)?.events.map((entry) => entry.eventId))
      .toEqual([
        `pte_${'7'.repeat(64)}`,
        `pte_${'8'.repeat(64)}`
      ]);
    expect(mergeTimelineWorkspace(first!, {
      ...next,
      timelineDigest: `ptl_${'d'.repeat(64)}`
    })).toBeNull();
    expect(mergeTimelineWorkspace(first!, {
      ...next,
      requestId: `ptr_${'e'.repeat(64)}`
    })).toBeNull();
    expect(mergeTimelineWorkspace(first!, {
      ...next,
      events: first!.events
    })).toBeNull();
    expect(mergeTimelineWorkspace(first!, {
      ...next,
      events: [event('8', 5)]
    })).toBeNull();
    expect(mergeTimelineWorkspace(first!, {
      ...next,
      matchedEventCount: 3
    })).toBeNull();
  });

  it('distinguishes complete empty history and classifies only known identities', () => {
    const empty = timelineWorkspaceFromResult({
      ...result(true),
      matchedEventCount: 0,
      events: []
    });
    expect(empty && timelineWorkspaceIsEmpty(empty)).toBe(true);
    expect(timelineRelatedIdentityKind(projectId)).toBe('project');
    expect(timelineRelatedIdentityKind(`env_${'e'.repeat(64)}`)).toBe('delivery');
    expect(timelineRelatedIdentityKind('untyped-native-reference')).toBe('unknown');
    expect(timelineIdentityHref(projectId))
      .toBe(`/academic-graph?project=${projectId}`);
    expect(timelineIdentityHref(`cap_${'f'.repeat(64)}`)).toBeNull();
  });

  it('keeps every strict fixture mode valid through content-bound pagination', async () => {
    const client = new QiongliAppClient(sourceFixtureTransport());
    const fixtureCatalogId = `pca_${'6'.repeat(64)}`;
    const fixtureProjectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const modes = [
      { mode: 'portfolio-activity', projectId: null },
      { mode: 'project-activity', projectId: fixtureProjectId },
      { mode: 'revision-history', projectId: null },
      { mode: 'merge-resolution-history', projectId: fixtureProjectId }
    ] satisfies TimelineSelection[];

    for (const selection of modes) {
      const firstRequest = timelineRequest(fixtureCatalogId, selection);
      expect(firstRequest).not.toBeNull();
      const first = await client.execute({
        action: 'load-semantic-timeline',
        request: firstRequest!
      });
      expect(first.type).toBe('semantic-timeline');
      if (first.type !== 'semantic-timeline') continue;
      const workspace = timelineWorkspaceFromResult(first.result);
      expect(workspace).not.toBeNull();
      if (!first.result.nextCursor || !workspace) continue;
      const nextRequest = timelineRequest(
        first.result.catalogId,
        selection,
        first.result.nextCursor
      );
      const next = await client.execute({
        action: 'load-semantic-timeline',
        request: nextRequest!
      });
      expect(next.type).toBe('semantic-timeline');
      if (next.type !== 'semantic-timeline') continue;
      const merged = mergeTimelineWorkspace(workspace, next.result);
      expect(merged?.events).toHaveLength(first.result.matchedEventCount);
      expect(merged?.nextCursor).toBeNull();
    }
  });
});
