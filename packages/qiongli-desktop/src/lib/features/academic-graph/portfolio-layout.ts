import type { AcademicGraphPortfolioSnapshot } from '@qiongli/app-api';

export interface AcademicGraphPortfolioLayoutNode {
  nodeId: string;
  label: string;
  nodeType: AcademicGraphPortfolioSnapshot['nodes'][number]['nodeType'];
  identityScope: AcademicGraphPortfolioSnapshot['nodes'][number]['identityScope'];
  x: number;
  y: number;
}

export interface AcademicGraphPortfolioLayoutEdge {
  edgeId: string;
  sourceNodeId: string;
  targetNodeId: string;
  relation: AcademicGraphPortfolioSnapshot['edges'][number]['relation'];
}

export interface AcademicGraphPortfolioLayout {
  portfolioId: string;
  width: number;
  height: number;
  nodes: AcademicGraphPortfolioLayoutNode[];
  edges: AcademicGraphPortfolioLayoutEdge[];
}

export function buildAcademicGraphPortfolioLayout(
  portfolio: AcademicGraphPortfolioSnapshot
): AcademicGraphPortfolioLayout {
  const projects = portfolio.nodes
    .filter((node) => node.identityScope === 'project')
    .sort(compareNodes);
  const shared = portfolio.nodes
    .filter((node) => node.identityScope === 'global')
    .sort(compareNodes);
  const nodes = [
    ...projects.map((node, index) => ({
      nodeId: node.nodeId,
      label: node.label,
      nodeType: node.nodeType,
      identityScope: node.identityScope,
      x: 32,
      y: 28 + index * 86
    })),
    ...shared.map((node, index) => ({
      nodeId: node.nodeId,
      label: node.label,
      nodeType: node.nodeType,
      identityScope: node.identityScope,
      x: 420,
      y: 28 + index * 86
    }))
  ];
  const nodeIds = new Set(nodes.map((node) => node.nodeId));
  if (nodes.length !== portfolio.nodeCount
    || portfolio.edges.some((edge) => !nodeIds.has(edge.sourceNodeId) || !nodeIds.has(edge.targetNodeId))) {
    throw new Error('academic graph portfolio layout received inconsistent topology');
  }
  return {
    portfolioId: portfolio.portfolioId,
    width: 620,
    height: Math.max(150, 56 + Math.max(projects.length, shared.length) * 86),
    nodes,
    edges: [...portfolio.edges]
      .sort((left, right) => left.edgeId.localeCompare(right.edgeId))
      .map((edge) => ({
        edgeId: edge.edgeId,
        sourceNodeId: edge.sourceNodeId,
        targetNodeId: edge.targetNodeId,
        relation: edge.relation
      }))
  };
}

function compareNodes(
  left: AcademicGraphPortfolioSnapshot['nodes'][number],
  right: AcademicGraphPortfolioSnapshot['nodes'][number]
): number {
  return left.nodeType.localeCompare(right.nodeType)
    || left.canonicalId.localeCompare(right.canonicalId)
    || left.nodeId.localeCompare(right.nodeId);
}
