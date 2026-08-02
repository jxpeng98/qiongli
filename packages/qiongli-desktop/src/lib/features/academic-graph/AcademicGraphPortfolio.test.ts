import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import type { AcademicGraphPortfolioSnapshot } from '@qiongli/app-api';

import AcademicGraphPortfolio from './AcademicGraphPortfolio.svelte';
import { buildAcademicGraphPortfolioLayout } from './portfolio-layout';

const firstProject = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const secondProject = 'prj_118f4d5a3b2c71008a9b0c1d2e3f4052';
const firstProjection = `grp_${'a'.repeat(64)}`;
const secondProjection = `grp_${'b'.repeat(64)}`;
const firstNode = `pnd_${'1'.repeat(64)}`;
const secondNode = `pnd_${'2'.repeat(64)}`;
const paperNode = `pnd_${'3'.repeat(64)}`;

const portfolio: AcademicGraphPortfolioSnapshot = {
  schemaVersion: 1,
  documentKind: 'qiongli-academic-graph-portfolio',
  portfolioId: `gpf_${'4'.repeat(64)}`,
  libraryRevision: 2,
  projectCount: 2,
  includedProjectCount: 2,
  skippedProjectCount: 0,
  nodeCount: 3,
  edgeCount: 2,
  projects: [
    { projectId: firstProject, displayName: 'First paper', lifecycle: 'active', health: 'ready', included: true, projectRevision: 1, projectionId: firstProjection },
    { projectId: secondProject, displayName: 'Second paper', lifecycle: 'active', health: 'ready', included: true, projectRevision: 1, projectionId: secondProjection }
  ],
  nodes: [
    projectNode(firstNode, firstProject, firstProjection, 'First paper', '1'),
    projectNode(secondNode, secondProject, secondProjection, 'Second paper', '2'),
    {
      nodeId: paperNode,
      nodeType: 'paper',
      identityScope: 'global',
      canonicalId: 'doi:10.1000/shared',
      label: 'Shared exact source',
      projectIds: [firstProject, secondProject],
      occurrences: [
        occurrence(firstProject, firstProjection, '3', 'Shared exact source'),
        occurrence(secondProject, secondProjection, '4', 'A different display label')
      ]
    }
  ],
  edges: [
    sharedEdge('1', firstNode, firstProject, firstProjection),
    sharedEdge('2', secondNode, secondProject, secondProjection)
  ]
};

describe('AcademicGraphPortfolio', () => {
  it('uses deterministic geometry independent of incoming node order', () => {
    const forward = buildAcademicGraphPortfolioLayout(portfolio);
    const reverse = buildAcademicGraphPortfolioLayout({
      ...portfolio,
      nodes: [...portfolio.nodes].reverse(),
      edges: [...portfolio.edges].reverse()
    });
    expect(forward).toEqual(reverse);
    expect(forward.nodes.filter((node) => node.identityScope === 'project').map((node) => node.x))
      .toEqual([32, 32]);
    expect(forward.nodes.find((node) => node.identityScope === 'global')?.x).toBe(420);
  });

  it('renders the semantic portfolio oracle and opens an exact contributing project', async () => {
    const onOpenProject = vi.fn();
    render(AcademicGraphPortfolio, { portfolio, onOpenProject });

    expect(screen.getByRole('heading', { name: 'Portfolio topology' })).toBeVisible();
    expect(screen.getAllByText('Shared exact source').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Shares source')).toHaveLength(2);
    await fireEvent.click(screen.getAllByText('2 source occurrences')[0]!);
    await fireEvent.click(screen.getAllByRole('button', { name: 'Open project graph' })[0]!);
    expect(onOpenProject).toHaveBeenCalledWith(firstProject);
  });
});

function projectNode(
  nodeId: string,
  projectId: string,
  projectionId: string,
  label: string,
  graphNode: string
): AcademicGraphPortfolioSnapshot['nodes'][number] {
  return {
    nodeId,
    nodeType: 'project',
    identityScope: 'project',
    canonicalId: projectId,
    label,
    projectIds: [projectId],
    occurrences: [{
      projectId,
      projectionId,
      graphNodeId: `nod_${graphNode.repeat(64)}`,
      label,
      artifactPath: 'context/project_manifest.json',
      sourceAnchor: 'project'
    }]
  };
}

function occurrence(
  projectId: string,
  projectionId: string,
  graphNode: string,
  label: string
): AcademicGraphPortfolioSnapshot['nodes'][number]['occurrences'][number] {
  return {
    projectId,
    projectionId,
    graphNodeId: `nod_${graphNode.repeat(64)}`,
    label,
    artifactPath: 'literature/literature_map.md',
    sourceAnchor: 'paper:shared'
  };
}

function sharedEdge(
  id: string,
  sourceNodeId: string,
  projectId: string,
  projectionId: string
): AcademicGraphPortfolioSnapshot['edges'][number] {
  return {
    edgeId: `ped_${id.repeat(64)}`,
    sourceNodeId,
    relation: 'shares-source',
    targetNodeId: paperNode,
    sharedCanonicalId: 'doi:10.1000/shared',
    rationale: 'The registered project projections contain the same exact global paper identifier.',
    evidenceLimit: 'Exact identifier reuse does not imply identical conclusions.',
    inferenceStrength: 'direct_evidence',
    confidence: 'high',
    status: 'observed',
    origins: [{
      projectId,
      projectionId,
      graphEdgeId: null,
      artifactPath: 'literature/literature_map.md',
      sourceAnchor: 'paper:shared'
    }]
  };
}
