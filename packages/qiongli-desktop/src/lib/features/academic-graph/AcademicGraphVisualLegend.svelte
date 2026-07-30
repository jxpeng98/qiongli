<script lang="ts">
  import type { AcademicGraphNodeType } from '@qiongli/app-api';
  import { Shapes } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';

  import type { AcademicGraphLayout } from './layout';
  import {
    academicGraphNodeVisual,
    academicGraphRelationFamilies,
    academicGraphRelationVisual,
    type AcademicGraphRelationFamily
  } from './visual-language';

  let {
    layout,
    hiddenNodeTypes,
    hiddenRelationFamilies,
    onToggleNodeType,
    onToggleRelationFamily
  }: {
    layout: AcademicGraphLayout;
    hiddenNodeTypes: AcademicGraphNodeType[];
    hiddenRelationFamilies: AcademicGraphRelationFamily[];
    onToggleNodeType: (nodeType: AcademicGraphNodeType) => void;
    onToggleRelationFamily: (family: AcademicGraphRelationFamily) => void;
  } = $props();

  let nodeTypes = $derived(
    [...new Set(layout.nodes.map((node) => node.nodeType))]
      .sort((left, right) => i18n.label(left).localeCompare(i18n.label(right)))
  );
  let relationCounts = $derived(new Map(
    academicGraphRelationFamilies.map((family) => [
      family,
      layout.edges.filter((edge) =>
        academicGraphRelationVisual(edge.relation).family === family).length
    ])
  ));
  let visibleRelationFamilies = $derived(
    academicGraphRelationFamilies.filter((family) => (relationCounts.get(family) ?? 0) > 0)
  );
</script>

<details class="visual-legend">
  <summary>
    <Shapes size={15} aria-hidden="true" />
    <span>{i18n.t('graph.visualLegend')}</span>
    <small>
      {i18n.t('graph.visualLegendSummary', {
        nodes: nodeTypes.length,
        relations: visibleRelationFamilies.length
      })}
    </small>
  </summary>
  <div class="legend-content">
    <section aria-labelledby="graph-node-visual-key">
      <h3 id="graph-node-visual-key">{i18n.t('graph.nodeVisuals')}</h3>
      <div class="legend-grid">
        {#each nodeTypes as nodeType (nodeType)}
          {@const visual = academicGraphNodeVisual(nodeType)}
          <button
            type="button"
            class:inactive={hiddenNodeTypes.includes(nodeType)}
            aria-pressed={!hiddenNodeTypes.includes(nodeType)}
            aria-label={i18n.t('graph.toggleNodeType', { type: i18n.label(nodeType) })}
            onclick={() => onToggleNodeType(nodeType)}
          >
            <span
              class="node-mark"
              data-shape={visual.shape}
              aria-hidden="true"
            >{visual.mark}</span>
            <span>{i18n.label(nodeType)}</span>
          </button>
        {/each}
      </div>
    </section>
    <section aria-labelledby="graph-relation-visual-key">
      <h3 id="graph-relation-visual-key">{i18n.t('graph.relationVisuals')}</h3>
      <div class="legend-grid relation-grid">
        {#each visibleRelationFamilies as family (family)}
          {@const sample = layout.edges.find((edge) =>
            academicGraphRelationVisual(edge.relation).family === family)}
          {@const visual = academicGraphRelationVisual(sample!.relation)}
          <button
            type="button"
            class:inactive={hiddenRelationFamilies.includes(family)}
            aria-pressed={!hiddenRelationFamilies.includes(family)}
            aria-label={i18n.t('graph.toggleRelationFamily', {
              family: i18n.t(`graph.relationFamily.${family}`)
            })}
            onclick={() => onToggleRelationFamily(family)}
          >
            <span
              class="relation-mark"
              data-line={visual.lineStyle}
              aria-hidden="true"
            >{visual.mark}</span>
            <span>{i18n.t(`graph.relationFamily.${family}`)}</span>
            <small>{relationCounts.get(family)}</small>
          </button>
        {/each}
      </div>
    </section>
    <p>{i18n.t('graph.visualLegendHint')}</p>
  </div>
</details>

<style>
  .visual-legend {
    position: relative;
    margin: 0 16px 12px;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    background: var(--color-surface-subtle);
  }
  summary {
    display: flex;
    min-height: 44px;
    align-items: center;
    gap: 7px;
    padding: 7px 10px;
    color: var(--color-ink);
    font-size: 11px;
    font-weight: 800;
    cursor: pointer;
    list-style: none;
    white-space: nowrap;
  }
  summary::-webkit-details-marker { display: none; }
  summary small {
    overflow: hidden;
    margin-left: auto;
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .legend-content {
    display: grid;
    gap: 12px;
    border-top: 1px solid var(--color-border);
    padding: 12px;
    background: white;
  }
  h3 { margin: 0 0 7px; color: var(--color-muted); font-size: 10px; letter-spacing: 0.04em; text-transform: uppercase; }
  .legend-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  button {
    display: inline-flex;
    min-height: 36px;
    align-items: center;
    gap: 7px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 4px 8px;
    color: var(--color-ink);
    background: white;
    font: inherit;
    font-size: 10px;
    font-weight: 700;
    white-space: nowrap;
  }
  button:hover { border-color: var(--color-accent); }
  button.inactive { color: var(--color-muted); background: #f1f5f9; opacity: 0.62; }
  button small { color: var(--color-muted); font-variant-numeric: tabular-nums; }
  .node-mark {
    display: inline-grid;
    width: 26px;
    height: 22px;
    flex: none;
    place-items: center;
    border: 1.5px solid #475569;
    background: #f8fafc;
    font-size: var(--font-size-micro);
    font-weight: 900;
  }
  .node-mark[data-shape='ellipse'] { border-radius: 50%; }
  .node-mark[data-shape='roundrectangle'] { border-radius: 6px; }
  .node-mark[data-shape='diamond'] { width: 19px; height: 19px; margin-inline: 3px; transform: rotate(45deg); }
  .node-mark[data-shape='diamond'] { color: transparent; }
  .node-mark[data-shape='hexagon'],
  .node-mark[data-shape='pentagon'],
  .node-mark[data-shape='triangle'] {
    border: 0;
    clip-path: polygon(25% 5%, 75% 5%, 100% 50%, 75% 95%, 25% 95%, 0 50%);
    color: white;
    background: #475569;
  }
  .node-mark[data-shape='pentagon'] { clip-path: polygon(50% 0, 100% 38%, 82% 100%, 18% 100%, 0 38%); }
  .node-mark[data-shape='triangle'] { clip-path: polygon(50% 0, 100% 100%, 0 100%); padding-top: 6px; }
  .node-mark[data-shape='barrel'] { border-radius: 45% / 24%; }
  .relation-mark {
    position: relative;
    display: inline-flex;
    width: 30px;
    flex: none;
    align-items: center;
    justify-content: flex-end;
    color: #475569;
    font-size: 12px;
  }
  .relation-mark::before {
    position: absolute;
    right: 7px;
    left: 0;
    border-top: 2px solid currentColor;
    content: '';
  }
  .relation-mark[data-line='dashed']::before { border-top-style: dashed; }
  .relation-mark[data-line='dotted']::before { border-top-style: dotted; }
  .legend-content > p { margin: 0; color: var(--color-muted); font-size: 10px; line-height: 1.45; }
  @media (max-width: 520px) {
    .visual-legend { margin-inline: 12px; }
    summary small { display: none; }
    .legend-grid { flex-wrap: nowrap; overflow-x: auto; padding-bottom: 4px; }
  }
</style>
