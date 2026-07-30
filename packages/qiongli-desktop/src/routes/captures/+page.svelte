<script lang="ts">
  import type {
    AppIntent,
    CaptureAssignmentPage,
    CaptureAssignmentView,
    CaptureDeliveryPage,
    CaptureDeliveryView,
    CaptureInboxEntry,
    CaptureResolutionPage,
    CaptureResolutionSelection,
    CaptureResolutionView
  } from '@qiongli/app-api';
  import {
    AlertTriangle,
    ArrowRight,
    CheckCircle2,
    FileInput,
    Files,
    Inbox,
    RefreshCw,
    ScanSearch
  } from '@lucide/svelte';

  import { useAppState } from '$lib/context';
  import CaptureConflicts from '$lib/features/captures/CaptureConflicts.svelte';
  import CaptureOutbox from '$lib/features/captures/CaptureOutbox.svelte';
  import CaptureWorkspaceTabs from '$lib/features/captures/CaptureWorkspaceTabs.svelte';
  import {
    artifactChangeStatus,
    canReviewCapture,
    captureStatus,
    coverageStatus,
    loadCaptureConflictState,
    loadCaptureDeliveryPage,
    loadCapturePresentationState,
    mergeAssignmentPages,
    mergeDeliveryPages,
    mergeResolutionPages,
    type CaptureWorkspaceMode
  } from '$lib/features/captures';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();

  let selectedProjectId = $state<string | null>(null);
  let requestedProjectId = $state<string | null>(null);
  let requestedProjectRevision = $state<number | null>(null);
  let manualRefreshInProgress = $state(false);
  let selectedCaptureId = $state<string | null>(null);
  let workspaceMode = $state<CaptureWorkspaceMode>('inbox');
  let deliveryPage = $state<CaptureDeliveryPage | null>(null);
  let assignmentPage = $state<CaptureAssignmentPage | null>(null);
  let resolutionPage = $state<CaptureResolutionPage | null>(null);
  let selectedEnvelopeId = $state<string | null>(null);
  let continuityContext = $state('');
  let deliveryRequestContext = $state<string | null>(null);
  let conflictRequestContext = $state<string | null>(null);
  let continuityLoading = $state(false);
  let deliveryLoadStatus = $state<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  let conflictLoadStatus = $state<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  let captureLoad = $state<{
    projectId: string;
    projectRevision: number;
    status: 'loading' | 'ready' | 'failed';
  } | null>(null);

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let inbox = $derived(
    app.captureInbox?.projectId === selectedProjectId
      && app.captureInbox.projectRevision === selectedProject?.semanticRevision
      ? app.captureInbox : null
  );
  let coverage = $derived(
    app.captureCoverage?.projectId === selectedProjectId
      && app.captureCoverage.projectRevision === selectedProject?.semanticRevision
      ? app.captureCoverage : null
  );
  let changes = $derived(
    app.artifactChanges?.projectId === selectedProjectId
      && app.artifactChanges.projectRevision === selectedProject?.semanticRevision
      ? app.artifactChanges : null
  );
  let captureLoadStatus = $derived(
    captureLoad?.projectId === selectedProjectId
      && captureLoad.projectRevision === selectedProject?.semanticRevision
      ? captureLoad.status : 'idle'
  );
  let workspaceCounts = $derived<Record<CaptureWorkspaceMode, number>>({
    inbox: inbox?.entries.length ?? 0,
    outbox: deliveryPage?.entries.length ?? 0,
    conflicts: (deliveryPage?.entries.filter((delivery) =>
      delivery.destination === null
      && delivery.state !== 'acknowledged'
      && delivery.state !== 'cancelled'
    ).length ?? 0) + (assignmentPage?.entries.filter((assignment) =>
      assignment.state === 'pending' || assignment.canResolve
    ).length ?? 0),
    coverage: coverage?.sources.length ?? 0
  });

  $effect(() => {
    if (projects.length === 0) {
      selectedProjectId = null;
      requestedProjectId = null;
      requestedProjectRevision = null;
      captureLoad = null;
      return;
    }
    if (!selectedProjectId || !projects.some((project) => project.projectId === selectedProjectId)) {
      selectedProjectId = projects[0].projectId;
      requestedProjectId = null;
      requestedProjectRevision = null;
      captureLoad = null;
    }
    if (selectedProject?.health === 'inspection-blocked') {
      requestedProjectId = selectedProjectId;
      requestedProjectRevision = selectedProject.semanticRevision;
      captureLoad = null;
      return;
    }
    if (
      selectedProject
      && (requestedProjectId !== selectedProject.projectId
        || requestedProjectRevision !== selectedProject.semanticRevision)
      && !app.loading
      && !manualRefreshInProgress
    ) {
      requestedProjectId = selectedProjectId;
      requestedProjectRevision = selectedProject.semanticRevision;
      void loadCaptureState(selectedProject.projectId, selectedProject.semanticRevision);
    }
  });

  $effect(() => {
    const context = selectedProject
      ? `${selectedProject.projectId}:${selectedProject.semanticRevision}`
      : '';
    if (continuityContext !== context) {
      continuityContext = context;
      deliveryPage = null;
      assignmentPage = null;
      resolutionPage = null;
      selectedEnvelopeId = null;
      deliveryRequestContext = null;
      conflictRequestContext = null;
      deliveryLoadStatus = 'idle';
      conflictLoadStatus = 'idle';
    }
    if (!selectedProject || captureLoadStatus !== 'ready') return;
    if (
      (workspaceMode === 'outbox' || workspaceMode === 'conflicts')
      && deliveryRequestContext !== context
      && !continuityLoading
    ) {
      deliveryRequestContext = context;
      void loadDeliveries(selectedProject.projectId);
    }
    if (
      workspaceMode === 'conflicts'
      && conflictRequestContext !== context
      && !continuityLoading
    ) {
      conflictRequestContext = context;
      void loadConflicts(selectedProject.projectId);
    }
  });

  async function loadCaptureState(projectId: string, projectRevision: number): Promise<void> {
    captureLoad = { projectId, projectRevision, status: 'loading' };
    const complete = await loadCapturePresentationState(
      projectId,
      projectRevision,
      (intent) => app.execute(intent)
    );
    if (
      selectedProjectId === projectId
      && selectedProject?.semanticRevision === projectRevision
    ) {
      captureLoad = { projectId, projectRevision, status: complete ? 'ready' : 'failed' };
    }
  }

  function chooseProject(event: Event): void {
    selectedProjectId = (event.currentTarget as HTMLSelectElement).value || null;
    selectedCaptureId = null;
    requestedProjectId = null;
    requestedProjectRevision = null;
    captureLoad = null;
    workspaceMode = 'inbox';
  }

  function chooseWorkspaceMode(mode: CaptureWorkspaceMode): void {
    workspaceMode = mode;
  }

  async function refreshInbox(): Promise<void> {
    if (!selectedProject || selectedProject.health === 'inspection-blocked') return;
    const projectId = selectedProject.projectId;
    manualRefreshInProgress = true;
    try {
      const refreshed = await app.execute({ action: 'refresh-research-library' });
      if (refreshed?.type !== 'snapshot') return;
      const current = refreshed.snapshot.researchLibrary.projects
        .find((project) => project.projectId === projectId);
      if (!current || current.health === 'inspection-blocked') return;
      requestedProjectId = current.projectId;
      requestedProjectRevision = current.semanticRevision;
      await loadCaptureState(current.projectId, current.semanticRevision);
      await refreshContinuityForMode();
    } finally {
      manualRefreshInProgress = false;
    }
  }

  async function loadDeliveries(projectId: string): Promise<void> {
    continuityLoading = true;
    deliveryLoadStatus = 'loading';
    try {
      const page = await loadCaptureDeliveryPage(projectId, null, (intent) => app.execute(intent));
      if (selectedProjectId === projectId) {
        deliveryPage = page;
        deliveryLoadStatus = page ? 'ready' : 'failed';
      }
    } finally {
      continuityLoading = false;
    }
  }

  async function loadConflicts(projectId: string): Promise<void> {
    continuityLoading = true;
    conflictLoadStatus = 'loading';
    try {
      const state = await loadCaptureConflictState(projectId, (intent) => app.execute(intent));
      if (selectedProjectId !== projectId) return;
      assignmentPage = state?.assignments ?? null;
      resolutionPage = state?.resolutions ?? null;
      conflictLoadStatus = state ? 'ready' : 'failed';
    } finally {
      continuityLoading = false;
    }
  }

  async function refreshContinuityForMode(): Promise<void> {
    if (!selectedProject) return;
    const projectId = selectedProject.projectId;
    if (workspaceMode === 'outbox' || workspaceMode === 'conflicts') {
      await loadDeliveries(projectId);
    }
    if (workspaceMode === 'conflicts') await loadConflicts(projectId);
  }

  async function retryContinuityLoad(): Promise<void> {
    deliveryRequestContext = continuityContext;
    conflictRequestContext = continuityContext;
    await refreshContinuityForMode();
  }

  async function loadMoreDeliveries(): Promise<void> {
    if (!selectedProject || !deliveryPage?.nextCursor) return;
    continuityLoading = true;
    try {
      const next = await loadCaptureDeliveryPage(
        selectedProject.projectId,
        deliveryPage.nextCursor,
        (intent) => app.execute(intent)
      );
      if (next) deliveryPage = mergeDeliveryPages(deliveryPage, next);
    } finally {
      continuityLoading = false;
    }
  }

  async function loadMoreAssignments(): Promise<void> {
    if (!selectedProject || !assignmentPage?.nextCursor) return;
    continuityLoading = true;
    try {
      const event = await app.execute({
        action: 'load-capture-assignments',
        request: {
          projectId: selectedProject.projectId,
          limit: 128,
          cursor: assignmentPage.nextCursor
        }
      });
      if (event?.type === 'capture-assignments') {
        assignmentPage = mergeAssignmentPages(assignmentPage, event.page);
      }
    } finally {
      continuityLoading = false;
    }
  }

  async function loadMoreResolutions(): Promise<void> {
    if (!selectedProject || !resolutionPage?.nextCursor) return;
    continuityLoading = true;
    try {
      const event = await app.execute({
        action: 'load-capture-resolutions',
        request: {
          projectId: selectedProject.projectId,
          limit: 64,
          cursor: resolutionPage.nextCursor
        }
      });
      if (event?.type === 'capture-resolutions') {
        resolutionPage = mergeResolutionPages(resolutionPage, event.page);
      }
    } finally {
      continuityLoading = false;
    }
  }

  async function inspectDelivery(delivery: CaptureDeliveryView): Promise<void> {
    selectedEnvelopeId = delivery.envelopeId;
    await app.execute({
      action: 'inspect-capture-delivery',
      envelopeId: delivery.envelopeId
    });
  }

  async function retryDelivery(
    delivery: CaptureDeliveryView,
    cause: Extract<AppIntent, { action: 'retry-capture-delivery' }>['cause']
  ): Promise<void> {
    await app.execute({
      action: 'retry-capture-delivery',
      envelopeId: delivery.envelopeId,
      expectedGeneration: delivery.generation,
      expectedRecordSha256: delivery.recordSha256,
      retriedAtUnix: Math.floor(Date.now() / 1_000),
      cause
    });
    await refreshContinuityForMode();
  }

  async function cancelDelivery(delivery: CaptureDeliveryView): Promise<void> {
    await app.execute({
      action: 'cancel-capture-delivery',
      envelopeId: delivery.envelopeId,
      expectedGeneration: delivery.generation,
      expectedRecordSha256: delivery.recordSha256,
      cancelledAtUnix: Math.floor(Date.now() / 1_000)
    });
    await refreshContinuityForMode();
  }

  async function previewAcknowledgement(
    delivery: CaptureDeliveryView,
    resultingProjectRevision: number
  ): Promise<void> {
    if (!delivery.destination) return;
    await app.execute({
      action: 'preview-capture-delivery-acknowledgement',
      envelopeId: delivery.envelopeId,
      destinationProjectId: delivery.destination.projectId,
      acceptedCaptureId: delivery.captureId,
      expectedProjectRevision: delivery.destination.expectedProjectRevision,
      resultingProjectRevision,
      acknowledgedAtUnix: Math.floor(Date.now() / 1_000),
      expectedGeneration: delivery.generation,
      expectedRecordSha256: delivery.recordSha256
    });
  }

  async function previewAssignment(
    delivery: CaptureDeliveryView,
    targetProjectId: string,
    decision: 'assign' | 'reject'
  ): Promise<void> {
    if (!targetProjectId) return;
    await app.execute({
      action: 'preview-capture-assignment',
      sourceEnvelopeId: delivery.envelopeId,
      targetProjectId,
      decision,
      decidedAtUnix: Math.floor(Date.now() / 1_000)
    });
  }

  async function inspectAssignment(assignment: CaptureAssignmentView): Promise<void> {
    await app.execute({
      action: 'inspect-capture-assignment',
      intentId: assignment.intentId
    });
  }

  async function loadResolutionPlan(assignment: CaptureAssignmentView): Promise<void> {
    if (!assignment.receiptId) return;
    await app.execute({
      action: 'preview-capture-resolution',
      assignmentReceiptId: assignment.receiptId,
      reviewedAtUnix: Math.floor(Date.now() / 1_000)
    });
  }

  async function previewResolution(
    assignment: CaptureAssignmentView,
    selections: CaptureResolutionSelection[]
  ): Promise<void> {
    if (!assignment.receiptId || selections.length === 0) return;
    await app.execute({
      action: 'preview-capture-resolution',
      assignmentReceiptId: assignment.receiptId,
      reviewedAtUnix: app.captureResolutionPlan?.reviewedAtUnix
        ?? Math.floor(Date.now() / 1_000),
      selections
    });
  }

  async function inspectResolution(resolution: CaptureResolutionView): Promise<void> {
    await app.execute({
      action: 'inspect-capture-resolution',
      projectId: resolution.targetProjectId,
      receiptId: resolution.receiptId
    });
  }

  async function importCapture(): Promise<void> {
    if (!selectedProject || selectedProject.health === 'inspection-blocked') return;
    const selection = await app.execute({
      action: 'select-capture-file',
      projectId: selectedProject.projectId
    });
    if (selection?.type !== 'capture-file-selected') return;
    await app.execute({
      action: 'preview-capture-intake',
      fileToken: selection.token
    });
  }

  async function inspectCapture(entry: CaptureInboxEntry): Promise<void> {
    if (!selectedProjectId) return;
    selectedCaptureId = entry.captureId;
    await app.execute({
      action: 'read-capture',
      projectId: selectedProjectId,
      captureId: entry.captureId
    });
  }

  async function reviewCapture(entry: CaptureInboxEntry): Promise<void> {
    if (!selectedProjectId) return;
    selectedCaptureId = entry.captureId;
    await app.execute({
      action: 'preview-capture-consolidation',
      projectId: selectedProjectId,
      captureId: entry.captureId
    });
  }

  function sentence(value: string): string {
    return i18n.label(value);
  }

  function captureDate(entry: CaptureInboxEntry): string {
    return i18n.date(entry.capturedAtUnix, true);
  }
</script>

<svelte:head>
  <title>{i18n.t('captures.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('captures.eyebrow')}
  title={i18n.t('captures.title')}
  description={i18n.t('captures.description')}
>
  {#snippet actions()}
    <label class="project-picker">
      <span>{i18n.t('captures.project')}</span>
      <select value={selectedProjectId ?? ''} onchange={chooseProject} disabled={app.loading || projects.length === 0}>
        {#each projects as project}
          <option value={project.projectId}>{project.displayName}</option>
        {/each}
      </select>
    </label>
    <button
      class="button-primary"
      type="button"
      disabled={app.loading || !selectedProject || selectedProject.health === 'inspection-blocked' || !app.snapshot?.capabilities.captureMutation}
      onclick={importCapture}
    >
      <FileInput size={16} aria-hidden="true" />{i18n.t('captures.import')}
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !selectedProject || selectedProject.health === 'inspection-blocked' || !app.snapshot?.capabilities.captureInbox}
      onclick={refreshInbox}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />{i18n.t('common.refresh')}
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section
    class="surface loading"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >
    <p>{i18n.t('captures.loadingLibrary')}</p>
  </section>
{:else if projects.length === 0}
  <section class="surface empty-state">
    <Inbox size={30} aria-hidden="true" />
    <h2>{i18n.t('captures.emptyTitle')}</h2>
    <p>{i18n.t('captures.emptyDetail')}</p>
    <a class="button-primary" href="/research-library">{i18n.t('captures.openLibrary')}</a>
  </section>
{:else if selectedProject?.health === 'inspection-blocked'}
  <section class="surface blocked-state">
    <AlertTriangle size={24} aria-hidden="true" />
    <div><h2>{i18n.t('captures.blocked')}</h2><p>{i18n.t('captures.blockedDetail')}</p></div>
  </section>
{:else if captureLoadStatus === 'failed'}
  <section class="surface load-failed" role="alert">
    <AlertTriangle size={24} aria-hidden="true" />
    <div>
      <h2>{i18n.t('captures.loadFailedTitle')}</h2>
      <p>{i18n.t('captures.loadFailedDetail')}</p>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshInbox}>
        <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('captures.retryInspection')}
      </button>
    </div>
  </section>
{:else if captureLoadStatus !== 'ready' || !inbox || !coverage || !changes}
  <section
    class="surface loading"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >
    <p>{i18n.t('captures.inspecting', { project: selectedProject?.displayName ?? '' })}</p>
  </section>
{:else}
  <section class="metrics" aria-label={i18n.t('captures.summaryAria')}>
    <article class="surface metric"><span class="metric-icon"><Inbox size={18} aria-hidden="true" /></span><div><strong>{inbox.entries.length}</strong><span>{i18n.t('captures.captures')}</span></div></article>
    <article class="surface metric"><span class="metric-icon attention"><ScanSearch size={18} aria-hidden="true" /></span><div><strong>{inbox.pendingReviewCount}</strong><span>{i18n.t('captures.pending')}</span></div></article>
    <article class="surface metric"><span class:warning={inbox.staleCount + inbox.conflictedCount > 0} class="metric-icon"><AlertTriangle size={18} aria-hidden="true" /></span><div><strong>{inbox.staleCount + inbox.conflictedCount}</strong><span>{i18n.t('captures.resolution')}</span></div></article>
    <article class="surface metric"><span class="metric-icon positive"><CheckCircle2 size={18} aria-hidden="true" /></span><div><strong>{inbox.appliedCount}</strong><span>{i18n.t('captures.consolidated')}</span></div></article>
  </section>

  <CaptureWorkspaceTabs
    mode={workspaceMode}
    counts={workspaceCounts}
    onChange={chooseWorkspaceMode}
  />

  {#if workspaceMode === 'coverage'}
    <div
      id="capture-panel-coverage"
      role="tabpanel"
      aria-labelledby="capture-tab-coverage"
    >
      <section class="surface coverage-panel" aria-labelledby="coverage-title">
    <div class="panel-heading">
      <div>
        <p class="eyebrow">{i18n.t('captures.coverageEyebrow')}</p>
        <h2 id="coverage-title">{i18n.t('captures.coverageTitle')}</h2>
        <p>{i18n.t('captures.coverageSummary', { captures: coverage.captureCount, unknown: coverage.unknownSourceCount })}</p>
      </div>
      <StatusBadge
        status={coverage.unknownSourceCount === 0 ? 'ready' : 'attention'}
        label={coverage.unknownSourceCount === 0 ? i18n.label('observed') : i18n.label('partial-coverage')}
      />
    </div>
    <p class="coverage-note">{i18n.t('captures.coverageNote')}</p>
    <div class="coverage-grid">
      {#each coverage.sources as source (source.source)}
        <article>
          <div class="coverage-source">
            <strong>{sentence(source.source)}</strong>
            <span>{source.delivery === 'unknown' ? i18n.label('unknown') : sentence(source.delivery)}</span>
          </div>
          <StatusBadge status={coverageStatus(source)} label={sentence(source.state)} />
          <small>{source.captureCount} {i18n.t('captures.captures')}</small>
        </article>
      {/each}
    </div>
      </section>

      <section class="surface change-panel" aria-labelledby="change-title">
    <div class="panel-heading">
      <div>
        <p class="eyebrow">{i18n.t('captures.changesEyebrow')}</p>
        <h2 id="change-title">{i18n.t('captures.changesTitle')}</h2>
        <p>{i18n.t('captures.changesSummary', { revision: changes.projectRevision, present: changes.presentArtifactCount, registered: changes.registeredArtifactCount })}</p>
      </div>
      <StatusBadge
        status={artifactChangeStatus(changes)}
        label={changes.state === 'current' ? i18n.t('captures.revisionCurrent') : i18n.t('captures.unattributedChange')}
      />
    </div>

    {#if changes.state === 'current'}
      <div class="change-summary current">
        <span class="change-icon"><CheckCircle2 size={20} aria-hidden="true" /></span>
        <div><strong>{i18n.t('captures.noDrift')}</strong><p>{i18n.t('captures.currentRevisionDetail', { revision: changes.projectRevision })}</p></div>
      </div>
    {:else}
      <div class="change-summary attention" role="status">
        <span class="change-icon"><AlertTriangle size={20} aria-hidden="true" /></span>
        <div>
          <strong>{i18n.t('captures.attributionRequired', { count: changes.unattributedCount })}</strong>
          {#if changes.changes[0]?.detection === 'exact'}
            <p>{i18n.t('captures.exactAttributionDetail')}</p>
            <ul>{#each changes.changes[0].relativePaths as path}<li><code>{path}</code></li>{/each}</ul>
          {:else}
            <p>{i18n.t('captures.driftAttributionDetail', { revision: changes.projectRevision })}</p>
          {/if}
        </div>
      </div>
    {/if}

    <div class="artifact-grid" aria-label={i18n.t('captures.artifactInventoryAria')}>
      {#each changes.artifacts as artifact (artifact.relativePath)}
        <article class:present={artifact.present}>
          <Files size={15} aria-hidden="true" />
          <span><strong>{sentence(artifact.artifact)}</strong><small>{artifact.relativePath}</small></span>
          <small>{artifact.present ? i18n.label('present') : i18n.label('not-present')}</small>
        </article>
      {/each}
    </div>
      </section>
    </div>

  {:else if workspaceMode === 'inbox'}
    <div
      id="capture-panel-inbox"
      role="tabpanel"
      aria-labelledby="capture-tab-inbox"
    >
      <section class="surface inbox-panel">
    <div class="panel-heading">
      <div>
        <p class="eyebrow">{i18n.t('captures.queueEyebrow')}</p>
        <h2>{selectedProject?.displayName}</h2>
        <p>{i18n.t('captures.queueSummary', { revision: inbox.projectRevision, stage: sentence(inbox.projectStage) })}</p>
      </div>
      <StatusBadge status={inbox.entries.length === 0 ? 'ready' : 'attention'} label={inbox.entries.length === 0 ? i18n.t('captures.clear') : i18n.t('captures.reviewAvailable')} />
    </div>

    {#if inbox.entries.length === 0}
      <div class="empty-inbox">
        <CheckCircle2 size={26} aria-hidden="true" />
        <h3>{i18n.t('captures.emptyInboxTitle')}</h3>
        <p>{i18n.t('captures.emptyInboxDetail')}</p>
      </div>
    {:else}
      <div class="capture-list">
        {#each inbox.entries as entry (entry.captureId)}
          <article class:selected={selectedCaptureId === entry.captureId}>
            <button class="capture-main" type="button" onclick={() => inspectCapture(entry)}>
              <span class="capture-title"><strong>{entry.summary}</strong><small>{entry.task}</small></span>
              <span class="capture-meta"><span>{sentence(entry.source)}</span><span>{sentence(entry.disposition)}</span><span>r{entry.baseRevision}</span></span>
              <span class="capture-date">{captureDate(entry)}</span>
              <StatusBadge status={captureStatus(entry)} label={sentence(entry.state)} />
              <ArrowRight size={17} aria-hidden="true" />
            </button>
            <button
              class="button-secondary review-button"
              type="button"
              disabled={app.loading || !canReviewCapture(entry)}
              onclick={() => reviewCapture(entry)}
            >
              {entry.state === 'applied' ? i18n.t('captures.consolidated') : i18n.label('review-plan')}
            </button>
          </article>
        {/each}
      </div>
    {/if}
      </section>

      {#if app.capture && app.capture.captureId === selectedCaptureId}
        <section class="surface detail-panel" aria-live="polite">
      <div class="panel-heading">
        <div><p class="eyebrow">{i18n.t('captures.detailEyebrow')}</p><h2>{i18n.t('captures.detailTitle')}</h2><p><code>{app.capture.captureId}</code></p></div>
        <button class="button-quiet" type="button" onclick={() => selectedCaptureId = null}>{i18n.t('common.close')}</button>
      </div>
      <p class="capture-summary">{app.capture.summary}</p>
      <div class="detail-grid">
        <section><h3>{i18n.label('academic-changes')}</h3>{#if app.capture.changes.length}<ul>{#each app.capture.changes as change}<li><strong>{sentence(change.area)}</strong><span>{change.summary}</span></li>{/each}</ul>{:else}<p>{i18n.t('common.none')}</p>{/if}</section>
        <section><h3>{i18n.label('decision-candidates')}</h3>{#if app.capture.decisions.length}<ul>{#each app.capture.decisions as decision}<li><strong>{sentence(decision.relation)}</strong><span>{decision.statement}</span><small>{decision.rationale}</small></li>{/each}</ul>{:else}<p>{i18n.t('common.none')}</p>{/if}</section>
        <section><h3>{i18n.label('evidence-references')}</h3>{#if app.capture.evidence.length}<ul>{#each app.capture.evidence as evidence}<li><code>{evidence.locator}</code><span>{evidence.relevance}</span>{#if evidence.limitation}<small>{evidence.limitation}</small>{/if}</li>{/each}</ul>{:else}<p>{i18n.t('common.none')}</p>{/if}</section>
        <section><h3>{i18n.t('captures.contradictions')}</h3>{#if app.capture.contradictions.length}<ul class="danger-list">{#each app.capture.contradictions as contradiction}<li><strong>{contradiction.statement}</strong><span>{contradiction.consequence}</span></li>{/each}</ul>{/if}{#if app.capture.nextActions.length}<ol>{#each app.capture.nextActions as action}<li>{action}</li>{/each}</ol>{:else if app.capture.contradictions.length === 0}<p>{i18n.t('common.none')}</p>{/if}</section>
      </div>
        </section>
      {/if}
    </div>
  {:else if workspaceMode === 'outbox'}
    {#if deliveryLoadStatus === 'failed'}
      <section class="surface load-failed" role="alert">
        <AlertTriangle size={24} aria-hidden="true" />
        <div>
          <h2>{i18n.t('captures.continuityLoadFailedTitle')}</h2>
          <p>{i18n.t('captures.continuityLoadFailedDetail')}</p>
          <button class="button-secondary" type="button" disabled={continuityLoading} onclick={retryContinuityLoad}>
            <RefreshCw size={16} class={continuityLoading ? 'spin' : undefined} aria-hidden="true" />
            {i18n.t('captures.retryContinuityLoad')}
          </button>
        </div>
      </section>
    {:else if deliveryLoadStatus !== 'ready'}
      <section
        class="surface loading"
        role="status"
        aria-busy="true"
        aria-live="polite"
        aria-atomic="true"
      >
        <p>{i18n.t('captures.loadingOutbox')}</p>
      </section>
    {:else}
      <CaptureOutbox
        entries={deliveryPage?.entries ?? []}
        currentProjectRevision={selectedProject?.semanticRevision ?? 1}
        {selectedEnvelopeId}
        loading={app.loading || continuityLoading}
        truncated={deliveryPage?.truncated ?? false}
        onInspect={inspectDelivery}
        onRetry={retryDelivery}
        onCancel={cancelDelivery}
        onAcknowledge={previewAcknowledgement}
        onLoadMore={loadMoreDeliveries}
      />
    {/if}
  {:else}
    {#if deliveryLoadStatus === 'failed' || conflictLoadStatus === 'failed'}
      <section class="surface load-failed" role="alert">
        <AlertTriangle size={24} aria-hidden="true" />
        <div>
          <h2>{i18n.t('captures.continuityLoadFailedTitle')}</h2>
          <p>{i18n.t('captures.continuityLoadFailedDetail')}</p>
          <button class="button-secondary" type="button" disabled={continuityLoading} onclick={retryContinuityLoad}>
            <RefreshCw size={16} class={continuityLoading ? 'spin' : undefined} aria-hidden="true" />
            {i18n.t('captures.retryContinuityLoad')}
          </button>
        </div>
      </section>
    {:else if deliveryLoadStatus !== 'ready' || conflictLoadStatus !== 'ready'}
      <section
        class="surface loading"
        role="status"
        aria-busy="true"
        aria-live="polite"
        aria-atomic="true"
      >
        <p>{i18n.t('captures.loadingConflicts')}</p>
      </section>
    {:else}
      <CaptureConflicts
        deliveries={deliveryPage?.entries ?? []}
        assignments={assignmentPage?.entries ?? []}
        resolutions={resolutionPage?.entries ?? []}
        projects={projects.map((project) => ({
          projectId: project.projectId,
          displayName: project.displayName
        }))}
        plan={app.captureResolutionPlan}
        loading={app.loading || continuityLoading}
        assignmentsTruncated={assignmentPage?.truncated ?? false}
        resolutionsTruncated={resolutionPage?.truncated ?? false}
        onPreviewAssignment={previewAssignment}
        onInspectAssignment={inspectAssignment}
        onLoadResolutionPlan={loadResolutionPlan}
        onPreviewResolution={previewResolution}
        onInspectResolution={inspectResolution}
        onLoadMoreAssignments={loadMoreAssignments}
        onLoadMoreResolutions={loadMoreResolutions}
      />
    {/if}
  {/if}
{/if}

<style>
  .project-picker { display: grid; gap: 4px; min-width: min(280px, 100%); }
  .project-picker span { color: var(--color-muted); font-size: 10px; font-weight: 800; letter-spacing: .05em; text-transform: uppercase; }
  select { min-height: 44px; border: 1px solid var(--color-border-strong); border-radius: 10px; padding: 7px 10px; color: var(--color-ink); background: white; font: inherit; font-size: 12px; }
  .loading, .empty-state { min-height: 160px; padding: 24px; }
  .loading { color: var(--color-muted); }
  .empty-state { display: grid; place-items: center; align-content: center; text-align: center; }
  .empty-state h2 { margin: 12px 0 0; color: var(--color-ink-strong); }
  .empty-state p { max-width: 620px; margin: 8px 0 18px; color: var(--color-muted); line-height: 1.6; }
  .blocked-state { display: flex; gap: 13px; padding: 22px; border-color: #fecaca; color: var(--color-danger); background: var(--color-danger-soft); }
  .load-failed { display: flex; gap: 13px; padding: 22px; border-color: #fed7aa; color: var(--color-warning); background: var(--color-warning-soft); }
  .blocked-state h2, .load-failed h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .blocked-state p, .load-failed p { margin: 6px 0 0; color: var(--color-muted); line-height: 1.55; }
  .load-failed button { display: inline-flex; align-items: center; gap: 7px; margin-top: 14px; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; margin-bottom: 10px; }
  .metric { display: flex; min-height: 62px; align-items: center; gap: 9px; padding: 10px; }
  .metric-icon { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .metric-icon.attention, .metric-icon.warning { color: var(--color-warning); background: var(--color-warning-soft); }
  .metric-icon.positive { color: var(--color-success); background: var(--color-success-soft); }
  .metric strong, .metric span { display: block; }
  .metric strong { color: var(--color-ink-strong); font-size: 21px; line-height: 1; }
  .metric div span { margin-top: 5px; color: var(--color-muted); font-size: 11px; font-weight: 700; }
  .coverage-panel, .change-panel, .inbox-panel, .detail-panel { padding: 14px; }
  .coverage-panel, .change-panel { margin-bottom: 10px; }
  .coverage-note { max-width: 760px; margin: 13px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .coverage-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 7px; margin-top: 10px; }
  .coverage-grid article { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 8px; border: 1px solid var(--color-border); border-radius: 9px; padding: 8px; background: var(--color-surface-subtle); }
  .coverage-source { min-width: 0; }
  .coverage-source strong, .coverage-source span, .coverage-grid small { display: block; }
  .coverage-source strong { overflow: hidden; color: var(--color-ink-strong); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
  .coverage-source span, .coverage-grid small { color: var(--color-muted); font-size: 10px; }
  .coverage-source span { margin-top: 4px; }
  .coverage-grid small { grid-column: 1 / -1; }
  .change-summary { display: flex; gap: 11px; margin-top: 16px; border: 1px solid var(--color-border); border-radius: 12px; padding: 14px; }
  .change-summary.current { color: var(--color-success); background: var(--color-success-soft); }
  .change-summary.attention { border-color: #fed7aa; color: var(--color-warning); background: var(--color-warning-soft); }
  .change-icon { flex: none; margin-top: 1px; }
  .change-summary strong { color: var(--color-ink-strong); font-size: 13px; }
  .change-summary p { max-width: 820px; margin: 5px 0 0; color: var(--color-ink); font-size: 12px; line-height: 1.55; }
  .change-summary ul { margin: 8px 0 0; padding-left: 18px; color: var(--color-ink); font-size: 11px; }
  .artifact-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 8px; margin-top: 14px; }
  .artifact-grid article { display: grid; min-width: 0; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 9px; border: 1px solid var(--color-border); border-radius: 10px; padding: 10px; color: var(--color-muted); background: var(--color-surface-subtle); }
  .artifact-grid article.present { border-color: var(--color-border-strong); color: var(--color-accent-strong); background: white; }
  .artifact-grid span { min-width: 0; }
  .artifact-grid strong, .artifact-grid small { display: block; }
  .artifact-grid strong { overflow: hidden; color: var(--color-ink-strong); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
  .artifact-grid small { overflow: hidden; color: var(--color-muted); font-size: var(--font-size-label); text-overflow: ellipsis; white-space: nowrap; }
  .artifact-grid span small { margin-top: 3px; }
  .panel-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; }
  .panel-heading h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; }
  .panel-heading > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }
  .capture-list { margin-top: 17px; border-top: 1px solid var(--color-border); }
  .capture-list article { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 10px; border-bottom: 1px solid var(--color-border); }
  .capture-list article.selected { margin-inline: -8px; border: 1px solid #7dd3fc; border-radius: 11px; padding-right: 8px; background: var(--color-accent-soft); }
  .capture-main { display: grid; min-height: 66px; grid-template-columns: minmax(220px, 1.5fr) minmax(160px, .8fr) 130px auto auto; align-items: center; gap: 11px; border: 0; padding: 8px 6px; color: inherit; background: transparent; text-align: left; cursor: pointer; }
  .capture-title strong, .capture-title small { display: block; }
  .capture-title strong { color: var(--color-ink-strong); font-size: 13px; line-height: 1.45; }
  .capture-title small { margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  .capture-meta { display: flex; flex-wrap: wrap; gap: 5px; }
  .capture-meta span { max-width: 100%; overflow: hidden; border: 1px solid var(--color-border); border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: white; font-size: 10px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
  .capture-date { color: var(--color-muted); font-size: 10px; }
  .review-button { min-height: 44px; padding: 6px 10px; font-size: 11px; }
  .empty-inbox { padding: 52px 20px; color: var(--color-muted); text-align: center; }
  .empty-inbox h3 { margin: 12px 0 0; color: var(--color-ink-strong); }
  .empty-inbox p { margin: 7px 0 0; }
  .detail-panel { margin-top: 10px; border-left: 3px solid var(--color-accent); }
  .capture-summary { margin: 18px 0 0; border-left: 3px solid var(--color-accent); padding: 3px 0 3px 13px; color: var(--color-ink); font-size: 14px; line-height: 1.65; }
  .detail-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 11px; margin-top: 17px; }
  .detail-grid section { border: 1px solid var(--color-border); border-radius: 12px; padding: 15px; background: var(--color-surface-subtle); }
  .detail-grid h3 { margin: 0; color: var(--color-ink-strong); font-size: 13px; }
  .detail-grid ul, .detail-grid ol { margin: 11px 0 0; padding-left: 20px; color: var(--color-ink); font-size: 12px; line-height: 1.55; }
  .detail-grid li + li { margin-top: 10px; }
  .detail-grid li strong, .detail-grid li span, .detail-grid li small { display: block; }
  .detail-grid li strong, .detail-grid code { color: var(--color-accent-strong); }
  .detail-grid li span { margin-top: 3px; }
  .detail-grid li small { margin-top: 3px; color: var(--color-muted); }
  .detail-grid section > p { color: var(--color-muted); font-size: 12px; }
  .danger-list { color: var(--color-danger) !important; }
  code { overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

  @media (max-width: 1180px) {
    .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .capture-main { grid-template-columns: minmax(200px, 1fr) minmax(150px, .7fr) auto auto; }
    .capture-date { display: none; }
  }
  @media (max-width: 760px) {
    .capture-list article { grid-template-columns: 1fr; padding-bottom: 10px; }
    .capture-main { grid-template-columns: 1fr auto; }
    .capture-meta { grid-column: 1 / -1; }
    .capture-main :global(.status) { justify-self: start; }
    .review-button { justify-self: start; margin-left: 8px; }
    .detail-grid { grid-template-columns: 1fr; }
  }
  @media (max-width: 520px) {
    .metrics { grid-template-columns: 1fr; }
    .coverage-panel, .change-panel, .inbox-panel, .detail-panel { padding: 17px; }
    .panel-heading { flex-direction: column; }
  }
</style>
