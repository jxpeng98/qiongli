import { fireEvent, render, screen } from '@testing-library/svelte';
import type {
  ContinuityOperationProgress,
  PortfolioMaintenanceResult,
  PortfolioQueryResult,
  PortfolioStatus
} from '@qiongli/app-api';
import { describe, expect, it, vi } from 'vitest';

import PortfolioFilters from './PortfolioFilters.svelte';
import PortfolioMaintenancePanel from './PortfolioMaintenancePanel.svelte';
import PortfolioResults from './PortfolioResults.svelte';
import PortfolioStatusPanel from './PortfolioStatusPanel.svelte';
import { portfolioWorkspaceFromResult } from '.';

const catalogId = `pca_${'1'.repeat(64)}`;
const projectId = `prj_${'2'.repeat(32)}`;
const status = {
  schemaVersion: 1,
  state: 'current',
  libraryRevision: 7,
  catalogId,
  catalogGeneration: 2,
  portfolioId: `gpf_${'3'.repeat(64)}`,
  contributionCount: 1,
  projectCount: 1,
  nodeCount: 0,
  edgeCount: 0,
  reasonCode: 'portfolio-current',
  capabilities: {
    canQuery: true,
    canReconcile: true,
    canRebuild: true,
    canDeleteDerivedState: false
  }
} satisfies PortfolioStatus;

describe('Portfolio workspace controls', () => {
  it('exposes only native maintenance capabilities and preserves destructive styling', async () => {
    const onPreviewMaintenance = vi.fn();
    render(PortfolioStatusPanel, {
      status,
      busy: false,
      onDoctor: vi.fn(),
      onPreviewMaintenance
    });

    expect(screen.getByRole('button', { name: 'Delete derived state' })).toBeDisabled();
    await fireEvent.click(screen.getByRole('button', { name: 'Reconcile changes' }));
    expect(onPreviewMaintenance).toHaveBeenCalledWith('reconcile');
    await fireEvent.click(screen.getByRole('button', { name: 'Full rebuild' }));
    expect(onPreviewMaintenance).toHaveBeenCalledWith('full-rebuild');
  });

  it('sends advanced filters only after explicit values are entered', async () => {
    const onApply = vi.fn();
    render(PortfolioFilters, {
      projects: [{
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
      }],
      disabled: false,
      onApply,
      onReset: vi.fn()
    });

    await fireEvent.change(screen.getByLabelText('Project'), {
      target: { value: projectId }
    });
    await fireEvent.click(screen.getByText('Advanced native filters'));
    await fireEvent.change(screen.getByLabelText('Shared identity type'), {
      target: { value: 'paper' }
    });
    await fireEvent.input(screen.getByLabelText('Shared canonical identity'), {
      target: { value: 'doi:10.1/example' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Apply filters' }));

    expect(onApply).toHaveBeenCalledWith({
      projectId,
      sharedIdentity: { nodeType: 'paper', canonicalId: 'doi:10.1/example' }
    });
  });

  it('announces native progress and only offers cancellation while cancellable', async () => {
    const onCancel = vi.fn();
    const progress = {
      schemaVersion: 1,
      operationId: `cop_${'4'.repeat(64)}`,
      operation: 'reconcile',
      phase: 'running',
      completedUnits: 1,
      totalUnits: 2,
      catalogId,
      cancellable: true,
      reasonCode: 'portfolio-operation-running'
    } satisfies ContinuityOperationProgress;
    const view = render(PortfolioMaintenancePanel, {
      doctor: null,
      doctorState: 'idle',
      progress,
      result: null,
      busy: false,
      onCancel
    });

    expect(screen.getByRole('progressbar', { name: 'Portfolio maintenance progress' }))
      .toHaveAttribute('aria-valuenow', '1');
    expect(screen.getByRole('status')).toHaveTextContent(
      'Reconcile: 50% complete. The approved native maintenance operation is running.'
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel maintenance' }));
    expect(onCancel).toHaveBeenCalledWith(progress.operationId);

    await view.rerender({
      doctor: null,
      doctorState: 'idle',
      progress: { ...progress, phase: 'cancelled', cancellable: false },
      result: null,
      busy: false,
      onCancel
    });
    expect(screen.queryByRole('button', { name: 'Cancel maintenance' })).not.toBeInTheDocument();
    expect(screen.getByRole('status')).not.toHaveTextContent('% complete');
  });

  it('preserves native result order and exposes its content-bound next page', async () => {
    const onLoadMore = vi.fn();
    const result = {
      schemaVersion: 1,
      requestId: `pqr_${'5'.repeat(64)}`,
      queryId: `pqy_${'6'.repeat(64)}`,
      catalogId,
      portfolioId: status.portfolioId,
      lineageDigest: `plg_${'7'.repeat(64)}`,
      matchedProjectCount: 2,
      matchedNodeCount: 0,
      matchedEdgeCount: 0,
      matchedLineageCount: 0,
      projectsTruncated: true,
      nodesTruncated: false,
      edgesTruncated: false,
      lineageTruncated: false,
      projects: [
        {
          resultId: 'project:first',
          projectId,
          displayName: 'First native project',
          stage: 'writing',
          lifecycle: 'active',
          health: 'ready',
          semanticRevision: 12,
          projectionId: `grp_${'8'.repeat(64)}`,
          nodeCount: 0,
          edgeCount: 0,
          lineageCount: 0
        },
        {
          resultId: 'project:second',
          projectId: `prj_${'9'.repeat(32)}`,
          displayName: 'Second native project',
          stage: 'review',
          lifecycle: 'archived',
          health: 'ready',
          semanticRevision: 4,
          projectionId: `grp_${'a'.repeat(64)}`,
          nodeCount: 0,
          edgeCount: 0,
          lineageCount: 0
        }
      ],
      nodes: [],
      edges: [],
      lineage: [],
      nextCursor: {
        cursorId: `pqc_${'b'.repeat(64)}`,
        queryId: `pqy_${'6'.repeat(64)}`,
        projectAfter: 'project:second'
      }
    } satisfies PortfolioQueryResult;

    render(PortfolioResults, {
      workspace: portfolioWorkspaceFromResult(result),
      loadingMore: false,
      onLoadMore
    });

    const projects = screen.getByRole('region', { name: 'Projects' });
    expect(projects.textContent?.indexOf('First native project'))
      .toBeLessThan(projects.textContent?.indexOf('Second native project') ?? -1);
    await fireEvent.click(screen.getByRole('button', { name: 'Load next native page' }));
    expect(onLoadMore).toHaveBeenCalledOnce();
  });

  it('reports retained canonical data after maintenance completion', () => {
    const result = {
      schemaVersion: 1,
      operationId: `cop_${'c'.repeat(64)}`,
      operation: 'delete-derived-state',
      libraryRevision: 7,
      catalogId: null,
      portfolioId: null,
      catalogChanged: true,
      rebuiltProjectCount: 0,
      reusedProjectCount: 0,
      removedProjectCount: 0,
      removedContributionCount: 1,
      derivedStateOnly: true
    } satisfies PortfolioMaintenanceResult;
    render(PortfolioMaintenancePanel, {
      doctor: null,
      doctorState: 'idle',
      progress: null,
      result,
      busy: false,
      onCancel: vi.fn()
    });
    expect(screen.getByRole('article', { name: 'Portfolio maintenance result' }))
      .toHaveTextContent(
        'Projects rebuilt: 0 · Projects reused: 0 · Projects removed: 0 · Contributions removed: 1'
      );
    expect(screen.getByRole('article', { name: 'Portfolio maintenance result' }))
      .toHaveTextContent('Canonical project artifacts were retained.');
    expect(screen.getByRole('status')).toHaveTextContent(
      'Delete derived state completed. Canonical academic artifacts are retained.'
    );
  });
});
