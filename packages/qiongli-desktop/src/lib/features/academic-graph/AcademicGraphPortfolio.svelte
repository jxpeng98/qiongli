<script lang="ts">
  import type { AcademicGraphPortfolioSnapshot } from '@qiongli/app-api';
  import { AlertTriangle, ArrowRight, Boxes, ExternalLink } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { ContentGrid, MetricCard, MetricGrid, SectionHeader, StatePanel } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  import { buildAcademicGraphPortfolioLayout } from './portfolio-layout';

  let {
    portfolio,
    disabled = false,
    showDetails = true,
    onOpenProject
  }: {
    portfolio: AcademicGraphPortfolioSnapshot;
    disabled?: boolean;
    showDetails?: boolean;
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
  <Card.Root class="portfolio-heading">
    <SectionHeader eyebrow={i18n.t('graph.portfolioEyebrow')} title={i18n.t('graph.portfolioTitle')} titleId="portfolio-title" description={i18n.t('graph.portfolioDescription')}>
      {#snippet metadata()}<span class="scope-badge">{i18n.t('graph.portfolioIncluded', { included: portfolio.includedProjectCount, total: portfolio.projectCount })}</span>{/snippet}
    </SectionHeader>
  </Card.Root>

  <MetricGrid label={i18n.t('graph.portfolioSummaryAria')}>
    <MetricCard value={portfolio.includedProjectCount} label={i18n.t('graph.portfolioProjects')} />
    <MetricCard value={portfolio.nodes.filter((node) => node.identityScope === 'global').length} label={i18n.t('graph.portfolioSharedIdentities')} tone="info" />
    <MetricCard value={portfolio.edgeCount} label={i18n.t('graph.portfolioRelations')} />
    <MetricCard value={portfolio.skippedProjectCount} label={i18n.t('graph.portfolioSkipped')} tone={portfolio.skippedProjectCount > 0 ? 'warning' : 'neutral'} />
  </MetricGrid>

  {#if portfolio.skippedProjectCount > 0}
    <StatePanel tone="warning" role="status" title={i18n.t('graph.portfolioSkippedTitle')}>
      {#snippet icon()}<AlertTriangle size={18} />{/snippet}
      <ul class="skipped-list">
        {#each portfolio.projects.filter((project) => !project.included) as project}
          <li>{project.displayName} · {i18n.label(project.health)}</li>
        {/each}
      </ul>
    </StatePanel>
  {/if}

  <Card.Root class="topology" role="region" aria-labelledby="portfolio-map-title">
    <SectionHeader variant="panel" level={3} eyebrow={i18n.t('graph.portfolioMapEyebrow')} title={i18n.t('graph.portfolioMapTitle')} titleId="portfolio-map-title">
      {#snippet metadata()}<span>{i18n.t('graph.portfolioExactOnly')}</span>{/snippet}
    </SectionHeader>
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
  </Card.Root>

  {#if showDetails}
  <ContentGrid columns={2} collapse="lg" class="portfolio-grid">
    <Card.Root role="region" aria-labelledby="portfolio-node-title">
      <SectionHeader variant="panel" level={3} eyebrow={i18n.t('graph.portfolioNodeEyebrow')} title={i18n.t('graph.portfolioNodeTitle')} titleId="portfolio-node-title" />
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
                    <Button variant="ghost" disabled={disabled} onclick={() => onOpenProject(occurrence.projectId)}>
                      <ExternalLink size={12} aria-hidden="true" />{i18n.t('graph.portfolioOpenProject')}
                    </Button>
                  </li>
                {/each}
              </ul>
            </details>
          </li>
        {/each}
      </ol>
    </Card.Root>

    <Card.Root role="region" aria-labelledby="portfolio-edge-title">
      <SectionHeader variant="panel" level={3} eyebrow={i18n.t('graph.portfolioEdgeEyebrow')} title={i18n.t('graph.portfolioEdgeTitle')} titleId="portfolio-edge-title" />
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
    </Card.Root>
  </ContentGrid>
  {/if}
</section>

<style>
  .portfolio { display: grid; gap: 9px; min-width: 0; }
  :global(.portfolio-heading) { padding: var(--ui-panel-padding); }
  .scope-badge { max-width: 100%; overflow: hidden; border-radius: var(--radius-pill); padding: 5px 9px; color: var(--color-info); background: var(--color-info-soft); font-size: var(--font-size-micro); font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
  .skipped-list { margin: 0; padding-left: 17px; font-size: var(--font-size-label); }
  :global(.topology [data-slot='section-header']) span { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 750; }
  :global(.topology) { min-width: 0; overflow: hidden; }
  .map-scroll { max-height: 390px; overflow-y: auto; padding: 9px; background: linear-gradient(var(--color-border) 1px, transparent 1px), linear-gradient(90deg, var(--color-border) 1px, transparent 1px); background-size: 20px 20px; }
  svg { display: block; width: 100%; max-width: 100%; height: auto; max-height: 360px; }
  line { stroke: var(--color-border-strong); stroke-width: 2; }
  g rect { fill: var(--color-control); stroke: var(--color-accent); stroke-width: 2; }g.shared rect { fill: var(--color-info-soft); stroke: var(--color-info); }
  text { fill: var(--color-ink); font-family: inherit; font-size: var(--font-size-micro); font-weight: 650; }.type { fill: var(--color-muted); font-size: var(--font-size-micro); text-transform: uppercase; }
  .map-note { margin: 0; border-top: 1px solid var(--color-border); padding: 8px 10px; color: var(--color-muted); font-size: var(--font-size-supporting); line-height: 1.5; }
  :global(.portfolio-grid > [data-slot='card']) { min-width: 0; overflow: hidden; }
  .identity-list, .relation-list { display: grid; gap: 7px; margin: 0; padding: 9px; list-style: none; }
  .identity-list > li, .relation-list > li { min-width: 0; border: 1px solid var(--color-border); border-radius: var(--radius-inset); padding: 10px; }
  .identity-list > li > div { display: flex; justify-content: space-between; gap: 8px; }.identity-list strong { font-size: var(--font-size-label); }.identity-list span { color: var(--color-muted); font-size: var(--font-size-label); }
  code { color: var(--color-muted); font-size: var(--font-size-label); overflow-wrap: anywhere; }
  details { margin-top: 7px; }summary { color: var(--color-accent-strong); font-size: var(--font-size-micro); font-weight: 750; cursor: pointer; }
  .occurrences, .relation-list details ul { display: grid; gap: 5px; margin: 6px 0 0; padding: 0; list-style: none; }
  .occurrences li { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; }
  .occurrences :global([data-slot='button']) { min-height: 44px; padding-inline: 0; color: var(--color-accent-strong); font-size: var(--font-size-label); font-weight: 750; }
  .statement { display: flex; flex-wrap: wrap; align-items: center; gap: 5px; margin: 0; font-size: var(--font-size-micro); }.statement span { color: var(--color-info); font-weight: 750; }
  .relation-list p:not(.statement) { margin: 6px 0 0; color: var(--color-muted); font-size: var(--font-size-micro); line-height: 1.5; }
  .relation-list .limit { border-top: 1px solid var(--color-border); padding-top: 6px; }
  .empty { display: flex; align-items: center; gap: 6px; margin: 0; padding: 9px; color: var(--color-muted); font-size: var(--font-size-label); }
  @media (max-width: 520px) { .occurrences li { flex-direction: column; } }
</style>
