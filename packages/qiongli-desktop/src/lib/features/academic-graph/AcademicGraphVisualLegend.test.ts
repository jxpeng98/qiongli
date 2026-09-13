import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import AcademicGraphVisualLegend from './AcademicGraphVisualLegend.svelte';
import type { AcademicGraphLayout } from './layout';

describe('AcademicGraphVisualLegend', () => {
  it('exposes shape and relation visibility as pressed keyboard controls', async () => {
    const onToggleNodeType = vi.fn();
    const onToggleRelationFamily = vi.fn();
    render(AcademicGraphVisualLegend, {
      layout: graphLayout(),
      hiddenNodeTypes: ['paper'],
      hiddenRelationFamilies: [],
      onToggleNodeType,
      onToggleRelationFamily
    });

    await fireEvent.click(screen.getByText('Visual key'));
    const project = screen.getByRole('button', { name: 'Toggle Project nodes' });
    const paper = screen.getByRole('button', { name: 'Toggle Paper nodes' });
    expect(project).toHaveAttribute('aria-pressed', 'true');
    expect(paper).toHaveAttribute('aria-pressed', 'false');

    await fireEvent.click(paper);
    expect(onToggleNodeType).toHaveBeenCalledWith('paper');
    await fireEvent.click(screen.getByRole('button', {
      name: 'Toggle Evidence relation family'
    }));
    expect(onToggleRelationFamily).toHaveBeenCalledWith('evidence');
  });
});

function graphLayout(): AcademicGraphLayout {
  return {
    schemaVersion: 1,
    algorithm: 'qiongli-layered-v1',
    layoutKey: 'layout',
    projectionId: `grp_${'a'.repeat(64)}`,
    indexId: `gix_${'b'.repeat(64)}`,
    width: 420,
    height: 220,
    bands: [],
    nodes: [
      {
        nodeId: `nod_${'1'.repeat(64)}`,
        canonicalId: 'PROJECT-001',
        label: 'Project',
        nodeType: 'project',
        layer: 'portfolio',
        column: 0,
        row: 0,
        x: 0,
        y: 0,
        width: 152,
        height: 52
      },
      {
        nodeId: `nod_${'2'.repeat(64)}`,
        canonicalId: 'PAPER-001',
        label: 'Paper',
        nodeType: 'paper',
        layer: 'literature',
        column: 1,
        row: 0,
        x: 200,
        y: 0,
        width: 152,
        height: 52
      }
    ],
    edges: [{
      edgeId: `edg_${'3'.repeat(64)}`,
      sourceNodeId: `nod_${'2'.repeat(64)}`,
      targetNodeId: `nod_${'1'.repeat(64)}`,
      relation: 'supports',
      x1: 0,
      y1: 0,
      x2: 200,
      y2: 0
    }]
  };
}
