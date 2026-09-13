import type {
  AppEvent,
  PortfolioQueryResult,
  PortfolioStatus
} from '@qiongli/app-api';
import { QiongliAppClient } from '@qiongli/app-api';
import { describe, expect, it, vi } from 'vitest';

import { sourceFixtureTransport } from '$lib/dev-transport';

import {
  loadPortfolioQueryPage,
  mergePortfolioQueryWorkspace,
  portfolioFilterKey,
  portfolioStatusCode,
  portfolioWorkspaceFromResult,
  portfolioWorkspaceIsEmpty
} from '.';

const catalogId = `pca_${'1'.repeat(64)}`;
const portfolioId = `gpf_${'2'.repeat(64)}`;
const queryId = `pqy_${'3'.repeat(64)}`;

const currentStatus = {
  schemaVersion: 1,
  state: 'current',
  libraryRevision: 7,
  catalogId,
  catalogGeneration: 2,
  portfolioId,
  contributionCount: 2,
  projectCount: 2,
  nodeCount: 3,
  edgeCount: 1,
  reasonCode: 'portfolio-current',
  capabilities: {
    canQuery: true,
    canReconcile: true,
    canRebuild: true,
    canDeleteDerivedState: true
  }
} satisfies PortfolioStatus;

function queryResult(
  projectResultId: string,
  truncated: boolean
): PortfolioQueryResult {
  return {
    schemaVersion: 1,
    requestId: `pqr_${(truncated ? '4' : '5').repeat(64)}`,
    queryId,
    catalogId,
    portfolioId,
    lineageDigest: `plg_${'6'.repeat(64)}`,
    matchedProjectCount: 2,
    matchedNodeCount: 0,
    matchedEdgeCount: 0,
    matchedLineageCount: 0,
    projectsTruncated: truncated,
    nodesTruncated: false,
    edgesTruncated: false,
    lineageTruncated: false,
    projects: [{
      resultId: projectResultId,
      projectId: `prj_${(truncated ? '7' : '8').repeat(32)}`,
      displayName: truncated ? 'First project' : 'Second project',
      stage: 'writing',
      lifecycle: 'active',
      health: 'ready',
      semanticRevision: 2,
      projectionId: `grp_${(truncated ? '9' : 'a').repeat(64)}`,
      nodeCount: 0,
      edgeCount: 0,
      lineageCount: 0
    }],
    nodes: [],
    edges: [],
    lineage: [],
    nextCursor: truncated ? {
      cursorId: `pqc_${'b'.repeat(64)}`,
      queryId,
      projectAfter: projectResultId
    } : null
  };
}

describe('Portfolio presentation model', () => {
  it.each([
    ['current', 'ready'],
    ['missing', 'missing'],
    ['stale', 'drifted'],
    ['recovery-required', 'recovery-required']
  ] as const)('maps catalog state %s to truthful status %s', (state, expected) => {
    const status = state === 'current'
      ? currentStatus
      : {
          ...currentStatus,
          state,
          catalogId: null,
          catalogGeneration: null,
          portfolioId: null,
          capabilities: { ...currentStatus.capabilities, canQuery: false }
        };
    expect(portfolioStatusCode(status)).toBe(expected);
  });

  it('creates a stable filter identity for every native filter field', () => {
    const first = portfolioFilterKey(catalogId, {
      projectId: `prj_${'c'.repeat(32)}`,
      stage: 'analysis',
      evidenceSignal: 'gap',
      manuscriptSection: 'Discussion',
      sharedIdentity: { nodeType: 'paper', canonicalId: 'doi:10.1/example' },
      captureSource: 'codex',
      captureDelivery: 'connected',
      deliveryState: 'acknowledged',
      assignmentOutcome: 'assigned',
      lineageId: `lin_${'d'.repeat(64)}`,
      text: 'portable provenance'
    });
    expect(first).not.toBe(portfolioFilterKey(catalogId, {
      projectId: `prj_${'c'.repeat(32)}`,
      stage: 'analysis',
      evidenceSignal: 'contradiction'
    }));
    expect(first).not.toBe(portfolioFilterKey(`pca_${'e'.repeat(64)}`, {}));
  });

  it('loads a catalog-bound page and rejects a foreign catalog or query cursor', async () => {
    const first = queryResult('project:first', true);
    const execute = vi.fn().mockResolvedValue({
      type: 'portfolio-query',
      result: first
    } as AppEvent);
    await expect(loadPortfolioQueryPage(catalogId, { stage: 'writing' }, null, execute))
      .resolves.toEqual(first);
    expect(execute).toHaveBeenCalledWith({
      action: 'query-portfolio',
      request: {
        catalogId,
        filters: { stage: 'writing' },
        limits: {
          projects: 32,
          nodes: 64,
          edges: 64,
          lineage: 64,
          maxBytes: 524_288
        }
      }
    });

    execute.mockResolvedValueOnce({
      type: 'portfolio-query',
      result: { ...first, catalogId: `pca_${'f'.repeat(64)}` }
    } as AppEvent);
    await expect(loadPortfolioQueryPage(catalogId, {}, null, execute)).resolves.toBeNull();

    execute.mockResolvedValueOnce({
      type: 'portfolio-query',
      result: { ...first, queryId: `pqy_${'0'.repeat(64)}` }
    } as AppEvent);
    await expect(loadPortfolioQueryPage(catalogId, {}, first.nextCursor, execute))
      .resolves.toBeNull();
  });

  it('appends only the matching native page and rejects identity drift or duplicates', () => {
    const first = queryResult('project:first', true);
    const next = queryResult('project:second', false);
    const workspace = portfolioWorkspaceFromResult(first);

    expect(mergePortfolioQueryWorkspace(workspace, next)?.projects.map((entry) => entry.resultId))
      .toEqual(['project:first', 'project:second']);
    expect(mergePortfolioQueryWorkspace(workspace, {
      ...next,
      catalogId: `pca_${'f'.repeat(64)}`
    })).toBeNull();
    expect(mergePortfolioQueryWorkspace(workspace, {
      ...next,
      projects: [first.projects[0]]
    })).toBeNull();
  });

  it('distinguishes a complete empty result from an unavailable workspace', () => {
    const empty = portfolioWorkspaceFromResult({
      ...queryResult('project:first', false),
      matchedProjectCount: 0,
      projects: []
    });
    expect(portfolioWorkspaceIsEmpty(empty)).toBe(true);
  });

  it('keeps the strict development fixture valid through query and maintenance polling', async () => {
    const client = new QiongliAppClient(sourceFixtureTransport());
    const statusEvent = await client.execute({ action: 'load-portfolio-status' });
    expect(statusEvent.type).toBe('portfolio-status');
    if (statusEvent.type !== 'portfolio-status' || !statusEvent.portfolio.catalogId) return;

    const first = await client.execute({
      action: 'query-portfolio',
      request: {
        catalogId: statusEvent.portfolio.catalogId,
        limits: {
          projects: 32,
          nodes: 64,
          edges: 64,
          lineage: 64,
          maxBytes: 524_288
        }
      }
    });
    expect(first.type).toBe('portfolio-query');
    if (first.type !== 'portfolio-query' || !first.result.nextCursor) return;

    const next = await client.execute({
      action: 'query-portfolio',
      request: {
        catalogId: statusEvent.portfolio.catalogId,
        limits: {
          projects: 32,
          nodes: 64,
          edges: 64,
          lineage: 64,
          maxBytes: 524_288
        },
        cursor: first.result.nextCursor
      }
    });
    expect(next.type === 'portfolio-query' && next.result.nextCursor).toBeNull();

    const preview = await client.execute({
      action: 'preview-portfolio-maintenance',
      operation: 'reconcile'
    });
    expect(preview.type).toBe('portfolio-maintenance-preview');
    if (preview.type !== 'portfolio-maintenance-preview') return;

    const queued = await client.execute({
      action: 'confirm-operation',
      token: preview.preview.token
    });
    expect(queued.type).toBe('continuity-operation-progress');
    if (queued.type !== 'continuity-operation-progress') return;

    const running = await client.execute({
      action: 'poll-continuity-operation',
      operationId: queued.progress.operationId
    });
    expect(running.type).toBe('continuity-operation-progress');
    const completed = await client.execute({
      action: 'poll-continuity-operation',
      operationId: queued.progress.operationId
    });
    expect(completed.type).toBe('portfolio-maintenance-completed');
  });
});
