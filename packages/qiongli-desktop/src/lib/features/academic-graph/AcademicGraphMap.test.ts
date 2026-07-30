import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import AcademicGraphMap from './AcademicGraphMap.svelte';
import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';

const firstNodeId = `nod_${'1'.repeat(64)}`;
const secondNodeId = `nod_${'2'.repeat(64)}`;
const layoutKey = `grp_${'a'.repeat(64)}:gix_${'b'.repeat(64)}:qiongli-layered-v1`;

describe('AcademicGraphMap', () => {
  it('keeps visual node selection keyboard-operable and synchronized by stable ID', async () => {
    const onSelect = vi.fn();
    const onSelectEdge = vi.fn();
    const { container } = render(AcademicGraphMap, {
      layout: graphLayout(),
      viewState: viewState(),
      onSelect,
      onSelectEdge
    });

    expect(screen.getByRole('region', { name: 'Scrollable Academic Graph map' }))
      .toBeVisible();
    expect(screen.getByText('Layered layout v1')).toBeVisible();
    const selected = screen.getByRole('button', {
      name: 'Project; type Project; layer Portfolio'
    });
    expect(selected).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByText('Project selected in the graph map.')).toBeVisible();

    await fireEvent.click(screen.getByRole('button', {
      name: 'Paper; type Paper; layer Literature'
    }));
    expect(onSelect).toHaveBeenCalledOnce();
    expect(onSelect).toHaveBeenCalledWith(secondNodeId);

    const firstEdge = screen.getByRole('button', {
      name: 'Paper; relation Informs; Project'
    });
    await fireEvent.keyDown(firstEdge, { key: 'Enter' });
    expect(onSelectEdge).toHaveBeenCalledOnce();
    expect(onSelectEdge).toHaveBeenCalledWith(`edg_${'3'.repeat(64)}`);

    expect(Array.from(container.querySelectorAll('.edges .edge-visual'), (edge) =>
      edge.getAttribute('marker-end'))).toEqual([
      'url(#academic-graph-arrow-triangle)',
      'url(#academic-graph-arrow-tee)',
      'url(#academic-graph-arrow-diamond)',
      'url(#academic-graph-arrow-square)'
    ]);
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
    edges: [
      {
        edgeId: `edg_${'3'.repeat(64)}`,
        sourceNodeId: secondNodeId,
        targetNodeId: firstNodeId,
        relation: 'informs',
        x1: 320,
        y1: 82,
        x2: 112,
        y2: 82
      },
      {
        edgeId: `edg_${'4'.repeat(64)}`,
        sourceNodeId: secondNodeId,
        targetNodeId: firstNodeId,
        relation: 'contradicts',
        x1: 320,
        y1: 88,
        x2: 112,
        y2: 88
      },
      {
        edgeId: `edg_${'5'.repeat(64)}`,
        sourceNodeId: secondNodeId,
        targetNodeId: firstNodeId,
        relation: 'derived-from',
        x1: 320,
        y1: 94,
        x2: 112,
        y2: 94
      },
      {
        edgeId: `edg_${'6'.repeat(64)}`,
        sourceNodeId: secondNodeId,
        targetNodeId: firstNodeId,
        relation: 'contains',
        x1: 320,
        y1: 100,
        x2: 112,
        y2: 100
      }
    ]
  };
}

function viewState(): AcademicGraphViewState {
  return {
    schemaVersion: 1,
    layoutKey,
    viewportMode: 'scroll',
    selectedNodeId: firstNodeId,
    selectedEdgeId: null,
    focusNodeId: firstNodeId,
    direction: 'both',
    matchingNodeIds: [],
    collapsedClusterIds: [],
    hiddenNodeTypes: [],
    hiddenRelationFamilies: []
  };
}
