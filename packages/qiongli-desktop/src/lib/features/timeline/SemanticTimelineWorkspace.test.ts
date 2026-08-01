import { fireEvent, render, screen } from '@testing-library/svelte';
import type {
  AppSnapshot,
  SemanticTimelineResult
} from '@qiongli/app-api';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import { i18n } from '$lib/i18n.svelte';

import TimelineControls from './TimelineControls.svelte';
import TimelineResults from './TimelineResults.svelte';
import type { TimelineWorkspace } from '.';

const projectId = `prj_${'1'.repeat(32)}`;
const projects = [{
  projectId,
  displayName: 'Trustworthy research agents',
  projectKind: 'article',
  stage: 'writing',
  lifecycle: 'active',
  semanticRevision: 12,
  registeredAtUnix: 1,
  lastOpenedAtUnix: null,
  academicallyUpdatedAtUnix: 2,
  health: 'ready',
  nextAction: 'open',
  rootLabel: 'trustworthy-research-agents',
  overview: {
    focalQuestion: null,
    thesis: null,
    evidencePosition: null,
    unresolvedRiskCount: 0,
    claimEvidenceCoveragePercent: 0,
    nextPriorities: []
  }
}] satisfies AppSnapshot['researchLibrary']['projects'];

function activity(
  identity: string,
  kind: SemanticTimelineResult['events'][number]['kind'],
  occurredAtUnix: number
): SemanticTimelineResult['events'][number] {
  return {
    eventId: `pte_${identity.repeat(64)}`,
    kind,
    occurredAtUnix,
    timestampSource: kind === 'capture-accepted'
      ? 'capture-captured-at'
      : 'delivery-transitioned-at',
    projectIds: [projectId],
    relatedIds: [
      kind === 'capture-accepted'
        ? `cap_${'2'.repeat(64)}`
        : `env_${'3'.repeat(64)}`
    ],
    fromProjectRevision: 12,
    toProjectRevision: kind === 'delivery-acknowledged' ? 13 : null,
    lifecycle: null,
    source: 'codex',
    delivery: kind === 'capture-accepted' ? 'portable' : 'connected',
    deliveryState: kind === 'delivery-acknowledged' ? 'acknowledged' : null,
    deliveryReason: kind === 'delivery-acknowledged'
      ? 'delivery-acknowledged'
      : null,
    deliveryGeneration: kind === 'delivery-acknowledged' ? 2 : null,
    assignmentOutcome: null,
    resolutionItemId: null,
    resolutionItemKind: null,
    resolutionDisposition: null
  };
}

function workspace(events: SemanticTimelineResult['events']): TimelineWorkspace {
  return {
    requestId: `ptr_${'4'.repeat(64)}`,
    queryId: `pty_${'5'.repeat(64)}`,
    catalogId: `pca_${'6'.repeat(64)}`,
    portfolioId: `gpf_${'7'.repeat(64)}`,
    timelineDigest: `ptl_${'8'.repeat(64)}`,
    projectId: null,
    view: 'activity',
    matchedEventCount: 3,
    events,
    nextCursor: {
      cursorId: `ptc_${'9'.repeat(64)}`,
      queryId: `pty_${'5'.repeat(64)}`,
      afterOccurredAtUnix: events.at(-1)?.occurredAtUnix ?? 0,
      afterEventId: events.at(-1)?.eventId ?? `pte_${'a'.repeat(64)}`
    },
    truncated: true
  };
}

beforeAll(() => {
  const values = new Map<string, string>();
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      clear: () => values.clear(),
      getItem: (key: string) => values.get(key) ?? null,
      removeItem: (key: string) => values.delete(key),
      setItem: (key: string, value: string) => values.set(key, value)
    }
  });
});

afterEach(async () => {
  await i18n.setLocale('en');
});

describe('Semantic Timeline workspace controls', () => {
  it('requires an exact project for project activity and submits the native mode', async () => {
    const onApply = vi.fn();
    render(TimelineControls, {
      projects,
      selection: { mode: 'portfolio-activity', projectId: null },
      disabled: false,
      onApply
    });

    await fireEvent.change(screen.getByLabelText('History mode'), {
      target: { value: 'project-activity' }
    });
    expect(screen.getByLabelText('Project scope')).toHaveValue(projectId);
    await fireEvent.click(screen.getByRole('button', { name: 'Load native history' }));
    expect(onApply).toHaveBeenCalledWith({
      mode: 'project-activity',
      projectId
    });
  });

  it('presents explicit timestamp evidence, opaque identities, and native order', async () => {
    const onLoadMore = vi.fn();
    const events = [
      activity('b', 'capture-accepted', 1_784_476_800),
      activity('c', 'delivery-acknowledged', 1_784_563_100)
    ];
    render(TimelineResults, {
      workspace: workspace(events),
      selection: { mode: 'portfolio-activity', projectId: null },
      projects,
      loadingMore: false,
      onLoadMore
    });

    expect(screen.getByText('Timestamp source: Capture record')).toBeInTheDocument();
    expect(screen.getByText('Timestamp source: Delivery transition record'))
      .toBeInTheDocument();
    expect(screen.getByText(/does not infer a human author/)).toBeInTheDocument();
    const result = screen.getByRole('region', { name: 'Native semantic history' });
    expect(result.textContent?.indexOf('Capture accepted'))
      .toBeLessThan(result.textContent?.indexOf('Delivery acknowledged') ?? -1);
    expect(screen.getAllByRole('link', { name: /Trustworthy research agents/ })[0])
      .toHaveAttribute('href', `/academic-graph?project=${projectId}`);
    await fireEvent.click(screen.getByRole('button', {
      name: 'Load next native history page'
    }));
    expect(onLoadMore).toHaveBeenCalledOnce();
  });

  it('announces a complete empty result and renders Chinese causal boundaries', async () => {
    await i18n.setLocale('zh-CN');
    render(TimelineResults, {
      workspace: {
        ...workspace([]),
        matchedEventCount: 0,
        nextCursor: null,
        truncated: false
      },
      selection: { mode: 'merge-resolution-history', projectId: null },
      projects,
      loadingMore: false,
      onLoadMore: vi.fn()
    });
    expect(screen.getByRole('heading', { name: '没有匹配的原生历史' }))
      .toBeInTheDocument();
    expect(screen.getByText(/不会根据事件相邻关系推断人类作者/))
      .toBeInTheDocument();
  });
});
