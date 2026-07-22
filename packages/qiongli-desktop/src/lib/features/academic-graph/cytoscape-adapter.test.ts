import cytoscape from 'cytoscape';
import { describe, expect, it, vi } from 'vitest';

import {
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
        layer: 'portfolio'
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
        relation: 'informs'
      },
      selectable: false
    });
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

function viewState(nodeId: string): AcademicGraphViewState {
  return {
    schemaVersion: 1,
    layoutKey,
    viewportMode: 'scroll',
    selectedNodeId: nodeId,
    focusNodeId: nodeId,
    direction: 'both'
  };
}
