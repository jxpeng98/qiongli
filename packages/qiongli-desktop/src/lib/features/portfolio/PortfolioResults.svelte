<script lang="ts">
  import { ArrowRight, Boxes, GitBranch, Link2, Network } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { MetricCard, MetricGrid, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  import { portfolioWorkspaceIsEmpty, type PortfolioQueryWorkspace } from '.';

  let {
    workspace,
    loadingMore,
    onLoadMore
  }: {
    workspace: PortfolioQueryWorkspace;
    loadingMore: boolean;
    onLoadMore: () => void;
  } = $props();
</script>

<section class="results" aria-labelledby="portfolio-results-title">
  <Card.Root class="results-header">
    <SectionHeader eyebrow={i18n.t('portfolio.resultsEyebrow')} title={i18n.t('portfolio.resultsTitle')} titleId="portfolio-results-title" description={i18n.t('portfolio.resultsDetail')}>
      {#snippet metadata()}<code>{workspace.queryId.slice(0, 16)}…</code>{/snippet}
    </SectionHeader>
  </Card.Root>

  <MetricGrid label={i18n.t('portfolio.querySummaryAria')}>
    <MetricCard value={workspace.matchedProjectCount} label={i18n.t('portfolio.projects')} />
    <MetricCard value={workspace.matchedNodeCount} label={i18n.t('portfolio.nodes')} />
    <MetricCard value={workspace.matchedEdgeCount} label={i18n.t('portfolio.edges')} />
    <MetricCard value={workspace.matchedLineageCount} label={i18n.t('portfolio.lineage')} />
  </MetricGrid>

  {#if portfolioWorkspaceIsEmpty(workspace)}
    <StatePanel centered title={i18n.t('portfolio.emptyResultsTitle')} description={i18n.t('portfolio.emptyResultsDetail')}>
      {#snippet icon()}<Boxes size={20} />{/snippet}
    </StatePanel>
  {:else}
    <div class="result-grid">
      <Card.Root class="result-section" role="region" aria-labelledby="portfolio-project-results">
        <header class="result-header">
          <SectionHeader level={3} title={i18n.t('portfolio.projectResults')} titleId="portfolio-project-results">
            {#snippet icon()}<Network size={17} />{/snippet}
            {#snippet metadata()}<span>{workspace.projects.length}/{workspace.matchedProjectCount}</span>{/snippet}
          </SectionHeader>
        </header>
        {#if workspace.projects.length === 0}
          <p class="section-empty">{i18n.t('portfolio.noProjectResults')}</p>
        {:else}
          <ol>
            {#each workspace.projects as project (project.resultId)}
              <li>
                <div class="row-heading">
                  <strong>{project.displayName}</strong>
                  <StatusBadge
                    status={project.health === 'ready' ? 'ready' : 'attention'}
                    label={i18n.label(project.health)}
                  />
                </div>
                <span>{i18n.label(project.stage)} · {i18n.label(project.lifecycle)} · r{project.semanticRevision}</span>
                <small>{i18n.t('portfolio.projectCounts', {
                  nodes: project.nodeCount,
                  edges: project.edgeCount,
                  lineage: project.lineageCount
                })}</small>
                <a href={`/academic-graph?project=${encodeURIComponent(project.projectId)}`}>
                  {i18n.t('portfolio.openGraph')}<ArrowRight size={13} aria-hidden="true" />
                </a>
              </li>
            {/each}
          </ol>
        {/if}
      </Card.Root>

      <Card.Root class="result-section" role="region" aria-labelledby="portfolio-node-results">
        <header class="result-header">
          <SectionHeader level={3} title={i18n.t('portfolio.nodeResults')} titleId="portfolio-node-results">
            {#snippet icon()}<Boxes size={17} />{/snippet}
            {#snippet metadata()}<span>{workspace.nodes.length}/{workspace.matchedNodeCount}</span>{/snippet}
          </SectionHeader>
        </header>
        {#if workspace.nodes.length === 0}
          <p class="section-empty">{i18n.t('portfolio.noNodeResults')}</p>
        {:else}
          <ol>
            {#each workspace.nodes as entry (entry.resultId)}
              <li>
                <div class="row-heading">
                  <strong>{entry.node.label}</strong>
                  <Badge variant="outline">{i18n.label(entry.node.nodeType)}</Badge>
                </div>
                <span>{i18n.label(entry.node.identityScope)} · <code>{entry.node.canonicalId}</code></span>
                <small><code>{entry.node.artifactPath}</code> · {entry.node.sourceAnchor}</small>
              </li>
            {/each}
          </ol>
        {/if}
      </Card.Root>

      <Card.Root class="result-section" role="region" aria-labelledby="portfolio-edge-results">
        <header class="result-header">
          <SectionHeader level={3} title={i18n.t('portfolio.edgeResults')} titleId="portfolio-edge-results">
            {#snippet icon()}<Link2 size={17} />{/snippet}
            {#snippet metadata()}<span>{workspace.edges.length}/{workspace.matchedEdgeCount}</span>{/snippet}
          </SectionHeader>
        </header>
        {#if workspace.edges.length === 0}
          <p class="section-empty">{i18n.t('portfolio.noEdgeResults')}</p>
        {:else}
          <ol>
            {#each workspace.edges as entry (entry.resultId)}
              <li>
                <div class="row-heading">
                  <strong>{i18n.label(entry.edge.relation)}</strong>
                  <Badge variant="outline">{i18n.label(entry.edge.status)}</Badge>
                </div>
                <span><code>{entry.edge.sourceNodeId.slice(0, 12)}…</code> → <code>{entry.edge.targetNodeId.slice(0, 12)}…</code></span>
                <p>{entry.edge.rationale}</p>
                <small>{i18n.t('portfolio.evidenceLimit')}: {entry.edge.evidenceLimit}</small>
              </li>
            {/each}
          </ol>
        {/if}
      </Card.Root>

      <Card.Root class="result-section" role="region" aria-labelledby="portfolio-lineage-results">
        <header class="result-header">
          <SectionHeader level={3} title={i18n.t('portfolio.lineageResults')} titleId="portfolio-lineage-results">
            {#snippet icon()}<GitBranch size={17} />{/snippet}
            {#snippet metadata()}<span>{workspace.lineage.length}/{workspace.matchedLineageCount}</span>{/snippet}
          </SectionHeader>
        </header>
        {#if workspace.lineage.length === 0}
          <p class="section-empty">{i18n.t('portfolio.noLineageResults')}</p>
        {:else}
          <ol>
            {#each workspace.lineage as entry (entry.lineageId)}
              <li>
                <div class="row-heading">
                  <strong>{i18n.label(entry.kind)}</strong>
                  <span>{i18n.date(entry.occurredAtUnix, true)}</span>
                </div>
                <span>{i18n.t('portfolio.relatedProjects', { count: entry.projectIds.length })}</span>
                {#if entry.fromProjectRevision !== null && entry.toProjectRevision !== null}
                  <small>r{entry.fromProjectRevision} → r{entry.toProjectRevision}</small>
                {/if}
                <details>
                  <summary>{i18n.t('portfolio.relatedEvidence')}</summary>
                  <ul class="ids">
                    {#each entry.relatedIds as id}<li><code>{id}</code></li>{/each}
                  </ul>
                </details>
              </li>
            {/each}
          </ol>
        {/if}
      </Card.Root>
    </div>
  {/if}

  {#if workspace.nextCursor}
    <Button class="load-more" variant="outline" disabled={loadingMore} onclick={onLoadMore}>
      {loadingMore ? i18n.t('portfolio.loadingMore') : i18n.t('portfolio.loadMore')}
    </Button>
  {/if}
</section>

<style>
  .results { display: grid; gap: 10px; min-width: 0; }
  :global(.results-header) { padding: var(--ui-panel-padding); }
  :global(.results-header) code { color: var(--color-muted); font-size: 10px; }
  p { margin: 0; }
  .result-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; min-width: 0; }
  :global(.result-section) { min-width: 0; overflow: hidden; }
  .result-header {
    --ui-icon-container-size: 28px;
    --ui-section-title-size: 13px;
    border-bottom: 1px solid var(--color-border);
    padding: 12px 13px;
  }
  .result-header span { color: var(--color-muted); font-size: 10px; }
  ol { display: grid; gap: 0; margin: 0; padding: 0; list-style: none; }
  ol > li { min-width: 0; border-bottom: 1px solid var(--color-border); padding: 11px 13px; }
  ol > li:last-child { border-bottom: 0; }
  .row-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; }
  .row-heading strong { min-width: 0; color: var(--color-ink-strong); font-size: 12px; overflow-wrap: anywhere; }
  ol > li > span, ol > li > small { display: block; margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  ol > li > p { margin-top: 7px; color: var(--color-ink); font-size: 11px; line-height: 1.45; }
  ol > li > a { display: inline-flex; align-items: center; gap: 4px; margin-top: 7px; color: var(--color-accent-strong); font-size: 11px; font-weight: 700; }
  code { overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
  details { margin-top: 7px; }
  summary { cursor: pointer; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  .ids { margin: 6px 0 0; padding-left: 16px; }
  .ids li { margin-top: 3px; color: var(--color-muted); font-size: var(--font-size-label); }
  .section-empty { padding: 12px; color: var(--color-muted); font-size: 11px; }
  :global(.load-more) { justify-self: center; }
  @media (max-width: 860px) {
    .result-grid { grid-template-columns: 1fr; }
  }
  @media (max-width: 520px) {
    :global(.results-header) { padding: 13px; }
  }
</style>
