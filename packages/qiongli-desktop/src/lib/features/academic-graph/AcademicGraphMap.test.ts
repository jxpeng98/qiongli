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
    render(AcademicGraphMap, {
      layout: graphLayout(),
      viewState: viewState(),
      onSelect
    });

    expect(screen.getByRole('region', { name: 'Scrollable Academic Graph map' }))
      .toBeVisible();
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
      edgeId: `edg_${'3'.repeat(64)}`,
      sourceNodeId: secondNodeId,
      targetNodeId: firstNodeId,
      relation: 'informs',
      x1: 320,
      y1: 82,
      x2: 112,
      y2: 82
    }]
  };
}

function viewState(): AcademicGraphViewState {
  return {
    schemaVersion: 1,
    layoutKey,
    viewportMode: 'scroll',
    selectedNodeId: firstNodeId,
    focusNodeId: firstNodeId,
    direction: 'both'
  };
}
