<script lang="ts">
  import type {
    AcademicGraphEntityReference,
    ProjectArtifactView
  } from '@qiongli/app-api';
  import { ExternalLink, FileSearch } from '@lucide/svelte';

  import ProjectArtifactViewer from '$lib/features/project-workspace/ProjectArtifactViewer.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  import type { AcademicGraphInspection } from './inspection';

  let {
    inspection,
    artifact = null,
    disabled = false,
    onPreview,
    onOpen
  }: {
    inspection: AcademicGraphInspection | null;
    artifact?: ProjectArtifactView | null;
    disabled?: boolean;
    onPreview: (entity: AcademicGraphEntityReference) => Promise<boolean>;
    onOpen: (entity: AcademicGraphEntityReference) => Promise<boolean>;
  } = $props();

  let openState = $state<'idle' | 'opening' | 'opened' | 'failed'>('idle');
  let previewState = $state<'idle' | 'loading' | 'loaded' | 'failed'>('idle');
  let showViewer = $state(false);
  let activeEntity = $state('');
  let previewButton = $state<HTMLButtonElement | null>(null);
  let visibleArtifact = $derived(
    showViewer
    && inspection
    && artifact?.projectId
    && artifact.entityKind === inspection.entity.kind
    && artifact.entityId === inspection.entity.id
      ? artifact
      : null
  );

  $effect(() => {
    const nextEntity = inspection ? `${inspection.entity.kind}:${inspection.entity.id}` : '';
    if (nextEntity !== activeEntity) {
      activeEntity = nextEntity;
      openState = 'idle';
      previewState = 'idle';
      showViewer = false;
    }
  });

  async function previewArtifact(): Promise<void> {
    if (!inspection || disabled || previewState === 'loading') return;
    previewState = 'loading';
    try {
      const loaded = await onPreview(inspection.entity);
      previewState = loaded ? 'loaded' : 'failed';
      showViewer = loaded;
    } catch {
      previewState = 'failed';
      showViewer = false;
    }
  }

  async function openArtifact(): Promise<void> {
    if (!inspection || disabled || openState === 'opening') return;
    openState = 'opening';
    try {
      openState = await onOpen(inspection.entity) ? 'opened' : 'failed';
    } catch {
      openState = 'failed';
    }
  }
</script>

<Card.Root class="inspector" role="region" aria-labelledby="graph-inspector-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('graph.inspectorEyebrow')}</p>
      <h2 id="graph-inspector-title">{i18n.t('graph.inspectorTitle')}</h2>
    </div>
    <span class="header-icon"><FileSearch size={20} aria-hidden="true" /></span>
  </header>

  {#if !inspection}
    <p class="empty">{i18n.t('graph.inspectorEmpty')}</p>
  {:else}
    <div class="inspector-body">
      <div class="identity">
        <span>{i18n.t(inspection.entity.kind === 'node' ? 'graph.node' : 'graph.edge')}</span>
        <h3>{inspection.title}</h3>
        <code>{inspection.entity.id}</code>
      </div>

      <dl>
        {#if inspection.nodeType}<div><dt>{i18n.t('graph.type')}</dt><dd>{i18n.label(inspection.nodeType)}</dd></div>{/if}
        {#if inspection.relation}<div><dt>{i18n.t('graph.relation')}</dt><dd>{i18n.label(inspection.relation)}</dd></div>{/if}
        {#if inspection.canonicalId}<div><dt>{i18n.t('graph.canonicalId')}</dt><dd>{inspection.canonicalId}</dd></div>{/if}
        <div><dt>{i18n.t('graph.layers')}</dt><dd>{inspection.layers.map((layer) => i18n.label(layer)).join(', ')}</dd></div>
        <div><dt>{i18n.t('graph.artifact')}</dt><dd><code>{inspection.artifactPath}</code></dd></div>
        <div><dt>{i18n.t('graph.anchor')}</dt><dd><code>{inspection.sourceAnchor}</code></dd></div>
        {#if inspection.inferenceStrength}<div><dt>{i18n.t('graph.strength')}</dt><dd>{i18n.label(inspection.inferenceStrength)}</dd></div>{/if}
        {#if inspection.confidence}<div><dt>{i18n.t('graph.confidence')}</dt><dd>{i18n.label(inspection.confidence)}</dd></div>{/if}
        {#if inspection.status}<div><dt>{i18n.t('graph.status')}</dt><dd>{i18n.label(inspection.status)}</dd></div>{/if}
      </dl>

      {#if inspection.rationale}
        <section class="detail"><h4>{i18n.t('graph.rationale')}</h4><p>{inspection.rationale}</p></section>
      {/if}
      {#if inspection.evidenceLimit}
        <section class="detail"><h4>{i18n.t('graph.evidenceLimit')}</h4><p>{inspection.evidenceLimit}</p></section>
      {/if}

      <section class="detail locations">
        <h4>{i18n.t('graph.manuscriptLocations')}</h4>
        {#if inspection.manuscriptLocations.length === 0}
          <p>{i18n.t('graph.noManuscriptLocations')}</p>
        {:else}
          <ul>
            {#each inspection.manuscriptLocations as location (location.nodeId)}
              <li><strong>{location.label}</strong><code>{location.sourceAnchor}</code></li>
            {/each}
          </ul>
        {/if}
      </section>

      <div class="artifact-actions">
        <Button
          disabled={disabled || previewState === 'loading'}
          bind:ref={previewButton}
          onclick={previewArtifact}
        >
          <FileSearch size={15} aria-hidden="true" />
          {previewState === 'loading'
            ? i18n.t('graph.previewingArtifact')
            : i18n.t('graph.previewArtifact')}
        </Button>
        <Button variant="outline" disabled={disabled || openState === 'opening'} onclick={openArtifact}>
          <ExternalLink size={15} aria-hidden="true" />
          {openState === 'opening' ? i18n.t('graph.openingArtifact') : i18n.t('graph.openArtifact')}
        </Button>
      </div>
      <p class:failed={previewState === 'failed' || openState === 'failed'} class="open-status" aria-live="polite">
        {previewState === 'loaded'
          ? i18n.t('graph.artifactPreviewed')
          : previewState === 'failed'
            ? i18n.t('graph.artifactPreviewFailed')
            : openState === 'opened'
              ? i18n.t('graph.artifactOpened')
              : openState === 'failed' ? i18n.t('graph.artifactOpenFailed') : ''}
      </p>
      {#if visibleArtifact}
        <ProjectArtifactViewer
          artifact={visibleArtifact}
          returnFocusTarget={previewButton ?? null}
          onClose={() => showViewer = false}
        />
      {/if}
    </div>
  {/if}
</Card.Root>

<style>
  :global(.inspector) { min-width: 0; margin-bottom: 9px; overflow: hidden; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 8px; border-bottom: 1px solid var(--color-border); padding: 8px 10px; }
  h2, h3, h4, p { margin: 0; }
  h2 { font-size: 16px; }
  .header-icon { display: inline-flex; color: var(--color-accent-strong); }
  .empty { padding: 12px 10px; color: var(--color-muted); font-size: var(--font-size-supporting); line-height: 1.5; }
  .inspector-body { display: grid; gap: 10px; padding: var(--ui-panel-padding); }
  .identity { display: grid; gap: 4px; }
  .identity > span { width: fit-content; max-width: 100%; overflow: hidden; border-radius: var(--radius-pill); padding: 2px 7px; color: var(--color-accent-strong); background: var(--color-accent-soft); font-size: var(--font-size-label); font-weight: 800; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
  .identity h3 { font-size: 15px; line-height: 1.35; }
  code { overflow-wrap: anywhere; color: var(--color-muted); font-size: var(--font-size-micro); }
  dl { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px 10px; margin: 0; }
  dl div { min-width: 0; }
  dt { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 750; letter-spacing: 0.04em; text-transform: uppercase; }
  dd { margin: 3px 0 0; overflow-wrap: anywhere; font-size: var(--font-size-label); font-weight: 680; }
  .detail { display: grid; gap: 5px; border-top: 1px solid var(--color-border); padding-top: 11px; }
  .detail h4 { font-size: var(--font-size-label); }
  .detail p, .locations li { color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.55; }
  .locations ul { display: grid; gap: 6px; margin: 0; padding-left: 18px; }
  .locations li strong, .locations li code { display: block; }
  .artifact-actions { display: flex; flex-wrap: wrap; gap: 8px; }
  .artifact-actions :global([data-slot='button']) { width: fit-content; }
  .open-status { min-height: 16px; color: var(--color-accent-strong); font-size: var(--font-size-micro); font-weight: 700; }
  .open-status.failed { color: var(--color-danger); }
  @media (max-width: 520px) { dl { grid-template-columns: 1fr; } }
</style>
