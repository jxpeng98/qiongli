import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { AcademicGraphRendererOptions } from './cytoscape-adapter';
import CytoscapeAcademicGraph from './CytoscapeAcademicGraph.svelte';
import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';

const rendererMocks = vi.hoisted(() => ({
  mount: vi.fn(),
  update: vi.fn(),
  fit: vi.fn(),
  destroy: vi.fn()
}));

vi.mock('./cytoscape-adapter', () => ({
  mountAcademicGraphRenderer: rendererMocks.mount
}));

const firstNodeId = `nod_${'1'.repeat(64)}`;
const secondNodeId = `nod_${'2'.repeat(64)}`;
const layoutKey = `grp_${'a'.repeat(64)}:gix_${'b'.repeat(64)}:qiongli-layered-v1`;

describe('CytoscapeAcademicGraph', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    rendererMocks.mount.mockReturnValue({
      update: rendererMocks.update,
      fit: rendererMocks.fit,
      destroy: rendererMocks.destroy
    });
  });

  it('lazy-mounts the renderer and bridges stable-ID selection and viewport reset', async () => {
    const onSelect = vi.fn();
    render(CytoscapeAcademicGraph, {
      layout: graphLayout(),
      viewState: viewState(),
      onSelect
    });

    await waitFor(() => expect(screen.getByText('Interactive canvas')).toBeVisible());
    expect(rendererMocks.mount).toHaveBeenCalledOnce();
    const options = rendererMocks.mount.mock.calls[0]![0] as AcademicGraphRendererOptions;
    expect(options.layout.layoutKey).toBe(layoutKey);
    expect(options.container).toBeInstanceOf(HTMLDivElement);

    await options.onSelect(secondNodeId);
    expect(onSelect).toHaveBeenCalledWith(secondNodeId);

    await fireEvent.click(screen.getByRole('button', { name: 'Fit graph' }));
    expect(rendererMocks.fit).toHaveBeenCalledOnce();
    expect(screen.getByRole('link', { name: 'Use node table' }))
      .toHaveAttribute('href', '#graph-nodes-title');
  });

  it('falls back to the deterministic semantic map when mounting fails', async () => {
    rendererMocks.mount.mockImplementationOnce(() => {
      throw new Error('renderer unavailable');
    });
    render(CytoscapeAcademicGraph, {
      layout: graphLayout(),
      viewState: viewState(),
      onSelect: vi.fn()
    });

    await waitFor(() => expect(screen.getByText(
      'The interactive renderer is unavailable. The deterministic semantic map and table remain available.'
    )).toBeVisible());
    expect(screen.getByRole('region', { name: 'Scrollable Academic Graph map' }))
      .toBeVisible();
    expect(screen.getByRole('button', {
      name: 'Project; type Project; layer Portfolio'
    })).toBeVisible();
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
      x1: 244,
      y1: 82,
      x2: 188,
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
