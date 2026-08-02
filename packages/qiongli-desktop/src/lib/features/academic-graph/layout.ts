import type {
  AcademicGraphDirection,
  AcademicGraphEdge,
  AcademicGraphLayer,
  AcademicGraphNode,
  AcademicGraphNodeType,
  AcademicGraphQueryResult,
  AcademicGraphRelation
} from '@qiongli/app-api';

import type {
  AcademicGraphRiskKind,
  AcademicGraphRiskOverlay,
  AcademicGraphRiskSeverity
} from './risk';
import type { AcademicGraphRelationFamily } from './visual-language';

export const ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION = 1 as const;
export const ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM = 'qiongli-layered-v1' as const;
export const ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM = 'qiongli-topology-v2' as const;
export const ACADEMIC_GRAPH_LAYOUT_ALGORITHM = ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM;

export type AcademicGraphLayoutAlgorithm =
  | typeof ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM
  | typeof ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM;
export type AcademicGraphViewportClass = 'compact' | 'standard' | 'wide';
export type AcademicGraphLayoutFallbackReason = 'layout-budget-exceeded' | 'topology-layout-failed';

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

const spineNodeTypeOrder = [
  'research-question',
  'contribution',
  'project',
  'claim',
  'method',
  'decision',
  'evidence',
  'paper',
  'literature-cluster',
  'concept',
  'gap',
  'idea',
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
const LANE_GAP = 16;
const COMPONENT_ROW_GAP = 2;
const BOTTOM_PADDING = 24;
const MINIMUM_WIDTH = 420;
const MINIMUM_HEIGHT = 220;
const DEFAULT_DESKTOP_LAYOUT_BUDGET_MS = 100;
const HARD_BOUND_LAYOUT_BUDGET_MS = 250;
const TOPOLOGY_SWEEP_COUNT = 4;

export interface AcademicGraphLayoutBand {
  layer: AcademicGraphLayer;
  x: number;
  width: number;
  nodeCount: number;
  laneCount?: number;
}

export interface AcademicGraphLayoutComponent {
  componentId: string;
  anchorNodeId: string;
  nodeIds: string[];
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface AcademicGraphLayoutCluster {
  clusterId: string;
  componentId: string;
  anchorNodeId: string;
  nodeIds: string[];
  label: string;
  layer: AcademicGraphLayer;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface AcademicGraphLayoutClusterEdge {
  clusterEdgeId: string;
  sourceClusterId: string;
  targetClusterId: string;
  edgeCount: number;
  routeOffset: number;
}

export interface AcademicGraphLayoutNode {
  nodeId: string;
  canonicalId: string;
  label: string;
  nodeType: AcademicGraphNodeType;
  layer: AcademicGraphLayer;
  column: number;
  row: number;
  lane?: number;
  componentId?: string;
  communityId?: string;
  spine?: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  riskSeverity?: AcademicGraphRiskSeverity | null;
  riskCount?: number;
  riskKinds?: AcademicGraphRiskKind[];
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
  routeOffset?: number;
  riskSeverity?: AcademicGraphRiskSeverity | null;
  riskCount?: number;
  riskKinds?: AcademicGraphRiskKind[];
  inferenceStrength?: AcademicGraphEdge['inferenceStrength'];
  confidence?: AcademicGraphEdge['confidence'];
  status?: AcademicGraphEdge['status'];
}

export interface AcademicGraphLayout {
  schemaVersion: typeof ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION;
  algorithm: AcademicGraphLayoutAlgorithm;
  layoutKey: string;
  projectId?: string;
  projectionId: string;
  indexId: string;
  viewportClass?: AcademicGraphViewportClass;
  fallbackReason?: AcademicGraphLayoutFallbackReason | null;
  width: number;
  height: number;
  bands: AcademicGraphLayoutBand[];
  components?: AcademicGraphLayoutComponent[];
  clusters?: AcademicGraphLayoutCluster[];
  clusterEdges?: AcademicGraphLayoutClusterEdge[];
  nodes: AcademicGraphLayoutNode[];
  edges: AcademicGraphLayoutEdge[];
}

export interface AcademicGraphLayoutOptions {
  viewportClass?: AcademicGraphViewportClass;
  previousLayout?: AcademicGraphLayout | null;
  performanceBudgetMs?: number;
  now?: () => number;
  forceFallback?: boolean;
}

export interface AcademicGraphViewState {
  schemaVersion: typeof ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION;
  layoutKey: string;
  viewportMode: 'scroll';
  selectedNodeId: string | null;
  selectedEdgeId: string | null;
  focusNodeId: string | null;
  direction: AcademicGraphDirection;
  matchingNodeIds: string[];
  collapsedClusterIds: string[];
  hiddenNodeTypes: AcademicGraphNodeType[];
  hiddenRelationFamilies: AcademicGraphRelationFamily[];
}

export interface AcademicGraphViewStateInput {
  selectedNodeId: string | null;
  selectedEdgeId?: string | null;
  focusNodeId: string | null;
  direction: AcademicGraphDirection;
  matchingNodeIds?: string[];
  collapsedClusterIds?: string[];
  hiddenNodeTypes?: AcademicGraphNodeType[];
  hiddenRelationFamilies?: AcademicGraphRelationFamily[];
}

export function academicGraphViewportClass(width: number): AcademicGraphViewportClass {
  if (!Number.isFinite(width) || width <= 0) return 'standard';
  if (width <= 520) return 'compact';
  if (width >= 1280) return 'wide';
  return 'standard';
}

/**
 * Produces rebuildable geometry from an already validated, bounded query result.
 * Semantic identity remains owned by the Rust projection and never depends on
 * these coordinates.
 */
export function buildAcademicGraphLayout(
  result: AcademicGraphQueryResult,
  riskOverlay?: AcademicGraphRiskOverlay,
  options: AcademicGraphLayoutOptions = {}
): AcademicGraphLayout {
  if (riskOverlay
    && (riskOverlay.projectionId !== result.projectionId || riskOverlay.indexId !== result.indexId)) {
    throw new Error('academic graph layout received a risk overlay from another projection');
  }
  validateLayoutInput(result);
  const risks = buildRiskMaps(riskOverlay);
  const viewportClass = options.viewportClass ?? 'standard';
  const now = options.now ?? (() => globalThis.performance?.now() ?? Date.now());
  const startedAt = now();
  if (!options.forceFallback) {
    try {
      const topologyLayout = buildTopologyLayoutV2(
        result,
        risks,
        viewportClass,
        options.previousLayout ?? null
      );
      const budget = options.performanceBudgetMs
        ?? (result.nodes.length > 100 || result.edges.length > 200
          ? HARD_BOUND_LAYOUT_BUDGET_MS
          : DEFAULT_DESKTOP_LAYOUT_BUDGET_MS);
      if (now() - startedAt <= budget) return topologyLayout;
      return buildLayeredLayoutV1(
        result,
        risks,
        viewportClass,
        'layout-budget-exceeded'
      );
    } catch {
      return buildLayeredLayoutV1(
        result,
        risks,
        viewportClass,
        'topology-layout-failed'
      );
    }
  }
  return buildLayeredLayoutV1(
    result,
    risks,
    viewportClass,
    'topology-layout-failed'
  );
}

interface LayoutRiskMaps {
  nodeRisks: Map<string, AcademicGraphRiskSeverity[]>;
  edgeRisks: Map<string, AcademicGraphRiskSeverity[]>;
  nodeRiskKinds: Map<string, AcademicGraphRiskKind[]>;
  edgeRiskKinds: Map<string, AcademicGraphRiskKind[]>;
}

interface TopologyComponent {
  componentId: string;
  anchorNodeId: string;
  nodeIds: string[];
  distances: Map<string, number>;
}

function buildRiskMaps(riskOverlay: AcademicGraphRiskOverlay | undefined): LayoutRiskMaps {
  const nodeRisks = new Map<string, AcademicGraphRiskSeverity[]>();
  const edgeRisks = new Map<string, AcademicGraphRiskSeverity[]>();
  const nodeRiskKinds = new Map<string, AcademicGraphRiskKind[]>();
  const edgeRiskKinds = new Map<string, AcademicGraphRiskKind[]>();
  for (const risk of riskOverlay?.entries ?? []) {
    for (const nodeId of risk.affectedNodeIds) {
      const values = nodeRisks.get(nodeId) ?? [];
      values.push(risk.severity);
      nodeRisks.set(nodeId, values);
      const kinds = nodeRiskKinds.get(nodeId) ?? [];
      kinds.push(risk.kind);
      nodeRiskKinds.set(nodeId, kinds);
    }
    if (risk.entity.kind === 'edge') {
      const values = edgeRisks.get(risk.entity.id) ?? [];
      values.push(risk.severity);
      edgeRisks.set(risk.entity.id, values);
      const kinds = edgeRiskKinds.get(risk.entity.id) ?? [];
      kinds.push(risk.kind);
      edgeRiskKinds.set(risk.entity.id, kinds);
    }
  }
  return { nodeRisks, edgeRisks, nodeRiskKinds, edgeRiskKinds };
}

function validateLayoutInput(result: AcademicGraphQueryResult): void {
  const nodeIds = new Set(result.nodes.map((node) => node.nodeId));
  if (nodeIds.size !== result.nodes.length
    || new Set(result.edges.map((edge) => edge.edgeId)).size !== result.edges.length
    || result.edges.some((edge) =>
      !nodeIds.has(edge.sourceNodeId) || !nodeIds.has(edge.targetNodeId))) {
    throw new Error('academic graph layout received inconsistent topology');
  }
}

function buildTopologyLayoutV2(
  result: AcademicGraphQueryResult,
  risks: LayoutRiskMaps,
  viewportClass: AcademicGraphViewportClass,
  previousLayout: AcademicGraphLayout | null
): AcademicGraphLayout {
  const orderedSourceNodes = [...result.nodes].sort(compareNodes);
  const adjacency = new Map(orderedSourceNodes.map((node) => [node.nodeId, [] as string[]]));
  for (const edge of result.edges) {
    adjacency.get(edge.sourceNodeId)!.push(edge.targetNodeId);
    adjacency.get(edge.targetNodeId)!.push(edge.sourceNodeId);
  }
  for (const neighbours of adjacency.values()) neighbours.sort(compareText);

  const components = buildTopologyComponents(orderedSourceNodes, adjacency);
  const componentByNode = new Map<string, TopologyComponent>();
  const componentRank = new Map<string, number>();
  const communityByNode = new Map<string, string>();
  for (const [rank, component] of components.entries()) {
    componentRank.set(component.componentId, rank);
    for (const nodeId of component.nodeIds) componentByNode.set(nodeId, component);
    assignTopologyCommunities(component, adjacency, communityByNode);
  }

  const canPreserve = previousLayout?.algorithm === ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM
    && previousLayout.projectId === result.projectId
    && previousLayout.viewportClass === viewportClass
    && previousLayout.nodes.length > 0
    && orderedSourceNodes.filter((node) =>
      previousLayout.nodes.some((previous) => previous.nodeId === node.nodeId)).length
      / previousLayout.nodes.length >= 0.9;
  const previousNodeById = new Map(
    canPreserve ? previousLayout!.nodes.map((node) => [node.nodeId, node]) : []
  );
  const orderedByLayer = orderTopologyNodes(
    orderedSourceNodes,
    adjacency,
    componentByNode,
    componentRank,
    communityByNode,
    previousNodeById,
    canPreserve
  );

  const activeLayers = bandOrder.filter((layer) => (orderedByLayer.get(layer)?.length ?? 0) > 0);
  const bands: AcademicGraphLayoutBand[] = [];
  const nodes: AcademicGraphLayoutNode[] = [];
  let bandX = HORIZONTAL_PADDING;
  for (const [column, layer] of activeLayers.entries()) {
    const layerNodes = orderedByLayer.get(layer) ?? [];
    const previousBand = canPreserve
      ? previousLayout!.bands.find((band) => band.layer === layer)
      : undefined;
    const laneCount = topologyLaneCount(
      layerNodes.length,
      viewportClass,
      previousBand?.laneCount
    );
    const width = Math.max(
      BAND_WIDTH,
      24 + laneCount * NODE_WIDTH + Math.max(0, laneCount - 1) * LANE_GAP
    );
    bands.push({ layer, x: bandX, width, nodeCount: layerNodes.length, laneCount });

    const slots = assignTopologySlots(
      layerNodes,
      laneCount,
      componentByNode,
      previousNodeById,
      canPreserve && previousBand?.laneCount === laneCount
    );
    for (const sourceNode of layerNodes) {
      const slot = slots.get(sourceNode.nodeId)!;
      const lane = slot % laneCount;
      const row = Math.floor(slot / laneCount);
      const component = componentByNode.get(sourceNode.nodeId)!;
      nodes.push({
        nodeId: sourceNode.nodeId,
        canonicalId: sourceNode.canonicalId,
        label: sourceNode.label,
        nodeType: sourceNode.nodeType,
        layer,
        column,
        row,
        lane,
        componentId: component.componentId,
        communityId: communityByNode.get(sourceNode.nodeId),
        spine: sourceNode.nodeId === component.anchorNodeId,
        x: bandX + 12 + lane * (NODE_WIDTH + LANE_GAP),
        y: BAND_HEADER_HEIGHT + row * (NODE_HEIGHT + NODE_GAP),
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
        riskSeverity: highestRisk(risks.nodeRisks.get(sourceNode.nodeId)),
        riskCount: risks.nodeRisks.get(sourceNode.nodeId)?.length ?? 0,
        riskKinds: uniqueRiskKinds(risks.nodeRiskKinds.get(sourceNode.nodeId))
      });
    }
    bandX += width + BAND_GAP;
  }

  nodes.sort((left, right) =>
    left.column - right.column
    || left.row - right.row
    || (left.lane ?? 0) - (right.lane ?? 0)
    || compareText(left.nodeId, right.nodeId));
  const positionedNodeById = new Map(nodes.map((node) => [node.nodeId, node]));
  const edges = buildPositionedEdges(
    result,
    positionedNodeById,
    risks.edgeRisks,
    risks.edgeRiskKinds
  );
  const layoutComponents = components.map((component) =>
    positionTopologyComponent(component, positionedNodeById));
  const clusters = buildTopologyClusters(nodes);
  const clusterEdges = buildTopologyClusterEdges(result, clusters);
  const maximumNodeBottom = Math.max(
    0,
    ...nodes.map((node) => node.y + node.height)
  );
  const width = Math.max(
    MINIMUM_WIDTH,
    activeLayers.length === 0 ? MINIMUM_WIDTH : bandX - BAND_GAP + HORIZONTAL_PADDING
  );
  const height = Math.max(
    MINIMUM_HEIGHT,
    maximumNodeBottom + BOTTOM_PADDING
  );

  return {
    schemaVersion: ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION,
    algorithm: ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM,
    layoutKey: layoutKey(
      result,
      ACADEMIC_GRAPH_TOPOLOGY_LAYOUT_ALGORITHM,
      viewportClass
    ),
    projectId: result.projectId,
    projectionId: result.projectionId,
    indexId: result.indexId,
    viewportClass,
    fallbackReason: null,
    width,
    height,
    bands,
    components: layoutComponents,
    clusters,
    clusterEdges,
    nodes,
    edges
  };
}

function buildLayeredLayoutV1(
  result: AcademicGraphQueryResult,
  risks: LayoutRiskMaps,
  viewportClass: AcademicGraphViewportClass,
  fallbackReason: AcademicGraphLayoutFallbackReason
): AcademicGraphLayout {
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
        riskSeverity: highestRisk(risks.nodeRisks.get(node.nodeId)),
        riskCount: risks.nodeRisks.get(node.nodeId)?.length ?? 0,
        riskKinds: uniqueRiskKinds(risks.nodeRiskKinds.get(node.nodeId))
      });
    }
  }

  const nodeById = new Map(nodes.map((node) => [node.nodeId, node]));
  const edges = buildPositionedEdges(
    result,
    nodeById,
    risks.edgeRisks,
    risks.edgeRiskKinds
  );

  const columnWidth = activeLayers.length === 0
    ? 0
    : activeLayers.length * BAND_WIDTH + (activeLayers.length - 1) * BAND_GAP;
  const maximumRows = Math.max(0, ...bands.map((band) => band.nodeCount));
  const rowsHeight = maximumRows === 0
    ? 0
    : maximumRows * NODE_HEIGHT + (maximumRows - 1) * NODE_GAP;

  return {
    schemaVersion: ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION,
    algorithm: ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM,
    layoutKey: layoutKey(
      result,
      ACADEMIC_GRAPH_LAYERED_LAYOUT_ALGORITHM,
      viewportClass
    ),
    projectId: result.projectId,
    projectionId: result.projectionId,
    indexId: result.indexId,
    viewportClass,
    fallbackReason,
    width: Math.max(MINIMUM_WIDTH, HORIZONTAL_PADDING * 2 + columnWidth),
    height: Math.max(MINIMUM_HEIGHT, BAND_HEADER_HEIGHT + rowsHeight + BOTTOM_PADDING),
    bands,
    components: [],
    clusters: [],
    clusterEdges: [],
    nodes,
    edges
  };
}

function buildTopologyComponents(
  nodes: AcademicGraphNode[],
  adjacency: Map<string, string[]>
): TopologyComponent[] {
  const nodeById = new Map(nodes.map((node) => [node.nodeId, node]));
  const visited = new Set<string>();
  const components: TopologyComponent[] = [];
  for (const start of nodes) {
    if (visited.has(start.nodeId)) continue;
    const queue = [start.nodeId];
    const nodeIds: string[] = [];
    visited.add(start.nodeId);
    for (let index = 0; index < queue.length; index += 1) {
      const nodeId = queue[index]!;
      nodeIds.push(nodeId);
      for (const neighbourId of adjacency.get(nodeId) ?? []) {
        if (visited.has(neighbourId)) continue;
        visited.add(neighbourId);
        queue.push(neighbourId);
      }
    }
    nodeIds.sort(compareText);
    const anchorNodeId = [...nodeIds].sort((leftId, rightId) => {
      const left = nodeById.get(leftId)!;
      const right = nodeById.get(rightId)!;
      return spineNodeTypeOrder.indexOf(left.nodeType)
        - spineNodeTypeOrder.indexOf(right.nodeType)
        || compareNodes(left, right);
    })[0]!;
    components.push({
      componentId: stableIdentity('cmp', nodeIds),
      anchorNodeId,
      nodeIds,
      distances: topologyDistances(anchorNodeId, new Set(nodeIds), adjacency)
    });
  }
  return components.sort((left, right) => {
    const leftAnchor = nodeById.get(left.anchorNodeId)!;
    const rightAnchor = nodeById.get(right.anchorNodeId)!;
    return spineNodeTypeOrder.indexOf(leftAnchor.nodeType)
      - spineNodeTypeOrder.indexOf(rightAnchor.nodeType)
      || compareNodes(leftAnchor, rightAnchor)
      || compareText(left.componentId, right.componentId);
  });
}

function topologyDistances(
  startNodeId: string,
  allowedNodeIds: Set<string>,
  adjacency: Map<string, string[]>
): Map<string, number> {
  const distances = new Map<string, number>([[startNodeId, 0]]);
  const queue = [startNodeId];
  for (let index = 0; index < queue.length; index += 1) {
    const nodeId = queue[index]!;
    const nextDistance = distances.get(nodeId)! + 1;
    for (const neighbourId of adjacency.get(nodeId) ?? []) {
      if (!allowedNodeIds.has(neighbourId) || distances.has(neighbourId)) continue;
      distances.set(neighbourId, nextDistance);
      queue.push(neighbourId);
    }
  }
  return distances;
}

function assignTopologyCommunities(
  component: TopologyComponent,
  adjacency: Map<string, string[]>,
  output: Map<string, string>
): void {
  const degree = (nodeId: string): number => adjacency.get(nodeId)?.length ?? 0;
  const localHubs = component.nodeIds
    .filter((nodeId) => nodeId !== component.anchorNodeId && degree(nodeId) >= 3)
    .filter((nodeId) => (adjacency.get(nodeId) ?? [])
      .every((neighbourId) => degree(nodeId) >= degree(neighbourId)))
    .sort((left, right) => degree(right) - degree(left) || compareText(left, right));
  const maximumHubCount = Math.max(1, Math.ceil(Math.sqrt(component.nodeIds.length) / 2));
  const hubs = [
    component.anchorNodeId,
    ...localHubs.slice(0, Math.max(0, maximumHubCount - 1))
  ];
  const allowedNodeIds = new Set(component.nodeIds);
  const distancesByHub = new Map(
    hubs.map((hubId) => [hubId, topologyDistances(hubId, allowedNodeIds, adjacency)])
  );
  for (const nodeId of component.nodeIds) {
    const hubId = [...hubs].sort((left, right) =>
      (distancesByHub.get(left)?.get(nodeId) ?? Number.MAX_SAFE_INTEGER)
      - (distancesByHub.get(right)?.get(nodeId) ?? Number.MAX_SAFE_INTEGER)
      || compareText(left, right))[0]!;
    output.set(nodeId, stableIdentity('com', [hubId]));
  }
}

function orderTopologyNodes(
  nodes: AcademicGraphNode[],
  adjacency: Map<string, string[]>,
  componentByNode: Map<string, TopologyComponent>,
  componentRank: Map<string, number>,
  communityByNode: Map<string, string>,
  previousNodeById: Map<string, AcademicGraphLayoutNode>,
  preservePreviousOrder: boolean
): Map<AcademicGraphLayer, AcademicGraphNode[]> {
  const grouped = new Map<AcademicGraphLayer, AcademicGraphNode[]>(
    bandOrder.map((layer) => [layer, []])
  );
  for (const node of nodes) grouped.get(primaryLayer(node))!.push(node);

  const baselineCompare = (left: AcademicGraphNode, right: AcademicGraphNode): number => {
    const leftComponent = componentByNode.get(left.nodeId)!;
    const rightComponent = componentByNode.get(right.nodeId)!;
    return (componentRank.get(leftComponent.componentId) ?? Number.MAX_SAFE_INTEGER)
      - (componentRank.get(rightComponent.componentId) ?? Number.MAX_SAFE_INTEGER)
      || compareText(communityByNode.get(left.nodeId) ?? '', communityByNode.get(right.nodeId) ?? '')
      || (leftComponent.distances.get(left.nodeId) ?? Number.MAX_SAFE_INTEGER)
        - (rightComponent.distances.get(right.nodeId) ?? Number.MAX_SAFE_INTEGER)
      || compareNodes(left, right);
  };

  for (const layer of bandOrder) {
    grouped.get(layer)!.sort((left, right) => {
      if (!preservePreviousOrder) return baselineCompare(left, right);
      const leftPrevious = previousNodeById.get(left.nodeId);
      const rightPrevious = previousNodeById.get(right.nodeId);
      if (leftPrevious && rightPrevious) {
        return leftPrevious.row - rightPrevious.row
          || (leftPrevious.lane ?? 0) - (rightPrevious.lane ?? 0)
          || compareText(left.nodeId, right.nodeId);
      }
      if (leftPrevious) return -1;
      if (rightPrevious) return 1;
      return baselineCompare(left, right);
    });
  }
  if (preservePreviousOrder) return grouped;

  for (let sweep = 0; sweep < TOPOLOGY_SWEEP_COUNT; sweep += 1) {
    const positions = new Map<string, number>();
    for (const layer of bandOrder) {
      for (const [index, node] of grouped.get(layer)!.entries()) {
        positions.set(node.nodeId, index);
      }
    }
    const layers = sweep % 2 === 0 ? bandOrder : [...bandOrder].reverse();
    for (const layer of layers) {
      grouped.get(layer)!.sort((left, right) => {
        const leftComponent = componentByNode.get(left.nodeId)!;
        const rightComponent = componentByNode.get(right.nodeId)!;
        const componentDifference =
          (componentRank.get(leftComponent.componentId) ?? Number.MAX_SAFE_INTEGER)
          - (componentRank.get(rightComponent.componentId) ?? Number.MAX_SAFE_INTEGER);
        if (componentDifference !== 0) return componentDifference;
        const leftBarycenter = topologyBarycenter(left.nodeId, adjacency, positions);
        const rightBarycenter = topologyBarycenter(right.nodeId, adjacency, positions);
        return leftBarycenter - rightBarycenter || baselineCompare(left, right);
      });
    }
  }
  return grouped;
}

function topologyBarycenter(
  nodeId: string,
  adjacency: Map<string, string[]>,
  positions: Map<string, number>
): number {
  const values = (adjacency.get(nodeId) ?? [])
    .map((neighbourId) => positions.get(neighbourId))
    .filter((value): value is number => value !== undefined);
  if (values.length === 0) return Number.MAX_SAFE_INTEGER;
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function topologyLaneCount(
  nodeCount: number,
  viewportClass: AcademicGraphViewportClass,
  previousLaneCount: number | undefined
): number {
  const maximumLanes = viewportClass === 'compact' ? 2 : viewportClass === 'wide' ? 4 : 3;
  const targetRows = viewportClass === 'compact' ? 10 : viewportClass === 'wide' ? 7 : 8;
  if (previousLaneCount
    && previousLaneCount <= maximumLanes
    && nodeCount <= previousLaneCount * targetRows + Math.ceil(targetRows / 4)) {
    return previousLaneCount;
  }
  return Math.max(1, Math.min(maximumLanes, Math.ceil(nodeCount / targetRows)));
}

function assignTopologySlots(
  nodes: AcademicGraphNode[],
  laneCount: number,
  componentByNode: Map<string, TopologyComponent>,
  previousNodeById: Map<string, AcademicGraphLayoutNode>,
  preservePreviousSlots: boolean
): Map<string, number> {
  const slots = new Map<string, number>();
  if (preservePreviousSlots) {
    const usedSlots = new Set<number>();
    for (const node of nodes) {
      const previous = previousNodeById.get(node.nodeId);
      if (!previous || previous.lane === undefined || previous.lane >= laneCount) continue;
      const slot = previous.row * laneCount + previous.lane;
      if (usedSlots.has(slot)) continue;
      slots.set(node.nodeId, slot);
      usedSlots.add(slot);
    }
    let nextSlot = usedSlots.size === 0 ? 0 : Math.max(...usedSlots) + 1;
    for (const node of nodes) {
      if (slots.has(node.nodeId)) continue;
      while (usedSlots.has(nextSlot)) nextSlot += 1;
      slots.set(node.nodeId, nextSlot);
      usedSlots.add(nextSlot);
      nextSlot += 1;
    }
    return slots;
  }

  let nextSlot = 0;
  let previousComponentId: string | null = null;
  for (const node of nodes) {
    const componentId = componentByNode.get(node.nodeId)!.componentId;
    if (previousComponentId !== null && previousComponentId !== componentId) {
      nextSlot = (Math.ceil(nextSlot / laneCount) + COMPONENT_ROW_GAP) * laneCount;
    }
    slots.set(node.nodeId, nextSlot);
    nextSlot += 1;
    previousComponentId = componentId;
  }
  return slots;
}

function buildPositionedEdges(
  result: AcademicGraphQueryResult,
  nodeById: Map<string, AcademicGraphLayoutNode>,
  edgeRisks: Map<string, AcademicGraphRiskSeverity[]>,
  edgeRiskKinds: Map<string, AcademicGraphRiskKind[]>
): AcademicGraphLayoutEdge[] {
  return [...result.edges]
    .sort((left, right) => compareText(left.edgeId, right.edgeId))
    .map((edge): AcademicGraphLayoutEdge => {
      const source = nodeById.get(edge.sourceNodeId)!;
      const target = nodeById.get(edge.targetNodeId)!;
      return {
        edgeId: edge.edgeId,
        sourceNodeId: edge.sourceNodeId,
        targetNodeId: edge.targetNodeId,
        relation: edge.relation,
        routeOffset: topologyRouteOffset(edge.edgeId),
        riskSeverity: highestRisk(edgeRisks.get(edge.edgeId)),
        riskCount: edgeRisks.get(edge.edgeId)?.length ?? 0,
        riskKinds: uniqueRiskKinds(edgeRiskKinds.get(edge.edgeId)),
        inferenceStrength: edge.inferenceStrength,
        confidence: edge.confidence,
        status: edge.status,
        ...connectNodeBoundaries(source, target)
      };
    });
}

function positionTopologyComponent(
  component: TopologyComponent,
  nodeById: Map<string, AcademicGraphLayoutNode>
): AcademicGraphLayoutComponent {
  const nodes = component.nodeIds.map((nodeId) => nodeById.get(nodeId)!);
  const x = Math.min(...nodes.map((node) => node.x));
  const y = Math.min(...nodes.map((node) => node.y));
  const right = Math.max(...nodes.map((node) => node.x + node.width));
  const bottom = Math.max(...nodes.map((node) => node.y + node.height));
  return {
    componentId: component.componentId,
    anchorNodeId: component.anchorNodeId,
    nodeIds: [...component.nodeIds],
    x,
    y,
    width: right - x,
    height: bottom - y
  };
}

function buildTopologyClusters(
  nodes: AcademicGraphLayoutNode[]
): AcademicGraphLayoutCluster[] {
  const grouped = new Map<string, AcademicGraphLayoutNode[]>();
  for (const node of nodes) {
    const communityId = node.communityId ?? stableIdentity('com', [node.nodeId]);
    const values = grouped.get(communityId) ?? [];
    values.push(node);
    grouped.set(communityId, values);
  }
  return [...grouped.entries()]
    .map(([communityId, values]): AcademicGraphLayoutCluster => {
      values.sort((left, right) =>
        Number(right.spine) - Number(left.spine)
        || left.row - right.row
        || left.column - right.column
        || compareText(left.nodeId, right.nodeId));
      const anchor = values[0]!;
      const centerX = values.reduce(
        (sum, node) => sum + node.x + node.width / 2,
        0
      ) / values.length;
      const centerY = values.reduce(
        (sum, node) => sum + node.y + node.height / 2,
        0
      ) / values.length;
      const layers = new Set(values.map((node) => node.layer));
      const width = 176;
      const height = 64;
      return {
        clusterId: `vcl_${communityId.replace(/^com_/, '')}`,
        componentId: anchor.componentId!,
        anchorNodeId: anchor.nodeId,
        nodeIds: values.map((node) => node.nodeId).sort(compareText),
        label: values.length > 1 ? `${anchor.label} +${values.length - 1}` : anchor.label,
        layer: layers.size === 1 ? anchor.layer : 'combined',
        x: Math.round(centerX - width / 2),
        y: Math.round(centerY - height / 2),
        width,
        height
      };
    })
    .sort((left, right) => compareText(left.clusterId, right.clusterId));
}

function buildTopologyClusterEdges(
  result: AcademicGraphQueryResult,
  clusters: AcademicGraphLayoutCluster[]
): AcademicGraphLayoutClusterEdge[] {
  const clusterByNode = new Map(
    clusters.flatMap((cluster) =>
      cluster.nodeIds.map((nodeId) => [nodeId, cluster.clusterId] as const))
  );
  const grouped = new Map<string, {
    sourceClusterId: string;
    targetClusterId: string;
    edgeCount: number;
  }>();
  for (const edge of [...result.edges].sort((left, right) =>
    compareText(left.edgeId, right.edgeId))) {
    const sourceClusterId = clusterByNode.get(edge.sourceNodeId)!;
    const targetClusterId = clusterByNode.get(edge.targetNodeId)!;
    if (sourceClusterId === targetClusterId) continue;
    const key = `${sourceClusterId}\u001f${targetClusterId}`;
    const current = grouped.get(key);
    if (current) {
      current.edgeCount += 1;
    } else {
      grouped.set(key, { sourceClusterId, targetClusterId, edgeCount: 1 });
    }
  }
  return [...grouped.values()]
    .sort((left, right) =>
      compareText(left.sourceClusterId, right.sourceClusterId)
      || compareText(left.targetClusterId, right.targetClusterId))
    .map((value): AcademicGraphLayoutClusterEdge => {
      const clusterEdgeId = stableIdentity(
        'vce',
        [value.sourceClusterId, value.targetClusterId]
      );
      return {
        clusterEdgeId,
        sourceClusterId: value.sourceClusterId,
        targetClusterId: value.targetClusterId,
        edgeCount: value.edgeCount,
        routeOffset: topologyRouteOffset(clusterEdgeId)
      };
    });
}

function topologyRouteOffset(edgeId: string): number {
  return (stableHash32(edgeId, 0x811c9dc5) % 5 - 2) * 10;
}

function layoutKey(
  result: AcademicGraphQueryResult,
  algorithm: AcademicGraphLayoutAlgorithm,
  viewportClass: AcademicGraphViewportClass
): string {
  return `${result.projectionId}:${result.indexId}:${algorithm}:${viewportClass}`;
}

function stableIdentity(prefix: string, values: string[]): string {
  const source = values.join('\u001f');
  const first = stableHash32(source, 0x811c9dc5).toString(16).padStart(8, '0');
  const second = stableHash32(source, 0x9e3779b9).toString(16).padStart(8, '0');
  return `${prefix}_${first}${second}`;
}

function stableHash32(value: string, seed: number): number {
  let hash = seed >>> 0;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash;
}

function highestRisk(values: AcademicGraphRiskSeverity[] | undefined): AcademicGraphRiskSeverity | null {
  if (!values || values.length === 0) return null;
  return values.includes('high') ? 'high' : 'medium';
}

function uniqueRiskKinds(values: AcademicGraphRiskKind[] | undefined): AcademicGraphRiskKind[] {
  return [...new Set(values ?? [])].sort(compareText);
}

/** Normalizes ephemeral UI state to the nodes exposed by one exact layout. */
export function buildAcademicGraphViewState(
  layout: AcademicGraphLayout,
  input: AcademicGraphViewStateInput
): AcademicGraphViewState {
  const nodeIds = new Set(layout.nodes.map((node) => node.nodeId));
  const edgeIds = new Set(layout.edges.map((edge) => edge.edgeId));
  const clusterIds = new Set((layout.clusters ?? []).map((cluster) => cluster.clusterId));
  return {
    schemaVersion: ACADEMIC_GRAPH_LAYOUT_SCHEMA_VERSION,
    layoutKey: layout.layoutKey,
    viewportMode: 'scroll',
    selectedNodeId: input.selectedNodeId && nodeIds.has(input.selectedNodeId)
      ? input.selectedNodeId
      : null,
    selectedEdgeId: input.selectedEdgeId && edgeIds.has(input.selectedEdgeId)
      ? input.selectedEdgeId
      : null,
    focusNodeId: input.focusNodeId && nodeIds.has(input.focusNodeId)
      ? input.focusNodeId
      : null,
    direction: input.direction,
    matchingNodeIds: [...new Set(input.matchingNodeIds ?? [])]
      .filter((nodeId) => nodeIds.has(nodeId))
      .sort(compareText),
    collapsedClusterIds: [...new Set(input.collapsedClusterIds ?? [])]
      .filter((clusterId) => clusterIds.has(clusterId))
      .sort(compareText),
    hiddenNodeTypes: [...new Set(input.hiddenNodeTypes ?? [])]
      .filter((nodeType) => nodeTypeOrder.includes(nodeType))
      .sort((left, right) => nodeTypeOrder.indexOf(left) - nodeTypeOrder.indexOf(right)),
    hiddenRelationFamilies: [...new Set(input.hiddenRelationFamilies ?? [])].sort(compareText)
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
