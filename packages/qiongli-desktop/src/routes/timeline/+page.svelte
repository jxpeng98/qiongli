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
  import { DescriptionGrid, PageLayout, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

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

<PageLayout
  eyebrow={i18n.t('timeline.eyebrow')}
  title={i18n.t('timeline.title')}
  description={i18n.t('timeline.description')}
>
  {#snippet actions()}
    <Button
      variant="outline"
      disabled={app.loading || libraryRevision === null}
      onclick={refreshTimeline}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </Button>
  {/snippet}

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
      <Button variant="outline" disabled={app.loading} onclick={refreshTimeline}>
        {i18n.t('timeline.retryStatus')}
      </Button>
    {/snippet}
  </StatePanel>
{:else}
    <Card.Root class="catalog" aria-labelledby="timeline-catalog-title">
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
      <DescriptionGrid columns={4} compact class="catalog-facts">
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
      </DescriptionGrid>
    </Card.Root>

    {#if status.state !== 'current' || !status.capabilities.canQuery}
      <StatePanel tone="warning" role="alert" title={i18n.t(`timeline.recovery.${status.state}.title`)} description={i18n.t(`timeline.recovery.${status.state}.detail`)}>
        {#snippet icon()}<Database size={22} />{/snippet}
        {#snippet actions()}
          <Button href="/portfolio">{i18n.t('timeline.openPortfolio')}</Button>
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
            <Button
              variant="outline"
              disabled={app.loading}
              onclick={() => loadFirstPage(activeSelection)}
            >
              {i18n.t('timeline.retryQuery')}
            </Button>
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
{/if}
</PageLayout>

<style>
  :global(.catalog) { min-width: 0; padding: var(--ui-panel-padding); }
  :global(.catalog-facts) { margin-top: 10px; }
  :global(.catalog) dt { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 700; }
  :global(.catalog) dd {
    min-width: 0;
    margin: 4px 0 0;
    color: var(--color-ink-strong);
    font-size: var(--font-size-label);
    font-weight: 700;
  }
  :global(.catalog) code { overflow-wrap: anywhere; font-size: var(--font-size-label); }
</style>
