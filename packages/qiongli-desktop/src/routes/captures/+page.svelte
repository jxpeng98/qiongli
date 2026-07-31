<script lang="ts">
  import { page } from '$app/state';
  import type {
    AppIntent,
    CaptureAssignmentPage,
    CaptureAssignmentView,
    CaptureDeliveryPage,
    CaptureDeliveryView,
    CaptureInboxEntry,
    CaptureResolutionPage,
    CaptureResolutionSelection,
    CaptureResolutionView,
    ResearchCapture
  } from '@qiongli/app-api';
  import {
    AlertTriangle,
    ArrowRight,
    CheckCircle2,
    FileInput,
    Inbox,
    RefreshCw,
    ScanSearch
  } from '@lucide/svelte';

  import { useAppState, useProjectWorkspace } from '$lib/context';
  import CaptureConflicts from '$lib/features/captures/CaptureConflicts.svelte';
  import CaptureOutbox from '$lib/features/captures/CaptureOutbox.svelte';
  import CaptureWorkspaceTabs from '$lib/features/captures/CaptureWorkspaceTabs.svelte';
  import ProjectArtifactViewer from '$lib/features/project-workspace/ProjectArtifactViewer.svelte';
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
  import { MetricCard, MetricGrid, PageHeader, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  const projectWorkspace = useProjectWorkspace();

  let selectedProjectId = $derived(projectWorkspace.projectId);
  let requestedProjectId = $state<string | null>(null);
  let requestedProjectRevision = $state<number | null>(null);
  let manualRefreshInProgress = $state(false);
  let selectedCaptureId = $state<string | null>(null);
  let selectedEvidencePath = $state<RegisteredArtifactPath | null>(null);
  let selectedEvidenceAnchor = $state<string | null>(null);
  let appliedCaptureDeepLink = $state('');
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
  let visibleEvidenceArtifact = $derived.by(() => {
    const project = selectedProject;
    const artifact = app.projectArtifact;
    return project
      && artifact?.projectId === project.projectId
      && artifact.projectRevision === project.semanticRevision
      && artifact.artifactPath === selectedEvidencePath
      && artifact.sourceAnchor === selectedEvidenceAnchor
        ? artifact
        : null;
  });
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
    const captureId = page.url.searchParams.get('capture');
    const entry = captureId
      ? inbox?.entries.find((candidate) => candidate.captureId === captureId)
      : null;
    if (!entry || !selectedProjectId) return;
    const key = `${selectedProjectId}:${entry.captureId}`;
    if (appliedCaptureDeepLink === key) return;
    appliedCaptureDeepLink = key;
    workspaceMode = 'inbox';
    void inspectCapture(entry);
  });

  $effect(() => {
    if (projects.length === 0) {
      requestedProjectId = null;
      requestedProjectRevision = null;
      captureLoad = null;
      return;
    }
    if (!selectedProjectId || !projects.some((project) => project.projectId === selectedProjectId)) {
      requestedProjectId = null;
      requestedProjectRevision = null;
      captureLoad = null;
      return;
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
    selectedEvidencePath = null;
    selectedEvidenceAnchor = null;
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

  type RegisteredArtifactReference = Extract<
    Extract<AppIntent, { action: 'read-project-artifact' }>['reference'],
    { kind: 'registered-artifact' }
  >;
  type RegisteredArtifactPath = RegisteredArtifactReference['artifactPath'];
  type CaptureEvidence = ResearchCapture['evidence'][number];

  const registeredArtifactPaths = new Set<RegisteredArtifactPath>([
    'context/project_manifest.json',
    'context/research_state.md',
    'context/decision_log.md',
    'context/stage_handoff.md',
    'context/boundary_review.md',
    'context/idea_funnel.md',
    'literature/literature_map.md',
    'evidence/claim-evidence-ledger.csv',
    'manuscript/claims_evidence_map.md',
    'graph/semantic_links.jsonl'
  ]);

  function captureEvidenceReference(evidence: CaptureEvidence): RegisteredArtifactReference | null {
    if (evidence.locatorKind !== 'artifact-anchor') return null;
    const separator = evidence.locator.indexOf('#');
    const artifactPath = (separator < 0
      ? evidence.locator
      : evidence.locator.slice(0, separator)) as RegisteredArtifactPath;
    if (!registeredArtifactPaths.has(artifactPath)) return null;
    const sourceAnchor = separator < 0 ? null : evidence.locator.slice(separator + 1);
    return {
      kind: 'registered-artifact',
      artifactPath,
      sourceAnchor: sourceAnchor || null
    };
  }

  function canPreviewCaptureEvidence(evidence: CaptureEvidence): boolean {
    const reference = captureEvidenceReference(evidence);
    return reference !== null
      && app.capture?.binding.baseRevision === selectedProject?.semanticRevision
      && changes?.artifacts.some((artifact) =>
        artifact.present && artifact.relativePath === reference.artifactPath) === true;
  }

  async function previewCaptureEvidence(evidence: CaptureEvidence): Promise<void> {
    const reference = captureEvidenceReference(evidence);
    if (!selectedProject || !reference || !canPreviewCaptureEvidence(evidence)) return;
    selectedEvidencePath = reference.artifactPath;
    selectedEvidenceAnchor = reference.sourceAnchor;
    const event = await app.execute({
      action: 'read-project-artifact',
      projectId: selectedProject.projectId,
      expectedProjectRevision: selectedProject.semanticRevision,
      reference,
      maxBytes: 64 * 1_024
    });
    if (event?.type !== 'project-artifact-read') {
      selectedEvidencePath = null;
      selectedEvidenceAnchor = null;
    }
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
    <Button
      type="button"
      disabled={app.loading || !selectedProject || selectedProject.health === 'inspection-blocked' || !app.snapshot?.capabilities.captureMutation}
      onclick={importCapture}
    >
      <FileInput size={16} aria-hidden="true" />{i18n.t('captures.import')}
    </Button>
    <Button
      variant="outline"
      type="button"
      disabled={app.loading || !selectedProject || selectedProject.health === 'inspection-blocked' || !app.snapshot?.capabilities.captureInbox}
      onclick={refreshInbox}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />{i18n.t('common.refresh')}
    </Button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <StatePanel
    centered
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('captures.loadingLibrary')}
  />
{:else if projects.length === 0}
  <StatePanel centered title={i18n.t('captures.emptyTitle')} description={i18n.t('captures.emptyDetail')}>
    {#snippet icon()}<Inbox size={22} />{/snippet}
    {#snippet actions()}<Button href="/research-library">{i18n.t('captures.openLibrary')}</Button>{/snippet}
  </StatePanel>
{:else if selectedProject?.health === 'inspection-blocked'}
  <StatePanel tone="danger" title={i18n.t('captures.blocked')} description={i18n.t('captures.blockedDetail')}>
    {#snippet icon()}<AlertTriangle size={19} />{/snippet}
  </StatePanel>
{:else if captureLoadStatus === 'failed'}
  <StatePanel tone="danger" role="alert" title={i18n.t('captures.loadFailedTitle')} description={i18n.t('captures.loadFailedDetail')}>
    {#snippet icon()}<AlertTriangle size={19} />{/snippet}
    {#snippet actions()}
      <Button variant="outline" disabled={app.loading} onclick={refreshInbox}>
        <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('captures.retryInspection')}
      </Button>
    {/snippet}
  </StatePanel>
{:else if captureLoadStatus !== 'ready' || !inbox || !coverage || !changes}
  <StatePanel
    centered
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('captures.inspecting', { project: selectedProject?.displayName ?? '' })}
  />
{:else}
  <div class="metrics-wrap">
    <MetricGrid label={i18n.t('captures.summaryAria')}>
      <MetricCard value={inbox.entries.length} label={i18n.t('captures.captures')}>
        {#snippet icon()}<Inbox size={18} />{/snippet}
      </MetricCard>
      <MetricCard value={inbox.pendingReviewCount} label={i18n.t('captures.pending')} tone={inbox.pendingReviewCount > 0 ? 'warning' : 'neutral'}>
        {#snippet icon()}<ScanSearch size={18} />{/snippet}
      </MetricCard>
      <MetricCard value={inbox.staleCount + inbox.conflictedCount} label={i18n.t('captures.resolution')} tone={inbox.staleCount + inbox.conflictedCount > 0 ? 'warning' : 'neutral'}>
        {#snippet icon()}<AlertTriangle size={18} />{/snippet}
      </MetricCard>
      <MetricCard value={inbox.appliedCount} label={i18n.t('captures.consolidated')} tone="success">
        {#snippet icon()}<CheckCircle2 size={18} />{/snippet}
      </MetricCard>
    </MetricGrid>
  </div>

  <CaptureWorkspaceTabs
    mode={workspaceMode}
    counts={workspaceCounts}
    onChange={chooseWorkspaceMode}
  >
  {#snippet panel(panelMode)}

  {#if panelMode === 'coverage'}
    <div>
      <Card.Root class="coverage-panel" role="region" aria-labelledby="coverage-title">
    <SectionHeader eyebrow={i18n.t('captures.coverageEyebrow')} title={i18n.t('captures.coverageTitle')} titleId="coverage-title" description={i18n.t('captures.coverageSummary', { captures: coverage.captureCount, unknown: coverage.unknownSourceCount })}>
      {#snippet metadata()}
        <StatusBadge
          status={coverage.unknownSourceCount === 0 ? 'ready' : 'attention'}
          label={coverage.unknownSourceCount === 0 ? i18n.label('observed') : i18n.label('partial-coverage')}
        />
      {/snippet}
    </SectionHeader>
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
      </Card.Root>

      <Card.Root class="change-panel" role="region" aria-labelledby="change-title">
    <SectionHeader eyebrow={i18n.t('captures.changesEyebrow')} title={i18n.t('captures.changesTitle')} titleId="change-title" description={i18n.t('captures.changesSummary', { revision: changes.projectRevision, present: changes.presentArtifactCount, registered: changes.registeredArtifactCount })}>
      {#snippet metadata()}
        <StatusBadge
          status={artifactChangeStatus(changes)}
          label={changes.state === 'current' ? i18n.t('captures.revisionCurrent') : i18n.t('captures.unattributedChange')}
        />
      {/snippet}
    </SectionHeader>

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

    <Button class="artifacts-link" variant="outline" href={projectWorkspace.href('/artifacts', selectedProjectId ?? undefined)}>
      <ScanSearch size={15} aria-hidden="true" />
      {i18n.t('captures.openArtifacts')}
      <ArrowRight size={14} aria-hidden="true" />
    </Button>
      </Card.Root>
    </div>

  {:else if panelMode === 'inbox'}
    <div>
      <Card.Root class="inbox-panel">
    <SectionHeader eyebrow={i18n.t('captures.queueEyebrow')} title={selectedProject?.displayName ?? ''} description={i18n.t('captures.queueSummary', { revision: inbox.projectRevision, stage: sentence(inbox.projectStage) })}>
      {#snippet metadata()}<StatusBadge status={inbox.entries.length === 0 ? 'ready' : 'attention'} label={inbox.entries.length === 0 ? i18n.t('captures.clear') : i18n.t('captures.reviewAvailable')} />{/snippet}
    </SectionHeader>

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
            <Button class="capture-main" variant="ghost" onclick={() => inspectCapture(entry)}>
              <span class="capture-title"><strong>{entry.summary}</strong><small>{entry.task}</small></span>
              <span class="capture-meta"><span>{sentence(entry.source)}</span><span>{sentence(entry.disposition)}</span><span>r{entry.baseRevision}</span></span>
              <span class="capture-date">{captureDate(entry)}</span>
              <StatusBadge status={captureStatus(entry)} label={sentence(entry.state)} />
              <ArrowRight size={17} aria-hidden="true" />
            </Button>
            <Button
              class="review-button"
              variant="outline"
              disabled={app.loading || !canReviewCapture(entry)}
              onclick={() => reviewCapture(entry)}
            >
              {entry.state === 'applied' ? i18n.t('captures.consolidated') : i18n.label('review-plan')}
            </Button>
          </article>
        {/each}
      </div>
    {/if}
      </Card.Root>

      {#if app.capture && app.capture.captureId === selectedCaptureId}
        <Card.Root class="detail-panel" aria-live="polite">
      <SectionHeader eyebrow={i18n.t('captures.detailEyebrow')} title={i18n.t('captures.detailTitle')} description={app.capture.captureId}>
        {#snippet actions()}
          <Button variant="ghost" onclick={() => {
            selectedCaptureId = null;
            selectedEvidencePath = null;
            selectedEvidenceAnchor = null;
          }}>{i18n.t('common.close')}</Button>
        {/snippet}
      </SectionHeader>
      <p class="capture-summary">{app.capture.summary}</p>
      <div class="detail-grid">
        <section><h3>{i18n.label('academic-changes')}</h3>{#if app.capture.changes.length}<ul>{#each app.capture.changes as change}<li><strong>{sentence(change.area)}</strong><span>{change.summary}</span></li>{/each}</ul>{:else}<p>{i18n.t('common.none')}</p>{/if}</section>
        <section><h3>{i18n.label('decision-candidates')}</h3>{#if app.capture.decisions.length}<ul>{#each app.capture.decisions as decision}<li><strong>{sentence(decision.relation)}</strong><span>{decision.statement}</span><small>{decision.rationale}</small></li>{/each}</ul>{:else}<p>{i18n.t('common.none')}</p>{/if}</section>
        <section><h3>{i18n.label('evidence-references')}</h3>{#if app.capture.evidence.length}<ul>{#each app.capture.evidence as evidence}<li><code>{evidence.locator}</code><span>{evidence.relevance}</span>{#if evidence.limitation}<small>{evidence.limitation}</small>{/if}{#if captureEvidenceReference(evidence)}<Button class="evidence-preview" variant="ghost" size="sm" disabled={app.loading || !canPreviewCaptureEvidence(evidence)} onclick={() => previewCaptureEvidence(evidence)}><ScanSearch size={13} aria-hidden="true" />{i18n.t('captures.previewEvidence')}</Button>{/if}</li>{/each}</ul>{:else}<p>{i18n.t('common.none')}</p>{/if}</section>
        <section><h3>{i18n.t('captures.contradictions')}</h3>{#if app.capture.contradictions.length}<ul class="danger-list">{#each app.capture.contradictions as contradiction}<li><strong>{contradiction.statement}</strong><span>{contradiction.consequence}</span></li>{/each}</ul>{/if}{#if app.capture.nextActions.length}<ol>{#each app.capture.nextActions as action}<li>{action}</li>{/each}</ol>{:else if app.capture.contradictions.length === 0}<p>{i18n.t('common.none')}</p>{/if}</section>
      </div>
      {#if visibleEvidenceArtifact}
        <ProjectArtifactViewer
          artifact={visibleEvidenceArtifact}
          onClose={() => {
            selectedEvidencePath = null;
            selectedEvidenceAnchor = null;
          }}
        />
      {/if}
        </Card.Root>
      {/if}
    </div>
  {:else if panelMode === 'outbox'}
    {#if deliveryLoadStatus === 'failed'}
      <StatePanel tone="danger" role="alert" title={i18n.t('captures.continuityLoadFailedTitle')} description={i18n.t('captures.continuityLoadFailedDetail')}>
        {#snippet icon()}<AlertTriangle size={19} />{/snippet}
        {#snippet actions()}
          <Button variant="outline" disabled={continuityLoading} onclick={retryContinuityLoad}>
            <RefreshCw size={16} class={continuityLoading ? 'spin' : undefined} aria-hidden="true" />
            {i18n.t('captures.retryContinuityLoad')}
          </Button>
        {/snippet}
      </StatePanel>
    {:else if deliveryLoadStatus !== 'ready'}
      <StatePanel
        centered
        role="status"
        busy
        live="polite"
        atomic
        description={i18n.t('captures.loadingOutbox')}
      />
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
      <StatePanel tone="danger" role="alert" title={i18n.t('captures.continuityLoadFailedTitle')} description={i18n.t('captures.continuityLoadFailedDetail')}>
        {#snippet icon()}<AlertTriangle size={19} />{/snippet}
        {#snippet actions()}
          <Button variant="outline" disabled={continuityLoading} onclick={retryContinuityLoad}>
            <RefreshCw size={16} class={continuityLoading ? 'spin' : undefined} aria-hidden="true" />
            {i18n.t('captures.retryContinuityLoad')}
          </Button>
        {/snippet}
      </StatePanel>
    {:else if deliveryLoadStatus !== 'ready' || conflictLoadStatus !== 'ready'}
      <StatePanel
        centered
        role="status"
        busy
        live="polite"
        atomic
        description={i18n.t('captures.loadingConflicts')}
      />
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
  {/snippet}
  </CaptureWorkspaceTabs>
{/if}

<style>
  .metrics-wrap { margin-bottom: 10px; }
  :global(.coverage-panel), :global(.change-panel), :global(.inbox-panel), :global(.detail-panel) { padding: 14px; }
  :global(.coverage-panel), :global(.change-panel) { margin-bottom: 10px; }
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
  .change-summary.attention { border-color: var(--color-warning-border); color: var(--color-warning-strong); background: var(--color-warning-soft); }
  .change-icon { flex: none; margin-top: 1px; }
  .change-summary strong { color: var(--color-ink-strong); font-size: 13px; }
  .change-summary p { max-width: 820px; margin: 5px 0 0; color: var(--color-ink); font-size: 12px; line-height: 1.55; }
  .change-summary ul { margin: 8px 0 0; padding-left: 18px; color: var(--color-ink); font-size: 11px; }
  :global(.artifacts-link) { width: fit-content; margin-top: 14px; white-space: nowrap; }
  .capture-list { margin-top: 17px; border-top: 1px solid var(--color-border); }
  .capture-list article { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 10px; border-bottom: 1px solid var(--color-border); }
  .capture-list article.selected { margin-inline: -8px; border: 1px solid var(--color-accent-border); border-radius: 11px; padding-right: 8px; background: var(--color-accent-soft); }
  :global(.capture-main) { display: grid; width: 100%; height: auto; min-height: 66px; grid-template-columns: minmax(220px, 1.5fr) minmax(160px, .8fr) 130px auto auto; align-items: center; gap: 11px; border: 0; padding: 8px 6px; color: inherit; background: transparent; text-align: left; white-space: normal; cursor: pointer; }
  .capture-title strong, .capture-title small { display: block; }
  .capture-title strong { color: var(--color-ink-strong); font-size: 13px; line-height: 1.45; }
  .capture-title small { margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  .capture-meta { display: flex; flex-wrap: wrap; gap: 5px; }
  .capture-meta span { max-width: 100%; overflow: hidden; border: 1px solid var(--color-border); border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: var(--color-control); font-size: 10px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
  .capture-date { color: var(--color-muted); font-size: 10px; }
  :global(.review-button) { min-height: 44px; padding: 6px 10px; font-size: 11px; }
  .empty-inbox { padding: 52px 20px; color: var(--color-muted); text-align: center; }
  .empty-inbox h3 { margin: 12px 0 0; color: var(--color-ink-strong); }
  .empty-inbox p { margin: 7px 0 0; }
  :global(.detail-panel) { margin-top: 10px; border-left: 3px solid var(--color-accent); }
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
  :global(.evidence-preview) { min-height: 34px; margin-top: 7px; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  .detail-grid + :global(.artifact-viewer) { margin-top: 12px; }
  .detail-grid section > p { color: var(--color-muted); font-size: 12px; }
  .danger-list { color: var(--color-danger) !important; }
  code { overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

  @media (max-width: 1180px) {
    :global(.capture-main) { grid-template-columns: minmax(200px, 1fr) minmax(150px, .7fr) auto auto; }
    .capture-date { display: none; }
  }
  @media (max-width: 760px) {
    .capture-list article { grid-template-columns: 1fr; padding-bottom: 10px; }
    :global(.capture-main) { grid-template-columns: 1fr auto; }
    .capture-meta { grid-column: 1 / -1; }
    :global(.capture-main) :global(.status) { justify-self: start; }
    :global(.review-button) { justify-self: start; margin-left: 8px; }
    .detail-grid { grid-template-columns: 1fr; }
  }
  @media (max-width: 520px) {
    :global(.coverage-panel), :global(.change-panel), :global(.inbox-panel), :global(.detail-panel) { padding: 14px; }
  }
</style>
