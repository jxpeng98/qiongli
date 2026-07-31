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
  import { PageHeader, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';

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
  <StatePanel centered role="status" busy live="polite" atomic description={i18n.t('timeline.loading')}>
    {#snippet icon()}<CalendarClock size={21} />{/snippet}
  </StatePanel>
{:else if !app.snapshot.capabilities.timeline || !app.snapshot.capabilities.portfolio}
  <StatePanel tone="warning" role="alert" title={i18n.t('timeline.unavailableTitle')} description={i18n.t('timeline.unavailableDetail')}>
    {#snippet icon()}<AlertTriangle size={23} />{/snippet}
  </StatePanel>
{:else if statusLoadState === 'failed' || !status}
  <StatePanel tone="danger" role="alert" title={i18n.t('timeline.statusFailedTitle')} description={i18n.t('timeline.statusFailedDetail')}>
    {#snippet icon()}<AlertTriangle size={23} />{/snippet}
    {#snippet actions()}
      <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshTimeline}>
        {i18n.t('timeline.retryStatus')}
      </button>
    {/snippet}
  </StatePanel>
{:else}
  <div class="workspace">
    <section class="surface catalog" aria-labelledby="timeline-catalog-title">
      <SectionHeader
        eyebrow={i18n.t('timeline.catalogEyebrow')}
        title={i18n.t('timeline.catalogTitle')}
        titleId="timeline-catalog-title"
        description={i18n.reason(status.reasonCode)}
      >
        {#snippet metadata()}
          <StatusBadge
            status={status.state === 'current' ? 'ready'
              : status.state === 'stale' ? 'drifted'
                : status.state === 'missing' ? 'missing' : 'recovery-required'}
            label={i18n.label(status.state)}
          />
        {/snippet}
      </SectionHeader>
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
      <StatePanel tone="warning" role="alert" title={i18n.t(`timeline.recovery.${status.state}.title`)} description={i18n.t(`timeline.recovery.${status.state}.detail`)}>
        {#snippet icon()}<Database size={22} />{/snippet}
        {#snippet actions()}
          <a class="button-primary" href="/portfolio">{i18n.t('timeline.openPortfolio')}</a>
        {/snippet}
      </StatePanel>
    {:else}
      <TimelineControls
        projects={app.snapshot.researchLibrary.projects}
        selection={activeSelection}
        disabled={app.loading}
        onApply={loadFirstPage}
      />

      {#if timelineLoadState === 'loading' || timelineLoadState === 'idle'}
        <StatePanel centered role="status" busy live="polite" atomic description={i18n.t('timeline.queryLoading')} />
      {:else if timelineLoadState === 'failed' || !workspace}
        <StatePanel tone="danger" role="alert" title={i18n.t('timeline.queryFailedTitle')} description={i18n.t('timeline.queryFailedDetail')}>
          {#snippet icon()}<AlertTriangle size={22} />{/snippet}
          {#snippet actions()}
            <button
              class="button-secondary"
              type="button"
              disabled={app.loading}
              onclick={() => loadFirstPage(activeSelection)}
            >
              {i18n.t('timeline.retryQuery')}
            </button>
          {/snippet}
        </StatePanel>
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
  .catalog { min-width: 0; padding: 16px; }
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
  @media (max-width: 760px) {
    .catalog dl { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
  @media (max-width: 460px) {
    .catalog dl { grid-template-columns: 1fr; }
  }
</style>
