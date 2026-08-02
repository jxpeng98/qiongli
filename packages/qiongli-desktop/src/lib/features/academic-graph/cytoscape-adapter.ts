import cytoscape from 'cytoscape';

import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';
import {
  academicGraphNodeVisual,
  academicGraphRelationVisual,
  compactAcademicGraphLabel
} from './visual-language';

const FIT_PADDING = 28;
const OVERVIEW_ZOOM_THRESHOLD = 0.5;
const COMPACT_LABEL_ZOOM_THRESHOLD = 0.82;
const DETAIL_LABEL_ZOOM_THRESHOLD = 1.35;
const ZOOM_STEP = 1.22;
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 2.5;

export interface AcademicGraphViewport {
  zoom: number;
  overview: boolean;
  extent: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  };
}

export interface AcademicGraphRenderer {
  update(layout: AcademicGraphLayout, viewState: AcademicGraphViewState): void;
  fit(): void;
  fitSelection(): void;
  zoomIn(): void;
  zoomOut(): void;
  destroy(): void;
}

export interface AcademicGraphRendererOptions {
  container: HTMLElement;
  layout: AcademicGraphLayout;
  viewState: AcademicGraphViewState;
  onSelect: (nodeId: string) => void | Promise<void>;
  onSelectEdge?: (edgeId: string) => void | Promise<void>;
  onViewportChange?: (viewport: AcademicGraphViewport) => void;
}

export function buildCytoscapeElements(
  layout: AcademicGraphLayout
): cytoscape.ElementDefinition[] {
  return [
    ...layout.nodes.map((node): cytoscape.ElementDefinition => ({
      group: 'nodes',
      data: {
        ...nodeVisualData(node.nodeType, node.label, node.canonicalId),
        id: node.nodeId,
        canonicalId: node.canonicalId,
        kind: 'semantic',
        nodeType: node.nodeType,
        layer: node.layer,
        componentId: node.componentId ?? 'unassigned',
        communityId: node.communityId ?? 'unassigned',
        spine: node.spine === true ? 'yes' : 'no',
        riskSeverity: node.riskSeverity ?? 'none',
        riskCount: node.riskCount,
        riskKinds: node.riskKinds?.join(', ') ?? ''
      },
      position: {
        x: node.x + node.width / 2,
        y: node.y + node.height / 2
      },
      selectable: true,
      grabbable: false,
      locked: true
    })),
    ...(layout.clusters ?? []).map((cluster): cytoscape.ElementDefinition => ({
      group: 'nodes',
      data: {
        id: cluster.clusterId,
        kind: 'overview',
        label: cluster.label,
        layer: cluster.layer,
        nodeCount: cluster.nodeIds.length,
        componentId: cluster.componentId
      },
      position: {
        x: cluster.x + cluster.width / 2,
        y: cluster.y + cluster.height / 2
      },
      selectable: false,
      grabbable: false,
      locked: true
    })),
    ...layout.edges.map((edge): cytoscape.ElementDefinition => ({
      group: 'edges',
      data: {
        ...relationVisualData(edge.relation),
        id: edge.edgeId,
        kind: 'semantic',
        source: edge.sourceNodeId,
        target: edge.targetNodeId,
        relation: edge.relation,
        routeOffset: edge.routeOffset ?? 0,
        riskSeverity: edge.riskSeverity ?? 'none',
        riskCount: edge.riskCount,
        riskKinds: edge.riskKinds?.join(', ') ?? '',
        inferenceStrength: edge.inferenceStrength ?? 'unknown',
        confidence: edge.confidence ?? 'unknown',
        status: edge.status ?? 'unknown'
      },
      selectable: true,
      grabbable: false
    })),
    ...(layout.clusterEdges ?? []).map((edge): cytoscape.ElementDefinition => ({
      group: 'edges',
      data: {
        id: edge.clusterEdgeId,
        kind: 'overview',
        source: edge.sourceClusterId,
        target: edge.targetClusterId,
        edgeCount: edge.edgeCount,
        routeOffset: edge.routeOffset
      },
      selectable: false,
      grabbable: false
    }))
  ];
}

export function academicGraphRendererSignature(layout: AcademicGraphLayout): string {
  return JSON.stringify({
    layoutKey: layout.layoutKey,
    elements: buildCytoscapeElements(layout)
  });
}

function nodeVisualData(
  nodeType: AcademicGraphLayout['nodes'][number]['nodeType'],
  label: string,
  canonicalId: string
): Record<string, string> {
  const visual = academicGraphNodeVisual(nodeType);
  const compactLabel = compactAcademicGraphLabel(label);
  return {
    label,
    compactLabel: `${visual.mark} · ${compactLabel}`,
    detailLabel: `${visual.mark} · ${compactLabel}\n${canonicalId}`,
    typeMark: visual.mark,
    nodeShape: visual.shape
  };
}

function relationVisualData(
  relation: AcademicGraphLayout['edges'][number]['relation']
): Record<string, string> {
  const visual = academicGraphRelationVisual(relation);
  return {
    relationFamily: visual.family,
    lineStyle: visual.lineStyle,
    arrowShape: visual.arrowShape,
    relationMark: visual.mark
  };
}

export function buildCytoscapeStyles(): cytoscape.StylesheetJson {
  return [
    {
      selector: 'node[kind = "semantic"]',
      style: {
        width: 152,
        height: 52,
        shape: 'roundrectangle',
        label: 'data(compactLabel)',
        'font-family': 'system-ui, -apple-system, sans-serif',
        'font-size': 11,
        'font-weight': 600,
        color: '#172033',
        'text-wrap': 'ellipsis',
        'text-max-width': '132px',
        'text-valign': 'center',
        'text-halign': 'center',
        'background-color': '#ffffff',
        'border-color': '#94a3b8',
        'border-width': 2,
        'overlay-opacity': 0
      }
    },
    {
      selector: 'node[nodeType = "research-question"], node[nodeType = "decision"]',
      style: { shape: 'diamond', width: 88, height: 70, 'text-max-width': '68px' }
    },
    {
      selector: 'node[nodeType = "idea"], node[nodeType = "concept"]',
      style: { shape: 'ellipse', width: 118, height: 58, 'text-max-width': '94px' }
    },
    {
      selector: 'node[nodeType = "contribution"], node[nodeType = "claim"]',
      style: { shape: 'hexagon', width: 128, height: 62, 'text-max-width': '98px' }
    },
    {
      selector: 'node[nodeType = "evidence"]',
      style: { shape: 'barrel', width: 132, height: 58, 'text-max-width': '100px' }
    },
    {
      selector: 'node[nodeType = "gap"]',
      style: { shape: 'triangle', width: 92, height: 76, 'text-max-width': '62px' }
    },
    {
      selector: 'node[nodeType = "method"]',
      style: { shape: 'pentagon', width: 112, height: 68, 'text-max-width': '82px' }
    },
    {
      selector: 'node[nodeType = "paper"], node[nodeType = "manuscript-section"], node[nodeType = "artifact"]',
      style: { shape: 'rectangle' }
    },
    {
      selector: 'node[layer = "portfolio"]',
      style: { 'border-color': '#0f766e' }
    },
    {
      selector: 'node[layer = "literature"]',
      style: { 'border-color': '#2563eb' }
    },
    {
      selector: 'node[layer = "idea-decision"]',
      style: { 'border-color': '#7c3aed' }
    },
    {
      selector: 'node[layer = "argument"]',
      style: { 'border-color': '#c2410c' }
    },
    {
      selector: 'node[layer = "manuscript"]',
      style: { 'border-color': '#be123c' }
    },
    {
      selector: 'node[layer = "combined"]',
      style: { 'border-color': '#475569' }
    },
    {
      selector: 'node[kind = "overview"]',
      style: {
        display: 'none',
        width: 176,
        height: 64,
        label: 'data(label)',
        'background-color': '#f8fafc',
        'border-color': '#475569',
        'border-style': 'dashed',
        'border-width': 3,
        'font-size': 12,
        'font-weight': 700,
        'text-max-width': '152px'
      }
    },
    {
      selector: 'node[spine = "yes"]',
      style: {
        'background-color': '#f0fdfa',
        'border-width': 3,
        'font-weight': 700
      }
    },
    {
      selector: 'node[riskSeverity = "medium"]',
      style: {
        'background-color': '#fff7ed',
        'border-color': '#f97316',
        'border-style': 'dashed'
      }
    },
    {
      selector: 'node[riskSeverity = "high"]',
      style: {
        'background-color': '#fef2f2',
        'border-color': '#dc2626',
        'border-width': 4
      }
    },
    {
      selector: 'node:selected, node.is-focused',
      style: {
        color: '#075985',
        'background-color': '#e0f2fe',
        'border-color': '#0369a1',
        'border-width': 4
      }
    },
    {
      selector: 'node.is-search-match',
      style: {
        'border-color': '#a16207',
        'border-style': 'double',
        'border-width': 5,
        'underlay-color': '#fef08a',
        'underlay-opacity': 0.64,
        'underlay-padding': 5
      }
    },
    {
      selector: 'edge',
      style: {
        width: 1.5,
        'line-color': '#94a3b8',
        'line-opacity': 0.72,
        'curve-style': 'unbundled-bezier',
        'control-point-distances': 'data(routeOffset)',
        'control-point-weights': 0.5,
        'target-arrow-shape': 'triangle',
        'target-arrow-color': '#64748b',
        'arrow-scale': 0.8,
        'overlay-opacity': 0
      }
    },
    {
      selector: 'edge[relationFamily = "evidence"]',
      style: {
        'line-style': 'solid',
        'target-arrow-shape': 'triangle'
      }
    },
    {
      selector: 'edge[relationFamily = "challenge"]',
      style: {
        width: 2.4,
        'line-color': '#b91c1c',
        'line-style': 'dashed',
        'target-arrow-color': '#b91c1c',
        'target-arrow-shape': 'tee'
      }
    },
    {
      selector: 'edge[relationFamily = "provenance"]',
      style: {
        'line-color': '#475569',
        'line-style': 'dotted',
        'target-arrow-color': '#475569',
        'target-arrow-shape': 'diamond'
      }
    },
    {
      selector: 'edge[relationFamily = "structure"]',
      style: {
        'line-color': '#334155',
        'line-style': 'solid',
        'target-arrow-color': '#334155',
        'target-arrow-shape': 'square'
      }
    },
    {
      selector: 'edge[relationFamily = "development"]',
      style: {
        'line-color': '#7c3aed',
        'line-style': 'dashed',
        'target-arrow-color': '#7c3aed',
        'target-arrow-shape': 'triangle'
      }
    },
    {
      selector: 'edge[confidence = "low"], edge[confidence = "unknown"]',
      style: {
        'line-opacity': 0.46,
        'line-style': 'dotted'
      }
    },
    {
      selector: 'edge[riskSeverity = "medium"]',
      style: {
        width: 3,
        'line-color': '#f97316',
        'target-arrow-color': '#f97316',
        'line-style': 'dashed'
      }
    },
    {
      selector: 'edge[riskSeverity = "high"]',
      style: {
        width: 4,
        'line-color': '#dc2626',
        'target-arrow-color': '#dc2626'
      }
    },
    {
      selector: 'edge[kind = "overview"]',
      style: {
        display: 'none',
        width: 'mapData(edgeCount, 1, 20, 2, 7)',
        'line-color': '#64748b',
        'line-opacity': 0.8,
        'target-arrow-color': '#475569'
      }
    },
    {
      selector: '.overview-hidden',
      style: {
        display: 'none'
      }
    },
    {
      selector: '.overview-visible',
      style: {
        display: 'element'
      }
    },
    {
      selector: '.is-muted',
      style: {
        opacity: 0.16
      }
    },
    {
      selector: 'edge.is-context-edge',
      style: {
        width: 2.5,
        'line-opacity': 0.92
      }
    },
    {
      selector: 'edge:selected',
      style: {
        width: 4,
        'line-color': '#0369a1',
        'target-arrow-color': '#0369a1',
        'line-opacity': 1
      }
    },
    {
      selector: '.explicit-cluster-visible',
      style: {
        display: 'element'
      }
    },
    {
      selector: 'node.is-collapsed-cluster',
      style: {
        'background-color': '#e0f2fe',
        'border-color': '#0369a1',
        'border-style': 'solid',
        'border-width': 4
      }
    },
    {
      selector: '.explicit-collapsed',
      style: {
        display: 'none'
      }
    },
    {
      selector: '.legend-hidden',
      style: {
        display: 'none'
      }
    },
    {
      selector: 'node.semantic-compact',
      style: {
        label: 'data(typeMark)',
        'font-size': 9,
        'text-max-width': '52px'
      }
    },
    {
      selector: 'node.semantic-detail',
      style: {
        label: 'data(detailLabel)',
        'font-size': 10,
        'text-wrap': 'wrap',
        'text-max-width': '132px'
      }
    }
  ];
}

export function applyAcademicGraphViewState(
  core: cytoscape.Core,
  layout: AcademicGraphLayout,
  viewState: AcademicGraphViewState
): void {
  assertSynchronized(layout, viewState);
  core.batch(() => {
    core.elements().removeClass(
      'is-focused is-muted is-context-edge is-search-match '
      + 'explicit-cluster-visible explicit-collapsed is-collapsed-cluster legend-hidden'
    );
    core.nodes().unselect();
    core.edges().unselect();
    if (viewState.selectedNodeId) {
      core.getElementById(viewState.selectedNodeId).select();
    }
    if (viewState.selectedEdgeId) {
      core.getElementById(viewState.selectedEdgeId).select();
    }
    for (const nodeId of viewState.matchingNodeIds) {
      core.getElementById(nodeId).addClass('is-search-match');
    }
    if (viewState.focusNodeId) {
      const focused = core.getElementById(viewState.focusNodeId);
      focused.addClass('is-focused');
      const context = focused.closedNeighborhood();
      core.elements('[kind = "semantic"]').difference(context).addClass('is-muted');
      context.edges().addClass('is-context-edge');
    }
    if (viewState.collapsedClusterIds.length > 0) {
      core.elements('[kind = "overview"]').addClass('explicit-cluster-visible');
      const collapsedNodeIds = new Set<string>();
      for (const cluster of layout.clusters ?? []) {
        if (!viewState.collapsedClusterIds.includes(cluster.clusterId)) continue;
        core.getElementById(cluster.clusterId).addClass('is-collapsed-cluster');
        for (const nodeId of cluster.nodeIds) {
          collapsedNodeIds.add(nodeId);
          core.getElementById(nodeId).addClass('explicit-collapsed');
        }
      }
      core.edges('[kind = "semantic"]').forEach((edge) => {
        if (
          collapsedNodeIds.has(edge.source().id())
          || collapsedNodeIds.has(edge.target().id())
        ) {
          edge.addClass('explicit-collapsed');
        }
      });
    }
    for (const nodeType of viewState.hiddenNodeTypes) {
      core.nodes(`[kind = "semantic"][nodeType = "${nodeType}"]`).addClass('legend-hidden');
    }
    for (const family of viewState.hiddenRelationFamilies) {
      core.edges(`[kind = "semantic"][relationFamily = "${family}"]`).addClass('legend-hidden');
    }
    core.edges('[kind = "semantic"]').forEach((edge) => {
      if (
        edge.source().hasClass('legend-hidden')
        || edge.target().hasClass('legend-hidden')
      ) {
        edge.addClass('legend-hidden');
      }
    });
  });
}

export function applyAcademicGraphZoomLevel(core: cytoscape.Core): boolean {
  const overviewElements = core.elements('[kind = "overview"]');
  const semanticElements = core.elements('[kind = "semantic"]');
  const overview = overviewElements.length > 0 && core.zoom() <= OVERVIEW_ZOOM_THRESHOLD;
  const compact = !overview && core.zoom() < COMPACT_LABEL_ZOOM_THRESHOLD;
  const detail = !overview && core.zoom() >= DETAIL_LABEL_ZOOM_THRESHOLD;
  core.batch(() => {
    overviewElements.toggleClass('overview-visible', overview);
    semanticElements.toggleClass('overview-hidden', overview);
    core.nodes('[kind = "semantic"]').toggleClass('semantic-compact', compact);
    core.nodes('[kind = "semantic"]').toggleClass('semantic-detail', detail);
  });
  return overview;
}

function fitSemanticGraph(core: cytoscape.Core): void {
  const overviewElements = core.elements('[kind = "overview"]');
  const semanticElements = core.elements('[kind = "semantic"]');
  core.batch(() => {
    overviewElements.removeClass('overview-visible');
    semanticElements.removeClass('overview-hidden');
  });
  core.resize();
  core.fit(semanticElements, FIT_PADDING);
  applyAcademicGraphZoomLevel(core);
}

function fitCurrentSelection(core: cytoscape.Core): void {
  const selectedNodes = core.nodes(':selected');
  const selectedEdges = core.edges(':selected');
  const selection = selectedNodes.union(selectedEdges).union(selectedEdges.connectedNodes());
  if (selection.length === 0) {
    fitSemanticGraph(core);
    return;
  }
  core.fit(selection, FIT_PADDING * 2);
  applyAcademicGraphZoomLevel(core);
}

function zoomAroundCenter(core: cytoscape.Core, factor: number): void {
  const level = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, core.zoom() * factor));
  core.zoom(level);
  applyAcademicGraphZoomLevel(core);
}

function viewportSnapshot(core: cytoscape.Core): AcademicGraphViewport {
  const extent = core.extent();
  return {
    zoom: core.zoom(),
    overview: applyAcademicGraphZoomLevel(core),
    extent: {
      x1: extent.x1,
      y1: extent.y1,
      x2: extent.x2,
      y2: extent.y2
    }
  };
}

export function mountAcademicGraphRenderer(
  options: AcademicGraphRendererOptions
): AcademicGraphRenderer {
  assertSynchronized(options.layout, options.viewState);
  let activeLayout = options.layout;
  let activeLayoutSignature = academicGraphRendererSignature(options.layout);
  const core = cytoscape({
    container: options.container,
    elements: buildCytoscapeElements(options.layout),
    style: buildCytoscapeStyles(),
    layout: { name: 'preset', fit: true, padding: FIT_PADDING, animate: false },
    minZoom: MIN_ZOOM,
    maxZoom: MAX_ZOOM,
    boxSelectionEnabled: false,
    autoungrabify: true,
    selectionType: 'single',
    motionBlur: false
  });

  const tapHandler = (event: cytoscape.EventObjectNode): void => {
    void options.onSelect(event.target.id());
  };
  const edgeTapHandler = (event: cytoscape.EventObjectEdge): void => {
    void options.onSelectEdge?.(event.target.id());
  };
  core.on('tap', 'node[kind = "semantic"]', tapHandler);
  core.on('tap', 'edge[kind = "semantic"]', edgeTapHandler);
  applyAcademicGraphViewState(core, options.layout, options.viewState);
  const viewportHandler = (): void => {
    options.onViewportChange?.(viewportSnapshot(core));
  };
  core.on('zoom pan', viewportHandler);
  viewportHandler();

  const resizeObserver = typeof ResizeObserver === 'undefined'
    ? null
    : new ResizeObserver(() => {
        fitSemanticGraph(core);
      });
  resizeObserver?.observe(options.container);

  return {
    update(layout, viewState): void {
      assertSynchronized(layout, viewState);
      // Selection, focus, and legend updates reuse the same immutable layout
      // object, so they stay O(1) here instead of serializing every element.
      if (layout !== activeLayout) {
        const nextSignature = academicGraphRendererSignature(layout);
        if (nextSignature !== activeLayoutSignature) {
          core.batch(() => {
            core.elements().remove();
            core.add(buildCytoscapeElements(layout));
          });
          core.layout({ name: 'preset', fit: false, animate: false }).run();
          fitSemanticGraph(core);
          activeLayoutSignature = nextSignature;
        }
        activeLayout = layout;
      }
      applyAcademicGraphViewState(core, layout, viewState);
    },
    fit(): void {
      fitSemanticGraph(core);
      viewportHandler();
    },
    fitSelection(): void {
      fitCurrentSelection(core);
      viewportHandler();
    },
    zoomIn(): void {
      zoomAroundCenter(core, ZOOM_STEP);
      viewportHandler();
    },
    zoomOut(): void {
      zoomAroundCenter(core, 1 / ZOOM_STEP);
      viewportHandler();
    },
    destroy(): void {
      resizeObserver?.disconnect();
      core.off('tap', 'node[kind = "semantic"]', tapHandler);
      core.off('tap', 'edge[kind = "semantic"]', edgeTapHandler);
      core.off('zoom pan', viewportHandler);
      core.destroy();
    }
  };
}

function assertSynchronized(
  layout: AcademicGraphLayout,
  viewState: AcademicGraphViewState
): void {
  if (layout.layoutKey !== viewState.layoutKey) {
    throw new Error('academic graph renderer received mismatched layout and view state');
  }
}
