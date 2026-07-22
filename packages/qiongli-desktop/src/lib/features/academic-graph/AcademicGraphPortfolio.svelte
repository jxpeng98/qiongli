<script lang="ts">
  import type { AcademicGraphPortfolioSnapshot } from '@qiongli/app-api';
  import { AlertTriangle, ArrowRight, Boxes, ExternalLink } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';

  import { buildAcademicGraphPortfolioLayout } from './portfolio-layout';

  let {
    portfolio,
    disabled = false,
    onOpenProject
  }: {
    portfolio: AcademicGraphPortfolioSnapshot;
    disabled?: boolean;
    onOpenProject: (projectId: string) => void;
  } = $props();

  let layout = $derived(buildAcademicGraphPortfolioLayout(portfolio));
  let layoutNodes = $derived(new Map(layout.nodes.map((node) => [node.nodeId, node])));
  let nodeLabels = $derived(new Map(portfolio.nodes.map((node) => [node.nodeId, node.label])));

  function shortId(value: string): string {
    return `${value.slice(0, 8)}…${value.slice(-6)}`;
  }
</script>

<section class="portfolio" aria-labelledby="portfolio-title">
  <header class="surface">
    <div>
      <p class="eyebrow">{i18n.t('graph.portfolioEyebrow')}</p>
      <h2 id="portfolio-title">{i18n.t('graph.portfolioTitle')}</h2>
      <p>{i18n.t('graph.portfolioDescription')}</p>
    </div>
    <strong>{i18n.t('graph.portfolioIncluded', { included: portfolio.includedProjectCount, total: portfolio.projectCount })}</strong>
  </header>

  <section class="portfolio-metrics" aria-label={i18n.t('graph.portfolioSummaryAria')}>
    <article class="surface"><strong>{portfolio.includedProjectCount}</strong><span>{i18n.t('graph.portfolioProjects')}</span></article>
    <article class="surface"><strong>{portfolio.nodes.filter((node) => node.identityScope === 'global').length}</strong><span>{i18n.t('graph.portfolioSharedIdentities')}</span></article>
    <article class="surface"><strong>{portfolio.edgeCount}</strong><span>{i18n.t('graph.portfolioRelations')}</span></article>
    <article class="surface"><strong>{portfolio.skippedProjectCount}</strong><span>{i18n.t('graph.portfolioSkipped')}</span></article>
  </section>

  {#if portfolio.skippedProjectCount > 0}
    <div class="surface skipped" role="status">
      <AlertTriangle size={17} aria-hidden="true" />
      <div>
        <strong>{i18n.t('graph.portfolioSkippedTitle')}</strong>
        <ul>
          {#each portfolio.projects.filter((project) => !project.included) as project}
            <li>{project.displayName} · {i18n.label(project.health)}</li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}

  <section class="surface topology" aria-labelledby="portfolio-map-title">
    <div class="panel-heading">
      <div><p class="eyebrow">{i18n.t('graph.portfolioMapEyebrow')}</p><h3 id="portfolio-map-title">{i18n.t('graph.portfolioMapTitle')}</h3></div>
      <span>{i18n.t('graph.portfolioExactOnly')}</span>
    </div>
    {#if layout.nodes.length === 0}
      <p class="empty"><Boxes size={18} aria-hidden="true" />{i18n.t('graph.portfolioEmpty')}</p>
    {:else}
      <div class="map-scroll" aria-hidden="true">
        <svg viewBox={`0 0 ${layout.width} ${layout.height}`} role="presentation">
          {#each layout.edges as edge (edge.edgeId)}
            {@const source = layoutNodes.get(edge.sourceNodeId)}
            {@const target = layoutNodes.get(edge.targetNodeId)}
            {#if source && target}
              <line x1={source.x + 158} y1={source.y + 25} x2={target.x} y2={target.y + 25} />
            {/if}
          {/each}
          {#each layout.nodes as node (node.nodeId)}
            <g transform={`translate(${node.x} ${node.y})`} class:shared={node.identityScope === 'global'}>
              <rect width="158" height="50" rx="9" />
              <text x="79" y="22" text-anchor="middle">{node.label.slice(0, 24)}</text>
              <text class="type" x="79" y="37" text-anchor="middle">{i18n.label(node.nodeType)}</text>
            </g>
          {/each}
        </svg>
      </div>
      <p class="map-note">{i18n.t('graph.portfolioMapNote')}</p>
    {/if}
  </section>

  <div class="portfolio-grid">
    <section class="surface" aria-labelledby="portfolio-node-title">
      <div class="panel-heading"><div><p class="eyebrow">{i18n.t('graph.portfolioNodeEyebrow')}</p><h3 id="portfolio-node-title">{i18n.t('graph.portfolioNodeTitle')}</h3></div></div>
      <ol class="identity-list">
        {#each portfolio.nodes as node (node.nodeId)}
          <li>
            <div><strong>{node.label}</strong><span>{i18n.label(node.nodeType)} · {node.projectIds.length} {i18n.t('graph.portfolioProjectUnits')}</span></div>
            <code>{node.canonicalId}</code>
            <details>
              <summary>{i18n.t('graph.portfolioOccurrences', { count: node.occurrences.length })}</summary>
              <ul class="occurrences">
                {#each node.occurrences as occurrence}
                  <li>
                    <span><code>{occurrence.artifactPath}</code> · <code>{occurrence.sourceAnchor}</code></span>
                    <button type="button" disabled={disabled} onclick={() => onOpenProject(occurrence.projectId)}>
                      <ExternalLink size={12} aria-hidden="true" />{i18n.t('graph.portfolioOpenProject')}
                    </button>
                  </li>
                {/each}
              </ul>
            </details>
          </li>
        {/each}
      </ol>
    </section>

    <section class="surface" aria-labelledby="portfolio-edge-title">
      <div class="panel-heading"><div><p class="eyebrow">{i18n.t('graph.portfolioEdgeEyebrow')}</p><h3 id="portfolio-edge-title">{i18n.t('graph.portfolioEdgeTitle')}</h3></div></div>
      {#if portfolio.edges.length === 0}
        <p class="empty">{i18n.t('graph.portfolioNoRelations')}</p>
      {:else}
        <ol class="relation-list">
          {#each portfolio.edges as edge (edge.edgeId)}
            <li>
              <p class="statement"><strong>{nodeLabels.get(edge.sourceNodeId) ?? shortId(edge.sourceNodeId)}</strong><ArrowRight size={13} aria-hidden="true" /><span>{i18n.label(edge.relation)}</span><ArrowRight size={13} aria-hidden="true" /><strong>{nodeLabels.get(edge.targetNodeId) ?? shortId(edge.targetNodeId)}</strong></p>
              <p>{edge.rationale}</p>
              {#if edge.sharedCanonicalId}<code>{edge.sharedCanonicalId}</code>{/if}
              <details><summary>{i18n.t('graph.portfolioProvenance', { count: edge.origins.length })}</summary><ul>{#each edge.origins as origin}<li><code>{origin.artifactPath}</code> · <code>{origin.sourceAnchor}</code></li>{/each}</ul></details>
              <p class="limit">{edge.evidenceLimit}</p>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  </div>
</section>

<style>
  .portfolio { display: grid; gap: 12px; min-width: 0; }
  header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; padding: 16px; }
  header h2, .panel-heading h3 { margin: 0; font-size: 17px; }
  header p:last-child { max-width: 760px; margin: 5px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.5; }
  header > strong { flex: 0 0 auto; border-radius: 999px; padding: 5px 9px; color: #075985; background: #e0f2fe; font-size: 10px; }
  .portfolio-metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
  .portfolio-metrics article { display: grid; gap: 3px; padding: 13px; }
  .portfolio-metrics strong { font-size: 21px; }.portfolio-metrics span { color: var(--color-muted); font-size: 10px; font-weight: 750; }
  .skipped { display: flex; gap: 9px; padding: 12px 14px; color: #92400e; background: #fffbeb; }
  .skipped strong { font-size: 11px; }.skipped ul { margin: 4px 0 0; padding-left: 17px; font-size: 10px; }
  .panel-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; border-bottom: 1px solid var(--color-border); padding: 13px 15px; }
  .panel-heading h3 { font-size: 15px; }.panel-heading > span { color: var(--color-muted); font-size: 9px; font-weight: 750; }
  .topology { min-width: 0; overflow: hidden; }
  .map-scroll { max-height: 390px; overflow: auto; padding: 12px; background: linear-gradient(#f8fafc 1px, transparent 1px), linear-gradient(90deg, #f8fafc 1px, transparent 1px); background-size: 20px 20px; }
  svg { display: block; width: 100%; min-width: 620px; max-height: 360px; }
  line { stroke: #94a3b8; stroke-width: 2; }
  g rect { fill: #fff; stroke: #0f766e; stroke-width: 2; }g.shared rect { fill: #eff6ff; stroke: #2563eb; }
  text { fill: #172033; font: 650 10px system-ui; }.type { fill: #64748b; font-size: 8px; text-transform: uppercase; }
  .map-note { margin: 0; border-top: 1px solid var(--color-border); padding: 9px 15px; color: var(--color-muted); font-size: 10px; }
  .portfolio-grid { display: grid; grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr); gap: 12px; min-width: 0; }
  .portfolio-grid > section { min-width: 0; overflow: hidden; }
  .identity-list, .relation-list { display: grid; gap: 8px; margin: 0; padding: 12px; list-style: none; }
  .identity-list > li, .relation-list > li { min-width: 0; border: 1px solid var(--color-border); border-radius: 9px; padding: 10px; }
  .identity-list > li > div { display: flex; justify-content: space-between; gap: 8px; }.identity-list strong { font-size: 11px; }.identity-list span { color: var(--color-muted); font-size: 9px; }
  code { color: var(--color-muted); font-size: 9px; overflow-wrap: anywhere; }
  details { margin-top: 7px; }summary { color: var(--color-accent-strong); font-size: 10px; font-weight: 750; cursor: pointer; }
  .occurrences, .relation-list details ul { display: grid; gap: 5px; margin: 6px 0 0; padding: 0; list-style: none; }
  .occurrences li { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; }
  button { display: inline-flex; align-items: center; gap: 4px; border: 0; padding: 0; color: var(--color-accent-strong); background: transparent; font: inherit; font-size: 9px; font-weight: 750; cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: 0.55; }
  .statement { display: flex; flex-wrap: wrap; align-items: center; gap: 5px; margin: 0; font-size: 10px; }.statement span { color: #1d4ed8; font-weight: 750; }
  .relation-list p:not(.statement) { margin: 6px 0 0; color: var(--color-muted); font-size: 10px; line-height: 1.5; }
  .relation-list .limit { border-top: 1px solid var(--color-border); padding-top: 6px; }
  .empty { display: flex; align-items: center; gap: 7px; margin: 0; padding: 16px; color: var(--color-muted); font-size: 11px; }
  @media (max-width: 900px) { .portfolio-grid { grid-template-columns: 1fr; } }
  @media (max-width: 700px) { .portfolio-metrics { grid-template-columns: 1fr 1fr; } }
  @media (max-width: 520px) { header { flex-direction: column; }.portfolio-metrics { grid-template-columns: 1fr; }.occurrences li { flex-direction: column; } }
</style>
