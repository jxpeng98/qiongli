<script lang="ts">
  import { AlertTriangle, CalendarClock, Database, RefreshCw } from '@lucide/svelte';

  import { useAppState, useProjectWorkspace } from '$lib/context';
  import TimelineControls from '$lib/features/timeline/TimelineControls.svelte';
  import TimelineResults from '$lib/features/timeline/TimelineResults.svelte';
  import {
    loadTimelinePage,
    mergeTimelineWorkspace,
    timelineSelectionKey,
    timelineWorkspaceFromResult,
    type TimelineSelection,
    type TimelineWorkspace
  } from '$lib/features/timeline';
  import { i18n } from '$lib/i18n.svelte';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';

  type LoadState = 'idle' | 'loading' | 'ready' | 'failed';

  const app = useAppState();
  const projectWorkspace = useProjectWorkspace();

  let requestedLibraryRevision = $state<number | null>(null);
  let statusLoadState = $state<LoadState>('idle');
  let timelineLoadState = $state<LoadState>('idle');
  let activeSelection = $state<TimelineSelection>({
    mode: 'portfolio-activity',
    projectId: null
  });
  let activeSelectionKey = $state('');
  let workspace = $state<TimelineWorkspace | null>(null);
  let loadingMore = $state(false);
  let observedProjectId = $state<string | null>(null);

  let libraryRevision = $derived(app.snapshot?.researchLibrary.revision ?? null);
  let status = $derived(
    app.portfolioStatus?.libraryRevision === libraryRevision
      ? app.portfolioStatus : null
  );

  $effect(() => {
    const projectId = projectWorkspace.projectId;
    if (!projectId || projectId === observedProjectId) return;
    observedProjectId = projectId;
    const selection: TimelineSelection = {
      mode: 'project-activity',
      projectId
    };
    activeSelection = selection;
    activeSelectionKey = '';
    workspace = null;
    timelineLoadState = 'idle';
    if (
      status?.state === 'current'
      && status.capabilities.canQuery
      && app.snapshot?.capabilities.timeline
    ) {
      void loadFirstPage(selection, status.catalogId);
    }
  });

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

  async function loadStatus(expectedLibraryRevision: number): Promise<void> {
    statusLoadState = 'loading';
    timelineLoadState = 'idle';
    workspace = null;
    activeSelectionKey = '';
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
    if (
      event.portfolio.state === 'current'
      && event.portfolio.capabilities.canQuery
      && app.snapshot.capabilities.timeline
    ) {
      await loadFirstPage(activeSelection, event.portfolio.catalogId);
    }
  }

  async function loadFirstPage(
    selection: TimelineSelection,
    explicitCatalogId?: string | null
  ): Promise<void> {
    if (
      selection.projectId
      && selection.projectId !== projectWorkspace.projectId
    ) {
      observedProjectId = selection.projectId;
      void projectWorkspace.selectProject(selection.projectId);
    }
    const catalogId = explicitCatalogId ?? status?.catalogId;
    if (!catalogId) return;
    const context = timelineSelectionKey(catalogId, selection);
    activeSelection = selection;
    activeSelectionKey = context;
    timelineLoadState = 'loading';
    workspace = null;
    const result = await loadTimelinePage(
      catalogId,
      selection,
      null,
      (intent) => app.execute(intent)
    );
    const nextWorkspace = result && timelineWorkspaceFromResult(result);
    if (
      !nextWorkspace
      || activeSelectionKey !== context
      || app.portfolioStatus?.catalogId !== catalogId
      || app.portfolioStatus.state !== 'current'
    ) {
      if (activeSelectionKey === context) timelineLoadState = 'failed';
      return;
    }
    workspace = nextWorkspace;
    timelineLoadState = 'ready';
  }

  async function loadMore(): Promise<void> {
    const current = workspace;
    const cursor = current?.nextCursor;
    if (!current || !cursor || loadingMore) return;
    const context = activeSelectionKey;
    loadingMore = true;
    try {
      const next = await loadTimelinePage(
        current.catalogId,
        activeSelection,
        cursor,
        (intent) => app.execute(intent)
      );
      const merged = next && activeSelectionKey === context
        ? mergeTimelineWorkspace(current, next)
        : null;
      if (!merged) {
        workspace = null;
        timelineLoadState = 'failed';
        return;
      }
      workspace = merged;
    } finally {
      loadingMore = false;
    }
  }

  function refreshTimeline(): void {
    if (libraryRevision === null) return;
    requestedLibraryRevision = libraryRevision;
    void loadStatus(libraryRevision);
  }
</script>

<svelte:head>
  <title>{i18n.t('timeline.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('timeline.eyebrow')}
  title={i18n.t('timeline.title')}
  description={i18n.t('timeline.description')}
>
  {#snippet actions()}
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || libraryRevision === null}
      onclick={refreshTimeline}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot || statusLoadState === 'loading' || statusLoadState === 'idle'}
  <section
    class="surface loading"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >
    <CalendarClock size={21} aria-hidden="true" />
    <p>{i18n.t('timeline.loading')}</p>
  </section>
{:else if !app.snapshot.capabilities.timeline || !app.snapshot.capabilities.portfolio}
  <section class="surface state-message" role="alert">
    <AlertTriangle size={23} aria-hidden="true" />
    <div>
      <h2>{i18n.t('timeline.unavailableTitle')}</h2>
      <p>{i18n.t('timeline.unavailableDetail')}</p>
    </div>
  </section>
{:else if statusLoadState === 'failed' || !status}
  <section class="surface state-message" role="alert">
    <AlertTriangle size={23} aria-hidden="true" />
    <div>
      <h2>{i18n.t('timeline.statusFailedTitle')}</h2>
      <p>{i18n.t('timeline.statusFailedDetail')}</p>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshTimeline}>
        {i18n.t('timeline.retryStatus')}
      </button>
    </div>
  </section>
{:else}
  <div class="workspace">
    <section class="surface catalog" aria-labelledby="timeline-catalog-title">
      <div class="catalog-heading">
        <div>
          <p class="eyebrow">{i18n.t('timeline.catalogEyebrow')}</p>
          <h2 id="timeline-catalog-title">{i18n.t('timeline.catalogTitle')}</h2>
          <p>{i18n.reason(status.reasonCode)}</p>
        </div>
        <StatusBadge
          status={status.state === 'current' ? 'ready'
            : status.state === 'stale' ? 'drifted'
              : status.state === 'missing' ? 'missing' : 'recovery-required'}
          label={i18n.label(status.state)}
        />
      </div>
      <dl>
        <div><dt>{i18n.t('timeline.libraryRevision')}</dt><dd>r{status.libraryRevision}</dd></div>
        <div>
          <dt>{i18n.t('timeline.catalogGeneration')}</dt>
          <dd>{status.catalogGeneration ?? i18n.t('common.none')}</dd>
        </div>
        <div>
          <dt>{i18n.t('timeline.catalogIdentity')}</dt>
          <dd><code>{status.catalogId ?? i18n.t('timeline.noCatalog')}</code></dd>
        </div>
        <div><dt>{i18n.t('timeline.catalogProjects')}</dt><dd>{status.projectCount}</dd></div>
      </dl>
    </section>

    {#if status.state !== 'current' || !status.capabilities.canQuery}
      <section class="surface recovery-message" role="alert">
        <Database size={22} aria-hidden="true" />
        <div>
          <h2>{i18n.t(`timeline.recovery.${status.state}.title`)}</h2>
          <p>{i18n.t(`timeline.recovery.${status.state}.detail`)}</p>
          <a class="button-primary" href="/portfolio">{i18n.t('timeline.openPortfolio')}</a>
        </div>
      </section>
    {:else}
      <TimelineControls
        projects={app.snapshot.researchLibrary.projects}
        selection={activeSelection}
        disabled={app.loading}
        onApply={loadFirstPage}
      />

      {#if timelineLoadState === 'loading' || timelineLoadState === 'idle'}
        <section
          class="surface loading"
          role="status"
          aria-busy="true"
          aria-live="polite"
          aria-atomic="true"
        >
          <p>{i18n.t('timeline.queryLoading')}</p>
        </section>
      {:else if timelineLoadState === 'failed' || !workspace}
        <section class="surface state-message" role="alert">
          <AlertTriangle size={22} aria-hidden="true" />
          <div>
            <h2>{i18n.t('timeline.queryFailedTitle')}</h2>
            <p>{i18n.t('timeline.queryFailedDetail')}</p>
            <button
              class="button-secondary"
              type="button"
              disabled={app.loading}
              onclick={() => loadFirstPage(activeSelection)}
            >
              {i18n.t('timeline.retryQuery')}
            </button>
          </div>
        </section>
      {:else}
        <TimelineResults
          {workspace}
          selection={activeSelection}
          projects={app.snapshot.researchLibrary.projects}
          {loadingMore}
          onLoadMore={loadMore}
        />
      {/if}
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
  .catalog { min-width: 0; padding: 16px; }
  .catalog-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }
  .catalog h2, .state-message h2, .recovery-message h2 {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: 16px;
  }
  .catalog-heading p:not(.eyebrow), .state-message p, .recovery-message p {
    margin: 5px 0 0;
    color: var(--color-muted);
    font-size: 11px;
    line-height: 1.5;
  }
  .catalog dl {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
    margin: 13px 0 0;
  }
  .catalog dl > div {
    min-width: 0;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    padding: 8px 10px;
    background: var(--color-surface-subtle);
  }
  .catalog dt { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 700; }
  .catalog dd {
    min-width: 0;
    margin: 4px 0 0;
    color: var(--color-ink-strong);
    font-size: 11px;
    font-weight: 700;
  }
  .catalog code { overflow-wrap: anywhere; font-size: var(--font-size-label); }
  .state-message, .recovery-message {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 18px;
    color: var(--color-warning);
  }
  .state-message button, .recovery-message a { margin-top: 11px; }
  .recovery-message { background: var(--color-warning-soft); }
  @media (max-width: 760px) {
    .catalog dl { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
  @media (max-width: 460px) {
    .catalog-heading { flex-direction: column; }
    .catalog dl { grid-template-columns: 1fr; }
  }
</style>
