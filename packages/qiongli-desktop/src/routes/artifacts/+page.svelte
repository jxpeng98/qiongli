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
  import { PageLayout, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

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

<PageLayout
  eyebrow={i18n.t('artifacts.eyebrow')}
  title={i18n.t('artifacts.title')}
  description={i18n.t('artifacts.description')}
>
  {#snippet actions()}
    <Button
      variant="outline"
      disabled={!project || app.loading}
      onclick={refreshInventory}
    >
      <RefreshCw size={15} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </Button>
  {/snippet}

{#if !project}
  <StatePanel tone="warning" description={i18n.t('projectWorkspace.none')}>
    {#snippet icon()}<AlertTriangle size={22} />{/snippet}
  </StatePanel>
{:else if loadState === 'failed'}
  <StatePanel tone="danger" role="alert" title={i18n.t('artifacts.loadFailed')} description={i18n.t('artifacts.loadFailedDetail')}>
    {#snippet icon()}<AlertTriangle size={22} />{/snippet}
  </StatePanel>
{:else if !inventory || loadState !== 'ready'}
  <StatePanel role="status" busy description={i18n.t('artifacts.loading')}>
    {#snippet icon()}<ScanSearch size={22} />{/snippet}
  </StatePanel>
{:else}
  <Card.Root class="inventory" aria-labelledby="artifact-inventory-title">
    <SectionHeader
      variant="panel"
      eyebrow={i18n.t('artifacts.inventoryEyebrow')}
      title={i18n.t('artifacts.inventoryTitle')}
      titleId="artifact-inventory-title"
      description={i18n.t('artifacts.inventorySummary', {
        present: inventory.presentArtifactCount,
        total: inventory.registeredArtifactCount,
        revision: inventory.projectRevision
      })}
    >
      {#snippet metadata()}
        <StatusBadge
          status={inventory.state === 'current' ? 'ready' : 'attention'}
          label={i18n.label(inventory.state)}
        />
      {/snippet}
    </SectionHeader>

    <div class="artifact-list">
      {#each inventory.artifacts as observation (observation.relativePath)}
        <article class:missing={!observation.present}>
          <span class="file-mark"><FileText size={17} aria-hidden="true" /></span>
          <span class="artifact-identity">
            <strong>{i18n.label(observation.artifact)}</strong>
            <code title={observation.relativePath}>{observation.relativePath}</code>
          </span>
          <StatusBadge
            status={observation.present ? 'ready' : 'missing'}
            label={i18n.label(observation.present ? 'present' : 'not-present')}
          />
          <Button
            class="artifact-action"
            variant="outline"
            size="sm"
            disabled={!observation.present || app.loading}
            onclick={() => previewArtifact(observation)}
          >
            <ScanSearch size={14} aria-hidden="true" />
            {i18n.t('artifacts.preview')}
          </Button>
        </article>
      {/each}
    </div>
  </Card.Root>

  {#if visibleArtifact}
    <ProjectArtifactViewer
      artifact={visibleArtifact}
      onClose={() => selectedPath = null}
    />
  {:else if selectedPath && app.loading}
    <StatePanel role="status" busy description={i18n.t('artifacts.previewLoading')}>
      {#snippet icon()}<ScanSearch size={18} />{/snippet}
    </StatePanel>
  {/if}

  <StatePanel
    tone="success"
    title={i18n.t('artifacts.boundaryTitle')}
    description={i18n.t('artifacts.boundary')}
    descriptionMode="tooltip"
  >
    {#snippet icon()}<CheckCircle2 size={18} />{/snippet}
  </StatePanel>
{/if}
</PageLayout>

<style>
  :global(.inventory) { overflow: hidden; }
  .artifact-list { display: grid; }
  .artifact-list article { display: grid; min-width: 0; grid-template-columns: auto minmax(0, 1fr) auto auto; align-items: center; gap: 7px; padding: 7px 10px; border-bottom: 1px solid var(--color-border); }
  .artifact-list article:last-child { border-bottom: 0; }
  .artifact-list article.missing { opacity: .68; }
  .file-mark { display: grid; width: 32px; height: 32px; place-items: center; border-radius: var(--radius-control); color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .artifact-identity { display: grid; min-width: 0; gap: 3px; }
  .artifact-identity strong { color: var(--color-ink-strong); font-size: var(--font-size-label); }
  .artifact-identity code { overflow: hidden; color: var(--color-muted); font-size: var(--font-size-micro); text-overflow: ellipsis; white-space: nowrap; }
  @media (max-width: 680px) {
    .artifact-list article { grid-template-columns: auto minmax(0, 1fr) auto; }
    :global(.artifact-action) { grid-column: 2 / -1; }
  }
</style>
