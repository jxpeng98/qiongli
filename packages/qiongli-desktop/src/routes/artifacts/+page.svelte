<script lang="ts">
  import type { RegisteredArtifactObservation } from '@qiongli/app-api';
  import {
    AlertTriangle,
    CheckCircle2,
    FileText,
    RefreshCw,
    ScanSearch
  } from '@lucide/svelte';

  import { useAppState, useProjectWorkspace } from '$lib/context';
  import ProjectArtifactViewer from '$lib/features/project-workspace/ProjectArtifactViewer.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';

  const app = useAppState();
  const workspace = useProjectWorkspace();

  let requestedKey = $state('');
  let loadState = $state<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  let selectedPath = $state<RegisteredArtifactObservation['relativePath'] | null>(null);

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let project = $derived(
    projects.find((candidate) => candidate.projectId === workspace.projectId) ?? null
  );
  let inventory = $derived.by(() => {
    const activeProject = project;
    const changes = app.artifactChanges;
    return activeProject
      && changes?.projectId === activeProject.projectId
      && changes.projectRevision === activeProject.semanticRevision
        ? changes
        : null;
  });
  let visibleArtifact = $derived.by(() => {
    const activeProject = project;
    const artifact = app.projectArtifact;
    return activeProject
      && artifact?.projectId === activeProject.projectId
      && artifact.projectRevision === activeProject.semanticRevision
      && artifact.artifactPath === selectedPath
        ? artifact
        : null;
  });

  $effect(() => {
    if (!project) {
      requestedKey = '';
      loadState = 'idle';
      selectedPath = null;
      return;
    }
    const key = `${project.projectId}:${project.semanticRevision}`;
    if (requestedKey === key || app.loading) return;
    requestedKey = key;
    selectedPath = null;
    void loadInventory(project.projectId, key);
  });

  async function loadInventory(projectId: string, key: string): Promise<void> {
    loadState = 'loading';
    const event = await app.execute({ action: 'load-artifact-changes', projectId });
    if (requestedKey !== key) return;
    loadState = event?.type === 'artifact-changes'
      && event.changes.projectId === projectId
      ? 'ready'
      : 'failed';
  }

  async function refreshInventory(): Promise<void> {
    if (!project || app.loading) return;
    requestedKey = '';
  }

  async function previewArtifact(observation: RegisteredArtifactObservation): Promise<void> {
    if (!project || !observation.present || app.loading) return;
    selectedPath = observation.relativePath;
    const event = await app.execute({
      action: 'read-project-artifact',
      projectId: project.projectId,
      expectedProjectRevision: project.semanticRevision,
      reference: {
        kind: 'registered-artifact',
        artifactPath: observation.relativePath,
        sourceAnchor: null
      },
      maxBytes: 128 * 1_024
    });
    if (event?.type !== 'project-artifact-read') selectedPath = null;
  }
</script>

<svelte:head>
  <title>{i18n.t('artifacts.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('artifacts.eyebrow')}
  title={i18n.t('artifacts.title')}
  description={i18n.t('artifacts.description')}
>
  {#snippet actions()}
    <button
      class="button-secondary"
      type="button"
      disabled={!project || app.loading}
      onclick={refreshInventory}
    >
      <RefreshCw size={15} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </button>
  {/snippet}
</PageHeader>

{#if !project}
  <section class="surface state-panel">
    <AlertTriangle size={22} aria-hidden="true" />
    <p>{i18n.t('projectWorkspace.none')}</p>
  </section>
{:else if loadState === 'failed'}
  <section class="surface state-panel" role="alert">
    <AlertTriangle size={22} aria-hidden="true" />
    <div>
      <h2>{i18n.t('artifacts.loadFailed')}</h2>
      <p>{i18n.t('artifacts.loadFailedDetail')}</p>
    </div>
  </section>
{:else if !inventory || loadState !== 'ready'}
  <section class="surface state-panel" role="status" aria-busy="true">
    <ScanSearch size={22} aria-hidden="true" />
    <p>{i18n.t('artifacts.loading')}</p>
  </section>
{:else}
  <section class="surface inventory" aria-labelledby="artifact-inventory-title">
    <header>
      <div>
        <p class="eyebrow">{i18n.t('artifacts.inventoryEyebrow')}</p>
        <h2 id="artifact-inventory-title">{i18n.t('artifacts.inventoryTitle')}</h2>
        <p>{i18n.t('artifacts.inventorySummary', {
          present: inventory.presentArtifactCount,
          total: inventory.registeredArtifactCount,
          revision: inventory.projectRevision
        })}</p>
      </div>
      <StatusBadge
        status={inventory.state === 'current' ? 'ready' : 'attention'}
        label={i18n.label(inventory.state)}
      />
    </header>

    <div class="artifact-list">
      {#each inventory.artifacts as observation (observation.relativePath)}
        <article class:missing={!observation.present}>
          <span class="file-mark"><FileText size={17} aria-hidden="true" /></span>
          <span class="artifact-identity">
            <strong>{i18n.label(observation.artifact)}</strong>
            <code>{observation.relativePath}</code>
          </span>
          <StatusBadge
            status={observation.present ? 'ready' : 'missing'}
            label={i18n.label(observation.present ? 'present' : 'not-present')}
          />
          <button
            class="button-secondary"
            type="button"
            disabled={!observation.present || app.loading}
            onclick={() => previewArtifact(observation)}
          >
            <ScanSearch size={14} aria-hidden="true" />
            {i18n.t('artifacts.preview')}
          </button>
        </article>
      {/each}
    </div>
  </section>

  {#if visibleArtifact}
    <ProjectArtifactViewer
      artifact={visibleArtifact}
      onClose={() => selectedPath = null}
    />
  {:else if selectedPath && app.loading}
    <section class="surface viewer-loading" role="status" aria-busy="true">
      <ScanSearch size={18} aria-hidden="true" />
      <span>{i18n.t('artifacts.previewLoading')}</span>
    </section>
  {/if}

  <section class="surface boundary">
    <CheckCircle2 size={18} aria-hidden="true" />
    <p>{i18n.t('artifacts.boundary')}</p>
  </section>
{/if}

<style>
  .state-panel, .viewer-loading, .boundary { display: flex; align-items: center; gap: 10px; padding: 16px; }
  .state-panel h2, .state-panel p, .boundary p { margin: 0; }
  .state-panel h2 { font-size: 14px; }
  .state-panel p, .boundary p { color: var(--color-muted); font-size: 11px; line-height: 1.5; }
  .inventory { overflow: hidden; margin-bottom: 12px; }
  .inventory > header { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; padding: 14px 16px; border-bottom: 1px solid var(--color-border); }
  .inventory h2 { margin: 0; font-size: 16px; }
  .inventory header p:not(.eyebrow) { margin: 4px 0 0; color: var(--color-muted); font-size: 11px; line-height: 1.45; }
  .artifact-list { display: grid; }
  .artifact-list article { display: grid; min-width: 0; grid-template-columns: auto minmax(0, 1fr) auto auto; align-items: center; gap: 10px; padding: 10px 14px; border-bottom: 1px solid var(--color-border); }
  .artifact-list article:last-child { border-bottom: 0; }
  .artifact-list article.missing { opacity: .68; }
  .file-mark { display: grid; width: 32px; height: 32px; place-items: center; border-radius: 8px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .artifact-identity { display: grid; min-width: 0; gap: 3px; }
  .artifact-identity strong { color: var(--color-ink-strong); font-size: 11px; }
  .artifact-identity code { overflow: hidden; color: var(--color-muted); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .artifact-list button { white-space: nowrap; }
  .viewer-loading { margin-bottom: 12px; color: var(--color-accent-strong); font-size: 11px; font-weight: 700; }
  .boundary { margin-top: 12px; color: var(--color-accent-strong); }
  @media (max-width: 680px) {
    .artifact-list article { grid-template-columns: auto minmax(0, 1fr) auto; }
    .artifact-list button { grid-column: 2 / -1; width: fit-content; }
  }
</style>
