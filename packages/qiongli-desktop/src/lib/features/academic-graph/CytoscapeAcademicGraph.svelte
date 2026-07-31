<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Focus,
    LocateFixed,
    TableProperties,
    ZoomIn,
    ZoomOut
  } from '@lucide/svelte';
  import type { StatusCode } from '@qiongli/app-api';
  import type { AcademicGraphNodeType } from '@qiongli/app-api';

  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/shared/ui';

  import AcademicGraphMinimap from './AcademicGraphMinimap.svelte';
  import AcademicGraphMap from './AcademicGraphMap.svelte';
  import AcademicGraphVisualLegend from './AcademicGraphVisualLegend.svelte';
  import type {
    AcademicGraphRenderer,
    AcademicGraphViewport
  } from './cytoscape-adapter';
  import {
    ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM,
    type AcademicGraphLayout,
    type AcademicGraphViewState
  } from './layout';
  import type { AcademicGraphRelationFamily } from './visual-language';

  let {
    layout,
    viewState,
    busy = false,
    onSelect,
    onSelectEdge,
    compact = false,
    status = 'ready',
    statusLabel,
    resultSummary,
    tableTarget = '#graph-nodes-title'
  }: {
    layout: AcademicGraphLayout;
    viewState: AcademicGraphViewState;
    busy?: boolean;
    onSelect: (nodeId: string) => void | Promise<void>;
    onSelectEdge: (edgeId: string) => void | Promise<void>;
    compact?: boolean;
    status?: StatusCode;
    statusLabel?: string;
    resultSummary?: string;
    tableTarget?: string;
  } = $props();

  let rendererHost = $state.raw<HTMLDivElement | null>(null);
  let renderer = $state.raw<AcademicGraphRenderer | null>(null);
  let rendererState = $state<'loading' | 'ready' | 'failed'>('loading');
  let viewport = $state<AcademicGraphViewport | null>(null);
  let hiddenNodeTypes = $state<AcademicGraphNodeType[]>([]);
  let hiddenRelationFamilies = $state<AcademicGraphRelationFamily[]>([]);
  let rendererViewState = $derived({
    ...viewState,
    hiddenNodeTypes,
    hiddenRelationFamilies
  });
  let selectedLabel = $derived(
    layout.nodes.find((node) => node.nodeId === viewState.selectedNodeId)?.label ?? null
  );
  let selectedEdgeLabel = $derived(
    layout.edges.find((edge) => edge.edgeId === viewState.selectedEdgeId)?.relation ?? null
  );
  let layoutLabel = $derived(
    layout.algorithm === ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM
      ? i18n.t('graph.layoutTopologyV2')
      : layout.fallbackReason
        ? i18n.t('graph.layoutLayeredFallback')
        : i18n.t('graph.mapAlgorithm')
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
          viewState: rendererViewState,
          onSelect: (nodeId) => onSelect(nodeId),
          onSelectEdge: (edgeId) => onSelectEdge(edgeId),
          onViewportChange: (nextViewport) => {
            viewport = nextViewport;
          }
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
      renderer.update(layout, rendererViewState);
    } catch {
      renderer.destroy();
      renderer = null;
      rendererState = 'failed';
    }
  });

  function fitGraph(): void {
    renderer?.fit();
  }

  function fitSelection(): void {
    renderer?.fitSelection();
  }

  function zoomIn(): void {
    renderer?.zoomIn();
  }

  function zoomOut(): void {
    renderer?.zoomOut();
  }

  function toggleNodeType(nodeType: AcademicGraphNodeType): void {
    hiddenNodeTypes = hiddenNodeTypes.includes(nodeType)
      ? hiddenNodeTypes.filter((value) => value !== nodeType)
      : [...hiddenNodeTypes, nodeType].sort();
  }

  function toggleRelationFamily(family: AcademicGraphRelationFamily): void {
    hiddenRelationFamilies = hiddenRelationFamilies.includes(family)
      ? hiddenRelationFamilies.filter((value) => value !== family)
      : [...hiddenRelationFamilies, family].sort();
  }

  function handleCanvasKeyboard(event: KeyboardEvent): void {
    if (rendererState !== 'ready') return;
    if (event.key === '+' || event.key === '=') {
      event.preventDefault();
      zoomIn();
      return;
    }
    if (event.key === '-' || event.key === '_') {
      event.preventDefault();
      zoomOut();
      return;
    }
    if (event.key === '0') {
      event.preventDefault();
      fitGraph();
      return;
    }
    if (event.key.toLowerCase() === 'f') {
      event.preventDefault();
      if (viewState.selectedNodeId || viewState.selectedEdgeId) fitSelection();
      else fitGraph();
    }
  }

</script>

{#if rendererState === 'failed'}
  <p class="renderer-fallback" role="status">{i18n.t('graph.rendererFailed')}</p>
  <AcademicGraphMap
    {layout}
    viewState={rendererViewState}
    {busy}
    {onSelect}
    {onSelectEdge}
  />
{:else}
  <section
    class="surface graph-renderer"
    class:workspace={compact}
    aria-labelledby="academic-graph-renderer-title"
    aria-busy={rendererState === 'loading' || busy}
  >
    <header>
      <div>
        <p class="eyebrow">{i18n.t('graph.rendererEyebrow')}</p>
        <h2 id="academic-graph-renderer-title">{i18n.t('graph.rendererTitle')}</h2>
      </div>
      <div class="header-status">
        {#if statusLabel}<StatusBadge {status} label={statusLabel} />{/if}
        <span
          class="algorithm"
          class:fallback={layout.fallbackReason !== null && layout.fallbackReason !== undefined}
        >
          {rendererState === 'ready'
            ? layoutLabel
            : i18n.t('graph.rendererLoading')}
        </span>
      </div>
    </header>

    <div class="renderer-intro">
      <p>{compact && resultSummary ? resultSummary : i18n.t('graph.rendererDescription')}</p>
      <div class="renderer-actions">
        <button
          class="button-quiet icon-action"
          type="button"
          aria-label={i18n.t('graph.zoomOut')}
          title={i18n.t('graph.zoomOut')}
          disabled={rendererState !== 'ready' || layout.nodes.length === 0}
          onclick={zoomOut}
        >
          <ZoomOut size={15} aria-hidden="true" />
        </button>
        <span class="zoom-value" aria-live="polite">
          {Math.round((viewport?.zoom ?? 1) * 100)}%
        </span>
        <button
          class="button-quiet icon-action"
          type="button"
          aria-label={i18n.t('graph.zoomIn')}
          title={i18n.t('graph.zoomIn')}
          disabled={rendererState !== 'ready' || layout.nodes.length === 0}
          onclick={zoomIn}
        >
          <ZoomIn size={15} aria-hidden="true" />
        </button>
        <button
          class="button-quiet icon-action"
          type="button"
          aria-label={i18n.t('graph.fitSelection')}
          title={i18n.t('graph.fitSelection')}
          disabled={rendererState !== 'ready'
            || (!viewState.selectedNodeId && !viewState.selectedEdgeId)}
          onclick={fitSelection}
        >
          <LocateFixed size={15} aria-hidden="true" />
        </button>
        <button
          class="button-quiet"
          type="button"
          disabled={rendererState !== 'ready' || layout.nodes.length === 0}
          onclick={fitGraph}
        >
          <Focus size={14} aria-hidden="true" />{i18n.t('graph.fitView')}
        </button>
        <a class="button-quiet" href={tableTarget}>
          <TableProperties size={14} aria-hidden="true" />{i18n.t('graph.useTable')}
        </a>
      </div>
    </div>

    <ul class="layer-legend" aria-label={i18n.t('graph.mapLegend')}>
      {#each layout.bands as band (band.layer)}
        <li data-layer={band.layer}>
          <span aria-hidden="true"></span>
          {i18n.label(band.layer)} · {band.nodeCount}
        </li>
      {/each}
    </ul>
    <AcademicGraphVisualLegend
      {layout}
      {hiddenNodeTypes}
      {hiddenRelationFamilies}
      onToggleNodeType={toggleNodeType}
      onToggleRelationFamily={toggleRelationFamily}
    />

    <button
      type="button"
      class="renderer-shell"
      class:busy
      aria-label={i18n.t('graph.canvasRegion')}
      aria-describedby="academic-graph-canvas-help"
      onkeydown={handleCanvasKeyboard}
    >
      <div class="cytoscape-host" bind:this={rendererHost} aria-hidden="true"></div>
      {#if rendererState === 'ready' && layout.nodes.length > 0}
        <AcademicGraphMinimap {layout} {viewport} />
      {/if}
      {#if rendererState === 'loading'}
        <p class="renderer-overlay" role="status">{i18n.t('graph.rendererLoading')}</p>
      {:else if layout.nodes.length === 0}
        <p class="renderer-overlay">{i18n.t('graph.mapEmpty')}</p>
      {/if}
    </button>
    <p id="academic-graph-canvas-help" class="sr-only">
      {i18n.t('graph.canvasKeyboardHelp')}
    </p>

    <p class="selection" aria-live="polite">
      {selectedEdgeLabel
        ? i18n.t('graph.edgeSelection', { relation: i18n.label(selectedEdgeLabel) })
        : selectedLabel
          ? i18n.t('graph.mapSelection', { label: selectedLabel })
          : ''}
    </p>
  </section>
{/if}

<style>
  .graph-renderer { min-width: 0; margin-bottom: 12px; overflow: hidden; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  h2 { margin: 0; font-size: 16px; }
  .algorithm { border: 1px solid var(--color-border); border-radius: 999px; padding: 4px 8px; color: var(--color-muted); background: var(--color-surface-subtle); font-size: 10px; font-weight: 750; white-space: nowrap; }
  .algorithm.fallback { border-color: var(--color-warning-border); color: var(--color-warning-strong); background: var(--color-warning-soft); }
  .header-status { display: flex; flex: none; align-items: center; gap: 7px; }
  .renderer-intro { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; padding: 12px 16px; }
  .renderer-intro > p { max-width: 820px; margin: 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .renderer-actions { display: flex; flex: 0 0 auto; gap: 6px; }
  .renderer-actions .button-quiet { min-height: 44px; padding: 5px 8px; text-decoration: none; }
  .renderer-actions .icon-action { width: 44px; justify-content: center; padding: 0; }
  .zoom-value { display: inline-grid; min-width: 42px; place-items: center; color: var(--color-muted); font-size: 10px; font-variant-numeric: tabular-nums; font-weight: 750; white-space: nowrap; }
  .layer-legend { display: flex; flex-wrap: wrap; gap: 6px 12px; margin: 0; padding: 0 16px 9px; color: var(--color-muted); font-size: 10px; font-weight: 700; list-style: none; }
  .layer-legend li { display: inline-flex; align-items: center; gap: 5px; white-space: nowrap; }
  .layer-legend li > span { width: 9px; height: 9px; border: 1px solid var(--color-border-strong); border-radius: 3px; background: var(--color-graph-canvas); }
  .layer-legend [data-layer='portfolio'] > span { background: var(--color-layer-portfolio); }
  .layer-legend [data-layer='literature'] > span { background: var(--color-layer-literature); }
  .layer-legend [data-layer='idea-decision'] > span { background: var(--color-layer-idea); }
  .layer-legend [data-layer='argument'] > span { background: var(--color-layer-argument); }
  .layer-legend [data-layer='manuscript'] > span { background: var(--color-layer-manuscript); }
  .layer-legend [data-layer='combined'] > span { background: var(--color-layer-combined); }
  .renderer-shell { position: relative; display: block; width: 100%; min-height: 340px; border: 0; border-top: 1px solid var(--color-border); padding: 0; color: inherit; background: var(--color-graph-canvas); font: inherit; text-align: initial; }
  .renderer-shell:focus-visible { outline: 3px solid color-mix(in srgb, var(--color-focus) 42%, transparent); outline-offset: -3px; }
  .workspace .renderer-shell { min-height: clamp(360px, calc(100vh - 330px), 620px); }
  .workspace .renderer-intro { align-items: center; padding-block: 8px; }
  .workspace .renderer-intro > p { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .cytoscape-host { position: absolute; inset: 0; }
  .renderer-shell.busy .cytoscape-host { pointer-events: none; opacity: 0.72; }
  .renderer-overlay { position: absolute; inset: 0; display: grid; place-items: center; margin: 0; padding: 24px; color: var(--color-muted); background: var(--color-graph-overlay); font-size: 12px; font-weight: 700; text-align: center; }
  .selection { min-height: 16px; margin: 7px 16px 10px; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  .renderer-fallback { margin: 0 0 8px; border-left: 3px solid var(--color-warning); padding: 7px 10px; color: var(--color-warning-strong); background: var(--color-warning-soft); font-size: 12px; }
  @media (max-width: 700px) {
    .renderer-intro { align-items: stretch; flex-direction: column; }
    .renderer-actions { flex-wrap: wrap; }
    .renderer-shell, .workspace .renderer-shell { min-height: 320px; }
  }
  @media (max-width: 520px) {
    header { align-items: flex-start; flex-direction: column; }
    .header-status { width: 100%; justify-content: space-between; }
    .workspace .renderer-intro > p { white-space: normal; }
  }
</style>
