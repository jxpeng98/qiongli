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
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  type ControlAction = 'pause' | 'recover' | 'resume' | 'cancel';

  const app = useAppState();
  let selectedProjectId = $state('');
  let loadedProjectKey = $state('');

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
    if (!selectedProjectId && readyProjects[0]) {
      selectedProjectId = readyProjects[0].projectId;
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

  function selectProject(event: Event): void {
    selectedProjectId = (event.currentTarget as HTMLSelectElement).value;
    loadedProjectKey = '';
  }

  async function controlRun(
    run: OrchestrationRunSummary,
    actionName: ControlAction
  ): Promise<void> {
    if (!selectedProject) return;
    if (actionName === 'cancel' && !window.confirm(i18n.t('orchestrator.cancelConfirm'))) return;
    await app.execute({
      action: 'control-orchestration',
      projectId: selectedProject.projectId,
      expectedProjectRevision: selectedProject.semanticRevision,
      runId: run.runId,
      expectedGeneration: run.generation,
      expectedDocumentSha256: run.documentSha256,
      actionName
    });
  }

  function evidenceLabel(run: OrchestrationRunSummary): string {
    if (run.status === 'completed') return i18n.t('orchestrator.evidenceCheckpointed');
    if (run.activeTaskId) return i18n.t('orchestrator.evidenceRequired');
    return i18n.t('orchestrator.evidenceAwaitingHandoff');
  }
</script>

<PageHeader
  eyebrow={i18n.t('orchestrator.hostEyebrow')}
  title={i18n.t('orchestrator.hostTitle')}
  description={i18n.t('orchestrator.hostDescription')}
>
  {#snippet actions()}
    <a class="button-secondary integration-link" href="/client-integrations">
      <Cable size={15} aria-hidden="true" />
      {i18n.t('backend.openIntegrations')}
    </a>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section class="surface loading" aria-busy="true">{i18n.t('common.loading')}</section>
{:else}
  <section class="surface boundary" aria-labelledby="orchestrator-boundary-title">
    <GitBranch size={23} aria-hidden="true" />
    <div>
      <p class="eyebrow">{i18n.t('orchestrator.controlPlaneEyebrow')}</p>
      <h2 id="orchestrator-boundary-title">{i18n.t('orchestrator.controlPlaneTitle')}</h2>
      <p>{i18n.t('orchestrator.controlPlaneDescription')}</p>
    </div>
    <StatusBadge
      status={activeHosts.length > 0 ? 'ready' : installedHosts.length > 0 ? 'attention' : 'missing'}
      label={activeHosts.length > 0
        ? i18n.t('orchestrator.hostObserved')
        : installedHosts.length > 0
          ? i18n.t('orchestrator.hostActionRequired')
          : i18n.t('orchestrator.hostInstallRequired')}
    />
  </section>

  <div class="summary-grid">
    <section class="surface summary-card" aria-labelledby="project-summary-title">
      <ShieldCheck size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.projectSummaryEyebrow')}</p>
        <h2 id="project-summary-title">{i18n.t('orchestrator.projectSummaryTitle')}</h2>
        <p>{i18n.t('orchestrator.projectSummaryDescription', { count: readyProjects.length })}</p>
      </div>
    </section>

    <section class="surface summary-card" aria-labelledby="host-summary-title">
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
    </section>
  </div>

  <section class="surface project-control" aria-labelledby="project-control-title">
    <div>
      <p class="eyebrow">{i18n.t('orchestrator.projectEyebrow')}</p>
      <h2 id="project-control-title">{i18n.t('orchestrator.projectTitle')}</h2>
    </div>
    {#if readyProjects.length > 0}
      <label>
        <span>{i18n.t('orchestrator.project')}</span>
        <select value={selectedProjectId} onchange={selectProject}>
          {#each readyProjects as project (project.projectId)}
            <option value={project.projectId}>
              {project.displayName} · r{project.semanticRevision}
            </option>
          {/each}
        </select>
      </label>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={loadRuns}>
        <RefreshCw size={15} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('orchestrator.refreshRuns')}
      </button>
    {:else}
      <div class="empty-project">
        <p>{i18n.t('orchestrator.noProjectHelp')}</p>
        <a href="/research-library">{i18n.t('backend.openLibrary')}</a>
      </div>
    {/if}
  </section>

  {#if selectedProject}
    <section aria-labelledby="run-list-title" aria-live="polite" aria-busy={app.loading}>
      <div class="section-title">
        <div>
          <p class="eyebrow">{i18n.t('orchestrator.runsEyebrow')}</p>
          <h2 id="run-list-title">{i18n.t('orchestrator.runsTitle')}</h2>
        </div>
        <span>{i18n.t('orchestrator.runCount', { count: selectedRuns.length })}</span>
      </div>

      {#if selectedRuns.length === 0}
        <div class="surface empty-runs">
          <p>{i18n.t('orchestrator.noRuns')}</p>
          <strong>{i18n.t('orchestrator.startInHost')}</strong>
        </div>
      {:else}
        <div class="run-list">
          {#each selectedRuns as run (run.runId)}
            <article class="surface run-card">
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

              <div class="progress" aria-label={i18n.t('orchestrator.progress')}>
                <span style={`width: ${(run.completedTaskCount / run.totalTaskCount) * 100}%`}></span>
              </div>

              <dl>
                <div><dt>{i18n.t('orchestrator.progress')}</dt><dd>{run.completedTaskCount} / {run.totalTaskCount}</dd></div>
                <div><dt>{i18n.t('orchestrator.currentRole')}</dt><dd>{run.activeRole ? i18n.label(run.activeRole) : i18n.t('orchestrator.awaitingHost')}</dd></div>
                <div><dt>{i18n.t('orchestrator.evidenceStatus')}</dt><dd>{evidenceLabel(run)}</dd></div>
                <div><dt>{i18n.t('orchestrator.reviewGate')}</dt><dd>{run.completedRoleCount} / {run.requiredRoleCount}</dd></div>
                <div><dt>{i18n.t('orchestrator.generation')}</dt><dd>{run.generation}</dd></div>
                <div><dt>{i18n.t('orchestrator.checkpoint')}</dt><dd><code>{run.documentSha256.slice(0, 12)}…</code></dd></div>
              </dl>

              <footer>
                <p>{run.canContinue
                  ? i18n.t('orchestrator.continueInHost')
                  : i18n.t('orchestrator.checkpointOnly')}</p>
                <div class="run-actions">
                  {#if run.canPause}
                    <button class="button-secondary" type="button" disabled={app.loading} onclick={() => controlRun(run, 'pause')}>
                      <CirclePause size={15} aria-hidden="true" />{i18n.t('orchestrator.pause')}
                    </button>
                  {/if}
                  {#if run.canRecover}
                    <button class="button-secondary" type="button" disabled={app.loading} onclick={() => controlRun(run, 'recover')}>
                      <RotateCcw size={15} aria-hidden="true" />{i18n.t('orchestrator.recover')}
                    </button>
                  {/if}
                  {#if run.canResume}
                    <button class="button-secondary" type="button" disabled={app.loading} onclick={() => controlRun(run, 'resume')}>
                      <CirclePlay size={15} aria-hidden="true" />{i18n.t('orchestrator.resume')}
                    </button>
                  {/if}
                  {#if run.canCancel}
                    <button class="button-danger" type="button" disabled={app.loading} onclick={() => controlRun(run, 'cancel')}>
                      <Square size={14} aria-hidden="true" />{i18n.t('orchestrator.cancel')}
                    </button>
                  {/if}
                </div>
              </footer>
            </article>
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  <section class="surface approval-gate" aria-labelledby="approval-gate-title">
    <ShieldCheck size={20} aria-hidden="true" />
    <div>
      <h2 id="approval-gate-title">{i18n.t('orchestrator.approvalGateTitle')}</h2>
      <p>{i18n.t('orchestrator.approvalGateDescription')}</p>
    </div>
    <StatusBadge status="attention" label={i18n.t('orchestrator.noArtifactPreview')} />
  </section>

  <section class="surface nonclaim" aria-labelledby="orchestrator-nonclaim-title">
    <ShieldCheck size={20} aria-hidden="true" />
    <div>
      <h2 id="orchestrator-nonclaim-title">{i18n.t('orchestrator.nonclaimTitle')}</h2>
      <p>{i18n.t('orchestrator.nonclaimDescription')}</p>
    </div>
  </section>
{/if}

<style>
  .loading { padding: 22px; color: var(--color-muted); }
  .integration-link { display: inline-flex; align-items: center; gap: 7px; text-decoration: none; }
  .boundary,
  .summary-card,
  .approval-gate,
  .nonclaim {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: start;
    gap: 13px;
    padding: 18px;
  }
  .boundary { border-left: 3px solid var(--color-accent); }
  .summary-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    margin-top: 10px;
  }
  .summary-card { grid-template-columns: auto minmax(0, 1fr); }
  .project-control {
    display: grid;
    grid-template-columns: minmax(190px, .75fr) minmax(260px, 1fr) auto;
    align-items: end;
    gap: 14px;
    margin-top: 10px;
    padding: 16px;
  }
  .project-control label span { display: block; margin-bottom: 5px; color: var(--color-muted); font-size: 10px; font-weight: 750; }
  .project-control select {
    width: 100%;
    min-height: 38px;
    border: 1px solid var(--color-border-strong);
    border-radius: 9px;
    padding: 6px 9px;
    color: var(--color-ink);
    background: white;
    font: inherit;
    font-size: 12px;
  }
  .empty-project { grid-column: 2 / -1; }
  .empty-project a { color: var(--color-accent-strong); font-weight: 750; }
  .section-title {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 12px;
    margin: 22px 0 10px;
  }
  .section-title > span { color: var(--color-muted); font-size: 11px; }
  .empty-runs { padding: 18px; }
  .empty-runs strong { display: block; margin-top: 6px; color: var(--color-ink); font-size: 12px; }
  .run-list { display: grid; gap: 10px; }
  .run-card { padding: 17px; }
  .run-card header,
  .run-card footer {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
  }
  .run-card header code { color: var(--color-muted); font-size: 9px; }
  .progress { height: 5px; overflow: hidden; margin: 14px 0; border-radius: 999px; background: var(--color-surface-subtle); }
  .progress span { display: block; height: 100%; border-radius: inherit; background: var(--color-accent); }
  .run-card dl { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 1px; margin: 0 0 14px; background: var(--color-border); }
  .run-card dl div { padding: 9px 10px; background: white; }
  .run-card dt { color: var(--color-muted); font-size: 9px; font-weight: 750; }
  .run-card dd { margin: 4px 0 0; color: var(--color-ink); font-size: 11px; font-weight: 700; }
  .run-card footer { align-items: center; border-top: 1px solid var(--color-border); padding-top: 12px; }
  .run-card footer p { max-width: 480px; }
  .run-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 6px; }
  .run-actions button { display: inline-flex; min-height: 34px; align-items: center; gap: 6px; font-size: 10px; }
  .approval-gate { margin-top: 14px; border-left: 3px solid var(--color-warning); }
  .nonclaim {
    grid-template-columns: auto minmax(0, 1fr);
    margin-top: 10px;
    color: var(--color-success);
  }
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
    .summary-grid { grid-template-columns: 1fr; }
    .project-control { grid-template-columns: 1fr; align-items: stretch; }
    .empty-project { grid-column: auto; }
    .run-card dl { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
  @media (max-width: 620px) {
    .boundary,
    .approval-gate { grid-template-columns: auto minmax(0, 1fr); }
    .boundary :global(.status-badge),
    .approval-gate :global(.status-badge) { grid-column: 1 / -1; justify-self: start; }
    .run-card header,
    .run-card footer,
    .section-title { align-items: stretch; flex-direction: column; }
    .run-actions { justify-content: flex-start; }
  }
  @media (max-width: 440px) {
    .run-card dl { grid-template-columns: 1fr; }
    .run-actions { flex-direction: column; }
    .run-actions button { justify-content: center; width: 100%; }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(.spin) { animation: none; }
  }
</style>
