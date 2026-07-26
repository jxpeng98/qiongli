import type {
  AppEvent,
  AppIntent,
  PortfolioQueryFilters,
  PortfolioQueryRequest,
  PortfolioQueryResult,
  PortfolioStatus,
  StatusCode
} from '@qiongli/app-api';

import type { FeatureDescriptor } from '../types';

export const portfolioFeature: FeatureDescriptor = {
  id: 'portfolio',
  label: 'Research Portfolio',
  route: '/portfolio',
  stage: 'r5c'
};

export const portfolioQueryLimits: PortfolioQueryRequest['limits'] = {
  projects: 32,
  nodes: 64,
  edges: 64,
  lineage: 64,
  maxBytes: 512 * 1_024
};

export interface PortfolioQueryWorkspace {
  requestId: string;
  queryId: string;
  catalogId: string;
  portfolioId: string;
  lineageDigest: string;
  matchedProjectCount: number;
  matchedNodeCount: number;
  matchedEdgeCount: number;
  matchedLineageCount: number;
  projects: PortfolioQueryResult['projects'];
  nodes: PortfolioQueryResult['nodes'];
  edges: PortfolioQueryResult['edges'];
  lineage: PortfolioQueryResult['lineage'];
  nextCursor: PortfolioQueryResult['nextCursor'];
  projectsTruncated: boolean;
  nodesTruncated: boolean;
  edgesTruncated: boolean;
  lineageTruncated: boolean;
}

type PortfolioQueryIntent = Extract<AppIntent, { action: 'query-portfolio' }>;

export function portfolioStatusCode(status: PortfolioStatus): StatusCode {
  switch (status.state) {
    case 'current': return 'ready';
    case 'missing': return 'missing';
    case 'stale': return 'drifted';
    case 'recovery-required': return 'recovery-required';
  }
}

export function portfolioFilterKey(
  catalogId: string,
  filters: PortfolioQueryFilters
): string {
  return JSON.stringify([
    catalogId,
    filters.projectId ?? null,
    filters.stage ?? null,
    filters.evidenceSignal ?? null,
    filters.manuscriptSection ?? null,
    filters.sharedIdentity?.nodeType ?? null,
    filters.sharedIdentity?.canonicalId ?? null,
    filters.captureSource ?? null,
    filters.captureDelivery ?? null,
    filters.deliveryState ?? null,
    filters.assignmentOutcome ?? null,
    filters.lineageId ?? null,
    filters.text ?? null
  ]);
}

export async function loadPortfolioQueryPage(
  catalogId: string,
  filters: PortfolioQueryFilters,
  cursor: PortfolioQueryResult['nextCursor'],
  execute: (intent: PortfolioQueryIntent) => Promise<AppEvent | null>
): Promise<PortfolioQueryResult | null> {
  try {
    const event = await execute({
      action: 'query-portfolio',
      request: {
        catalogId,
        ...(Object.keys(filters).length > 0 ? { filters } : {}),
        limits: portfolioQueryLimits,
        ...(cursor ? { cursor } : {})
      }
    });
    if (event?.type !== 'portfolio-query' || event.result.catalogId !== catalogId) {
      return null;
    }
    if (cursor && event.result.queryId !== cursor.queryId) return null;
    return event.result;
  } catch {
    return null;
  }
}

export function portfolioWorkspaceFromResult(
  result: PortfolioQueryResult
): PortfolioQueryWorkspace {
  return { ...result };
}

export function mergePortfolioQueryWorkspace(
  current: PortfolioQueryWorkspace,
  next: PortfolioQueryResult
): PortfolioQueryWorkspace | null {
  if (
    current.catalogId !== next.catalogId
    || current.portfolioId !== next.portfolioId
    || current.queryId !== next.queryId
    || current.lineageDigest !== next.lineageDigest
    || current.matchedProjectCount !== next.matchedProjectCount
    || current.matchedNodeCount !== next.matchedNodeCount
    || current.matchedEdgeCount !== next.matchedEdgeCount
    || current.matchedLineageCount !== next.matchedLineageCount
  ) return null;

  if (
    hasDuplicate(current.projects, next.projects, (entry) => entry.resultId)
    || hasDuplicate(current.nodes, next.nodes, (entry) => entry.resultId)
    || hasDuplicate(current.edges, next.edges, (entry) => entry.resultId)
    || hasDuplicate(current.lineage, next.lineage, (entry) => entry.lineageId)
  ) return null;

  return {
    ...next,
    projects: [...current.projects, ...next.projects],
    nodes: [...current.nodes, ...next.nodes],
    edges: [...current.edges, ...next.edges],
    lineage: [...current.lineage, ...next.lineage]
  };
}

export function portfolioWorkspaceIsEmpty(result: PortfolioQueryWorkspace): boolean {
  return result.projects.length === 0
    && result.nodes.length === 0
    && result.edges.length === 0
    && result.lineage.length === 0;
}

function hasDuplicate<T>(
  current: readonly T[],
  next: readonly T[],
  identity: (item: T) => string
): boolean {
  const identities = new Set(current.map(identity));
  return next.some((entry) => identities.has(identity(entry)));
}
