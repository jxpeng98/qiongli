<script lang="ts">
  import {
    Cable,
    CirclePause,
    CirclePlay,
    GitBranch,
    RefreshCw,
    RotateCcw,
    ShieldCheck,
    Square,
    Workflow
  } from '@lucide/svelte';

  import type { OrchestrationRunSummary } from '@qiongli/app-api';
  import { ActionGroup, ContentGrid, DescriptionGrid, PageLayout, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { useAppState, useProjectWorkspace } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  type ControlAction = 'pause' | 'recover' | 'resume' | 'cancel';

  const app = useAppState();
  const projectWorkspace = useProjectWorkspace();
  let selectedProjectId = $derived(projectWorkspace.projectId);
  let observedProjectId = $state<string | null>(null);
  let loadedProjectKey = $state('');
  let pendingCancelRunId = $state<string | null>(null);

  let readyProjects = $derived(
    app.snapshot?.researchLibrary.projects.filter(
      (project) => project.lifecycle === 'active' && project.health === 'ready'
    ) ?? []
  );
  let selectedProject = $derived(
    readyProjects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let installedHosts = $derived(
    app.snapshot?.integrations.filter(
      (integration) =>
        integration.managedContent.source === 'ready'
        && integration.managedContent.registration === 'ready'
        && integration.managedContent.mcpAttachment === 'ready'
    ) ?? []
  );
  let activeHosts = $derived(
    app.snapshot?.integrations.filter(
      (integration) => integration.connection.state === 'connected'
    ) ?? []
  );
  let selectedRuns = $derived(
    app.orchestrationRuns?.projectId === selectedProjectId
      ? app.orchestrationRuns.runs
      : []
  );

  $effect(() => {
    if (selectedProjectId !== observedProjectId) {
      observedProjectId = selectedProjectId;
      loadedProjectKey = '';
      pendingCancelRunId = null;
    }
    if (selectedProject) {
      const key = `${selectedProject.projectId}:${selectedProject.semanticRevision}`;
      if (key !== loadedProjectKey) {
        loadedProjectKey = key;
        void loadRuns();
      }
    }
  });

  async function loadRuns(): Promise<void> {
    if (!selectedProject) return;
    await app.execute({
      action: 'load-orchestration',
      projectId: selectedProject.projectId,
      expectedProjectRevision: selectedProject.semanticRevision
    });
  }

  async function controlRun(
    run: OrchestrationRunSummary,
    actionName: ControlAction
  ): Promise<void> {
    if (!selectedProject) return;
    const event = await app.execute({
      action: 'control-orchestration',
      projectId: selectedProject.projectId,
      expectedProjectRevision: selectedProject.semanticRevision,
      runId: run.runId,
      expectedGeneration: run.generation,
      expectedDocumentSha256: run.documentSha256,
      actionName
    });
    if (actionName === 'cancel' && event) pendingCancelRunId = null;
  }

  function toggleCancelConfirmation(runId: string): void {
    pendingCancelRunId = pendingCancelRunId === runId ? null : runId;
  }

  function evidenceLabel(run: OrchestrationRunSummary): string {
    if (run.status === 'completed') return i18n.t('orchestrator.evidenceCheckpointed');
    if (run.activeTaskId) return i18n.t('orchestrator.evidenceRequired');
    return i18n.t('orchestrator.evidenceAwaitingHandoff');
  }
</script>

<svelte:head>
  <title>{i18n.t('orchestrator.hostTitle')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageLayout
  eyebrow={i18n.t('orchestrator.hostEyebrow')}
  title={i18n.t('orchestrator.hostTitle')}
  description={i18n.t('orchestrator.hostDescription')}
>
  {#snippet actions()}
    <Button variant="outline" href="/client-integrations">
      <Cable size={15} aria-hidden="true" />
      {i18n.t('backend.openIntegrations')}
    </Button>
  {/snippet}

{#if !app.snapshot}
  <StatePanel
    centered
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('common.loading')}
  />
{:else}
  <StatePanel tone="info" title={i18n.t('orchestrator.controlPlaneTitle')} description={i18n.t('orchestrator.controlPlaneDescription')}>
    {#snippet icon()}<GitBranch size={19} />{/snippet}
    {#snippet metadata()}
      <StatusBadge
        status={activeHosts.length > 0 ? 'ready' : installedHosts.length > 0 ? 'attention' : 'missing'}
        label={activeHosts.length > 0
          ? i18n.t('orchestrator.hostObserved')
          : installedHosts.length > 0
            ? i18n.t('orchestrator.hostActionRequired')
            : i18n.t('orchestrator.hostInstallRequired')}
      />
    {/snippet}
  </StatePanel>

  <ContentGrid columns={2} collapse="md">
    <Card.Root class="summary-card" aria-labelledby="project-summary-title">
      <ShieldCheck size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.projectSummaryEyebrow')}</p>
        <h2 id="project-summary-title">{i18n.t('orchestrator.projectSummaryTitle')}</h2>
        <p>{i18n.t('orchestrator.projectSummaryDescription', { count: readyProjects.length })}</p>
      </div>
    </Card.Root>

    <Card.Root class="summary-card" aria-labelledby="host-summary-title">
      <Workflow size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.hostsEyebrow')}</p>
        <h2 id="host-summary-title">{i18n.t('orchestrator.hostsTitle')}</h2>
        <p>
          {installedHosts.length > 0
            ? installedHosts.map((host) => host.label).join(' · ')
            : i18n.t('orchestrator.noHostReady')}
        </p>
      </div>
    </Card.Root>
  </ContentGrid>

  <Card.Root class="project-control" aria-labelledby="project-control-title">
    <div>
      <p class="eyebrow">{i18n.t('orchestrator.projectEyebrow')}</p>
      <h2 id="project-control-title">{i18n.t('orchestrator.projectTitle')}</h2>
    </div>
    {#if selectedProject}
      <div class="selected-project">
        <span>{i18n.t('orchestrator.project')}</span>
        <strong>{selectedProject.displayName} · r{selectedProject.semanticRevision}</strong>
      </div>
      <Button variant="outline" disabled={app.loading} onclick={loadRuns}>
        <RefreshCw size={15} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('orchestrator.refreshRuns')}
      </Button>
    {:else}
      <div class="empty-project">
        <p>{i18n.t('orchestrator.noProjectHelp')}</p>
        <a href="/research-library">{i18n.t('backend.openLibrary')}</a>
      </div>
    {/if}
  </Card.Root>

  {#if selectedProject}
    <section aria-labelledby="run-list-title" aria-live="polite" aria-busy={app.loading}>
      <SectionHeader eyebrow={i18n.t('orchestrator.runsEyebrow')} title={i18n.t('orchestrator.runsTitle')} titleId="run-list-title">
        {#snippet metadata()}<span>{i18n.t('orchestrator.runCount', { count: selectedRuns.length })}</span>{/snippet}
      </SectionHeader>

      {#if selectedRuns.length === 0}
        <StatePanel centered title={i18n.t('orchestrator.noRuns')} description={i18n.t('orchestrator.startInHost')} />
      {:else}
        <div class="run-list">
          {#each selectedRuns as run (run.runId)}
            <Card.Root class="run-card">
              <header>
                <div>
                  <p class="eyebrow">{run.hostDriven
                    ? i18n.t('orchestrator.hostRun')
                    : i18n.t('orchestrator.legacyRun')}</p>
                  <h3>{run.activeTaskId ?? run.nextTaskId ?? i18n.t('orchestrator.runTerminal')}</h3>
                  <code>{run.runId}</code>
                </div>
                <StatusBadge status={run.recoveryRequired ? 'recovery-required' : run.status === 'completed' ? 'ready' : run.status === 'cancelled' ? 'disabled' : 'attention'} label={i18n.label(run.status)} />
              </header>

              <Progress
                class="run-progress"
                max={run.totalTaskCount}
                value={run.completedTaskCount}
                aria-label={i18n.t('orchestrator.progress')}
              />

              <DescriptionGrid columns={3} compact class="run-facts">
                <div><dt>{i18n.t('orchestrator.progress')}</dt><dd>{run.completedTaskCount} / {run.totalTaskCount}</dd></div>
                <div><dt>{i18n.t('orchestrator.currentRole')}</dt><dd>{run.activeRole ? i18n.label(run.activeRole) : i18n.t('orchestrator.awaitingHost')}</dd></div>
                <div><dt>{i18n.t('orchestrator.evidenceStatus')}</dt><dd>{evidenceLabel(run)}</dd></div>
                <div><dt>{i18n.t('orchestrator.reviewGate')}</dt><dd>{run.completedRoleCount} / {run.requiredRoleCount}</dd></div>
                <div><dt>{i18n.t('orchestrator.generation')}</dt><dd>{run.generation}</dd></div>
                <div><dt>{i18n.t('orchestrator.checkpoint')}</dt><dd><code>{run.documentSha256.slice(0, 12)}…</code></dd></div>
              </DescriptionGrid>

              <footer>
                <p>{!run.hostDriven
                  ? i18n.t('orchestrator.legacyCheckpointOnly')
                  : run.canContinue
                    ? i18n.t('orchestrator.continueInHost')
                    : i18n.t('orchestrator.checkpointOnly')}</p>
                <ActionGroup class="run-actions" compact align="end" label={i18n.t('orchestrator.runsTitle')}>
                  {#if run.canPause}
                    <Button variant="outline" size="sm" disabled={app.loading} onclick={() => controlRun(run, 'pause')}>
                      <CirclePause size={15} aria-hidden="true" />{i18n.t('orchestrator.pause')}
                    </Button>
                  {/if}
                  {#if run.canRecover}
                    <Button variant="outline" size="sm" disabled={app.loading} onclick={() => controlRun(run, 'recover')}>
                      <RotateCcw size={15} aria-hidden="true" />{i18n.t('orchestrator.recover')}
                    </Button>
                  {/if}
                  {#if run.canResume}
                    <Button variant="outline" size="sm" disabled={app.loading} onclick={() => controlRun(run, 'resume')}>
                      <CirclePlay size={15} aria-hidden="true" />{i18n.t('orchestrator.resume')}
                    </Button>
                  {/if}
                  {#if run.canCancel}
                    <Button
                      variant="destructive"
                      size="sm"
                      disabled={app.loading}
                      aria-expanded={pendingCancelRunId === run.runId}
                      aria-controls={`cancel-confirmation-${run.runId}`}
                      onclick={() => toggleCancelConfirmation(run.runId)}
                    >
                      <Square size={14} aria-hidden="true" />{i18n.t('orchestrator.cancel')}
                    </Button>
                  {/if}
                </ActionGroup>
              </footer>
              {#if run.canCancel && pendingCancelRunId === run.runId}
                <AlertDialog.Root open onOpenChange={(open) => !open && (pendingCancelRunId = null)}>
                  <AlertDialog.Content id={`cancel-confirmation-${run.runId}`}>
                    <AlertDialog.Header>
                      <AlertDialog.Title>{i18n.t('orchestrator.cancelConfirm')}</AlertDialog.Title>
                      <AlertDialog.Description>{i18n.t('orchestrator.checkpointOnly')}</AlertDialog.Description>
                    </AlertDialog.Header>
                    <AlertDialog.Footer>
                      <Button variant="outline" disabled={app.loading} onclick={() => pendingCancelRunId = null}>{i18n.t('orchestrator.keepRun')}</Button>
                      <Button variant="destructive" disabled={app.loading} onclick={() => controlRun(run, 'cancel')}>{i18n.t('orchestrator.confirmCancel')}</Button>
                    </AlertDialog.Footer>
                  </AlertDialog.Content>
                </AlertDialog.Root>
              {/if}
            </Card.Root>
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  <StatePanel tone="warning" title={i18n.t('orchestrator.approvalGateTitle')} description={i18n.t('orchestrator.approvalGateDescription')}>
    {#snippet icon()}<ShieldCheck size={19} />{/snippet}
    {#snippet metadata()}
    <StatusBadge status="attention" label={i18n.t('orchestrator.noArtifactPreview')} />
    {/snippet}
  </StatePanel>

  <StatePanel tone="success" title={i18n.t('orchestrator.nonclaimTitle')} description={i18n.t('orchestrator.nonclaimDescription')}>
    {#snippet icon()}<ShieldCheck size={19} />{/snippet}
  </StatePanel>
{/if}
</PageLayout>

<style>
  :global(.summary-card) {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: start;
    gap: 9px;
    padding: var(--ui-panel-padding);
  }
  :global(.project-control) {
    display: grid;
    grid-template-columns: minmax(190px, .75fr) minmax(260px, 1fr) auto;
    align-items: end;
    gap: 10px;
    padding: var(--ui-panel-padding);
  }
  .selected-project { min-width: 0; }
  .selected-project span {
    display: block;
    margin-bottom: 5px;
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
  }
  .selected-project strong {
    display: block;
    overflow: hidden;
    color: var(--color-ink-strong);
    font-size: 13px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty-project { grid-column: 2 / -1; }
  .empty-project a { color: var(--color-accent-strong); font-weight: 750; }
  section { display: grid; min-width: 0; gap: var(--ui-section-gap); }
  section :global(.section-header) { width: 100%; }
  section :global(.section-header span) { color: var(--color-muted); font-size: 11px; }
  .run-list { display: grid; gap: 10px; }
  :global(.run-card) { padding: var(--ui-panel-padding); }
  :global(.run-card) header,
  :global(.run-card) footer {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }
  :global(.run-card) header code { color: var(--color-muted); font-size: var(--font-size-label); }
  :global(.run-progress) { margin: 10px 0; }
  :global(.run-facts) { margin-bottom: 10px; }
  :global(.run-card) dt { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 750; }
  :global(.run-card) dd { margin: 4px 0 0; color: var(--color-ink); font-size: 11px; font-weight: 700; }
  :global(.run-card) footer { align-items: center; border-top: 1px solid var(--color-border); padding-top: 8px; }
  :global(.run-card) footer p { max-width: 480px; }
  :global(.run-actions) { justify-content: flex-end; }
  h2, h3, p { margin-top: 0; }
  h2 { margin-bottom: 6px; color: var(--color-ink-strong); font-size: 16px; }
  h3 { margin-bottom: 4px; color: var(--color-ink-strong); font-size: 15px; }
  p { margin-bottom: 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .eyebrow {
    margin-bottom: 5px;
    color: var(--color-accent-strong);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: .1em;
    text-transform: uppercase;
  }
  @media (max-width: 840px) {
    :global(.project-control) { grid-template-columns: 1fr; align-items: stretch; }
    .empty-project { grid-column: auto; }
  }
  @media (max-width: 620px) {
    :global(.run-card) header,
    :global(.run-card) footer { align-items: stretch; flex-direction: column; }
    :global(.run-actions) { justify-content: flex-start; }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(.spin) { animation: none; }
  }
</style>
