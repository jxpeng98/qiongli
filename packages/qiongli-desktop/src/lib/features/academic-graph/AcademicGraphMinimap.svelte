<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';

  import type { AcademicGraphViewport } from './cytoscape-adapter';
  import type { AcademicGraphLayout } from './layout';

  let {
    layout,
    viewport
  }: {
    layout: AcademicGraphLayout;
    viewport: AcademicGraphViewport | null;
  } = $props();

  let visibleExtent = $derived.by(() => {
    const extent = viewport?.extent;
    if (!extent) return { x: 0, y: 0, width: layout.width, height: layout.height };
    const x1 = Math.max(0, Math.min(layout.width, extent.x1));
    const y1 = Math.max(0, Math.min(layout.height, extent.y1));
    const x2 = Math.max(x1, Math.min(layout.width, extent.x2));
    const y2 = Math.max(y1, Math.min(layout.height, extent.y2));
    return {
      x: x1,
      y: y1,
      width: Math.max(8, x2 - x1),
      height: Math.max(8, y2 - y1)
    };
  });
</script>

<div
  class="minimap"
  role="img"
  aria-label={i18n.t('graph.minimapAria', {
    zoom: Math.round((viewport?.zoom ?? 1) * 100)
  })}
>
  <svg viewBox={`0 0 ${layout.width} ${layout.height}`} preserveAspectRatio="xMidYMid meet">
    {#each layout.bands as band (band.layer)}
      <rect
        class="band"
        x={band.x}
        y="8"
        width={band.width}
        height={Math.max(8, layout.height - 16)}
      ></rect>
    {/each}
    {#each layout.edges as edge (edge.edgeId)}
      <line x1={edge.x1} y1={edge.y1} x2={edge.x2} y2={edge.y2}></line>
    {/each}
    {#each layout.nodes as node (node.nodeId)}
      <rect
        class:spine={node.spine === true}
        x={node.x}
        y={node.y}
        width={node.width}
        height={node.height}
        rx="8"
      ></rect>
    {/each}
    <rect
      class="viewport"
      x={visibleExtent.x}
      y={visibleExtent.y}
      width={visibleExtent.width}
      height={visibleExtent.height}
      rx="7"
    ></rect>
  </svg>
</div>

<style>
  .minimap {
    position: absolute;
    right: 12px;
    bottom: 12px;
    z-index: 2;
    width: 164px;
    height: 104px;
    overflow: hidden;
    border: 1px solid rgb(100 116 139 / 0.64);
    border-radius: 9px;
    background: var(--color-surface);
    box-shadow: 0 4px 16px rgb(15 23 42 / 0.14);
    pointer-events: none;
  }
  svg { display: block; width: 100%; height: 100%; }
  .band { fill: rgb(226 232 240 / 0.4); stroke: #cbd5e1; stroke-width: 1; }
  line { stroke: #94a3b8; stroke-width: 1.5; stroke-opacity: 0.62; vector-effect: non-scaling-stroke; }
  rect:not(.band, .viewport) { fill: #fff; stroke: #64748b; stroke-width: 1.5; vector-effect: non-scaling-stroke; }
  rect.spine { fill: #ccfbf1; stroke: #0f766e; stroke-width: 2.5; }
  .viewport { fill: rgb(14 165 233 / 0.08); stroke: #0369a1; stroke-width: 3; vector-effect: non-scaling-stroke; }
  @media (max-width: 520px) {
    .minimap { width: 118px; height: 76px; }
  }
</style>
