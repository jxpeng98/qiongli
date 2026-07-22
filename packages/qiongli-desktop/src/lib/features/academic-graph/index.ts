import type {
  AcademicGraphDirection,
  AcademicGraphLayer,
  AcademicGraphNodeType,
  AcademicGraphQuery,
  AcademicGraphRelation,
  AppEvent,
  AppIntent
} from '@qiongli/app-api';

import type { FeatureDescriptor } from '../types';

export * from './layout';
export * from './inspection';

export const academicGraphFeature: FeatureDescriptor = {
  id: 'academic-graph',
  label: 'Academic Graph',
  route: '/academic-graph',
  stage: 'available'
};

export const academicGraphNodeTypes: AcademicGraphNodeType[] = [
  'artifact', 'claim', 'concept', 'contribution', 'decision', 'evidence', 'gap', 'idea',
  'literature-cluster', 'manuscript-section', 'method', 'paper', 'project',
  'research-question', 'task'
];

export const academicGraphRelations: AcademicGraphRelation[] = [
  'addresses-gap', 'appears-in-section', 'belongs-to-cluster', 'bounded-by', 'cited-by',
  'cites', 'combines-with', 'competes-with', 'complements', 'contains', 'contradicts',
  'defines', 'derived-from', 'extends', 'extends-project', 'forked-from', 'informs',
  'motivates', 'operationalizes', 'shares-concept', 'shares-source', 'supersedes',
  'supports', 'uses-method', 'weakens'
];

export const academicGraphLayers: AcademicGraphLayer[] = [
  'argument', 'combined', 'idea-decision', 'literature', 'manuscript', 'portfolio'
];

export interface AcademicGraphFilters {
  focusNodeId: string | null;
  direction: AcademicGraphDirection;
  nodeType: AcademicGraphNodeType | null;
  relation: AcademicGraphRelation | null;
  layer: AcademicGraphLayer | null;
  text: string;
}

export function buildAcademicGraphQuery(
  expectedProjectionId: string,
  filters: AcademicGraphFilters
): AcademicGraphQuery {
  const text = filters.text.trim();
  return {
    expectedProjectionId,
    focusNodeId: filters.focusNodeId,
    direction: filters.direction,
    nodeTypes: filters.nodeType ? [filters.nodeType] : [],
    relations: filters.relation ? [filters.relation] : [],
    layers: filters.layer ? [filters.layer] : [],
    canonicalId: null,
    text: text.length > 0 ? text : null,
    maxNodes: 100,
    maxEdges: 200
  };
}

type AcademicGraphIntent = Extract<AppIntent, {
  action: 'load-academic-graph' | 'query-academic-graph';
}>;

export async function loadAcademicGraphPresentationState(
  projectId: string,
  projectRevision: number,
  execute: (intent: AcademicGraphIntent) => Promise<AppEvent | null>
): Promise<boolean> {
  try {
    const graphEvent = await execute({ action: 'load-academic-graph', projectId });
    if (
      graphEvent?.type !== 'academic-graph'
      || graphEvent.graph.projectId !== projectId
      || graphEvent.graph.projectRevision !== projectRevision
    ) return false;

    const queryEvent = await execute({
      action: 'query-academic-graph',
      projectId,
      query: buildAcademicGraphQuery(graphEvent.graph.projectionId, {
        focusNodeId: null,
        direction: 'both',
        nodeType: null,
        relation: null,
        layer: null,
        text: ''
      })
    });
    return queryEvent?.type === 'academic-graph-query'
      && queryEvent.result.projectId === projectId
      && queryEvent.result.projectRevision === projectRevision
      && queryEvent.result.projectionId === graphEvent.graph.projectionId;
  } catch {
    return false;
  }
}
