<script lang="ts">
  import { onMount } from 'svelte';
  import { Focus, TableProperties } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';

  import AcademicGraphMap from './AcademicGraphMap.svelte';
  import type { AcademicGraphRenderer } from './cytoscape-adapter';
  import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';

  let {
    layout,
    viewState,
    busy = false,
    onSelect
  }: {
    layout: AcademicGraphLayout;
    viewState: AcademicGraphViewState;
    busy?: boolean;
    onSelect: (nodeId: string) => void | Promise<void>;
  } = $props();

  let rendererHost = $state.raw<HTMLDivElement | null>(null);
  let renderer = $state.raw<AcademicGraphRenderer | null>(null);
  let rendererState = $state<'loading' | 'ready' | 'failed'>('loading');
  let selectedLabel = $derived(
    layout.nodes.find((node) => node.nodeId === viewState.selectedNodeId)?.label ?? null
  );

  onMount(() => {
    let disposed = false;
    const container = rendererHost;
    if (!container) {
      rendererState = 'failed';
      return;
    }
    void import('./cytoscape-adapter')
      .then(({ mountAcademicGraphRenderer }) => {
        if (disposed) return;
        renderer = mountAcademicGraphRenderer({
          container,
          layout,
          viewState,
          onSelect: (nodeId) => onSelect(nodeId)
        });
        rendererState = 'ready';
      })
      .catch(() => {
        if (!disposed) rendererState = 'failed';
      });
    return () => {
      disposed = true;
      renderer?.destroy();
      renderer = null;
    };
  });

  $effect(() => {
    if (!renderer) return;
    try {
      renderer.update(layout, viewState);
    } catch {
      renderer.destroy();
      renderer = null;
      rendererState = 'failed';
    }
  });

  function fitGraph(): void {
    renderer?.fit();
  }
</script>

{#if rendererState === 'failed'}
  <p class="renderer-fallback" role="status">{i18n.t('graph.rendererFailed')}</p>
  <AcademicGraphMap {layout} {viewState} {busy} {onSelect} />
{:else}
  <section
    class="surface graph-renderer"
    aria-labelledby="academic-graph-renderer-title"
    aria-busy={rendererState === 'loading' || busy}
  >
    <header>
      <div>
        <p class="eyebrow">{i18n.t('graph.rendererEyebrow')}</p>
        <h2 id="academic-graph-renderer-title">{i18n.t('graph.rendererTitle')}</h2>
      </div>
      <span class="algorithm">
        {rendererState === 'ready'
          ? i18n.t('graph.rendererReady')
          : i18n.t('graph.rendererLoading')}
      </span>
    </header>

    <div class="renderer-intro">
      <p>{i18n.t('graph.rendererDescription')}</p>
      <div class="renderer-actions">
        <button
          class="button-quiet"
          type="button"
          disabled={rendererState !== 'ready' || layout.nodes.length === 0}
          onclick={fitGraph}
        >
          <Focus size={14} aria-hidden="true" />{i18n.t('graph.fitView')}
        </button>
        <a class="button-quiet" href="#graph-nodes-title">
          <TableProperties size={14} aria-hidden="true" />{i18n.t('graph.useTable')}
        </a>
      </div>
    </div>

    <ul class="legend" aria-label={i18n.t('graph.mapLegend')}>
      {#each layout.bands as band (band.layer)}
        <li data-layer={band.layer}>
          <span aria-hidden="true"></span>
          {i18n.label(band.layer)} · {band.nodeCount}
        </li>
      {/each}
    </ul>

    <div class="renderer-shell" class:busy>
      <div class="cytoscape-host" bind:this={rendererHost} aria-hidden="true"></div>
      {#if rendererState === 'loading'}
        <p class="renderer-overlay" role="status">{i18n.t('graph.rendererLoading')}</p>
      {:else if layout.nodes.length === 0}
        <p class="renderer-overlay">{i18n.t('graph.mapEmpty')}</p>
      {/if}
    </div>

    <p class="selection" aria-live="polite">
      {selectedLabel ? i18n.t('graph.mapSelection', { label: selectedLabel }) : ''}
    </p>
  </section>
{/if}

<style>
  .graph-renderer { min-width: 0; margin-bottom: 12px; overflow: hidden; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  h2 { margin: 0; font-size: 16px; }
  .algorithm { border: 1px solid var(--color-border); border-radius: 999px; padding: 4px 8px; color: var(--color-muted); background: var(--color-surface-subtle); font-size: 10px; font-weight: 750; }
  .renderer-intro { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; padding: 12px 16px; }
  .renderer-intro > p { max-width: 820px; margin: 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .renderer-actions { display: flex; flex: 0 0 auto; gap: 6px; }
  .renderer-actions .button-quiet { min-height: 32px; padding: 5px 8px; text-decoration: none; }
  .legend { display: flex; flex-wrap: wrap; gap: 6px 12px; margin: 0; padding: 0 16px 12px; color: var(--color-muted); font-size: 10px; font-weight: 700; list-style: none; }
  .legend li { display: inline-flex; align-items: center; gap: 5px; }
  .legend li > span { width: 9px; height: 9px; border: 1px solid #94a3b8; border-radius: 3px; background: #f8fafc; }
  .legend [data-layer='portfolio'] > span { background: #ccfbf1; }
  .legend [data-layer='literature'] > span { background: #dbeafe; }
  .legend [data-layer='idea-decision'] > span { background: #ede9fe; }
  .legend [data-layer='argument'] > span { background: #ffedd5; }
  .legend [data-layer='manuscript'] > span { background: #ffe4e6; }
  .legend [data-layer='combined'] > span { background: #e2e8f0; }
  .renderer-shell { position: relative; min-height: 340px; border-top: 1px solid var(--color-border); background: #f8fafc; }
  .cytoscape-host { position: absolute; inset: 0; }
  .renderer-shell.busy .cytoscape-host { pointer-events: none; opacity: 0.72; }
  .renderer-overlay { position: absolute; inset: 0; display: grid; place-items: center; margin: 0; padding: 24px; color: var(--color-muted); background: rgb(248 250 252 / 0.84); font-size: 12px; font-weight: 700; text-align: center; }
  .selection { min-height: 16px; margin: 7px 16px 10px; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  .renderer-fallback { margin: 0 0 8px; border-left: 3px solid var(--color-warning); padding: 7px 10px; color: #854d0e; background: var(--color-warning-soft); font-size: 12px; }
  @media (max-width: 700px) {
    .renderer-intro { align-items: stretch; flex-direction: column; }
    .renderer-actions { flex-wrap: wrap; }
    .renderer-shell { min-height: 300px; }
  }
  @media (max-width: 520px) { header { align-items: flex-start; flex-direction: column; } }
</style>
