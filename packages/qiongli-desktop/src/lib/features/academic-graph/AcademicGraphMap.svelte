<script lang="ts">
  import {
    ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM,
    type AcademicGraphLayout,
    type AcademicGraphViewState
  } from './layout';
  import {
    academicGraphNodeVisual,
    academicGraphRelationVisual
  } from './visual-language';

  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  let {
    layout,
    viewState,
    busy = false,
    onSelect,
    onSelectEdge
  }: {
    layout: AcademicGraphLayout;
    viewState: AcademicGraphViewState;
    busy?: boolean;
    onSelect: (nodeId: string) => void | Promise<void>;
    onSelectEdge: (edgeId: string) => void | Promise<void>;
  } = $props();

  let synchronized = $derived(layout.layoutKey === viewState.layoutKey);
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
  let mapTitle = $derived(
    layout.algorithm === ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM
      ? i18n.t('graph.mapTopologyTitle')
      : i18n.t('graph.mapTitle')
  );
  let mapDescription = $derived(
    layout.algorithm === ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM
      ? i18n.t('graph.mapTopologyDescription')
      : i18n.t('graph.mapDescription')
  );
  let nodeTypeById = $derived(new Map(
    layout.nodes.map((node) => [node.nodeId, node.nodeType])
  ));

  function isNodeHidden(nodeId: string): boolean {
    const nodeType = nodeTypeById.get(nodeId);
    return nodeType ? viewState.hiddenNodeTypes.includes(nodeType) : false;
  }

  function edgeHidden(edge: AcademicGraphLayout['edges'][number]): boolean {
    const relationVisual = academicGraphRelationVisual(edge.relation);
    return viewState.hiddenRelationFamilies.includes(relationVisual.family)
      || isNodeHidden(edge.sourceNodeId)
      || isNodeHidden(edge.targetNodeId);
  }

  function edgeLabel(edge: AcademicGraphLayout['edges'][number]): string {
    const source = layout.nodes.find((node) => node.nodeId === edge.sourceNodeId)?.label
      ?? edge.sourceNodeId;
    const target = layout.nodes.find((node) => node.nodeId === edge.targetNodeId)?.label
      ?? edge.targetNodeId;
    return i18n.t('graph.mapEdgeAria', {
      source,
      relation: i18n.label(edge.relation),
      target
    });
  }

  function selectEdge(edgeId: string): void {
    if (!busy) void onSelectEdge(edgeId);
  }

  function handleEdgeKeydown(event: KeyboardEvent, edgeId: string): void {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    selectEdge(edgeId);
  }
</script>

<Card.Root class="graph-map" role="region" aria-labelledby="academic-graph-map-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('graph.mapEyebrow')}</p>
      <h2 id="academic-graph-map-title">{mapTitle}</h2>
    </div>
    <span class="algorithm">{layoutLabel}</span>
  </header>

  <p class="description">{mapDescription}</p>

  {#if layout.nodes.length === 0}
    <p class="empty-copy">{i18n.t('graph.mapEmpty')}</p>
  {:else}
    <ul class="legend" aria-label={i18n.t('graph.mapLegend')}>
      {#each layout.bands as band (band.layer)}
        <li data-layer={band.layer}>
          <span aria-hidden="true"></span>
          {i18n.label(band.layer)} · {band.nodeCount}
        </li>
      {/each}
    </ul>

    <div
      class="map-scroll"
      role="region"
      aria-label={i18n.t('graph.mapRegion')}
    >
      <div
        class="map-canvas"
        style={`width: ${layout.width}px; height: ${layout.height}px;`}
        data-layout-algorithm={layout.algorithm}
      >
        {#each layout.bands as band (band.layer)}
          <div
            class="band"
            data-layer={band.layer}
            style={`left: ${band.x}px; width: ${band.width}px; height: ${layout.height - 16}px;`}
            aria-hidden="true"
          >
            <strong>{i18n.label(band.layer)}</strong>
          </div>
        {/each}

        <svg
          class="edges"
          viewBox={`0 0 ${layout.width} ${layout.height}`}
          width={layout.width}
          height={layout.height}
          aria-label={i18n.t('graph.mapRelationsAria')}
        >
          <defs>
            <marker
              id="academic-graph-arrow-triangle"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="5"
              markerHeight="5"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z"></path>
            </marker>
            <marker
              id="academic-graph-arrow-tee"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path class="marker-tee" d="M 8 1 L 8 9"></path>
            </marker>
            <marker
              id="academic-graph-arrow-diamond"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <polygon points="1,5 5,1 9,5 5,9"></polygon>
            </marker>
            <marker
              id="academic-graph-arrow-square"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <rect x="2" y="2" width="7" height="7"></rect>
            </marker>
          </defs>
          {#each layout.edges as edge (edge.edgeId)}
            {@const relationVisual = academicGraphRelationVisual(edge.relation)}
            <g
              class="edge"
              class:hidden={edgeHidden(edge)}
              class:selected={edge.edgeId === viewState.selectedEdgeId}
              class:risk-medium={edge.riskSeverity === 'medium'}
              class:risk-high={edge.riskSeverity === 'high'}
              data-relation-family={relationVisual.family}
              role="button"
              tabindex={busy || edgeHidden(edge) ? -1 : 0}
              aria-label={edgeLabel(edge)}
              aria-pressed={edge.edgeId === viewState.selectedEdgeId}
              aria-disabled={busy}
              onclick={() => selectEdge(edge.edgeId)}
              onkeydown={(event) => handleEdgeKeydown(event, edge.edgeId)}
            >
              <line
                class="edge-visual"
                x1={edge.x1}
                y1={edge.y1}
                x2={edge.x2}
                y2={edge.y2}
                marker-end={`url(#academic-graph-arrow-${relationVisual.arrowShape})`}
                aria-hidden="true"
              ></line>
              <line
                class="edge-hit-target"
                x1={edge.x1}
                y1={edge.y1}
                x2={edge.x2}
                y2={edge.y2}
                aria-hidden="true"
              ></line>
            </g>
          {/each}
        </svg>

        {#each layout.nodes as node (node.nodeId)}
          {@const nodeVisual = academicGraphNodeVisual(node.nodeType)}
          <Button
            class={[
              'map-node',
              viewState.hiddenNodeTypes.includes(node.nodeType) ? 'hidden' : '',
              synchronized && node.nodeId === viewState.selectedNodeId ? 'selected' : '',
              synchronized && node.nodeId === viewState.focusNodeId ? 'focused' : '',
              node.riskSeverity === 'medium' ? 'risk-medium' : '',
              node.riskSeverity === 'high' ? 'risk-high' : ''
            ].filter(Boolean).join(' ')}
            variant="ghost"
            data-layer={node.layer}
            data-shape={nodeVisual.shape}
            style={`left: ${node.x}px; top: ${node.y}px; width: ${node.width}px; height: ${node.height}px;`}
            title={node.canonicalId}
            aria-label={i18n.t('graph.mapNodeAria', {
              label: node.label,
              type: i18n.label(node.nodeType),
              layer: i18n.label(node.layer),
              risk: node.riskSeverity
                ? i18n.t('graph.mapNodeRisk', {
                    severity: i18n.t(`graph.riskSeverity.${node.riskSeverity}`)
                  })
                : ''
            })}
            aria-pressed={synchronized && node.nodeId === viewState.selectedNodeId}
            disabled={busy}
            onclick={() => onSelect(node.nodeId)}
          >
            <b aria-hidden="true">{nodeVisual.mark}</b>
            <span>{i18n.label(node.nodeType)}</span>
            <strong>{node.label}</strong>
          </Button>
        {/each}
      </div>
    </div>

    <p class="selection" aria-live="polite">
      {selectedEdgeLabel
        ? i18n.t('graph.edgeSelection', { relation: i18n.label(selectedEdgeLabel) })
        : selectedLabel ? i18n.t('graph.mapSelection', { label: selectedLabel }) : ''}
    </p>
  {/if}
</Card.Root>

<style>
  :global(.graph-map) { min-width: 0; margin-bottom: 12px; overflow: hidden; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  h2 { margin: 0; font-size: 16px; }
  .algorithm { border: 1px solid var(--color-border); border-radius: 999px; padding: 4px 8px; color: var(--color-muted); background: var(--color-surface-subtle); font-size: 10px; font-weight: 750; white-space: nowrap; }
  .description { max-width: 850px; margin: 12px 16px; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .legend { display: flex; flex-wrap: wrap; gap: 6px 12px; margin: 0; padding: 0 16px 12px; color: var(--color-muted); font-size: 10px; font-weight: 700; list-style: none; }
  .legend li { display: inline-flex; align-items: center; gap: 5px; }
  .legend li > span { width: 9px; height: 9px; border: 1px solid var(--color-border-strong); border-radius: 3px; background: var(--color-graph-canvas); }
  .map-scroll { overflow: auto; border-top: 1px solid var(--color-border); background: var(--color-graph-canvas); overscroll-behavior: contain; }
  .map-canvas { position: relative; min-width: 100%; }
  .band { position: absolute; top: 8px; box-sizing: border-box; border: 1px dashed var(--color-border); border-radius: 12px; background: var(--glass-control); }
  .band strong { display: block; overflow: hidden; padding: 13px 12px 8px; color: var(--color-muted); font-size: 10px; letter-spacing: 0.04em; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
  .edges { position: absolute; inset: 0; overflow: visible; pointer-events: none; }
  .edge { pointer-events: stroke; cursor: pointer; }
  .edge.hidden { display: none; }
  .edge-visual { stroke: #94a3b8; stroke-width: 1.5; stroke-opacity: 0.72; }
  .edge-hit-target { stroke: transparent; stroke-width: 16; }
  .edge[data-relation-family='challenge'] .edge-visual { stroke: #b91c1c; stroke-width: 2.4; stroke-dasharray: 7 5; }
  .edge[data-relation-family='provenance'] .edge-visual { stroke: #475569; stroke-dasharray: 2 5; }
  .edge[data-relation-family='structure'] .edge-visual { stroke: #334155; stroke-width: 2; }
  .edge[data-relation-family='development'] .edge-visual { stroke: #7c3aed; stroke-dasharray: 8 4; }
  .edge.risk-medium .edge-visual { stroke: #f97316; stroke-width: 3; stroke-dasharray: 6 4; }
  .edge.risk-high .edge-visual { stroke: #dc2626; stroke-width: 4; }
  .edge.selected .edge-visual { stroke: var(--color-accent-strong); stroke-width: 4; stroke-opacity: 1; }
  .edge:focus-visible .edge-hit-target { stroke: rgb(3 105 161 / 0.34); stroke-width: 9; }
  .edges marker path,
  .edges marker polygon,
  .edges marker rect { fill: #64748b; }
  .edges marker .marker-tee {
    fill: none;
    stroke: #64748b;
    stroke-width: 2.5;
  }
  :global(.map-node) { position: absolute; display: grid; height: auto; grid-template-columns: auto minmax(0, 1fr); align-content: center; gap: 1px 6px; box-sizing: border-box; overflow: hidden; border: 1px solid var(--color-border-strong); border-left-width: 4px; border-radius: 9px; padding: 6px 8px; color: var(--color-ink); background: var(--color-control); box-shadow: var(--shadow-card); font: inherit; text-align: left; white-space: normal; cursor: pointer; }
  :global(.map-node.hidden) { display: none; }
  :global(.map-node > b) { grid-row: 1 / 3; align-self: center; color: var(--color-muted); font-size: var(--font-size-label); letter-spacing: 0.02em; }
  :global(.map-node > span) { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 800; letter-spacing: 0.04em; text-transform: uppercase; }
  :global(.map-node > strong) { overflow: hidden; font-size: 11px; line-height: 1.25; text-overflow: ellipsis; white-space: nowrap; }
  :global(.map-node:hover:not(:disabled)) { border-color: var(--color-accent); box-shadow: 0 4px 10px rgb(3 105 161 / 0.14); }
  :global(.map-node:focus-visible) { z-index: 2; outline: 3px solid rgb(3 105 161 / 0.34); outline-offset: 2px; }
  :global(.map-node.risk-medium) { border-color: var(--color-warning); border-style: dashed; background: var(--color-warning-soft); }
  :global(.map-node.risk-high) { border-color: var(--color-danger); border-width: 3px; border-left-width: 5px; background: var(--color-danger-soft); }
  :global(.map-node.selected), :global(.map-node.focused) { border-color: var(--color-accent-strong); color: var(--color-accent-strong); background: var(--color-accent-soft); }
  :global(.map-node:disabled) { cursor: progress; opacity: 0.68; }
  :global(.map-node[data-shape='ellipse']) { border-radius: 999px; padding-inline: 14px; }
  :global(.map-node[data-shape='diamond']) { border-radius: 2px 11px; }
  :global(.map-node[data-shape='hexagon']) { clip-path: polygon(7% 0, 93% 0, 100% 50%, 93% 100%, 7% 100%, 0 50%); padding-inline: 14px; }
  :global(.map-node[data-shape='triangle']) { clip-path: polygon(50% 0, 100% 100%, 0 100%); padding: 17px 21px 5px; }
  :global(.map-node[data-shape='pentagon']) { clip-path: polygon(50% 0, 100% 32%, 82% 100%, 18% 100%, 0 32%); padding: 12px 18px 5px; }
  :global(.map-node[data-shape='barrel']) { border-radius: 42% / 24%; padding-inline: 17px; }
  [data-layer='portfolio'] { border-left-color: #0f766e; }
  [data-layer='literature'] { border-left-color: #2563eb; }
  [data-layer='idea-decision'] { border-left-color: #7c3aed; }
  [data-layer='argument'] { border-left-color: #c2410c; }
  [data-layer='manuscript'] { border-left-color: #be123c; }
  [data-layer='combined'] { border-left-color: #475569; }
  .legend [data-layer='portfolio'] > span { background: var(--color-layer-portfolio); }
  .legend [data-layer='literature'] > span { background: var(--color-layer-literature); }
  .legend [data-layer='idea-decision'] > span { background: var(--color-layer-idea); }
  .legend [data-layer='argument'] > span { background: var(--color-layer-argument); }
  .legend [data-layer='manuscript'] > span { background: var(--color-layer-manuscript); }
  .legend [data-layer='combined'] > span { background: var(--color-layer-combined); }
  .selection { min-height: 16px; margin: 7px 16px 10px; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  .empty-copy { margin: 0; padding: 24px 16px; color: var(--color-muted); }
  @media (max-width: 520px) { header { align-items: flex-start; flex-direction: column; } }
</style>
