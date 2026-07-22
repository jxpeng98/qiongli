import cytoscape from 'cytoscape';

import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';

const FIT_PADDING = 28;

export interface AcademicGraphRenderer {
  update(layout: AcademicGraphLayout, viewState: AcademicGraphViewState): void;
  fit(): void;
  destroy(): void;
}

export interface AcademicGraphRendererOptions {
  container: HTMLElement;
  layout: AcademicGraphLayout;
  viewState: AcademicGraphViewState;
  onSelect: (nodeId: string) => void | Promise<void>;
}

export function buildCytoscapeElements(
  layout: AcademicGraphLayout
): cytoscape.ElementDefinition[] {
  return [
    ...layout.nodes.map((node): cytoscape.ElementDefinition => ({
      group: 'nodes',
      data: {
        id: node.nodeId,
        canonicalId: node.canonicalId,
        label: node.label,
        nodeType: node.nodeType,
        layer: node.layer,
        riskSeverity: node.riskSeverity ?? 'none',
        riskCount: node.riskCount
      },
      position: {
        x: node.x + node.width / 2,
        y: node.y + node.height / 2
      },
      selectable: true,
      grabbable: false,
      locked: true
    })),
    ...layout.edges.map((edge): cytoscape.ElementDefinition => ({
      group: 'edges',
      data: {
        id: edge.edgeId,
        source: edge.sourceNodeId,
        target: edge.targetNodeId,
        relation: edge.relation,
        riskSeverity: edge.riskSeverity ?? 'none',
        riskCount: edge.riskCount
      },
      selectable: false,
      grabbable: false
    }))
  ];
}

export function buildCytoscapeStyles(): cytoscape.StylesheetJson {
  return [
    {
      selector: 'node',
      style: {
        width: 152,
        height: 52,
        shape: 'roundrectangle',
        label: 'data(label)',
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
      selector: 'edge',
      style: {
        width: 1.5,
        'line-color': '#94a3b8',
        'line-opacity': 0.72,
        'curve-style': 'bezier',
        'target-arrow-shape': 'triangle',
        'target-arrow-color': '#64748b',
        'arrow-scale': 0.8,
        'overlay-opacity': 0
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
    core.nodes().unselect().removeClass('is-focused');
    if (viewState.selectedNodeId) {
      core.getElementById(viewState.selectedNodeId).select();
    }
    if (viewState.focusNodeId) {
      core.getElementById(viewState.focusNodeId).addClass('is-focused');
    }
  });
}

export function mountAcademicGraphRenderer(
  options: AcademicGraphRendererOptions
): AcademicGraphRenderer {
  assertSynchronized(options.layout, options.viewState);
  let activeLayoutSignature = layoutSignature(options.layout);
  const core = cytoscape({
    container: options.container,
    elements: buildCytoscapeElements(options.layout),
    style: buildCytoscapeStyles(),
    layout: { name: 'preset', fit: true, padding: FIT_PADDING, animate: false },
    minZoom: 0.35,
    maxZoom: 2.5,
    boxSelectionEnabled: false,
    autoungrabify: true,
    selectionType: 'single',
    motionBlur: false
  });

  const tapHandler = (event: cytoscape.EventObjectNode): void => {
    void options.onSelect(event.target.id());
  };
  core.on('tap', 'node', tapHandler);
  applyAcademicGraphViewState(core, options.layout, options.viewState);

  const resizeObserver = typeof ResizeObserver === 'undefined'
    ? null
    : new ResizeObserver(() => {
        core.resize();
        core.fit(undefined, FIT_PADDING);
      });
  resizeObserver?.observe(options.container);

  return {
    update(layout, viewState): void {
      assertSynchronized(layout, viewState);
      const nextSignature = layoutSignature(layout);
      if (nextSignature !== activeLayoutSignature) {
        core.batch(() => {
          core.elements().remove();
          core.add(buildCytoscapeElements(layout));
        });
        core.layout({ name: 'preset', fit: false, animate: false }).run();
        core.fit(undefined, FIT_PADDING);
        activeLayoutSignature = nextSignature;
      }
      applyAcademicGraphViewState(core, layout, viewState);
    },
    fit(): void {
      core.resize();
      core.fit(undefined, FIT_PADDING);
    },
    destroy(): void {
      resizeObserver?.disconnect();
      core.off('tap', 'node', tapHandler);
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

function layoutSignature(layout: AcademicGraphLayout): string {
  const nodes = layout.nodes
    .map((node) => `${node.nodeId}:${node.x}:${node.y}`)
    .join('|');
  const edges = layout.edges.map((edge) => edge.edgeId).join('|');
  return `${layout.layoutKey}|${nodes}|${edges}`;
}
