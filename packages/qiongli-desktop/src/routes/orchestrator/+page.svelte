<script lang="ts">
  import { AlertTriangle, CheckCircle2, GitBranch, Network, Pause, Play, RefreshCw, RotateCcw, ShieldCheck, Square } from '@lucide/svelte';

  import type {
    OrchestrationRunSummary,
    StatusCode
  } from '@qiongli/app-api';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let selectedProjectId = $state('');
  let executionMode = $state<'solo' | 'duo' | 'triad'>('solo');
  let loadedReference = $state('');

  let runnableProjects = $derived(
    app.snapshot?.researchLibrary.projects.filter(
      (project) => project.lifecycle === 'active' && project.health === 'ready'
    ) ?? []
  );
  let selectedProject = $derived(
    runnableProjects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let doctor = $derived(
    app.orchestrationDoctor?.projectId === selectedProjectId
      ? app.orchestrationDoctor
      : null
  );
  let runs = $derived(
    app.orchestrationRuns?.projectId === selectedProjectId
      ? app.orchestrationRuns.runs
      : []
  );
  let doctorStatus = $derived.by<StatusCode>(() => {
    if (!doctor) return 'unavailable';
    if (doctor.recoveryRequiredCount > 0) return 'recovery-required';
    if (doctor.backendReadiness === 'ready') return 'ready';
    if (doctor.backendReadiness === 'disabled') return 'disabled';
    return 'attention';
  });

  $effect(() => {
    if (!runnableProjects.some((project) => project.projectId === selectedProjectId)) {
      selectedProjectId = runnableProjects[0]?.projectId ?? '';
    }
  });

  $effect(() => {
    const project = selectedProject;
    if (!project || !app.snapshot?.capabilities.orchestration) return;
    const reference = `${project.projectId}:${project.semanticRevision}`;
    if (loadedReference === reference) return;
    loadedReference = reference;
    void loadOrchestration(project.projectId, project.semanticRevision);
  });

  function loadOrchestration(projectId: string, revision: number): Promise<unknown> {
    return app.execute({
      action: 'load-orchestration',
      projectId,
      expectedProjectRevision: revision
    });
  }

  function refresh(): Promise<unknown> | undefined {
    if (!selectedProject) return;
    return loadOrchestration(selectedProject.projectId, selectedProject.semanticRevision);
  }

  function previewTest(): Promise<unknown> | undefined {
    if (!selectedProject || !doctor?.runnable) return;
    return app.execute({
      action: 'preview-orchestration-test',
      projectId: selectedProject.projectId,
      expectedProjectRevision: selectedProject.semanticRevision,
      executionMode
    });
  }

  function runReference(run: OrchestrationRunSummary) {
    if (!selectedProject) throw new Error('orchestration-project-unavailable');
    return {
      projectId: selectedProject.projectId,
      expectedProjectRevision: selectedProject.semanticRevision,
      runId: run.runId,
      expectedGeneration: run.generation,
      expectedDocumentSha256: run.documentSha256
    };
  }

  function previewContinue(run: OrchestrationRunSummary): Promise<unknown> {
    return app.execute({
      action: 'preview-orchestration-continue',
      ...runReference(run)
    });
  }

  function control(
    run: OrchestrationRunSummary,
    actionName: 'pause' | 'recover' | 'resume' | 'cancel'
  ): Promise<unknown> {
    return app.execute({
      action: 'control-orchestration',
      ...runReference(run),
      actionName
    });
  }

  function cancelRun(run: OrchestrationRunSummary): Promise<unknown> | undefined {
    if (!globalThis.confirm(i18n.t('orchestrator.cancelConfirm'))) return;
    return control(run, 'cancel');
  }

  function runStatus(status: OrchestrationRunSummary['status']): StatusCode {
    if (status === 'completed') return 'ready';
    if (status === 'failed') return 'blocked';
    if (status === 'cancelled') return 'disabled';
    if (status === 'paused') return 'attention';
    return 'ready';
  }
</script>

<PageHeader
  eyebrow={i18n.t('orchestrator.eyebrow')}
  title={i18n.t('orchestrator.title')}
  description={i18n.t('orchestrator.description')}
/>

{#if !app.snapshot}
  <section class="surface loading" aria-busy="true">{i18n.t('common.loading')}</section>
{:else if runnableProjects.length === 0}
  <section class="surface empty" role="status">
    <GitBranch size={24} aria-hidden="true" />
    <div>
      <h2>{i18n.t('orchestrator.noProject')}</h2>
      <p>{i18n.t('orchestrator.noProjectHelp')}</p>
      <a href="/research-library">{i18n.t('backend.openLibrary')}</a>
    </div>
  </section>
{:else}
  <section class="surface controls" aria-labelledby="orchestrator-project-title">
    <div>
      <p class="eyebrow">{i18n.t('orchestrator.projectEyebrow')}</p>
      <h2 id="orchestrator-project-title">{i18n.t('orchestrator.projectTitle')}</h2>
    </div>
    <label>
      <span>{i18n.t('orchestrator.project')}</span>
      <select bind:value={selectedProjectId} disabled={app.loading}>
        {#each runnableProjects as project}
          <option value={project.projectId}>{project.displayName} · r{project.semanticRevision}</option>
        {/each}
      </select>
    </label>
    <button class="button-secondary" type="button" disabled={app.loading} onclick={refresh}>
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </button>
  </section>

  <div class="status-grid">
    <section class="surface status-card" aria-labelledby="contract-title">
      <ShieldCheck size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.contractEyebrow')}</p>
        <h2 id="contract-title">{i18n.t('orchestrator.contractTitle')}</h2>
        <p>{i18n.t('orchestrator.contractHelp')}</p>
      </div>
      <StatusBadge
        status={doctor ? 'ready' : 'unavailable'}
        label={doctor ? i18n.label(doctor.workflowContractStatus) : i18n.t('common.loading')}
      />
    </section>

    <section class="surface status-card" aria-labelledby="runtime-title">
      <Network size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.runtimeEyebrow')}</p>
        <h2 id="runtime-title">{i18n.t('orchestrator.runtimeTitle')}</h2>
        <p>{doctor ? i18n.t('orchestrator.runCount', { count: doctor.runCount }) : i18n.t('common.loading')}</p>
      </div>
      <StatusBadge status={doctorStatus} label={doctor ? i18n.label(doctor.backendReadiness) : i18n.t('common.loading')} />
    </section>
  </div>

  {#if doctor?.recoveryRequiredCount}
    <div class="recovery-banner" role="alert">
      <AlertTriangle size={19} aria-hidden="true" />
      <div>
        <strong>{i18n.t('orchestrator.recoveryTitle')}</strong>
        <span>{i18n.t('orchestrator.recoveryHelp')}</span>
      </div>
    </div>
  {/if}

  <section class="surface test-panel" aria-labelledby="test-title">
    <div>
      <p class="eyebrow">{i18n.t('orchestrator.testEyebrow')}</p>
      <h2 id="test-title">{i18n.t('orchestrator.testTitle')}</h2>
      <p>{i18n.t('orchestrator.testHelp')}</p>
    </div>
    <label>
      <span>{i18n.t('orchestrator.mode')}</span>
      <select bind:value={executionMode} disabled={app.loading || !doctor?.runnable}>
        <option value="solo">{i18n.t('orchestrator.modeSolo')}</option>
        <option value="duo">{i18n.t('orchestrator.modeDuo')}</option>
        <option value="triad">{i18n.t('orchestrator.modeTriad')}</option>
      </select>
    </label>
    <button
      class="button-primary"
      type="button"
      disabled={app.loading || !doctor?.runnable}
      onclick={previewTest}
    >
      <Play size={16} aria-hidden="true" />
      {i18n.t('orchestrator.previewTest')}
    </button>
    {#if doctor && !doctor.runnable}
      <p class="requirement" role="status">
        {doctor.activeRunCount > 0
          ? i18n.t('orchestrator.activeRunExists')
          : i18n.t('orchestrator.backendUnavailable')}
      </p>
    {/if}
  </section>

  <section aria-labelledby="runs-title">
    <div class="section-title">
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.runsEyebrow')}</p>
        <h2 id="runs-title">{i18n.t('orchestrator.runsTitle')}</h2>
      </div>
      <span>{runs.length}</span>
    </div>

    {#if runs.length === 0}
      <div class="surface empty compact" role="status">
        <CheckCircle2 size={20} aria-hidden="true" />
        <p>{i18n.t('orchestrator.noRuns')}</p>
      </div>
    {:else}
      <div class="run-list">
        {#each runs as run (run.runId)}
          <article class="surface run-card">
            <header>
              <div>
                <p class="run-id">{run.runId}</p>
                <h3>{i18n.label(run.executionMode)} · {run.profileId}</h3>
              </div>
              <StatusBadge status={runStatus(run.status)} label={i18n.label(run.status)} />
            </header>
            <dl>
              <div><dt>{i18n.t('orchestrator.progress')}</dt><dd>{run.completedTaskCount}/{run.totalTaskCount}</dd></div>
              <div><dt>{i18n.t('orchestrator.nextTask')}</dt><dd>{run.nextTaskId ?? '—'}</dd></div>
              <div><dt>{i18n.t('orchestrator.generation')}</dt><dd>{run.generation}</dd></div>
            </dl>
            {#if run.recoveryRequired}
              <p class="run-warning"><AlertTriangle size={15} aria-hidden="true" />{i18n.t('orchestrator.interrupted')}</p>
            {/if}
            <div class="run-actions">
              {#if run.canContinue}
                <button class="button-primary" type="button" disabled={app.loading} onclick={() => previewContinue(run)}>
                  <Play size={15} aria-hidden="true" />{i18n.t('orchestrator.continue')}
                </button>
              {/if}
              {#if run.canPause}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => control(run, 'pause')}>
                  <Pause size={15} aria-hidden="true" />{i18n.t('orchestrator.pause')}
                </button>
              {/if}
              {#if run.canRecover}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => control(run, 'recover')}>
                  <RotateCcw size={15} aria-hidden="true" />{i18n.t('orchestrator.recover')}
                </button>
              {/if}
              {#if run.canResume}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => control(run, 'resume')}>
                  <Play size={15} aria-hidden="true" />{i18n.t('orchestrator.resume')}
                </button>
              {/if}
              {#if run.canCancel}
                <button class="button-danger" type="button" disabled={app.loading} onclick={() => cancelRun(run)}>
                  <Square size={14} aria-hidden="true" />{i18n.t('orchestrator.cancel')}
                </button>
              {/if}
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </section>

  {#if app.orchestrationExecution}
    <section class="surface output-panel" aria-labelledby="output-title" aria-live="polite">
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.outputEyebrow')}</p>
        <h2 id="output-title">{i18n.t('orchestrator.outputTitle')}</h2>
        <p>{i18n.t('orchestrator.outputHelp')}</p>
      </div>
      {#each app.orchestrationExecution.roleOutputs as output}
        <article class="role-output">
          <header>
            <strong>{i18n.label(output.role)} · {output.taskId}</strong>
            <span>{output.outputSha256.slice(0, 12)}…</span>
          </header>
          <div>{output.content}</div>
        </article>
      {/each}
    </section>
  {/if}
{/if}

<style>
  .loading,
  .empty {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 22px;
  }

  .empty h2,
  .empty p { margin: 0; }
  .empty p { color: var(--color-muted); }
  .empty a { color: var(--color-accent-strong); font-weight: 750; }
  .compact { padding: 18px; }

  .controls,
  .test-panel {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) minmax(220px, 320px) auto;
    align-items: end;
    gap: 18px;
    margin-bottom: 16px;
    padding: 20px;
  }

  h2, h3, p { margin-top: 0; }
  .eyebrow {
    margin-bottom: 5px;
    color: var(--color-accent-strong);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: .1em;
    text-transform: uppercase;
  }

  label { display: grid; gap: 7px; color: var(--color-muted); font-size: 12px; font-weight: 750; }
  select {
    min-height: 40px;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    padding: 0 10px;
    color: var(--color-ink-strong);
    background: white;
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
    margin-bottom: 16px;
  }

  .status-card {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: start;
    gap: 13px;
    padding: 19px;
  }
  .status-card h2 { margin-bottom: 5px; font-size: 16px; }
  .status-card p:last-child { margin-bottom: 0; color: var(--color-muted); font-size: 13px; }

  .recovery-banner {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
    border: 1px solid #fed7aa;
    border-radius: 12px;
    padding: 14px 16px;
    color: #9a3412;
    background: #fff7ed;
  }
  .recovery-banner strong,
  .recovery-banner span { display: block; }
  .recovery-banner span { margin-top: 3px; font-size: 13px; }

  .test-panel { align-items: center; }
  .test-panel p { margin-bottom: 0; color: var(--color-muted); font-size: 13px; }
  .requirement { grid-column: 1 / -1; }

  .section-title {
    display: flex;
    align-items: end;
    justify-content: space-between;
    margin: 24px 2px 12px;
  }
  .section-title h2 { margin-bottom: 0; }
  .section-title > span {
    min-width: 28px;
    border-radius: 999px;
    padding: 4px 8px;
    text-align: center;
    color: var(--color-accent-strong);
    background: var(--color-surface-subtle);
    font-weight: 800;
  }

  .run-list { display: grid; gap: 12px; }
  .run-card { padding: 18px; }
  .run-card header {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: 16px;
  }
  .run-id { margin-bottom: 4px; color: var(--color-muted); font-family: ui-monospace, monospace; font-size: 11px; }
  .run-card h3 { margin-bottom: 0; font-size: 15px; }
  .run-card dl { display: flex; flex-wrap: wrap; gap: 22px; margin: 16px 0; }
  .run-card dl div { display: grid; gap: 3px; }
  .run-card dt { color: var(--color-muted); font-size: 11px; font-weight: 750; }
  .run-card dd { margin: 0; font-size: 13px; font-weight: 760; }
  .run-warning { display: flex; align-items: center; gap: 7px; color: #9a3412; font-size: 12px; }
  .run-actions { display: flex; flex-wrap: wrap; gap: 8px; }

  .output-panel { margin-top: 22px; padding: 20px; }
  .output-panel > div > p:last-child { color: var(--color-muted); font-size: 13px; }
  .role-output {
    margin-top: 12px;
    border: 1px solid var(--color-border);
    border-radius: 10px;
    overflow: hidden;
  }
  .role-output header {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    background: var(--color-surface-subtle);
    font-size: 12px;
  }
  .role-output header span { color: var(--color-muted); font-family: ui-monospace, monospace; }
  .role-output > div {
    max-height: 420px;
    overflow: auto;
    padding: 14px;
    white-space: pre-wrap;
    line-height: 1.6;
  }

  button:focus-visible,
  select:focus-visible,
  a:focus-visible {
    outline: 2px solid var(--color-accent-strong);
    outline-offset: 2px;
  }

  @media (max-width: 860px) {
    .controls,
    .test-panel,
    .status-grid { grid-template-columns: 1fr; }
    .controls button,
    .test-panel button { justify-self: start; }
  }
</style>
