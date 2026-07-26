<script lang="ts">
  import { ArrowRight, Boxes, GitBranch, Link2, Network } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/shared/ui';

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
  <header class="surface">
    <div>
      <p class="eyebrow">{i18n.t('portfolio.resultsEyebrow')}</p>
      <h2 id="portfolio-results-title">{i18n.t('portfolio.resultsTitle')}</h2>
      <p>{i18n.t('portfolio.resultsDetail')}</p>
    </div>
    <code>{workspace.queryId.slice(0, 16)}…</code>
  </header>

  <div class="summary" aria-label={i18n.t('portfolio.querySummaryAria')}>
    <article class="surface">
      <strong>{workspace.matchedProjectCount}</strong>
      <span>{i18n.t('portfolio.projects')}</span>
    </article>
    <article class="surface">
      <strong>{workspace.matchedNodeCount}</strong>
      <span>{i18n.t('portfolio.nodes')}</span>
    </article>
    <article class="surface">
      <strong>{workspace.matchedEdgeCount}</strong>
      <span>{i18n.t('portfolio.edges')}</span>
    </article>
    <article class="surface">
      <strong>{workspace.matchedLineageCount}</strong>
      <span>{i18n.t('portfolio.lineage')}</span>
    </article>
  </div>

  {#if portfolioWorkspaceIsEmpty(workspace)}
    <div class="surface empty">
      <Boxes size={26} aria-hidden="true" />
      <h3>{i18n.t('portfolio.emptyResultsTitle')}</h3>
      <p>{i18n.t('portfolio.emptyResultsDetail')}</p>
    </div>
  {:else}
    <div class="result-grid">
      <section class="surface result-section" aria-labelledby="portfolio-project-results">
        <header>
          <div><Network size={17} aria-hidden="true" /><h3 id="portfolio-project-results">{i18n.t('portfolio.projectResults')}</h3></div>
          <span>{workspace.projects.length}/{workspace.matchedProjectCount}</span>
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
      </section>

      <section class="surface result-section" aria-labelledby="portfolio-node-results">
        <header>
          <div><Boxes size={17} aria-hidden="true" /><h3 id="portfolio-node-results">{i18n.t('portfolio.nodeResults')}</h3></div>
          <span>{workspace.nodes.length}/{workspace.matchedNodeCount}</span>
        </header>
        {#if workspace.nodes.length === 0}
          <p class="section-empty">{i18n.t('portfolio.noNodeResults')}</p>
        {:else}
          <ol>
            {#each workspace.nodes as entry (entry.resultId)}
              <li>
                <div class="row-heading">
                  <strong>{entry.node.label}</strong>
                  <span class="plain-badge">{i18n.label(entry.node.nodeType)}</span>
                </div>
                <span>{i18n.label(entry.node.identityScope)} · <code>{entry.node.canonicalId}</code></span>
                <small><code>{entry.node.artifactPath}</code> · {entry.node.sourceAnchor}</small>
              </li>
            {/each}
          </ol>
        {/if}
      </section>

      <section class="surface result-section" aria-labelledby="portfolio-edge-results">
        <header>
          <div><Link2 size={17} aria-hidden="true" /><h3 id="portfolio-edge-results">{i18n.t('portfolio.edgeResults')}</h3></div>
          <span>{workspace.edges.length}/{workspace.matchedEdgeCount}</span>
        </header>
        {#if workspace.edges.length === 0}
          <p class="section-empty">{i18n.t('portfolio.noEdgeResults')}</p>
        {:else}
          <ol>
            {#each workspace.edges as entry (entry.resultId)}
              <li>
                <div class="row-heading">
                  <strong>{i18n.label(entry.edge.relation)}</strong>
                  <span class="plain-badge">{i18n.label(entry.edge.status)}</span>
                </div>
                <span><code>{entry.edge.sourceNodeId.slice(0, 12)}…</code> → <code>{entry.edge.targetNodeId.slice(0, 12)}…</code></span>
                <p>{entry.edge.rationale}</p>
                <small>{i18n.t('portfolio.evidenceLimit')}: {entry.edge.evidenceLimit}</small>
              </li>
            {/each}
          </ol>
        {/if}
      </section>

      <section class="surface result-section" aria-labelledby="portfolio-lineage-results">
        <header>
          <div><GitBranch size={17} aria-hidden="true" /><h3 id="portfolio-lineage-results">{i18n.t('portfolio.lineageResults')}</h3></div>
          <span>{workspace.lineage.length}/{workspace.matchedLineageCount}</span>
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
      </section>
    </div>
  {/if}

  {#if workspace.nextCursor}
    <button class="button-secondary load-more" type="button" disabled={loadingMore} onclick={onLoadMore}>
      {loadingMore ? i18n.t('portfolio.loadingMore') : i18n.t('portfolio.loadMore')}
    </button>
  {/if}
</section>

<style>
  .results { display: grid; gap: 10px; min-width: 0; }
  .results > header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 15px;
  }
  h2, h3, p { margin: 0; }
  h2 { color: var(--color-ink-strong); font-size: 17px; }
  .results > header p { margin-top: 5px; color: var(--color-muted); font-size: 12px; }
  .results > header code { color: var(--color-muted); font-size: 10px; }
  .summary { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; }
  .summary article { display: grid; gap: 2px; padding: 11px 13px; }
  .summary strong { color: var(--color-ink-strong); font-size: 19px; }
  .summary span { color: var(--color-muted); font-size: 10px; font-weight: 750; text-transform: uppercase; }
  .empty { padding: 28px; text-align: center; color: var(--color-muted); }
  .empty h3 { margin-top: 8px; color: var(--color-ink-strong); }
  .empty p { margin-top: 6px; font-size: 12px; }
  .result-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; min-width: 0; }
  .result-section { min-width: 0; overflow: hidden; }
  .result-section > header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    border-bottom: 1px solid var(--color-border);
    padding: 12px 13px;
  }
  .result-section > header div { display: flex; align-items: center; gap: 7px; color: var(--color-accent-strong); }
  .result-section h3 { color: var(--color-ink-strong); font-size: 13px; }
  .result-section > header > span { color: var(--color-muted); font-size: 10px; }
  ol { display: grid; gap: 0; margin: 0; padding: 0; list-style: none; }
  ol > li { min-width: 0; border-bottom: 1px solid var(--color-border); padding: 11px 13px; }
  ol > li:last-child { border-bottom: 0; }
  .row-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; }
  .row-heading strong { min-width: 0; color: var(--color-ink-strong); font-size: 12px; overflow-wrap: anywhere; }
  ol > li > span, ol > li > small { display: block; margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  ol > li > p { margin-top: 7px; color: var(--color-ink); font-size: 11px; line-height: 1.45; }
  ol > li > a { display: inline-flex; align-items: center; gap: 4px; margin-top: 7px; color: var(--color-accent-strong); font-size: 11px; font-weight: 700; }
  code { overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
  .plain-badge { border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: var(--color-surface-subtle); font-size: 10px; white-space: nowrap; }
  details { margin-top: 7px; }
  summary { cursor: pointer; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  .ids { margin: 6px 0 0; padding-left: 16px; }
  .ids li { margin-top: 3px; color: var(--color-muted); font-size: 9px; }
  .section-empty { padding: 18px 13px; color: var(--color-muted); font-size: 11px; }
  .load-more { justify-self: center; }
  @media (max-width: 860px) {
    .result-grid { grid-template-columns: 1fr; }
  }
  @media (max-width: 520px) {
    .results > header { flex-direction: column; }
    .summary { grid-template-columns: 1fr 1fr; }
  }
</style>
