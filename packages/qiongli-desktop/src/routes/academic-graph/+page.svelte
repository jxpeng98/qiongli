<script lang="ts">
  import { page } from '$app/state';
  import type {
    AcademicGraphDirection,
    AcademicGraphEntityReference,
    AcademicGraphLayer,
    AcademicGraphNodeType,
    AcademicGraphPathQuery,
    AcademicGraphQueryResult,
    AcademicGraphRelation
  } from '@qiongli/app-api';
  import {
    AlertTriangle,
    ChevronLeft,
    ChevronRight,
    Layers3,
    Network,
    RefreshCw,
    RotateCcw,
    Search
  } from '@lucide/svelte';

  import { useAppState, useProjectWorkspace } from '$lib/context';
  import AcademicGraphInspector from '$lib/features/academic-graph/AcademicGraphInspector.svelte';
  import AcademicGraphPathFinder from '$lib/features/academic-graph/AcademicGraphPathFinder.svelte';
  import AcademicGraphPortfolio from '$lib/features/academic-graph/AcademicGraphPortfolio.svelte';
  import AcademicGraphReadinessPanel from '$lib/features/academic-graph/AcademicGraphReadinessPanel.svelte';
  import AcademicGraphRevisionComparison from '$lib/features/academic-graph/AcademicGraphRevisionComparison.svelte';
  import AcademicGraphRiskOverlay from '$lib/features/academic-graph/AcademicGraphRiskOverlay.svelte';
  import CytoscapeAcademicGraph from '$lib/features/academic-graph/CytoscapeAcademicGraph.svelte';
  import {
    ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM,
    AcademicGraphRequestSequence,
    type AcademicGraphFocusHistory,
    type AcademicGraphLayout,
    type AcademicGraphViewportClass,
    academicGraphLayers,
    academicGraphNodeTypes,
    academicGraphQueryMatchesScope,
    academicGraphRelations,
    academicGraphNodeVisual,
    academicGraphRelationVisual,
    academicGraphReadinessStatus,
    academicGraphViewportClass,
    artifactForAcademicGraphEntity,
    buildAcademicGraphInspection,
    buildAcademicGraphLayout,
    buildAcademicGraphQuery,
    buildAcademicGraphRiskOverlay,
    buildAcademicGraphViewState,
    canRenderAcademicGraph,
    effectiveAcademicGraphReadiness,
    findAcademicGraphMatches,
    loadAcademicGraphPresentationState,
    moveAcademicGraphFocus,
    pushAcademicGraphFocus
  } from '$lib/features/academic-graph';
  import { i18n } from '$lib/i18n.svelte';
  import { ContentGrid, PageLayout, ResponsiveDataView, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Input } from '$lib/components/ui/input';
  import { NativeSelect } from '$lib/components/ui/native-select';

  const app = useAppState();
  const projectWorkspace = useProjectWorkspace();

  let selectedProjectId = $derived(projectWorkspace.projectId);
  let viewMode = $state<'project' | 'portfolio'>('project');
  let requestedProjectId = $state<string | null>(null);
  let requestedProjectRevision = $state<number | null>(null);
  let requestedPortfolioRevision = $state<number | null>(null);
  let observedProjectId = $state<string | null>(null);
  let appliedEntityDeepLink = $state('');
  let loadState = $state<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  let manualRefreshInProgress = $state(false);
  let queryInProgress = $state(false);
  let queryFailed = $state(false);
  let selectedNodeId = $state<string | null>(null);
  let selectedEdgeId = $state<string | null>(null);
  let textFilter = $state('');
  let nodeType = $state<AcademicGraphNodeType | ''>('');
  let relation = $state<AcademicGraphRelation | ''>('');
  let layer = $state<AcademicGraphLayer | ''>('');
  let direction = $state<AcademicGraphDirection>('both');
  let maxDepth = $state<1 | 2 | 3>(1);
  let focusHistory = $state<AcademicGraphFocusHistory>({ entries: [], index: -1 });
  let collapsedClusterIds = $state<string[]>([]);
  let searchFocused = $state(false);
  let layoutViewportClass = $state<AcademicGraphViewportClass>('standard');
  let graphLayout = $state<AcademicGraphLayout | null>(null);
  let previousGraphLayout: AcademicGraphLayout | null = null;
  let activeGraphQueryCount = 0;
  const graphLoadSequence = new AcademicGraphRequestSequence();
  const graphQuerySequence = new AcademicGraphRequestSequence();

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let portfolio = $derived(
    app.academicGraphPortfolio?.libraryRevision === app.snapshot?.researchLibrary.revision
      ? app.academicGraphPortfolio : null
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
  let readiness = $derived(
    app.academicGraphReadiness?.projectId === selectedProjectId
      && app.academicGraphReadiness.projectionId === graph?.projectionId
      ? app.academicGraphReadiness : null
  );
  let pathResult = $derived(
    app.academicGraphPath?.projectId === selectedProjectId
      && app.academicGraphPath.projectRevision === selectedProject?.semanticRevision
      && app.academicGraphPath.projectionId === graph?.projectionId
      && app.academicGraphPath.indexId === result?.indexId
      ? app.academicGraphPath : null
  );
  let nodeLabels = $derived(new Map(
    graph?.nodes.map((node) => [node.nodeId, node.label]) ?? []
  ));
  let searchMatches = $derived(findAcademicGraphMatches(graph?.nodes ?? [], textFilter));
  let matchingNodeIds = $derived(
    findAcademicGraphMatches(graph?.nodes ?? [], textFilter, graph?.nodes.length ?? 0)
      .map((match) => match.nodeId)
  );
  let activeCollapsedClusterIds = $derived(
    collapsedClusterIds.filter((clusterId) =>
      graphLayout?.clusters?.some((cluster) => cluster.clusterId === clusterId) ?? false)
  );
  let canFocusBack = $derived(focusHistory.index > 0);
  let canFocusForward = $derived(
    focusHistory.index >= 0 && focusHistory.index < focusHistory.entries.length - 1
  );
  let riskOverlay = $derived(result ? buildAcademicGraphRiskOverlay(result) : null);
  let graphViewState = $derived(graphLayout ? buildAcademicGraphViewState(graphLayout, {
    selectedNodeId,
    selectedEdgeId,
    focusNodeId: selectedNodeId,
    direction,
    matchingNodeIds,
    collapsedClusterIds: activeCollapsedClusterIds
  }) : null);
  let showGraphCanvas = $derived(
    readiness && result ? canRenderAcademicGraph(readiness, result) : false
  );
  let readinessState = $derived(
    readiness && result ? effectiveAcademicGraphReadiness(readiness, result) : null
  );
  let readinessStatus = $derived(
    readinessState ? academicGraphReadinessStatus(readinessState) : 'unavailable'
  );
  let selectedEntity = $derived<AcademicGraphEntityReference | null>(
    selectedEdgeId
      ? { kind: 'edge', id: selectedEdgeId }
      : selectedNodeId ? { kind: 'node', id: selectedNodeId } : null
  );
  let inspection = $derived(graph ? buildAcademicGraphInspection(graph, selectedEntity) : null);
  let inspectionArtifact = $derived(
    artifactForAcademicGraphEntity(app.projectArtifact, graph, selectedEntity)
  );
  let canInspectProject = $derived(
    selectedProject?.health === 'ready' && app.snapshot?.capabilities.academicGraph === true
  );
  let canInspectPortfolio = $derived(
    projects.length > 0 && app.snapshot?.capabilities.academicGraph === true
  );
  let canInspect = $derived(viewMode === 'portfolio' ? canInspectPortfolio : canInspectProject);

  $effect(() => {
    if (typeof window === 'undefined') return;
    const updateViewportClass = (): void => {
      const next = academicGraphViewportClass(window.innerWidth);
      if (next !== layoutViewportClass) layoutViewportClass = next;
    };
    updateViewportClass();
    window.addEventListener('resize', updateViewportClass);
    return () => window.removeEventListener('resize', updateViewportClass);
  });

  $effect(() => {
    const activeGraph = graph;
    const activeResult = result;
    const entityId = page.url.searchParams.get('entity');
    if (!activeGraph || !activeResult || !entityId || viewMode !== 'project') return;
    const key = `${activeGraph.projectionId}:${entityId}`;
    if (appliedEntityDeepLink === key) return;
    if (activeGraph.nodes.some((node) => node.nodeId === entityId)) {
      appliedEntityDeepLink = key;
      void focusNode(entityId, false);
    } else if (activeGraph.edges.some((edge) => edge.edgeId === entityId)) {
      appliedEntityDeepLink = key;
      selectedNodeId = null;
      selectedEdgeId = entityId;
    }
  });

  $effect(() => {
    const currentResult: AcademicGraphQueryResult | null = result;
    const currentRiskOverlay = riskOverlay;
    const currentViewportClass = layoutViewportClass;
    if (!currentResult) {
      graphLayout = null;
      return;
    }
    const nextLayout = buildAcademicGraphLayout(
      currentResult,
      currentRiskOverlay ?? undefined,
      {
        viewportClass: currentViewportClass,
        previousLayout: previousGraphLayout
      }
    );
    graphLayout = nextLayout;
    previousGraphLayout = nextLayout;
  });

  $effect(() => {
    if (selectedProjectId !== observedProjectId) {
      observedProjectId = selectedProjectId;
      viewMode = 'project';
      resetFilters();
      resetRequestState();
    }
    if (projects.length === 0) {
      requestedProjectId = null;
      requestedProjectRevision = null;
      loadState = 'idle';
      return;
    }
    if (!selectedProjectId || !projects.some((project) => project.projectId === selectedProjectId)) {
      resetRequestState();
      return;
    }
    if (viewMode === 'portfolio') {
      if (!canInspectPortfolio) {
        loadState = 'idle';
        return;
      }
      const libraryRevision = app.snapshot?.researchLibrary.revision;
      if (libraryRevision !== undefined
        && requestedPortfolioRevision !== libraryRevision
        && !app.loading
        && !manualRefreshInProgress) {
        requestedPortfolioRevision = libraryRevision;
        void loadPortfolio(libraryRevision);
      }
      return;
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
    const scope = projectScope(projectId, projectRevision);
    const request = graphLoadSequence.begin(scope);
    loadState = 'loading';
    selectedNodeId = null;
    selectedEdgeId = null;
    queryFailed = false;
    const complete = await loadAcademicGraphPresentationState(
      projectId,
      projectRevision,
      (intent) => app.execute(
        intent,
        () => graphLoadSequence.isCurrent(request, activeProjectScope())
      )
    );
    if (graphLoadSequence.isCurrent(request, activeProjectScope())) {
      loadState = complete ? 'ready' : 'failed';
    }
  }

  async function loadPortfolio(libraryRevision: number): Promise<void> {
    const scope = portfolioScope(libraryRevision);
    const request = graphLoadSequence.begin(scope);
    loadState = 'loading';
    selectedNodeId = null;
    selectedEdgeId = null;
    queryFailed = false;
    const event = await app.execute(
      { action: 'load-academic-graph-portfolio' },
      () => graphLoadSequence.isCurrent(request, activePortfolioScope())
    );
    if (graphLoadSequence.isCurrent(request, activePortfolioScope())) {
      loadState = event?.type === 'academic-graph-portfolio'
        && event.portfolio.libraryRevision === libraryRevision ? 'ready' : 'failed';
    }
  }

  function chooseView(event: Event): void {
    viewMode = (event.currentTarget as HTMLSelectElement).value as 'project' | 'portfolio';
    resetFilters();
    resetRequestState();
  }

  function resetRequestState(): void {
    graphLoadSequence.invalidate();
    graphQuerySequence.invalidate();
    requestedProjectId = null;
    requestedProjectRevision = null;
    requestedPortfolioRevision = null;
    loadState = 'idle';
    selectedNodeId = null;
    selectedEdgeId = null;
    queryFailed = false;
  }

  function resetFilters(): void {
    graphQuerySequence.invalidate();
    textFilter = '';
    nodeType = '';
    relation = '';
    layer = '';
    direction = 'both';
    maxDepth = 1;
    selectedNodeId = null;
    selectedEdgeId = null;
    queryFailed = false;
    focusHistory = { entries: [], index: -1 };
    collapsedClusterIds = [];
    searchFocused = false;
  }

  async function refreshGraph(): Promise<void> {
    if (!selectedProject && viewMode === 'project') return;
    const projectId = selectedProject?.projectId ?? null;
    manualRefreshInProgress = true;
    try {
      const refreshed = await app.execute({ action: 'refresh-research-library' });
      if (refreshed?.type !== 'snapshot') return;
      if (viewMode === 'portfolio') {
        requestedPortfolioRevision = refreshed.snapshot.researchLibrary.revision;
        await loadPortfolio(refreshed.snapshot.researchLibrary.revision);
        return;
      }
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

  function openPortfolioProject(projectId: string): void {
    if (!projects.some((project) => project.projectId === projectId)) return;
    viewMode = 'project';
    resetFilters();
    resetRequestState();
    void projectWorkspace.selectProject(projectId);
  }

  async function runQuery(focusNodeId = selectedNodeId): Promise<void> {
    if (!selectedProject || !graph || queryInProgress) return;
    const project = selectedProject;
    const activeGraph = graph;
    const scope = graphScope(project.projectId, project.semanticRevision, activeGraph.projectionId);
    const request = graphQuerySequence.begin(scope);
    activeGraphQueryCount += 1;
    queryInProgress = true;
    queryFailed = false;
    try {
      const event = await app.execute({
        action: 'query-academic-graph',
        projectId: project.projectId,
        query: buildAcademicGraphQuery(activeGraph.projectionId, {
          focusNodeId,
          direction,
          maxDepth,
          nodeType: nodeType || null,
          relation: relation || null,
          layer: layer || null,
          text: textFilter
        })
      }, () => graphQuerySequence.isCurrent(request, activeGraphScope()));
      const current = graphQuerySequence.isCurrent(request, activeGraphScope());
      if (current) {
        queryFailed = !academicGraphQueryMatchesScope(
          event,
          project.projectId,
          project.semanticRevision,
          activeGraph.projectionId
        );
      }
    } finally {
      activeGraphQueryCount = Math.max(0, activeGraphQueryCount - 1);
      queryInProgress = activeGraphQueryCount > 0;
    }
  }

  async function applyFilters(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    selectedEdgeId = null;
    if (textFilter.trim().length > 0) selectedNodeId = null;
    await runQuery(textFilter.trim().length > 0 ? null : selectedNodeId);
  }

  async function selectNode(nodeId: string): Promise<void> {
    await focusNode(nodeId, false);
  }

  async function selectSearchMatch(nodeId: string): Promise<void> {
    await focusNode(nodeId, true);
  }

  async function focusNode(nodeId: string, keepSearchFocus: boolean): Promise<void> {
    selectedEdgeId = null;
    selectedNodeId = nodeId;
    textFilter = '';
    focusHistory = pushAcademicGraphFocus(focusHistory, nodeId);
    searchFocused = keepSearchFocus;
    await runQuery(nodeId);
  }

  function inspectEdge(edgeId: string): void {
    selectedEdgeId = edgeId;
  }

  function inspectRisk(entity: AcademicGraphEntityReference): void {
    if (entity.kind === 'edge') {
      selectedEdgeId = entity.id;
      return;
    }
    selectedEdgeId = null;
    selectedNodeId = entity.id;
  }

  async function openArtifact(entity: AcademicGraphEntityReference): Promise<boolean> {
    if (!selectedProject || !graph) return false;
    const event = await app.execute({
      action: 'open-academic-graph-artifact',
      projectId: selectedProject.projectId,
      expectedProjectRevision: graph.projectRevision,
      expectedProjectionId: graph.projectionId,
      entity
    });
    return event?.type === 'academic-graph-artifact-opened'
      && event.projectId === selectedProject.projectId
      && event.projectRevision === graph.projectRevision
      && event.projectionId === graph.projectionId
      && event.entity.kind === entity.kind
      && event.entity.id === entity.id;
  }

  async function previewArtifact(entity: AcademicGraphEntityReference): Promise<boolean> {
    if (!selectedProject || !graph) return false;
    const event = await app.execute({
      action: 'read-project-artifact',
      projectId: selectedProject.projectId,
      expectedProjectRevision: graph.projectRevision,
      reference: {
        kind: 'academic-graph-entity',
        expectedProjectionId: graph.projectionId,
        entity
      },
      maxBytes: 64 * 1_024
    });
    return event?.type === 'project-artifact-read'
      && event.artifact.projectId === selectedProject.projectId
      && event.artifact.projectRevision === graph.projectRevision
      && event.artifact.projectionId === graph.projectionId
      && event.artifact.entityKind === entity.kind
      && event.artifact.entityId === entity.id;
  }

  async function queryPath(
    query: Omit<AcademicGraphPathQuery, 'expectedProjectionId'>
  ): Promise<boolean> {
    if (!selectedProject || !graph || !result) return false;
    const event = await app.execute({
      action: 'query-academic-graph-path',
      projectId: selectedProject.projectId,
      query: { ...query, expectedProjectionId: graph.projectionId }
    });
    return event?.type === 'academic-graph-path'
      && event.result.projectId === selectedProject.projectId
      && event.result.projectRevision === graph.projectRevision
      && event.result.projectionId === graph.projectionId
      && event.result.indexId === result.indexId
      && event.result.sourceNodeId === query.sourceNodeId
      && event.result.targetNodeId === query.targetNodeId
      && event.result.maxHops === query.maxHops;
  }

  async function navigateFocus(offset: -1 | 1): Promise<void> {
    const next = moveAcademicGraphFocus(focusHistory, offset);
    if (next.index === focusHistory.index) return;
    focusHistory = next;
    const nodeId = next.entries[next.index];
    if (!nodeId || !graph?.nodes.some((node) => node.nodeId === nodeId)) return;
    selectedEdgeId = null;
    selectedNodeId = nodeId;
    textFilter = '';
    await runQuery(nodeId);
  }

  async function updateDirection(event: Event): Promise<void> {
    direction = (event.currentTarget as HTMLSelectElement).value as AcademicGraphDirection;
    if (selectedNodeId) await runQuery(selectedNodeId);
  }

  async function updateDepth(event: Event): Promise<void> {
    maxDepth = Number((event.currentTarget as HTMLSelectElement).value) as 1 | 2 | 3;
    if (selectedNodeId) await runQuery(selectedNodeId);
  }

  function toggleCluster(clusterId: string): void {
    collapsedClusterIds = collapsedClusterIds.includes(clusterId)
      ? collapsedClusterIds.filter((value) => value !== clusterId)
      : [...collapsedClusterIds, clusterId].sort();
  }

  function collapseAllClusters(): void {
    collapsedClusterIds = [...(graphLayout?.clusters ?? [])]
      .map((cluster) => cluster.clusterId)
      .sort();
  }

  function expandAllClusters(): void {
    collapsedClusterIds = [];
  }

  async function resetOverview(): Promise<void> {
    textFilter = '';
    nodeType = '';
    relation = '';
    layer = '';
    direction = 'both';
    maxDepth = 1;
    selectedNodeId = null;
    selectedEdgeId = null;
    focusHistory = { entries: [], index: -1 };
    collapsedClusterIds = [];
    await runQuery(null);
  }

  function shortId(value: string): string {
    return `${value.slice(0, 8)}…${value.slice(-6)}`;
  }

  function nodeLabel(nodeId: string): string {
    return nodeLabels.get(nodeId) ?? shortId(nodeId);
  }

  function projectScope(projectId: string, projectRevision: number): string {
    return `project:${projectId}:${projectRevision}`;
  }

  function activeProjectScope(): string | null {
    return selectedProject
      ? projectScope(selectedProject.projectId, selectedProject.semanticRevision)
      : null;
  }

  function portfolioScope(libraryRevision: number): string {
    return `portfolio:${libraryRevision}`;
  }

  function activePortfolioScope(): string | null {
    const libraryRevision = app.snapshot?.researchLibrary.revision;
    return viewMode === 'portfolio' && libraryRevision !== undefined
      ? portfolioScope(libraryRevision)
      : null;
  }

  function graphScope(
    projectId: string,
    projectRevision: number,
    projectionId: string
  ): string {
    return `${projectScope(projectId, projectRevision)}:${projectionId}`;
  }

  function activeGraphScope(): string | null {
    return selectedProject && graph
      ? graphScope(selectedProject.projectId, selectedProject.semanticRevision, graph.projectionId)
      : null;
  }
</script>

<svelte:head>
  <title>{i18n.t('graph.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageLayout
  eyebrow={i18n.t('graph.eyebrow')}
  title={i18n.t('graph.title')}
  description={i18n.t('graph.description')}
>
  {#snippet actions()}
    <label class="project-picker">
      <span>{i18n.t('graph.view')}</span>
      <NativeSelect value={viewMode} onchange={chooseView} disabled={app.loading || projects.length === 0}>
        <option value="project">{i18n.t('graph.projectView')}</option>
        <option value="portfolio">{i18n.t('graph.portfolioView')}</option>
      </NativeSelect>
    </label>
    <Button variant="outline" disabled={app.loading || !canInspect} onclick={refreshGraph}>
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('graph.rebuild')}
    </Button>
  {/snippet}

{#if !app.snapshot}
  <StatePanel
    centered
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('graph.loadingLibrary')}
  />
{:else if projects.length === 0}
  <StatePanel centered title={i18n.t('graph.emptyTitle')} description={i18n.t('graph.emptyDetail')}>
    {#snippet icon()}<Network size={22} />{/snippet}
    {#snippet actions()}<Button href="/research-library">{i18n.t('graph.openLibrary')}</Button>{/snippet}
  </StatePanel>
{:else if !canInspect}
  <StatePanel tone="warning" role="status" title={i18n.t('graph.unavailableTitle')} description={i18n.t('graph.unavailableDetail')}>
    {#snippet icon()}<AlertTriangle size={19} />{/snippet}
    {#snippet actions()}
      <Button class="state-action" href="/research-library">
        {i18n.t('graph.openLibrary')}
      </Button>
    {/snippet}
  </StatePanel>
{:else if loadState === 'failed'}
  <StatePanel tone="danger" role="alert" title={i18n.t('graph.failedTitle')} description={i18n.t('graph.failedDetail')}>
    {#snippet icon()}<AlertTriangle size={19} />{/snippet}
    {#snippet actions()}
      <Button variant="outline" disabled={app.loading} onclick={refreshGraph}>
        <RefreshCw size={16} aria-hidden="true" />{i18n.t('graph.retry')}
      </Button>
    {/snippet}
  </StatePanel>
{:else if viewMode === 'portfolio' && (loadState !== 'ready' || !portfolio)}
  <StatePanel
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('graph.portfolioLoading')}
  >
    {#snippet icon()}<Network size={19} />{/snippet}
  </StatePanel>
{:else if viewMode === 'portfolio' && portfolio}
  <AcademicGraphPortfolio
    {portfolio}
    disabled={app.loading}
    onOpenProject={openPortfolioProject}
  />
{:else if loadState !== 'ready' || !graph || !result || !readiness}
  <StatePanel
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('graph.loading', { project: selectedProject?.displayName ?? '' })}
  >
    {#snippet icon()}<Network size={19} />{/snippet}
  </StatePanel>
{:else}
  {#if !showGraphCanvas}
    <AcademicGraphReadinessPanel {readiness} {result} />
    <Card.Root class="readiness-actions" role="navigation" aria-label={i18n.t('graph.readinessActions')}>
      <Button variant="outline" href="/research-library">{i18n.t('graph.openLibrary')}</Button>
      <Button
        disabled={app.loading || manualRefreshInProgress}
        onclick={refreshGraph}
      >
        <RefreshCw size={15} aria-hidden="true" />{i18n.t('graph.rebuild')}
      </Button>
    </Card.Root>
  {/if}

  {#if readiness.semanticNodeCount > 0}
    <Card.Root class="filters-card">
    <form class="filters" onsubmit={applyFilters} aria-label={i18n.t('graph.filters')}>
      <div class="search-field">
        <label for="academic-graph-search">{i18n.t('graph.search')}</label>
        <div class="search-control">
          <div class="search-input">
            <Search size={15} aria-hidden="true" />
            <Input
              id="academic-graph-search"
              bind:value={textFilter}
              maxlength={256}
              autocomplete="off"
              role="combobox"
              aria-autocomplete="list"
              aria-expanded={searchFocused && textFilter.trim().length > 0}
              aria-controls={searchFocused && textFilter.trim().length > 0
                ? 'academic-graph-search-results'
                : undefined}
              placeholder={i18n.t('graph.searchPlaceholder')}
              onfocus={() => searchFocused = true}
              onblur={() => searchFocused = false}
            />
          </div>
          {#if searchFocused && textFilter.trim().length > 0}
            <div id="academic-graph-search-results" class="search-results" role="listbox">
              {#if searchMatches.length > 0}
                {#each searchMatches as match (match.nodeId)}
                  <Button
                    variant="ghost"
                    role="option"
                    aria-selected={match.nodeId === selectedNodeId}
                    onmousedown={(event) => event.preventDefault()}
                    onclick={() => selectSearchMatch(match.nodeId)}
                  >
                    <strong>{match.label}</strong>
                    <span>{match.canonicalId}</span>
                  </Button>
                {/each}
              {:else}
                <p role="status">{i18n.t('graph.searchNoMatches')}</p>
              {/if}
            </div>
          {/if}
        </div>
      </div>
      <label><span>{i18n.t('graph.relation')}</span><NativeSelect bind:value={relation}><option value="">{i18n.t('graph.allRelations')}</option>{#each academicGraphRelations as value}<option value={value}>{i18n.label(value)}</option>{/each}</NativeSelect></label>
      <label><span>{i18n.t('graph.layer')}</span><NativeSelect bind:value={layer}><option value="">{i18n.t('graph.allLayers')}</option>{#each academicGraphLayers as value}<option value={value}>{i18n.label(value)}</option>{/each}</NativeSelect></label>
      <Button type="submit" disabled={app.loading || queryInProgress}>
        <Search size={16} aria-hidden="true" />{queryInProgress ? i18n.t('graph.querying') : i18n.t('graph.apply')}
      </Button>
      <details class="advanced-filters">
        <summary>{i18n.t('graph.advancedFilters')}</summary>
        <div>
          <label><span>{i18n.t('graph.nodeType')}</span><NativeSelect bind:value={nodeType}><option value="">{i18n.t('graph.allTypes')}</option>{#each academicGraphNodeTypes as value}<option value={value}>{i18n.label(value)}</option>{/each}</NativeSelect></label>
        </div>
      </details>
    </form>
    </Card.Root>

    {#if queryFailed}
      <div class="query-failure-wrap">
        <StatePanel tone="danger" role="alert" title={i18n.t('graph.queryFailedTitle')} description={i18n.t('graph.queryFailedDetail')}>
          {#snippet icon()}<AlertTriangle size={18} />{/snippet}
          {#snippet metadata()}
            <Button
              variant="outline"
              disabled={queryInProgress}
              onclick={() => runQuery(textFilter.trim().length > 0 ? null : selectedNodeId)}
            >
              <RefreshCw size={15} aria-hidden="true" />{i18n.t('graph.retryQuery')}
            </Button>
          {/snippet}
        </StatePanel>
      </div>
    {/if}

    {#if graphLayout}
      <div class="exploration-bar" role="toolbar" aria-label={i18n.t('graph.explorationToolbar')}>
        <div class="history-actions">
          <Button
            class="icon-button"
            variant="ghost"
            size="icon"
            aria-label={i18n.t('graph.focusBack')}
            title={i18n.t('graph.focusBack')}
            disabled={!canFocusBack || queryInProgress}
            onclick={() => navigateFocus(-1)}
          >
            <ChevronLeft size={16} aria-hidden="true" />
          </Button>
          <Button
            class="icon-button"
            variant="ghost"
            size="icon"
            aria-label={i18n.t('graph.focusForward')}
            title={i18n.t('graph.focusForward')}
            disabled={!canFocusForward || queryInProgress}
            onclick={() => navigateFocus(1)}
          >
            <ChevronRight size={16} aria-hidden="true" />
          </Button>
        </div>
        <span class="focus-context" role="status">
          {selectedNodeId
            ? i18n.t('graph.focused', { label: nodeLabel(selectedNodeId) })
            : i18n.t('graph.overviewMode')}
        </span>
        <label class="compact-control">
          <span>{i18n.t('graph.direction')}</span>
          <NativeSelect value={direction} disabled={!selectedNodeId || queryInProgress} onchange={updateDirection}>
            <option value="both">{i18n.t('graph.both')}</option>
            <option value="incoming">{i18n.t('graph.incoming')}</option>
            <option value="outgoing">{i18n.t('graph.outgoing')}</option>
          </NativeSelect>
        </label>
        <label class="compact-control">
          <span>{i18n.t('graph.depth')}</span>
          <NativeSelect value={String(maxDepth)} disabled={!selectedNodeId || queryInProgress} onchange={updateDepth}>
            <option value="1">{i18n.t('graph.depthValue', { depth: 1 })}</option>
            <option value="2">{i18n.t('graph.depthValue', { depth: 2 })}</option>
            <option value="3">{i18n.t('graph.depthValue', { depth: 3 })}</option>
          </NativeSelect>
        </label>
        <details class="community-menu">
          <summary>
            <Layers3 size={15} aria-hidden="true" />
            {i18n.t('graph.communities')}
            <span>{activeCollapsedClusterIds.length}/{graphLayout.clusters?.length ?? 0}</span>
          </summary>
          <div>
            <div class="community-actions">
              <Button variant="ghost" onclick={expandAllClusters}>
                {i18n.t('graph.expandAll')}
              </Button>
              <Button variant="ghost" onclick={collapseAllClusters}>
                {i18n.t('graph.collapseAll')}
              </Button>
            </div>
            {#each graphLayout.clusters ?? [] as cluster (cluster.clusterId)}
              <label>
                <Checkbox
                  checked={activeCollapsedClusterIds.includes(cluster.clusterId)}
                  onclick={() => toggleCluster(cluster.clusterId)}
                />
                <span>{cluster.label}</span>
                <small>{cluster.nodeIds.length}</small>
              </label>
            {:else}
              <p>{i18n.t('graph.noCommunities')}</p>
            {/each}
          </div>
        </details>
        <Button
          class="reset-overview"
          variant="ghost"
          disabled={queryInProgress}
          onclick={resetOverview}
        >
          <RotateCcw size={14} aria-hidden="true" />{i18n.t('graph.resetOverview')}
        </Button>
      </div>
    {/if}

    {#if showGraphCanvas && graphLayout && graphViewState}
      <CytoscapeAcademicGraph
        layout={graphLayout}
        viewState={graphViewState}
        busy={queryInProgress}
        onSelect={selectNode}
        onSelectEdge={inspectEdge}
        compact
        status={readinessStatus}
        statusLabel={readinessState
          ? i18n.t(`graph.readiness.badge.${readinessState}`)
          : undefined}
        resultSummary={graphLayout.algorithm === ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM
          ? i18n.t('graph.resultSummaryTopology', {
              nodes: result.matchedNodeCount,
              edges: result.matchedEdgeCount,
              components: graphLayout.components?.length ?? 0
            })
          : i18n.t('graph.resultSummary', {
              nodes: result.matchedNodeCount,
              edges: result.matchedEdgeCount
            })}
        tableTarget="#graph-data-summary"
      />
      <AcademicGraphReadinessPanel {readiness} {result} compact />
    {/if}

    {#if inspection}
      <AcademicGraphInspector
        {inspection}
        artifact={inspectionArtifact}
        disabled={app.loading || queryInProgress}
        onPreview={previewArtifact}
        onOpen={openArtifact}
      />
    {/if}

    <details class="workspace-disclosure">
      <summary>
        <span>{i18n.t('graph.riskTitle')} · {i18n.t('graph.comparisonTitle')}</span>
        <StatusBadge
          status={riskOverlay && riskOverlay.riskCount > 0 ? 'attention' : 'ready'}
          label={riskOverlay
            ? i18n.t('graph.riskCount', { count: riskOverlay.riskCount })
            : i18n.t('graph.riskClear')}
        />
      </summary>
      <ContentGrid columns={2} collapse="lg" class="disclosure-body">
        {#if riskOverlay}
          <AcademicGraphRiskOverlay
            overlay={riskOverlay}
            disabled={app.loading || queryInProgress}
            onInspect={inspectRisk}
          />
        {/if}
        <AcademicGraphRevisionComparison
          comparison={app.academicGraphComparison?.projectId === selectedProjectId
            && app.academicGraphComparison.afterProjectionId === graph.projectionId
            ? app.academicGraphComparison : null}
          disabled={app.loading || queryInProgress}
          onInspect={inspectRisk}
        />
      </ContentGrid>
    </details>

    <details class="workspace-disclosure">
      <summary><span>{i18n.t('graph.pathTitle')}</span></summary>
      <div class="disclosure-body">
        <AcademicGraphPathFinder
          {graph}
          result={pathResult}
          disabled={app.loading || queryInProgress}
          onQuery={queryPath}
        />
      </div>
    </details>

    <details class="workspace-disclosure">
      <summary id="graph-data-summary">
        <span>{i18n.t('graph.nodeTable')} · {i18n.t('graph.edgeList')}</span>
        <span class="summary-count">{result.nodes.length} · {result.edges.length}</span>
      </summary>
      <ContentGrid columns={2} collapse="lg" class="disclosure-body">
        <Card.Root class="table-panel" role="region" aria-labelledby="graph-nodes-title">
          <SectionHeader variant="panel" eyebrow={i18n.t('graph.tableEyebrow')} title={i18n.t('graph.nodeTable')} titleId="graph-nodes-title">
            {#snippet metadata()}<StatusBadge status={result.nodes.length > 0 ? 'ready' : 'missing'} label={`${result.nodes.length}`} />{/snippet}
          </SectionHeader>
          {#if result.nodes.length === 0}
            <p class="empty-copy">{i18n.t('graph.noNodes')}</p>
          {:else}
            <ResponsiveDataView label={i18n.t('graph.nodeTable')}>
              {#snippet desktop()}
                <table>
                  <thead><tr><th scope="col">{i18n.t('graph.label')}</th><th scope="col">{i18n.t('graph.type')}</th><th scope="col">{i18n.t('graph.layers')}</th><th scope="col">{i18n.t('graph.canonicalId')}</th></tr></thead>
                  <tbody>
                    {#each result.nodes as node}
                      <tr class:selected={node.nodeId === selectedNodeId}>
                        <th scope="row"><Button class="node-select" variant="ghost" aria-pressed={node.nodeId === selectedNodeId} onclick={() => selectNode(node.nodeId)}>{node.label}</Button></th>
                        <td class="visual-type">
                          <b aria-hidden="true">{academicGraphNodeVisual(node.nodeType).mark}</b>
                          {i18n.label(node.nodeType)}
                        </td>
                        <td>{node.layers.map((value) => i18n.label(value)).join(', ')}</td>
                        <td><code title={node.canonicalId}>{node.canonicalId}</code></td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              {/snippet}
              {#snippet mobile()}
                <ol class="node-cards">
                  {#each result.nodes as node}
                    <li class:selected={node.nodeId === selectedNodeId}>
                      <Button class="node-card-select" variant="ghost" aria-pressed={node.nodeId === selectedNodeId} onclick={() => selectNode(node.nodeId)}>
                        <strong>{node.label}</strong>
                        <span>{academicGraphNodeVisual(node.nodeType).mark} {i18n.label(node.nodeType)}</span>
                      </Button>
                      <dl>
                        <div><dt>{i18n.t('graph.layers')}</dt><dd>{node.layers.map((value) => i18n.label(value)).join(', ')}</dd></div>
                        <div><dt>{i18n.t('graph.canonicalId')}</dt><dd><code>{node.canonicalId}</code></dd></div>
                      </dl>
                    </li>
                  {/each}
                </ol>
              {/snippet}
            </ResponsiveDataView>
          {/if}
        </Card.Root>

        <Card.Root class="edge-panel" role="region" aria-labelledby="graph-edges-title">
          <SectionHeader variant="panel" eyebrow={i18n.t('graph.listEyebrow')} title={i18n.t('graph.edgeList')} titleId="graph-edges-title">
            {#snippet metadata()}<StatusBadge status={result.edges.length > 0 ? 'ready' : 'missing'} label={`${result.edges.length}`} />{/snippet}
          </SectionHeader>
          {#if result.edges.length === 0}
            <p class="empty-copy">{i18n.t('graph.noEdges')}</p>
          {:else}
            <ol class="edge-list">
              {#each result.edges as edge}
                <li class:selected={edge.edgeId === selectedEdgeId}>
                  <div class="edge-statement">
                    <strong>{nodeLabel(edge.sourceNodeId)}</strong>
                    <span title={i18n.t(
                      `graph.relationFamily.${academicGraphRelationVisual(edge.relation).family}`
                    )}>
                      <b aria-hidden="true">{academicGraphRelationVisual(edge.relation).mark}</b>
                      {i18n.label(edge.relation)}
                    </span>
                    <strong>{nodeLabel(edge.targetNodeId)}</strong>
                  </div>
                  <p>{edge.rationale}</p>
                  <dl><div><dt>{i18n.t('graph.strength')}</dt><dd>{i18n.label(edge.inferenceStrength)}</dd></div><div><dt>{i18n.t('graph.confidence')}</dt><dd>{i18n.label(edge.confidence)}</dd></div><div><dt>{i18n.t('graph.status')}</dt><dd>{i18n.label(edge.status)}</dd></div></dl>
                  <details><summary>{i18n.t('graph.evidenceLimit')}</summary><p>{edge.evidenceLimit}</p></details>
                  <Button class="inspect-relation" variant="ghost" aria-pressed={edge.edgeId === selectedEdgeId} onclick={() => inspectEdge(edge.edgeId)}>{i18n.t('graph.inspectRelation')}</Button>
                </li>
              {/each}
            </ol>
          {/if}
        </Card.Root>
      </ContentGrid>
    </details>
  {/if}

  {#if graph.diagnostics.length > 0}
    <details class="workspace-disclosure diagnostics-disclosure">
      <summary>
        <span>{i18n.t('graph.diagnosticList')}</span>
        <StatusBadge status="attention" label={`${graph.diagnostics.length}`} />
      </summary>
      <Card.Root class="diagnostics" role="region" aria-labelledby="graph-diagnostics-title">
        <SectionHeader variant="panel" eyebrow={i18n.t('graph.repairEyebrow')} title={i18n.t('graph.diagnosticList')} titleId="graph-diagnostics-title" />
        <ul>{#each graph.diagnostics as diagnostic}<li><strong>{i18n.label(diagnostic.code)}</strong><span>{diagnostic.artifactPath}{diagnostic.sourceAnchor ? ` · ${diagnostic.sourceAnchor}` : ''}</span></li>{/each}</ul>
      </Card.Root>
    </details>
  {/if}
{/if}
</PageLayout>

<style>
  .project-picker, .filters label, .search-field { display: grid; gap: 5px; color: var(--color-muted); font-size: var(--font-size-label); font-weight: 750; }
  .project-picker :global([data-slot='native-select-wrapper']), .filters label :global([data-slot='native-select-wrapper']) { width: 100%; }
  :global(.state-action) { width: fit-content; text-decoration: none; }
  :global(.readiness-actions) { display: flex; align-items: center; justify-content: flex-end; gap: 6px; margin-top: -3px; margin-bottom: 9px; padding: 7px 10px; }
  :global(.readiness-actions [data-slot='button']) { text-decoration: none; }
  .filters { display: grid; grid-template-columns: minmax(190px, 1.4fr) repeat(2, minmax(125px, 1fr)) auto; align-items: end; gap: 8px; padding: var(--ui-panel-padding); }
  .query-failure-wrap { margin: 10px 0; }
  .search-control { position: relative; min-width: 0; }
  .search-input { display: flex; align-items: center; gap: 7px; border: 1px solid var(--color-border-strong); border-radius: var(--radius-control); padding-left: 9px; background: var(--color-control); }
  .search-input:focus-within { box-shadow: 0 0 0 3px rgb(3 105 161 / 0.24); }
  .search-field :global([data-slot='input']) { width: 100%; border: 0; padding-left: 0; box-shadow: none !important; }
  .search-results { position: absolute; top: calc(100% + 5px); left: 0; z-index: 20; display: grid; width: min(420px, calc(100vw - 36px)); max-height: 286px; overflow-y: auto; border: 1px solid var(--color-border-strong); border-radius: var(--radius-inset); padding: 5px; color: var(--color-ink); background: var(--color-surface); box-shadow: var(--shadow-overlay); }
  .search-results :global([data-slot='button']) { display: grid; width: 100%; height: auto; min-height: 48px; align-content: center; justify-content: stretch; gap: 2px; padding: 7px 9px; color: inherit; text-align: left; white-space: normal; }
  .search-results :global([data-slot='button']:hover), .search-results :global([data-slot='button']:focus-visible) { background: var(--color-accent-soft); }
  :global(.search-results [data-slot='button'] strong), :global(.search-results [data-slot='button'] span) { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  :global(.search-results [data-slot='button'] span) { color: var(--color-muted); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: var(--font-size-micro); font-weight: 600; }
  .search-results p { margin: 0; padding: 13px 9px; color: var(--color-muted); font-size: var(--font-size-label); }
  .advanced-filters { grid-column: 1 / -1; border-top: 1px solid var(--color-border); padding-top: 8px; }
  .advanced-filters > summary { width: fit-content; color: var(--color-accent-strong); font-size: var(--font-size-label); font-weight: 750; cursor: pointer; white-space: nowrap; }
  .advanced-filters > div { display: grid; grid-template-columns: minmax(180px, 0.5fr); gap: 10px; padding-top: 10px; }
  .exploration-bar { position: relative; z-index: 4; display: flex; min-width: 0; align-items: center; flex-wrap: wrap; gap: 7px; margin: 10px 0; border: 1px solid var(--color-accent-border); border-radius: var(--radius-inset); padding: 7px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .history-actions { display: flex; flex: none; gap: 4px; }
  .exploration-bar :global(.icon-button) { width: 44px; min-height: 44px; justify-content: center; padding: 0; }
  .focus-context { flex: 1 1 180px; min-width: 120px; overflow: hidden; font-size: var(--font-size-label); font-weight: 760; text-overflow: ellipsis; white-space: nowrap; }
  .compact-control { display: flex; flex: none; align-items: center; gap: 6px; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 750; white-space: nowrap; }
  .compact-control :global([data-slot='native-select-wrapper']) { width: auto; }
  .community-menu { position: relative; flex: none; }
  .community-menu > summary { display: inline-flex; min-height: 44px; align-items: center; gap: 6px; box-sizing: border-box; border: 1px solid var(--color-border-strong); border-radius: var(--radius-control); padding: 6px 9px; color: var(--color-ink); background: var(--color-control); font-size: var(--font-size-label); font-weight: 750; cursor: pointer; list-style: none; white-space: nowrap; }
  .community-menu > summary::-webkit-details-marker { display: none; }
  .community-menu > summary span { color: var(--color-muted); font-size: var(--font-size-micro); font-variant-numeric: tabular-nums; }
  .community-menu > div { position: absolute; top: calc(100% + 6px); right: 0; z-index: 30; display: grid; width: min(330px, calc(100vw - 32px)); max-height: 340px; overflow-y: auto; border: 1px solid var(--color-border-strong); border-radius: var(--radius-inset); padding: 7px; color: var(--color-ink); background: var(--color-surface); box-shadow: var(--shadow-overlay); }
  .community-menu > div > label { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; min-height: 44px; align-items: center; gap: 8px; border-radius: var(--radius-control-inner); padding: 5px 7px; cursor: pointer; }
  .community-menu > div > label:hover { background: var(--color-surface-subtle); }
  .community-menu label span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .community-menu label small { color: var(--color-muted); font-size: var(--font-size-micro); }
  .community-menu p { margin: 0; padding: 12px 8px; color: var(--color-muted); font-size: var(--font-size-label); }
  .community-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 5px; border-bottom: 1px solid var(--color-border); padding-bottom: 6px; }
  .community-actions :global([data-slot='button']) { min-height: 40px; justify-content: center; }
  :global(.reset-overview) { flex: none; min-height: 44px; }
  .workspace-disclosure {
    min-width: 0;
    margin: 0 0 10px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    background: var(--color-control);
  }
  .workspace-disclosure > summary {
    display: flex;
    min-height: 48px;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    color: var(--color-ink);
    font-size: var(--font-size-supporting);
    font-weight: 760;
    cursor: pointer;
  }
  .workspace-disclosure > summary > span:first-child {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .summary-count {
    flex: none;
    color: var(--color-muted);
    font-size: var(--font-size-micro);
    white-space: nowrap;
  }
  .disclosure-body {
    min-width: 0;
    border-top: 1px solid var(--color-border);
    padding: 10px;
    background: var(--color-surface-subtle);
  }
  :global(.table-panel), :global(.edge-panel), :global(.diagnostics) { min-width: 0; overflow: hidden; }
  table { width: 100%; table-layout: fixed; border-collapse: collapse; font-size: var(--font-size-supporting); }
  th, td { border-bottom: 1px solid var(--color-border); padding: 8px 10px; text-align: left; vertical-align: top; }
  thead th { color: var(--color-muted); background: var(--color-surface-subtle); font-size: var(--font-size-micro); letter-spacing: 0.04em; text-transform: uppercase; }
  th, td { overflow-wrap: anywhere; }
  tbody th :global(.node-select) { width: 100%; height: auto; min-height: 44px; justify-content: flex-start; padding-inline: 0; color: var(--color-accent-strong); font-weight: 720; text-align: left; white-space: normal; }
  tbody tr.selected { background: var(--color-accent-soft); }
  .visual-type { white-space: nowrap; }
  .visual-type b { display: inline-grid; min-width: 22px; height: 22px; place-items: center; margin-right: 5px; border: 1px solid var(--color-border); border-radius: var(--radius-control-inner); color: var(--color-muted); font-size: var(--font-size-micro); }
  td code { display: block; max-width: 150px; overflow: hidden; color: var(--color-muted); font-size: var(--font-size-label); text-overflow: ellipsis; white-space: nowrap; }
  .edge-list { display: grid; gap: 0; margin: 0; padding: 0; list-style: none; }
  .edge-list li { border-bottom: 1px solid var(--color-border); padding: 8px 10px; }
  .edge-list li.selected { background: var(--color-accent-soft); }
  .edge-list li:last-child { border-bottom: 0; }
  .edge-statement { display: grid; gap: 3px; }
  .edge-statement span { display: inline-flex; width: fit-content; align-items: center; gap: 4px; border-radius: var(--radius-pill); padding: 2px 7px; color: var(--color-accent-strong); background: var(--color-accent-soft); font-size: var(--font-size-micro); font-weight: 800; white-space: nowrap; }
  .edge-statement span b { font-size: var(--font-size-supporting); }
  .edge-list p { margin: 8px 0; color: var(--color-muted); font-size: var(--font-size-supporting); line-height: 1.55; }
  dl { display: flex; flex-wrap: wrap; gap: 10px; margin: 0; }
  dl div { display: flex; gap: 4px; }
  dt { color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 700; }
  dd { margin: 0; font-size: var(--font-size-micro); font-weight: 750; }
  .edge-list details { margin-top: 8px; color: var(--color-muted); font-size: var(--font-size-label); }
  .edge-list summary { cursor: pointer; font-weight: 700; }
  :global(.inspect-relation) { min-height: 44px; margin-top: 5px; padding-inline: 0; color: var(--color-accent-strong); font-size: var(--font-size-label); font-weight: 750; }
  .empty-copy { margin: 0; padding: 12px 10px; color: var(--color-muted); }
  .node-cards { display: grid; gap: 7px; margin: 0; padding: 9px; list-style: none; }
  .node-cards > li { min-width: 0; border: 1px solid var(--color-border); border-radius: var(--radius-inset); padding: 8px; background: var(--color-surface-subtle); }
  .node-cards > li.selected { border-color: var(--color-accent-border); background: var(--color-accent-soft); }
  .node-cards :global(.node-card-select) { display: grid; width: 100%; height: auto; justify-items: start; gap: 3px; padding: 6px; text-align: left; white-space: normal; }
  :global(.node-cards .node-card-select span) { color: var(--color-muted); font-size: var(--font-size-micro); }
  .node-cards dl { display: grid; gap: 6px; margin-top: 7px; }
  .node-cards dl div { display: grid; gap: 2px; }
  .node-cards dd { min-width: 0; overflow-wrap: anywhere; }
  :global(.diagnostics) { margin: 0; }
  :global(.diagnostics) ul { display: grid; gap: 8px; margin: 0; padding: 10px 24px; }
  :global(.diagnostics) li span { display: block; margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-label); }
  @media (max-width: 1040px) { .filters { grid-template-columns: 1fr 1fr; } }
  @media (max-width: 520px) {
    .filters { grid-template-columns: 1fr; }
    .advanced-filters > div { grid-template-columns: 1fr; }
    :global(.readiness-actions) { align-items: stretch; flex-direction: column; }
    :global(.readiness-actions) :global([data-slot='button']) { justify-content: center; width: 100%; }
  }
</style>
