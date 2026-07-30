import cytoscape from 'cytoscape';
import { describe, expect, it, vi } from 'vitest';

import {
  academicGraphRendererSignature,
  applyAcademicGraphZoomLevel,
  applyAcademicGraphViewState,
  buildCytoscapeElements,
  buildCytoscapeStyles
} from './cytoscape-adapter';
import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';

const firstNodeId = `nod_${'1'.repeat(64)}`;
const secondNodeId = `nod_${'2'.repeat(64)}`;
const edgeId = `edg_${'3'.repeat(64)}`;
const layoutKey = `grp_${'a'.repeat(64)}:gix_${'b'.repeat(64)}:qiongli-layered-v1`;

describe('Cytoscape Academic Graph adapter', () => {
  it('maps the frozen layout to preset renderer elements without changing identities', () => {
    const elements = buildCytoscapeElements(graphLayout());

    expect(elements).toHaveLength(3);
    expect(elements.map((element) => element.data.id)).toEqual([
      firstNodeId,
      secondNodeId,
      edgeId
    ]);
    expect(elements[0]).toMatchObject({
      group: 'nodes',
      data: {
        id: firstNodeId,
        canonicalId: 'PROJECT-001',
        nodeType: 'project',
        layer: 'portfolio',
        spine: 'no',
        typeMark: 'PR',
        nodeShape: 'roundrectangle'
      },
      position: { x: 112, y: 82 },
      locked: true,
      grabbable: false
    });
    expect(elements[2]).toMatchObject({
      group: 'edges',
      data: {
        id: edgeId,
        source: secondNodeId,
        target: firstNodeId,
        relation: 'informs',
        relationFamily: 'evidence',
        lineStyle: 'solid',
        arrowShape: 'triangle',
        routeOffset: 0
      },
      selectable: true
    });
  });

  it('invalidates renderer elements when visual metadata changes without geometry drift', () => {
    const layout = graphLayout();
    const initial = academicGraphRendererSignature(layout);

    layout.nodes[0] = {
      ...layout.nodes[0]!,
      label: 'Project at risk',
      riskSeverity: 'high',
      riskCount: 1,
      riskKinds: ['gap']
    };
    expect(academicGraphRendererSignature(layout)).not.toBe(initial);

    const nodeUpdated = academicGraphRendererSignature(layout);
    layout.edges[0] = {
      ...layout.edges[0]!,
      confidence: 'low',
      status: 'rejected',
      riskSeverity: 'high',
      riskCount: 2,
      riskKinds: ['contradiction', 'rejected-relation']
    };
    expect(academicGraphRendererSignature(layout)).not.toBe(nodeUpdated);
  });

  it('applies exact stable-ID selection and focus to a headless renderer', () => {
    const layout = graphLayout();
    const warning = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const core = cytoscape({
      headless: true,
      styleEnabled: true,
      elements: buildCytoscapeElements(layout),
      style: buildCytoscapeStyles(),
      layout: { name: 'preset' }
    });
    try {
      applyAcademicGraphViewState(core, layout, viewState(secondNodeId));
      expect(core.getElementById(firstNodeId).selected()).toBe(false);
      expect(core.getElementById(secondNodeId).selected()).toBe(true);
      expect(core.getElementById(secondNodeId).hasClass('is-focused')).toBe(true);
      expect(core.getElementById(edgeId).hasClass('is-context-edge')).toBe(true);
      expect(core.getElementById(firstNodeId).position()).toEqual({ x: 112, y: 82 });
      expect(warning).not.toHaveBeenCalled();
    } finally {
      core.destroy();
      warning.mockRestore();
    }
  });

  it('rejects view state from another layout before renderer mutation', () => {
    const layout = graphLayout();
    const core = cytoscape({ headless: true, elements: buildCytoscapeElements(layout) });
    try {
      expect(() => applyAcademicGraphViewState(core, layout, {
        ...viewState(firstNodeId),
        layoutKey: 'stale-layout'
      })).toThrow('mismatched layout and view state');
    } finally {
      core.destroy();
    }
  });

  it('collapses semantic records into deterministic overview clusters at low zoom', () => {
    const layout = graphLayout();
    const firstClusterId = `vcl_${'4'.repeat(16)}`;
    const secondClusterId = `vcl_${'5'.repeat(16)}`;
    layout.clusters = [
      {
        clusterId: firstClusterId,
        componentId: `cmp_${'6'.repeat(16)}`,
        anchorNodeId: firstNodeId,
        nodeIds: [firstNodeId],
        label: 'Project',
        layer: 'portfolio',
        x: 24,
        y: 56,
        width: 176,
        height: 64
      },
      {
        clusterId: secondClusterId,
        componentId: `cmp_${'7'.repeat(16)}`,
        anchorNodeId: secondNodeId,
        nodeIds: [secondNodeId],
        label: 'Paper',
        layer: 'literature',
        x: 232,
        y: 56,
        width: 176,
        height: 64
      }
    ];
    layout.clusterEdges = [{
      clusterEdgeId: `vce_${'8'.repeat(16)}`,
      sourceClusterId: secondClusterId,
      targetClusterId: firstClusterId,
      edgeCount: 1,
      routeOffset: 10
    }];
    const core = cytoscape({
      headless: true,
      styleEnabled: true,
      elements: buildCytoscapeElements(layout),
      style: buildCytoscapeStyles(),
      layout: { name: 'preset' }
    });
    try {
      applyAcademicGraphViewState(core, layout, {
        ...viewState(secondNodeId),
        selectedEdgeId: edgeId,
        matchingNodeIds: [secondNodeId],
        collapsedClusterIds: [firstClusterId]
      });
      expect(core.getElementById(edgeId).selected()).toBe(true);
      expect(core.getElementById(secondNodeId).hasClass('is-search-match')).toBe(true);
      expect(core.getElementById(firstNodeId).hasClass('explicit-collapsed')).toBe(true);
      expect(core.getElementById(edgeId).hasClass('explicit-collapsed')).toBe(true);
      expect(core.getElementById(firstClusterId).hasClass('is-collapsed-cluster')).toBe(true);
      expect(core.getElementById(secondClusterId).hasClass('explicit-cluster-visible')).toBe(true);

      core.zoom(0.4);
      expect(applyAcademicGraphZoomLevel(core)).toBe(true);
      expect(core.getElementById(firstNodeId).hasClass('overview-hidden')).toBe(true);
      expect(core.getElementById(firstClusterId).hasClass('overview-visible')).toBe(true);
      expect(core.getElementById(firstClusterId).selectable()).toBe(false);

      core.zoom(1);
      expect(applyAcademicGraphZoomLevel(core)).toBe(false);
      expect(core.getElementById(firstNodeId).hasClass('overview-hidden')).toBe(false);
      expect(core.getElementById(firstClusterId).hasClass('overview-visible')).toBe(false);

      core.zoom(0.7);
      applyAcademicGraphZoomLevel(core);
      expect(core.getElementById(firstNodeId).hasClass('semantic-compact')).toBe(true);
      expect(core.getElementById(firstNodeId).hasClass('semantic-detail')).toBe(false);

      core.zoom(1.5);
      applyAcademicGraphZoomLevel(core);
      expect(core.getElementById(firstNodeId).hasClass('semantic-compact')).toBe(false);
      expect(core.getElementById(firstNodeId).hasClass('semantic-detail')).toBe(true);
    } finally {
      core.destroy();
    }
  });

  it('synchronizes legend visibility without deleting semantic records', () => {
    const layout = graphLayout();
    const core = cytoscape({
      headless: true,
      styleEnabled: true,
      elements: buildCytoscapeElements(layout),
      style: buildCytoscapeStyles(),
      layout: { name: 'preset' }
    });
    try {
      applyAcademicGraphViewState(core, layout, {
        ...viewState(null),
        hiddenNodeTypes: ['paper'],
        hiddenRelationFamilies: []
      });
      expect(core.getElementById(firstNodeId).hasClass('legend-hidden')).toBe(false);
      expect(core.getElementById(secondNodeId).hasClass('legend-hidden')).toBe(true);
      expect(core.getElementById(edgeId).hasClass('legend-hidden')).toBe(true);
      expect(core.nodes('[kind = "semantic"]')).toHaveLength(2);

      applyAcademicGraphViewState(core, layout, {
        ...viewState(null),
        hiddenNodeTypes: [],
        hiddenRelationFamilies: ['evidence']
      });
      expect(core.getElementById(secondNodeId).hasClass('legend-hidden')).toBe(false);
      expect(core.getElementById(edgeId).hasClass('legend-hidden')).toBe(true);
    } finally {
      core.destroy();
    }
  });
});

function graphLayout(): AcademicGraphLayout {
  return {
    schemaVersion: 1,
    algorithm: 'qiongli-layered-v1',
    layoutKey,
    projectionId: `grp_${'a'.repeat(64)}`,
    indexId: `gix_${'b'.repeat(64)}`,
    width: 420,
    height: 220,
    bands: [
      { layer: 'portfolio', x: 24, width: 176, nodeCount: 1 },
      { layer: 'literature', x: 232, width: 176, nodeCount: 1 }
    ],
    nodes: [
      {
        nodeId: firstNodeId,
        canonicalId: 'PROJECT-001',
        label: 'Project',
        nodeType: 'project',
        layer: 'portfolio',
        column: 0,
        row: 0,
        x: 36,
        y: 56,
        width: 152,
        height: 52
      },
      {
        nodeId: secondNodeId,
        canonicalId: 'PAPER-001',
        label: 'Paper',
        nodeType: 'paper',
        layer: 'literature',
        column: 1,
        row: 0,
        x: 244,
        y: 56,
        width: 152,
        height: 52
      }
    ],
    edges: [{
      edgeId,
      sourceNodeId: secondNodeId,
      targetNodeId: firstNodeId,
      relation: 'informs',
      x1: 244,
      y1: 82,
      x2: 188,
      y2: 82
    }]
  };
}

function viewState(nodeId: string | null): AcademicGraphViewState {
  return {
    schemaVersion: 1,
    layoutKey,
    viewportMode: 'scroll',
    selectedNodeId: nodeId,
    selectedEdgeId: null,
    focusNodeId: nodeId,
    direction: 'both',
    matchingNodeIds: [],
    collapsedClusterIds: [],
    hiddenNodeTypes: [],
    hiddenRelationFamilies: []
  };
}
