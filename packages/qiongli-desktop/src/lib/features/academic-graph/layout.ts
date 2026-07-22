import type {
  AcademicGraphDirection,
  AcademicGraphLayer,
  AcademicGraphNode,
  AcademicGraphNodeType,
  AcademicGraphQueryResult,
  AcademicGraphRelation
} from '@qiongli/app-api';

import type { AcademicGraphRiskOverlay, AcademicGraphRiskSeverity } from './risk';

export const ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION = 1 as const;
export const ACADEMIC_GRAPH_LAYOUT_ALGORITHM = 'qiongli-layered-v1' as const;

const bandOrder = [
  'portfolio',
  'literature',
  'idea-decision',
  'argument',
  'manuscript',
  'combined'
] as const satisfies readonly AcademicGraphLayer[];

const nodeTypeOrder = [
  'project',
  'paper',
  'literature-cluster',
  'concept',
  'gap',
  'research-question',
  'idea',
  'decision',
  'method',
  'claim',
  'evidence',
  'contribution',
  'manuscript-section',
  'task',
  'artifact'
] as const satisfies readonly AcademicGraphNodeType[];

const HORIZONTAL_PADDING = 24;
const BAND_WIDTH = 176;
const BAND_GAP = 32;
const BAND_HEADER_HEIGHT = 56;
const NODE_WIDTH = 152;
const NODE_HEIGHT = 52;
const NODE_GAP = 22;
const BOTTOM_PADDING = 24;
const MINIMUM_WIDTH = 420;
const MINIMUM_HEIGHT = 220;

export interface AcademicGraphLayoutBand {
  layer: AcademicGraphLayer;
  x: number;
  width: number;
  nodeCount: number;
}

export interface AcademicGraphLayoutNode {
  nodeId: string;
  canonicalId: string;
  label: string;
  nodeType: AcademicGraphNodeType;
  layer: AcademicGraphLayer;
  column: number;
  row: number;
  x: number;
  y: number;
  width: number;
  height: number;
  riskSeverity?: AcademicGraphRiskSeverity | null;
  riskCount?: number;
}

export interface AcademicGraphLayoutEdge {
  edgeId: string;
  sourceNodeId: string;
  targetNodeId: string;
  relation: AcademicGraphRelation;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  riskSeverity?: AcademicGraphRiskSeverity | null;
  riskCount?: number;
}

export interface AcademicGraphLayout {
  schemaVersion: typeof ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION;
  algorithm: typeof ACADEMIC_GRAPH_LAYOUT_ALGORITHM;
  layoutKey: string;
  projectionId: string;
  indexId: string;
  width: number;
  height: number;
  bands: AcademicGraphLayoutBand[];
  nodes: AcademicGraphLayoutNode[];
  edges: AcademicGraphLayoutEdge[];
}

export interface AcademicGraphViewState {
  schemaVersion: typeof ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION;
  layoutKey: string;
  viewportMode: 'scroll';
  selectedNodeId: string | null;
  focusNodeId: string | null;
  direction: AcademicGraphDirection;
}

export interface AcademicGraphViewStateInput {
  selectedNodeId: string | null;
  focusNodeId: string | null;
  direction: AcademicGraphDirection;
}

/**
 * Produces rebuildable geometry from an already validated, bounded query result.
 * Semantic identity remains owned by the Rust projection and never depends on
 * these coordinates.
 */
export function buildAcademicGraphLayout(
  result: AcademicGraphQueryResult,
  riskOverlay?: AcademicGraphRiskOverlay
): AcademicGraphLayout {
  if (riskOverlay
    && (riskOverlay.projectionId !== result.projectionId || riskOverlay.indexId !== result.indexId)) {
    throw new Error('academic graph layout received a risk overlay from another projection');
  }
  const nodeRisks = new Map<string, AcademicGraphRiskSeverity[]>();
  const edgeRisks = new Map<string, AcademicGraphRiskSeverity[]>();
  for (const risk of riskOverlay?.entries ?? []) {
    for (const nodeId of risk.affectedNodeIds) {
      const values = nodeRisks.get(nodeId) ?? [];
      values.push(risk.severity);
      nodeRisks.set(nodeId, values);
    }
    if (risk.entity.kind === 'edge') {
      const values = edgeRisks.get(risk.entity.id) ?? [];
      values.push(risk.severity);
      edgeRisks.set(risk.entity.id, values);
    }
  }
  const grouped = new Map<AcademicGraphLayer, AcademicGraphNode[]>();
  for (const layer of bandOrder) grouped.set(layer, []);
  for (const node of result.nodes) grouped.get(primaryLayer(node))!.push(node);

  const activeLayers = bandOrder.filter((layer) => grouped.get(layer)!.length > 0);
  const bands: AcademicGraphLayoutBand[] = [];
  const nodes: AcademicGraphLayoutNode[] = [];

  for (const [column, layer] of activeLayers.entries()) {
    const bandX = HORIZONTAL_PADDING + column * (BAND_WIDTH + BAND_GAP);
    const orderedNodes = [...grouped.get(layer)!].sort(compareNodes);
    bands.push({ layer, x: bandX, width: BAND_WIDTH, nodeCount: orderedNodes.length });
    for (const [row, node] of orderedNodes.entries()) {
      nodes.push({
        nodeId: node.nodeId,
        canonicalId: node.canonicalId,
        label: node.label,
        nodeType: node.nodeType,
        layer,
        column,
        row,
        x: bandX + (BAND_WIDTH - NODE_WIDTH) / 2,
        y: BAND_HEADER_HEIGHT + row * (NODE_HEIGHT + NODE_GAP),
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
        riskSeverity: highestRisk(nodeRisks.get(node.nodeId)),
        riskCount: nodeRisks.get(node.nodeId)?.length ?? 0
      });
    }
  }

  const nodeById = new Map(nodes.map((node) => [node.nodeId, node]));
  const edges = [...result.edges].sort((left, right) => compareText(left.edgeId, right.edgeId))
    .map((edge): AcademicGraphLayoutEdge => {
      const source = nodeById.get(edge.sourceNodeId);
      const target = nodeById.get(edge.targetNodeId);
      if (!source || !target) {
        throw new Error('academic graph layout received an edge with an unavailable endpoint');
      }
      const endpoints = connectNodeBoundaries(source, target);
      return {
        edgeId: edge.edgeId,
        sourceNodeId: edge.sourceNodeId,
        targetNodeId: edge.targetNodeId,
        relation: edge.relation,
        riskSeverity: highestRisk(edgeRisks.get(edge.edgeId)),
        riskCount: edgeRisks.get(edge.edgeId)?.length ?? 0,
        ...endpoints
      };
    });

  const columnWidth = activeLayers.length === 0
    ? 0
    : activeLayers.length * BAND_WIDTH + (activeLayers.length - 1) * BAND_GAP;
  const maximumRows = Math.max(0, ...bands.map((band) => band.nodeCount));
  const rowsHeight = maximumRows === 0
    ? 0
    : maximumRows * NODE_HEIGHT + (maximumRows - 1) * NODE_GAP;

  return {
    schemaVersion: ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION,
    algorithm: ACADEMIC_GRAPH_LAYOUT_ALGORITHM,
    layoutKey: `${result.projectionId}:${result.indexId}:${ACADEMIC_GRAPH_LAYOUT_ALGORITHM}`,
    projectionId: result.projectionId,
    indexId: result.indexId,
    width: Math.max(MINIMUM_WIDTH, HORIZONTAL_PADDING * 2 + columnWidth),
    height: Math.max(MINIMUM_HEIGHT, BAND_HEADER_HEIGHT + rowsHeight + BOTTOM_PADDING),
    bands,
    nodes,
    edges
  };
}

function highestRisk(values: AcademicGraphRiskSeverity[] | undefined): AcademicGraphRiskSeverity | null {
  if (!values || values.length === 0) return null;
  return values.includes('high') ? 'high' : 'medium';
}

/** Normalizes ephemeral UI state to the nodes exposed by one exact layout. */
export function buildAcademicGraphViewState(
  layout: AcademicGraphLayout,
  input: AcademicGraphViewStateInput
): AcademicGraphViewState {
  const nodeIds = new Set(layout.nodes.map((node) => node.nodeId));
  return {
    schemaVersion: ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION,
    layoutKey: layout.layoutKey,
    viewportMode: 'scroll',
    selectedNodeId: input.selectedNodeId && nodeIds.has(input.selectedNodeId)
      ? input.selectedNodeId
      : null,
    focusNodeId: input.focusNodeId && nodeIds.has(input.focusNodeId)
      ? input.focusNodeId
      : null,
    direction: input.direction
  };
}

function primaryLayer(node: AcademicGraphNode): AcademicGraphLayer {
  return bandOrder.find((layer) => node.layers.includes(layer)) ?? 'combined';
}

function compareNodes(left: AcademicGraphNode, right: AcademicGraphNode): number {
  return nodeTypeOrder.indexOf(left.nodeType) - nodeTypeOrder.indexOf(right.nodeType)
    || compareText(left.canonicalId, right.canonicalId)
    || compareText(left.nodeId, right.nodeId);
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function connectNodeBoundaries(
  source: AcademicGraphLayoutNode,
  target: AcademicGraphLayoutNode
): Pick<AcademicGraphLayoutEdge, 'x1' | 'y1' | 'x2' | 'y2'> {
  const sourceCenter = { x: source.x + source.width / 2, y: source.y + source.height / 2 };
  const targetCenter = { x: target.x + target.width / 2, y: target.y + target.height / 2 };
  const deltaX = targetCenter.x - sourceCenter.x;
  const deltaY = targetCenter.y - sourceCenter.y;
  if (deltaX === 0 && deltaY === 0) {
    return {
      x1: sourceCenter.x,
      y1: source.y,
      x2: targetCenter.x,
      y2: target.y
    };
  }
  const sourceScale = boundaryScale(deltaX, deltaY, source.width, source.height);
  const targetScale = boundaryScale(deltaX, deltaY, target.width, target.height);
  return {
    x1: sourceCenter.x + deltaX * sourceScale,
    y1: sourceCenter.y + deltaY * sourceScale,
    x2: targetCenter.x - deltaX * targetScale,
    y2: targetCenter.y - deltaY * targetScale
  };
}

function boundaryScale(
  deltaX: number,
  deltaY: number,
  width: number,
  height: number
): number {
  const horizontal = deltaX === 0 ? Number.POSITIVE_INFINITY : width / 2 / Math.abs(deltaX);
  const vertical = deltaY === 0 ? Number.POSITIVE_INFINITY : height / 2 / Math.abs(deltaY);
  return Math.min(horizontal, vertical);
}
