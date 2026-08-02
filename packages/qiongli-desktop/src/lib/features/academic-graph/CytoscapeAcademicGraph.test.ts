import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { AcademicGraphRendererOptions } from './cytoscape-adapter';
import CytoscapeAcademicGraph from './CytoscapeAcademicGraph.svelte';
import type { AcademicGraphLayout, AcademicGraphViewState } from './layout';

const rendererMocks = vi.hoisted(() => ({
  mount: vi.fn(),
  update: vi.fn(),
  fit: vi.fn(),
  fitSelection: vi.fn(),
  zoomIn: vi.fn(),
  zoomOut: vi.fn(),
  destroy: vi.fn()
}));

vi.mock('./cytoscape-adapter', () => ({
  mountAcademicGraphRenderer: rendererMocks.mount
}));

const firstNodeId = `nod_${'1'.repeat(64)}`;
const secondNodeId = `nod_${'2'.repeat(64)}`;
const layoutKey = `grp_${'a'.repeat(64)}:gix_${'b'.repeat(64)}:qiongli-topology-v2:standard`;

describe('CytoscapeAcademicGraph', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    rendererMocks.mount.mockReturnValue({
      update: rendererMocks.update,
      fit: rendererMocks.fit,
      fitSelection: rendererMocks.fitSelection,
      zoomIn: rendererMocks.zoomIn,
      zoomOut: rendererMocks.zoomOut,
      destroy: rendererMocks.destroy
    });
  });

  it('lazy-mounts the renderer and bridges stable-ID selection and viewport reset', async () => {
    const onSelect = vi.fn();
    const onSelectEdge = vi.fn();
    render(CytoscapeAcademicGraph, {
      layout: graphLayout(),
      viewState: viewState(),
      onSelect,
      onSelectEdge
    });

    await waitFor(() => expect(screen.getByText('Topology layout v2')).toBeVisible());
    expect(rendererMocks.mount).toHaveBeenCalledOnce();
    const options = rendererMocks.mount.mock.calls[0]![0] as AcademicGraphRendererOptions;
    expect(options.layout.layoutKey).toBe(layoutKey);
    expect(options.container).toBeInstanceOf(HTMLDivElement);

    await options.onSelect(secondNodeId);
    expect(onSelect).toHaveBeenCalledWith(secondNodeId);
    await options.onSelectEdge?.(`edg_${'3'.repeat(64)}`);
    expect(onSelectEdge).toHaveBeenCalledWith(`edg_${'3'.repeat(64)}`);

    await fireEvent.click(screen.getByRole('button', { name: 'Zoom out' }));
    expect(rendererMocks.zoomOut).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByRole('button', { name: 'Zoom in' }));
    expect(rendererMocks.zoomIn).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByRole('button', { name: 'Fit selection' }));
    expect(rendererMocks.fitSelection).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByRole('button', { name: 'Fit graph' }));
    expect(rendererMocks.fit).toHaveBeenCalledOnce();
    expect(screen.getByRole('link', { name: 'Use node table' }))
      .toHaveAttribute('href', '#graph-nodes-title');

    await fireEvent.click(screen.getByText('Visual key'));
    await fireEvent.click(screen.getByRole('button', { name: 'Toggle Paper nodes' }));
    await waitFor(() => expect(rendererMocks.update).toHaveBeenLastCalledWith(
      expect.anything(),
      expect.objectContaining({ hiddenNodeTypes: ['paper'] })
    ));

    const canvas = screen.getByRole('button', {
      name: 'Interactive Academic Graph canvas'
    });
    expect(canvas).toBeEnabled();
    await fireEvent.keyDown(canvas, { key: '+' });
    await fireEvent.keyDown(canvas, { key: '-' });
    await fireEvent.keyDown(canvas, { key: '0' });
    await fireEvent.keyDown(canvas, { key: 'f' });
    expect(rendererMocks.zoomIn).toHaveBeenCalledTimes(2);
    expect(rendererMocks.zoomOut).toHaveBeenCalledTimes(2);
    expect(rendererMocks.fit).toHaveBeenCalledTimes(2);
    expect(rendererMocks.fitSelection).toHaveBeenCalledTimes(2);
  });

  it('falls back to the deterministic semantic map when mounting fails', async () => {
    rendererMocks.mount.mockImplementationOnce(() => {
      throw new Error('renderer unavailable');
    });
    render(CytoscapeAcademicGraph, {
      layout: graphLayout(),
      viewState: viewState(),
      onSelect: vi.fn(),
      onSelectEdge: vi.fn()
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

  it('labels a budget fallback without presenting it as topology v2', async () => {
    render(CytoscapeAcademicGraph, {
      layout: {
        ...graphLayout(),
        algorithm: 'qiongli-layered-v1',
        fallbackReason: 'layout-budget-exceeded',
        layoutKey: `${layoutKey}:fallback`
      },
      viewState: {
        ...viewState(),
        layoutKey: `${layoutKey}:fallback`
      },
      onSelect: vi.fn(),
      onSelectEdge: vi.fn()
    });

    await waitFor(() => expect(screen.getByText('Layered fallback')).toBeVisible());
    expect(screen.queryByText('Topology layout v2')).not.toBeInTheDocument();
  });
});

function graphLayout(): AcademicGraphLayout {
  return {
    schemaVersion: 1,
    algorithm: 'qiongli-topology-v2',
    layoutKey,
    projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
    projectionId: `grp_${'a'.repeat(64)}`,
    indexId: `gix_${'b'.repeat(64)}`,
    viewportClass: 'standard',
    fallbackReason: null,
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
    selectedEdgeId: null,
    focusNodeId: firstNodeId,
    direction: 'both',
    matchingNodeIds: [],
    collapsedClusterIds: [],
    hiddenNodeTypes: [],
    hiddenRelationFamilies: []
  };
}
