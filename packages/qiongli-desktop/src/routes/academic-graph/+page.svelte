<script lang="ts">
  import type {
    AcademicGraphDirection,
    AcademicGraphLayer,
    AcademicGraphNodeType,
    AcademicGraphRelation
  } from '@qiongli/app-api';
  import { AlertTriangle, Network, RefreshCw, Search, X } from '@lucide/svelte';

  import { useAppState } from '$lib/context';
  import CytoscapeAcademicGraph from '$lib/features/academic-graph/CytoscapeAcademicGraph.svelte';
  import {
    academicGraphLayers,
    academicGraphNodeTypes,
    academicGraphRelations,
    buildAcademicGraphLayout,
    buildAcademicGraphQuery,
    buildAcademicGraphViewState,
    loadAcademicGraphPresentationState
  } from '$lib/features/academic-graph';
  import { i18n } from '$lib/i18n.svelte';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';

  const app = useAppState();

  let selectedProjectId = $state<string | null>(null);
  let requestedProjectId = $state<string | null>(null);
  let requestedProjectRevision = $state<number | null>(null);
  let loadState = $state<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  let manualRefreshInProgress = $state(false);
  let queryInProgress = $state(false);
  let selectedNodeId = $state<string | null>(null);
  let textFilter = $state('');
  let nodeType = $state<AcademicGraphNodeType | ''>('');
  let relation = $state<AcademicGraphRelation | ''>('');
  let layer = $state<AcademicGraphLayer | ''>('');
  let direction = $state<AcademicGraphDirection>('both');

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let graph = $derived(
    app.academicGraph?.projectId === selectedProjectId
      && app.academicGraph.projectRevision === selectedProject?.semanticRevision
      ? app.academicGraph : null
  );
  let result = $derived(
    app.academicGraphQuery?.projectId === selectedProjectId
      && app.academicGraphQuery.projectRevision === selectedProject?.semanticRevision
      && app.academicGraphQuery.projectionId === graph?.projectionId
      ? app.academicGraphQuery : null
  );
  let nodeLabels = $derived(new Map(
    result?.nodes.map((node) => [node.nodeId, node.label]) ?? []
  ));
  let graphLayout = $derived(result ? buildAcademicGraphLayout(result) : null);
  let graphViewState = $derived(graphLayout ? buildAcademicGraphViewState(graphLayout, {
    selectedNodeId,
    focusNodeId: selectedNodeId,
    direction
  }) : null);
  let canInspect = $derived(
    selectedProject?.health === 'ready' && app.snapshot?.capabilities.academicGraph === true
  );

  $effect(() => {
    if (projects.length === 0) {
      selectedProjectId = null;
      requestedProjectId = null;
      requestedProjectRevision = null;
      loadState = 'idle';
      return;
    }
    if (!selectedProjectId || !projects.some((project) => project.projectId === selectedProjectId)) {
      selectedProjectId = projects[0].projectId;
      resetRequestState();
    }
    if (!canInspect && selectedProject) {
      requestedProjectId = selectedProject.projectId;
      requestedProjectRevision = selectedProject.semanticRevision;
      loadState = 'idle';
      return;
    }
    if (
      selectedProject
      && canInspect
      && (requestedProjectId !== selectedProject.projectId
        || requestedProjectRevision !== selectedProject.semanticRevision)
      && !app.loading
      && !manualRefreshInProgress
    ) {
      requestedProjectId = selectedProject.projectId;
      requestedProjectRevision = selectedProject.semanticRevision;
      void loadGraph(selectedProject.projectId, selectedProject.semanticRevision);
    }
  });

  async function loadGraph(projectId: string, projectRevision: number): Promise<void> {
    loadState = 'loading';
    selectedNodeId = null;
    const complete = await loadAcademicGraphPresentationState(
      projectId,
      projectRevision,
      (intent) => app.execute(intent)
    );
    if (selectedProjectId === projectId && selectedProject?.semanticRevision === projectRevision) {
      loadState = complete ? 'ready' : 'failed';
    }
  }

  function chooseProject(event: Event): void {
    selectedProjectId = (event.currentTarget as HTMLSelectElement).value || null;
    resetFilters();
    resetRequestState();
  }

  function resetRequestState(): void {
    requestedProjectId = null;
    requestedProjectRevision = null;
    loadState = 'idle';
    selectedNodeId = null;
  }

  function resetFilters(): void {
    textFilter = '';
    nodeType = '';
    relation = '';
    layer = '';
    direction = 'both';
    selectedNodeId = null;
  }

  async function refreshGraph(): Promise<void> {
    if (!selectedProject) return;
    const projectId = selectedProject.projectId;
    manualRefreshInProgress = true;
    try {
      const refreshed = await app.execute({ action: 'refresh-research-library' });
      if (refreshed?.type !== 'snapshot') return;
      const current = refreshed.snapshot.researchLibrary.projects
        .find((project) => project.projectId === projectId);
      if (!current || current.health !== 'ready') return;
      requestedProjectId = current.projectId;
      requestedProjectRevision = current.semanticRevision;
      await loadGraph(current.projectId, current.semanticRevision);
    } finally {
      manualRefreshInProgress = false;
    }
  }

  async function runQuery(focusNodeId = selectedNodeId): Promise<void> {
    if (!selectedProject || !graph || queryInProgress) return;
    queryInProgress = true;
    try {
      const event = await app.execute({
        action: 'query-academic-graph',
        projectId: selectedProject.projectId,
        query: buildAcademicGraphQuery(graph.projectionId, {
          focusNodeId,
          direction,
          nodeType: nodeType || null,
          relation: relation || null,
          layer: layer || null,
          text: textFilter
        })
      });
      if (
        event?.type !== 'academic-graph-query'
        || event.result.projectId !== selectedProject.projectId
        || event.result.projectRevision !== selectedProject.semanticRevision
        || event.result.projectionId !== graph.projectionId
      ) {
        loadState = 'failed';
      }
    } finally {
      queryInProgress = false;
    }
  }

  async function applyFilters(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    await runQuery();
  }

  async function selectNode(nodeId: string): Promise<void> {
    selectedNodeId = nodeId;
    await runQuery(nodeId);
  }

  async function clearFocus(): Promise<void> {
    selectedNodeId = null;
    await runQuery(null);
  }

  function shortId(value: string): string {
    return `${value.slice(0, 8)}…${value.slice(-6)}`;
  }

  function nodeLabel(nodeId: string): string {
    return nodeLabels.get(nodeId) ?? shortId(nodeId);
  }
</script>

<PageHeader
  eyebrow={i18n.t('graph.eyebrow')}
  title={i18n.t('graph.title')}
  description={i18n.t('graph.description')}
>
  {#snippet actions()}
    <label class="project-picker">
      <span>{i18n.t('graph.project')}</span>
      <select value={selectedProjectId ?? ''} onchange={chooseProject} disabled={app.loading || projects.length === 0}>
        {#each projects as project}
          <option value={project.projectId}>{project.displayName}</option>
        {/each}
      </select>
    </label>
    <button class="button-secondary" type="button" disabled={app.loading || !canInspect} onclick={refreshGraph}>
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('graph.rebuild')}
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section class="surface state-panel" aria-busy="true"><p>{i18n.t('graph.loadingLibrary')}</p></section>
{:else if projects.length === 0}
  <section class="surface empty-state">
    <Network size={30} aria-hidden="true" />
    <h2>{i18n.t('graph.emptyTitle')}</h2>
    <p>{i18n.t('graph.emptyDetail')}</p>
    <a class="button-primary" href="/research-library">{i18n.t('graph.openLibrary')}</a>
  </section>
{:else if !canInspect}
  <section class="surface state-panel state-warning" role="status">
    <AlertTriangle size={24} aria-hidden="true" />
    <div><h2>{i18n.t('graph.unavailableTitle')}</h2><p>{i18n.t('graph.unavailableDetail')}</p></div>
  </section>
{:else if loadState === 'failed'}
  <section class="surface state-panel state-danger" role="alert">
    <AlertTriangle size={24} aria-hidden="true" />
    <div>
      <h2>{i18n.t('graph.failedTitle')}</h2>
      <p>{i18n.t('graph.failedDetail')}</p>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshGraph}>
        <RefreshCw size={16} aria-hidden="true" />{i18n.t('graph.retry')}
      </button>
    </div>
  </section>
{:else if loadState !== 'ready' || !graph || !result}
  <section class="surface state-panel" aria-busy="true">
    <Network size={24} aria-hidden="true" />
    <p>{i18n.t('graph.loading', { project: selectedProject?.displayName ?? '' })}</p>
  </section>
{:else}
  <section class="metrics" aria-label={i18n.t('graph.summaryAria')}>
    <article class="surface metric"><strong>{graph.nodeCount}</strong><span>{i18n.t('graph.nodes')}</span></article>
    <article class="surface metric"><strong>{graph.edgeCount}</strong><span>{i18n.t('graph.relations')}</span></article>
    <article class="surface metric"><strong>{graph.presentSourceCount}/{graph.sourceCount}</strong><span>{i18n.t('graph.sources')}</span></article>
    <article class="surface metric"><strong>{graph.diagnosticCount}</strong><span>{i18n.t('graph.diagnostics')}</span></article>
  </section>

  <form class="surface filters" onsubmit={applyFilters} aria-label={i18n.t('graph.filters')}>
    <label class="search-field">
      <span>{i18n.t('graph.search')}</span>
      <div><Search size={15} aria-hidden="true" /><input bind:value={textFilter} maxlength="256" placeholder={i18n.t('graph.searchPlaceholder')} /></div>
    </label>
    <label><span>{i18n.t('graph.nodeType')}</span><select bind:value={nodeType}><option value="">{i18n.t('graph.allTypes')}</option>{#each academicGraphNodeTypes as value}<option value={value}>{i18n.label(value)}</option>{/each}</select></label>
    <label><span>{i18n.t('graph.relation')}</span><select bind:value={relation}><option value="">{i18n.t('graph.allRelations')}</option>{#each academicGraphRelations as value}<option value={value}>{i18n.label(value)}</option>{/each}</select></label>
    <label><span>{i18n.t('graph.layer')}</span><select bind:value={layer}><option value="">{i18n.t('graph.allLayers')}</option>{#each academicGraphLayers as value}<option value={value}>{i18n.label(value)}</option>{/each}</select></label>
    <label><span>{i18n.t('graph.direction')}</span><select bind:value={direction} disabled={!selectedNodeId}><option value="both">{i18n.t('graph.both')}</option><option value="incoming">{i18n.t('graph.incoming')}</option><option value="outgoing">{i18n.t('graph.outgoing')}</option></select></label>
    <button class="button-primary" type="submit" disabled={app.loading || queryInProgress}>
      <Search size={16} aria-hidden="true" />{queryInProgress ? i18n.t('graph.querying') : i18n.t('graph.apply')}
    </button>
  </form>

  {#if selectedNodeId}
    <div class="focus-bar" role="status">
      <span>{i18n.t('graph.focused', { label: nodeLabel(selectedNodeId) })}</span>
      <button class="button-quiet" type="button" disabled={queryInProgress} onclick={clearFocus}>
        <X size={14} aria-hidden="true" />{i18n.t('graph.clearFocus')}
      </button>
    </div>
  {/if}

  <p class="result-summary" aria-live="polite">
    {i18n.t('graph.resultSummary', { nodes: result.matchedNodeCount, edges: result.matchedEdgeCount })}
  </p>
  {#if result.nodesTruncated || result.edgesTruncated}
    <p class="truncation" role="status">{i18n.t('graph.truncated')}</p>
  {/if}

  {#if graphLayout && graphViewState}
    <CytoscapeAcademicGraph
      layout={graphLayout}
      viewState={graphViewState}
      busy={queryInProgress}
      onSelect={selectNode}
    />
  {/if}

  <div class="inspection-grid">
    <section class="surface table-panel" aria-labelledby="graph-nodes-title">
      <div class="panel-heading"><div><p class="eyebrow">{i18n.t('graph.tableEyebrow')}</p><h2 id="graph-nodes-title">{i18n.t('graph.nodeTable')}</h2></div><StatusBadge status={result.nodes.length > 0 ? 'ready' : 'missing'} label={`${result.nodes.length}`} /></div>
      {#if result.nodes.length === 0}
        <p class="empty-copy">{i18n.t('graph.noNodes')}</p>
      {:else}
        <div class="table-scroll">
          <table>
            <thead><tr><th scope="col">{i18n.t('graph.label')}</th><th scope="col">{i18n.t('graph.type')}</th><th scope="col">{i18n.t('graph.layers')}</th><th scope="col">{i18n.t('graph.canonicalId')}</th></tr></thead>
            <tbody>
              {#each result.nodes as node}
                <tr class:selected={node.nodeId === selectedNodeId}>
                  <th scope="row"><button type="button" aria-pressed={node.nodeId === selectedNodeId} onclick={() => selectNode(node.nodeId)}>{node.label}</button></th>
                  <td>{i18n.label(node.nodeType)}</td>
                  <td>{node.layers.map((value) => i18n.label(value)).join(', ')}</td>
                  <td><code title={node.canonicalId}>{node.canonicalId}</code></td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>

    <section class="surface edge-panel" aria-labelledby="graph-edges-title">
      <div class="panel-heading"><div><p class="eyebrow">{i18n.t('graph.listEyebrow')}</p><h2 id="graph-edges-title">{i18n.t('graph.edgeList')}</h2></div><StatusBadge status={result.edges.length > 0 ? 'ready' : 'missing'} label={`${result.edges.length}`} /></div>
      {#if result.edges.length === 0}
        <p class="empty-copy">{i18n.t('graph.noEdges')}</p>
      {:else}
        <ol class="edge-list">
          {#each result.edges as edge}
            <li>
              <div class="edge-statement"><strong>{nodeLabel(edge.sourceNodeId)}</strong><span>{i18n.label(edge.relation)}</span><strong>{nodeLabel(edge.targetNodeId)}</strong></div>
              <p>{edge.rationale}</p>
              <dl><div><dt>{i18n.t('graph.strength')}</dt><dd>{i18n.label(edge.inferenceStrength)}</dd></div><div><dt>{i18n.t('graph.confidence')}</dt><dd>{i18n.label(edge.confidence)}</dd></div><div><dt>{i18n.t('graph.status')}</dt><dd>{i18n.label(edge.status)}</dd></div></dl>
              <details><summary>{i18n.t('graph.evidenceLimit')}</summary><p>{edge.evidenceLimit}</p></details>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  </div>

  {#if graph.diagnostics.length > 0}
    <section class="surface diagnostics" aria-labelledby="graph-diagnostics-title">
      <div class="panel-heading"><div><p class="eyebrow">{i18n.t('graph.repairEyebrow')}</p><h2 id="graph-diagnostics-title">{i18n.t('graph.diagnosticList')}</h2></div></div>
      <ul>{#each graph.diagnostics as diagnostic}<li><strong>{i18n.label(diagnostic.code)}</strong><span>{diagnostic.artifactPath}{diagnostic.sourceAnchor ? ` · ${diagnostic.sourceAnchor}` : ''}</span></li>{/each}</ul>
    </section>
  {/if}
{/if}

<style>
  .project-picker, .filters label { display: grid; gap: 5px; color: var(--color-muted); font-size: 11px; font-weight: 750; }
  .project-picker select, .filters select, .filters input { min-height: 38px; border: 1px solid var(--color-border-strong); border-radius: 9px; padding: 7px 9px; color: var(--color-ink); background: white; font: inherit; }
  .state-panel { display: flex; align-items: flex-start; gap: 12px; padding: 22px; }
  .state-panel h2, .empty-state h2 { margin: 0 0 5px; font-size: 17px; }
  .state-panel p, .empty-state p { margin: 0; color: var(--color-muted); line-height: 1.55; }
  .state-panel button { margin-top: 12px; }
  .state-warning { border-color: #fde68a; background: var(--color-warning-soft); }
  .state-danger { border-color: #fecaca; background: var(--color-danger-soft); }
  .empty-state { display: grid; justify-items: center; padding: 48px 24px; text-align: center; }
  .empty-state .button-primary { margin-top: 14px; text-decoration: none; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-bottom: 12px; }
  .metric { display: grid; gap: 3px; padding: 15px; }
  .metric strong { color: var(--color-ink-strong); font-size: 24px; letter-spacing: -0.04em; }
  .metric span { color: var(--color-muted); font-size: 12px; font-weight: 680; }
  .filters { display: grid; grid-template-columns: minmax(190px, 1.4fr) repeat(4, minmax(125px, 1fr)) auto; align-items: end; gap: 10px; padding: 14px; }
  .search-field div { display: flex; align-items: center; gap: 7px; border: 1px solid var(--color-border-strong); border-radius: 9px; padding-left: 9px; background: white; }
  .search-field div:focus-within { box-shadow: 0 0 0 3px rgb(3 105 161 / 0.24); }
  .search-field input { width: 100%; border: 0; padding-left: 0; box-shadow: none !important; }
  .focus-bar { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 10px; border: 1px solid #bae6fd; border-radius: 10px; padding: 8px 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); font-size: 12px; font-weight: 700; }
  .focus-bar .button-quiet { min-height: 32px; padding: 5px 9px; }
  .result-summary { margin: 12px 2px 8px; color: var(--color-muted); font-size: 12px; }
  .truncation { margin: 0 0 8px; border-left: 3px solid var(--color-warning); padding: 6px 9px; color: #854d0e; background: var(--color-warning-soft); font-size: 12px; }
  .inspection-grid { display: grid; grid-template-columns: minmax(0, 1.45fr) minmax(300px, 0.85fr); gap: 12px; }
  .table-panel, .edge-panel, .diagnostics { min-width: 0; overflow: hidden; }
  .panel-heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  .panel-heading h2 { margin: 0; font-size: 16px; }
  .table-scroll { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th, td { border-bottom: 1px solid var(--color-border); padding: 10px 12px; text-align: left; vertical-align: top; }
  thead th { color: var(--color-muted); background: var(--color-surface-subtle); font-size: 10px; letter-spacing: 0.04em; text-transform: uppercase; }
  tbody th { min-width: 190px; }
  tbody th button { border: 0; padding: 0; color: var(--color-accent-strong); background: transparent; font: inherit; font-weight: 720; text-align: left; cursor: pointer; }
  tbody tr.selected { background: var(--color-accent-soft); }
  td code { display: block; max-width: 150px; overflow: hidden; color: var(--color-muted); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
  .edge-list { display: grid; gap: 0; margin: 0; padding: 0; list-style: none; }
  .edge-list li { border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  .edge-list li:last-child { border-bottom: 0; }
  .edge-statement { display: grid; gap: 3px; }
  .edge-statement span { width: fit-content; border-radius: 999px; padding: 2px 7px; color: var(--color-accent-strong); background: var(--color-accent-soft); font-size: 10px; font-weight: 800; }
  .edge-list p { margin: 8px 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  dl { display: flex; flex-wrap: wrap; gap: 10px; margin: 0; }
  dl div { display: flex; gap: 4px; }
  dt { color: var(--color-muted); font-size: 10px; font-weight: 700; }
  dd { margin: 0; font-size: 10px; font-weight: 750; }
  details { margin-top: 8px; color: var(--color-muted); font-size: 11px; }
  summary { cursor: pointer; font-weight: 700; }
  .empty-copy { margin: 0; padding: 24px 16px; color: var(--color-muted); }
  .diagnostics { margin-top: 12px; }
  .diagnostics ul { display: grid; gap: 8px; margin: 0; padding: 14px 32px; }
  .diagnostics li span { display: block; margin-top: 2px; color: var(--color-muted); font-size: 11px; }
  @media (max-width: 1120px) { .filters { grid-template-columns: repeat(3, minmax(0, 1fr)); } .inspection-grid { grid-template-columns: 1fr; } }
  @media (max-width: 760px) { .metrics, .filters { grid-template-columns: 1fr 1fr; } }
  @media (max-width: 520px) { .metrics, .filters { grid-template-columns: 1fr; } }
</style>
