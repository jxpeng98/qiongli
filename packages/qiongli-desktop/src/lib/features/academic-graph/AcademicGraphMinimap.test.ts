import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import AcademicGraphMinimap from './AcademicGraphMinimap.svelte';
import type { AcademicGraphLayout } from './layout';

describe('AcademicGraphMinimap', () => {
  it('exposes an accessible viewport summary without becoming a second authority', () => {
    const { container } = render(AcademicGraphMinimap, {
      layout: graphLayout(),
      viewport: {
        zoom: 1,
        overview: false,
        extent: { x1: 20, y1: 30, x2: 220, y2: 150 }
      }
    });

    expect(screen.getByRole('img', {
      name: 'Academic Graph minimap showing the current viewport at 100 percent zoom.'
    })).toBeVisible();
    expect(container.querySelectorAll('rect.viewport')).toHaveLength(1);
    expect(container.querySelectorAll('rect:not(.viewport)')).toHaveLength(2);
  });
});

function graphLayout(): AcademicGraphLayout {
  return {
    schemaVersion: 1,
    algorithm: 'qiongli-topology-v2',
    layoutKey: 'layout',
    projectionId: `grp_${'a'.repeat(64)}`,
    indexId: `gix_${'b'.repeat(64)}`,
    width: 420,
    height: 220,
    bands: [{ layer: 'argument', x: 12, width: 396, nodeCount: 1 }],
    nodes: [{
      nodeId: `nod_${'1'.repeat(64)}`,
      canonicalId: 'CLM-001',
      label: 'Claim',
      nodeType: 'claim',
      layer: 'argument',
      column: 0,
      row: 0,
      x: 80,
      y: 70,
      width: 152,
      height: 52
    }],
    edges: []
  };
}
