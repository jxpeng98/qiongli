<script lang="ts">
  import type {
    PortfolioMaintenancePreview,
    PortfolioQueryFilters
  } from '@qiongli/app-api';
  import { AlertTriangle, Database, RefreshCw } from '@lucide/svelte';

  import { useAppState } from '$lib/context';
  import {
    loadPortfolioQueryPage,
    mergePortfolioQueryWorkspace,
    portfolioFilterKey,
    portfolioWorkspaceFromResult,
    type PortfolioQueryWorkspace
  } from '$lib/features/portfolio';
  import PortfolioFilters from '$lib/features/portfolio/PortfolioFilters.svelte';
  import PortfolioMaintenancePanel from '$lib/features/portfolio/PortfolioMaintenancePanel.svelte';
  import PortfolioResults from '$lib/features/portfolio/PortfolioResults.svelte';
  import PortfolioStatusPanel from '$lib/features/portfolio/PortfolioStatusPanel.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { PageHeader } from '$lib/shared/ui';

  type PortfolioMaintenanceOperation = PortfolioMaintenancePreview['operation'];
  type LoadState = 'idle' | 'loading' | 'ready' | 'failed';

  const app = useAppState();

  let requestedLibraryRevision = $state<number | null>(null);
  let statusLoadState = $state<LoadState>('idle');
  let queryLoadState = $state<LoadState>('idle');
  let doctorLoadState = $state<LoadState>('idle');
  let queryWorkspace = $state<PortfolioQueryWorkspace | null>(null);
  let activeFilters = $state<PortfolioQueryFilters>({});
  let activeFilterKey = $state('');
  let loadingMore = $state(false);

  let libraryRevision = $derived(app.snapshot?.researchLibrary.revision ?? null);
  let status = $derived(
    app.portfolioStatus?.libraryRevision === libraryRevision
      ? app.portfolioStatus : null
  );
  let operationActive = $derived(
    app.continuityOperationProgress?.phase === 'queued'
      || app.continuityOperationProgress?.phase === 'running'
  );

  $effect(() => {
    if (
      libraryRevision !== null
      && requestedLibraryRevision !== libraryRevision
      && !app.loading
    ) {
      requestedLibraryRevision = libraryRevision;
      void loadStatus(libraryRevision);
    }
  });

  $effect(() => {
    const progress = app.continuityOperationProgress;
    if (!progress || (progress.phase !== 'queued' && progress.phase !== 'running')) return;
    const timer = window.setTimeout(() => {
      void pollMaintenance(progress.operationId);
    }, 350);
    return () => window.clearTimeout(timer);
  });

  async function loadStatus(expectedLibraryRevision: number): Promise<void> {
    statusLoadState = 'loading';
    queryLoadState = 'idle';
    queryWorkspace = null;
    activeFilterKey = '';
    const event = await app.execute({ action: 'load-portfolio-status' });
    if (
      event?.type !== 'portfolio-status'
      || event.portfolio.libraryRevision !== expectedLibraryRevision
      || app.snapshot?.researchLibrary.revision !== expectedLibraryRevision
    ) {
      statusLoadState = 'failed';
      return;
    }
    statusLoadState = 'ready';
    if (event.portfolio.state === 'current' && event.portfolio.capabilities.canQuery) {
      await loadFirstPage(activeFilters, event.portfolio.catalogId);
    }
  }

  async function loadFirstPage(
    filters: PortfolioQueryFilters,
    explicitCatalogId?: string | null
  ): Promise<void> {
    const catalogId = explicitCatalogId ?? status?.catalogId;
    if (!catalogId) return;
    const context = portfolioFilterKey(catalogId, filters);
    activeFilters = filters;
    activeFilterKey = context;
    queryLoadState = 'loading';
    queryWorkspace = null;
    const result = await loadPortfolioQueryPage(
      catalogId,
      filters,
      null,
      (intent) => app.execute(intent)
    );
    if (
      !result
      || activeFilterKey !== context
      || status?.catalogId !== catalogId
    ) {
      if (activeFilterKey === context) queryLoadState = 'failed';
      return;
    }
    queryWorkspace = portfolioWorkspaceFromResult(result);
    queryLoadState = 'ready';
  }

  async function loadMore(): Promise<void> {
    const current = queryWorkspace;
    const cursor = current?.nextCursor;
    if (!current || !cursor || loadingMore) return;
    const context = activeFilterKey;
    loadingMore = true;
    try {
      const next = await loadPortfolioQueryPage(
        current.catalogId,
        activeFilters,
        cursor,
        (intent) => app.execute(intent)
      );
      const merged = next && activeFilterKey === context
        ? mergePortfolioQueryWorkspace(current, next)
        : null;
      if (!merged) {
        queryWorkspace = null;
        queryLoadState = 'failed';
        return;
      }
      queryWorkspace = merged;
    } finally {
      loadingMore = false;
    }
  }

  async function loadDoctor(): Promise<void> {
    const currentStatus = status;
    if (!currentStatus) return;
    doctorLoadState = 'loading';
    const event = await app.execute({ action: 'load-portfolio-doctor' });
    if (
      event?.type !== 'portfolio-doctor'
      || event.doctor.libraryRevision !== currentStatus.libraryRevision
      || (currentStatus.state === 'current'
        && event.doctor.catalogId !== currentStatus.catalogId)
    ) {
      doctorLoadState = 'failed';
      return;
    }
    doctorLoadState = 'ready';
  }

  async function previewMaintenance(operation: PortfolioMaintenanceOperation): Promise<void> {
    const currentStatus = status;
    if (!currentStatus || operationActive) return;
    const event = await app.execute({
      action: 'preview-portfolio-maintenance',
      operation
    });
    if (
      event?.type !== 'portfolio-maintenance-preview'
      || event.maintenance.expectedLibraryRevision !== currentStatus.libraryRevision
      || event.maintenance.expectedCatalogId !== currentStatus.catalogId
    ) {
      app.closePreview();
    }
  }

  async function cancelMaintenance(operationId: string): Promise<void> {
    const event = await app.execute({
      action: 'cancel-continuity-operation',
      operationId
    });
    if (
      event?.type === 'continuity-operation-progress'
      && event.progress.phase !== 'queued'
      && event.progress.phase !== 'running'
      && libraryRevision !== null
    ) {
      await loadStatus(libraryRevision);
    }
  }

  async function pollMaintenance(operationId: string): Promise<void> {
    const event = await app.execute({
      action: 'poll-continuity-operation',
      operationId
    });
    if (
      (event?.type === 'portfolio-maintenance-completed'
        || (
          event?.type === 'continuity-operation-progress'
          && event.progress.phase !== 'queued'
          && event.progress.phase !== 'running'
        ))
      && libraryRevision !== null
    ) {
      await loadStatus(libraryRevision);
    }
  }

  function resetFilters(): void {
    void loadFirstPage({});
  }

  function refreshPortfolio(): void {
    if (libraryRevision === null) return;
    requestedLibraryRevision = libraryRevision;
    void loadStatus(libraryRevision);
  }
</script>

<PageHeader
  eyebrow={i18n.t('portfolio.eyebrow')}
  title={i18n.t('portfolio.title')}
  description={i18n.t('portfolio.description')}
>
  {#snippet actions()}
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || libraryRevision === null || operationActive}
      onclick={refreshPortfolio}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot || statusLoadState === 'loading' || statusLoadState === 'idle'}
  <section class="surface loading" aria-busy="true" aria-live="polite">
    <Database size={21} aria-hidden="true" />
    <p>{i18n.t('portfolio.loading')}</p>
  </section>
{:else if !app.snapshot.capabilities.portfolio}
  <section class="surface state-message" role="alert">
    <AlertTriangle size={23} aria-hidden="true" />
    <div><h2>{i18n.t('portfolio.unavailableTitle')}</h2><p>{i18n.t('portfolio.unavailableDetail')}</p></div>
  </section>
{:else if statusLoadState === 'failed' || !status}
  <section class="surface state-message" role="alert">
    <AlertTriangle size={23} aria-hidden="true" />
    <div>
      <h2>{i18n.t('portfolio.statusFailedTitle')}</h2>
      <p>{i18n.t('portfolio.statusFailedDetail')}</p>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshPortfolio}>
        {i18n.t('portfolio.retryStatus')}
      </button>
    </div>
  </section>
{:else}
  <div class="workspace">
    <PortfolioStatusPanel
      {status}
      busy={app.loading || operationActive}
      onDoctor={loadDoctor}
      onPreviewMaintenance={previewMaintenance}
    />

    <PortfolioMaintenancePanel
      doctor={app.portfolioDoctor?.libraryRevision === status.libraryRevision
        ? app.portfolioDoctor : null}
      doctorState={doctorLoadState}
      progress={app.continuityOperationProgress}
      result={app.portfolioMaintenanceResult}
      busy={app.loading}
      onCancel={cancelMaintenance}
    />

    {#if status.state === 'current'}
      <PortfolioFilters
        projects={app.snapshot.researchLibrary.projects}
        disabled={app.loading || operationActive}
        onApply={loadFirstPage}
        onReset={resetFilters}
      />

      {#if queryLoadState === 'loading' || queryLoadState === 'idle'}
        <section class="surface loading" aria-busy="true" aria-live="polite">
          <p>{i18n.t('portfolio.queryLoading')}</p>
        </section>
      {:else if queryLoadState === 'failed' || !queryWorkspace}
        <section class="surface state-message" role="alert">
          <AlertTriangle size={22} aria-hidden="true" />
          <div>
            <h2>{i18n.t('portfolio.queryFailedTitle')}</h2>
            <p>{i18n.t('portfolio.queryFailedDetail')}</p>
            <button
              class="button-secondary"
              type="button"
              disabled={app.loading}
              onclick={() => loadFirstPage(activeFilters)}
            >
              {i18n.t('portfolio.retryQuery')}
            </button>
          </div>
        </section>
      {:else}
        <PortfolioResults
          workspace={queryWorkspace}
          {loadingMore}
          onLoadMore={loadMore}
        />
      {/if}
    {:else}
      <section class="surface recovery-message">
        <AlertTriangle size={22} aria-hidden="true" />
        <div>
          <h2>{i18n.t(`portfolio.recovery.${status.state}.title`)}</h2>
          <p>{i18n.t(`portfolio.recovery.${status.state}.detail`)}</p>
        </div>
      </section>
    {/if}
  </div>
{/if}

<style>
  .workspace { display: grid; gap: 10px; min-width: 0; }
  .loading {
    display: flex;
    min-height: 120px;
    align-items: center;
    justify-content: center;
    gap: 9px;
    padding: 22px;
    color: var(--color-muted);
    font-size: 13px;
  }
  .state-message, .recovery-message {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 18px;
    color: var(--color-warning);
  }
  .state-message h2, .recovery-message h2 {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: 16px;
  }
  .state-message p, .recovery-message p {
    margin: 5px 0 0;
    color: var(--color-muted);
    font-size: 12px;
    line-height: 1.5;
  }
  .state-message button { margin-top: 11px; }
  .recovery-message { background: var(--color-warning-soft); }
</style>
