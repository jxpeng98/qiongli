import type {
  AcademicGraphEdge,
  AcademicGraphEntityReference,
  AcademicGraphQueryResult
} from '@qiongli/app-api';

export const ACADEMIC_GRAPH_RISK_SCHEMA_VERSION = 1 as const;

export type AcademicGraphRiskKind =
  | 'contradiction'
  | 'gap'
  | 'rejected-relation'
  | 'low-confidence';

export type AcademicGraphRiskSeverity = 'high' | 'medium';

export interface AcademicGraphRiskEntry {
  riskId: string;
  kind: AcademicGraphRiskKind;
  severity: AcademicGraphRiskSeverity;
  entity: AcademicGraphEntityReference;
  sourceLabel: string;
  relation: AcademicGraphEdge['relation'] | null;
  targetLabel: string | null;
  rationale: string;
  evidenceLimit: string | null;
  artifactPath: string;
  sourceAnchor: string;
  affectedNodeIds: string[];
}

export interface AcademicGraphRiskOverlay {
  schemaVersion: typeof ACADEMIC_GRAPH_RISK_SCHEMA_VERSION;
  projectionId: string;
  indexId: string;
  riskCount: number;
  highRiskCount: number;
  affectedNodeCount: number;
  affectedEdgeCount: number;
  entries: AcademicGraphRiskEntry[];
}

/**
 * Builds a presentation-only overlay from one already validated query result.
 * It does not infer new scholarly relations or mutate canonical graph facts.
 */
export function buildAcademicGraphRiskOverlay(
  result: AcademicGraphQueryResult
): AcademicGraphRiskOverlay {
  const nodeById = new Map(result.nodes.map((node) => [node.nodeId, node]));
  const entries: AcademicGraphRiskEntry[] = [];

  for (const node of result.nodes) {
    if (node.nodeType !== 'gap') continue;
    entries.push({
      riskId: `node:${node.nodeId}:gap`,
      kind: 'gap',
      severity: 'high',
      entity: { kind: 'node', id: node.nodeId },
      sourceLabel: node.label,
      relation: null,
      targetLabel: null,
      rationale: node.label,
      evidenceLimit: null,
      artifactPath: node.artifactPath,
      sourceAnchor: node.sourceAnchor,
      affectedNodeIds: [node.nodeId]
    });
  }

  for (const edge of result.edges) {
    const sourceLabel = nodeById.get(edge.sourceNodeId)?.label ?? edge.sourceNodeId;
    const targetLabel = nodeById.get(edge.targetNodeId)?.label ?? edge.targetNodeId;
    const base = {
      entity: { kind: 'edge', id: edge.edgeId } as const,
      sourceLabel,
      relation: edge.relation,
      targetLabel,
      rationale: edge.rationale,
      evidenceLimit: edge.evidenceLimit,
      artifactPath: edge.artifactPath,
      sourceAnchor: edge.sourceAnchor,
      affectedNodeIds: [edge.sourceNodeId, edge.targetNodeId].sort(compareText)
    };
    if (edge.relation === 'contradicts') {
      entries.push({
        ...base,
        riskId: `edge:${edge.edgeId}:contradiction`,
        kind: 'contradiction',
        severity: 'high'
      });
    }
    if (edge.status === 'rejected') {
      entries.push({
        ...base,
        riskId: `edge:${edge.edgeId}:rejected-relation`,
        kind: 'rejected-relation',
        severity: 'high'
      });
    }
    if (edge.confidence === 'low' || edge.confidence === 'unknown') {
      entries.push({
        ...base,
        riskId: `edge:${edge.edgeId}:low-confidence`,
        kind: 'low-confidence',
        severity: 'medium'
      });
    }
    if (edge.inferenceStrength === 'unsupported_gap'
      && !entries.some((entry) => entry.riskId === `edge:${edge.edgeId}:gap`)) {
      entries.push({
        ...base,
        riskId: `edge:${edge.edgeId}:gap`,
        kind: 'gap',
        severity: 'high'
      });
    }
  }

  entries.sort((left, right) => severityOrder(left.severity) - severityOrder(right.severity)
    || riskKindOrder(left.kind) - riskKindOrder(right.kind)
    || compareText(left.riskId, right.riskId));

  const affectedNodes = new Set(entries.flatMap((entry) => entry.affectedNodeIds));
  const affectedEdges = new Set(
    entries.filter((entry) => entry.entity.kind === 'edge').map((entry) => entry.entity.id)
  );
  return {
    schemaVersion: ACADEMIC_GRAPH_RISK_SCHEMA_VERSION,
    projectionId: result.projectionId,
    indexId: result.indexId,
    riskCount: entries.length,
    highRiskCount: entries.filter((entry) => entry.severity === 'high').length,
    affectedNodeCount: affectedNodes.size,
    affectedEdgeCount: affectedEdges.size,
    entries
  };
}

function severityOrder(value: AcademicGraphRiskSeverity): number {
  return value === 'high' ? 0 : 1;
}

function riskKindOrder(value: AcademicGraphRiskKind): number {
  return ['contradiction', 'gap', 'rejected-relation', 'low-confidence'].indexOf(value);
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
