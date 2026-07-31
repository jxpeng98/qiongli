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
  import { PageHeader, StatePanel } from '$lib/components/app';

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

<svelte:head>
  <title>{i18n.t('portfolio.title')} · {i18n.t('app.name')}</title>
</svelte:head>

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
  <StatePanel centered role="status" busy live="polite" atomic description={i18n.t('portfolio.loading')}>
    {#snippet icon()}<Database size={21} />{/snippet}
  </StatePanel>
{:else if !app.snapshot.capabilities.portfolio}
  <StatePanel tone="warning" role="alert" title={i18n.t('portfolio.unavailableTitle')} description={i18n.t('portfolio.unavailableDetail')}>
    {#snippet icon()}<AlertTriangle size={23} />{/snippet}
  </StatePanel>
{:else if statusLoadState === 'failed' || !status}
  <StatePanel tone="danger" role="alert" title={i18n.t('portfolio.statusFailedTitle')} description={i18n.t('portfolio.statusFailedDetail')}>
    {#snippet icon()}<AlertTriangle size={23} />{/snippet}
    {#snippet actions()}
      <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshPortfolio}>
        {i18n.t('portfolio.retryStatus')}
      </button>
    {/snippet}
  </StatePanel>
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
        <StatePanel centered role="status" busy live="polite" atomic description={i18n.t('portfolio.queryLoading')} />
      {:else if queryLoadState === 'failed' || !queryWorkspace}
        <StatePanel tone="danger" role="alert" title={i18n.t('portfolio.queryFailedTitle')} description={i18n.t('portfolio.queryFailedDetail')}>
          {#snippet icon()}<AlertTriangle size={22} />{/snippet}
          {#snippet actions()}
            <button
              class="button-secondary"
              type="button"
              disabled={app.loading}
              onclick={() => loadFirstPage(activeFilters)}
            >
              {i18n.t('portfolio.retryQuery')}
            </button>
          {/snippet}
        </StatePanel>
      {:else}
        <PortfolioResults
          workspace={queryWorkspace}
          {loadingMore}
          onLoadMore={loadMore}
        />
      {/if}
    {:else}
      <StatePanel tone="warning" role="alert" title={i18n.t(`portfolio.recovery.${status.state}.title`)} description={i18n.t(`portfolio.recovery.${status.state}.detail`)}>
        {#snippet icon()}<AlertTriangle size={22} />{/snippet}
      </StatePanel>
    {/if}
  </div>
{/if}

<style>
  .workspace { display: grid; gap: 10px; min-width: 0; }
</style>
