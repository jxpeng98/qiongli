import type {
  AcademicGraphNodeType,
  AcademicGraphRelation
} from '@qiongli/app-api';

export type AcademicGraphRelationFamily =
  | 'evidence'
  | 'challenge'
  | 'provenance'
  | 'structure'
  | 'development';

export interface AcademicGraphNodeVisual {
  nodeType: AcademicGraphNodeType;
  mark: string;
  shape:
    | 'ellipse'
    | 'diamond'
    | 'hexagon'
    | 'rectangle'
    | 'roundrectangle'
    | 'barrel'
    | 'triangle'
    | 'pentagon';
}

export interface AcademicGraphRelationVisual {
  relation: AcademicGraphRelation;
  family: AcademicGraphRelationFamily;
  lineStyle: 'solid' | 'dashed' | 'dotted';
  arrowShape: 'triangle' | 'tee' | 'diamond' | 'square';
  mark: string;
}

export const academicGraphNodeVisuals = {
  project: { nodeType: 'project', mark: 'PR', shape: 'roundrectangle' },
  'research-question': { nodeType: 'research-question', mark: 'RQ', shape: 'diamond' },
  idea: { nodeType: 'idea', mark: 'ID', shape: 'ellipse' },
  contribution: { nodeType: 'contribution', mark: 'CO', shape: 'hexagon' },
  concept: { nodeType: 'concept', mark: 'CX', shape: 'ellipse' },
  'literature-cluster': {
    nodeType: 'literature-cluster',
    mark: 'LC',
    shape: 'roundrectangle'
  },
  paper: { nodeType: 'paper', mark: 'PA', shape: 'rectangle' },
  claim: { nodeType: 'claim', mark: 'CL', shape: 'hexagon' },
  evidence: { nodeType: 'evidence', mark: 'EV', shape: 'barrel' },
  decision: { nodeType: 'decision', mark: 'DE', shape: 'diamond' },
  gap: { nodeType: 'gap', mark: 'GA', shape: 'triangle' },
  method: { nodeType: 'method', mark: 'ME', shape: 'pentagon' },
  'manuscript-section': {
    nodeType: 'manuscript-section',
    mark: '§',
    shape: 'rectangle'
  },
  artifact: { nodeType: 'artifact', mark: 'AR', shape: 'rectangle' },
  task: { nodeType: 'task', mark: 'TA', shape: 'roundrectangle' }
} as const satisfies Record<AcademicGraphNodeType, AcademicGraphNodeVisual>;

const evidenceRelations = new Set<AcademicGraphRelation>([
  'supports',
  'weakens',
  'informs',
  'operationalizes',
  'uses-method',
  'addresses-gap'
]);
const challengeRelations = new Set<AcademicGraphRelation>([
  'contradicts',
  'competes-with',
  'supersedes'
]);
const provenanceRelations = new Set<AcademicGraphRelation>([
  'cites',
  'cited-by',
  'belongs-to-cluster',
  'derived-from',
  'shares-source',
  'shares-concept',
  'forked-from',
  'extends-project'
]);
const structureRelations = new Set<AcademicGraphRelation>([
  'contains',
  'defines',
  'appears-in-section',
  'bounded-by'
]);

export const academicGraphRelationFamilies: AcademicGraphRelationFamily[] = [
  'evidence',
  'challenge',
  'provenance',
  'structure',
  'development'
];

export function academicGraphNodeVisual(
  nodeType: AcademicGraphNodeType
): AcademicGraphNodeVisual {
  return academicGraphNodeVisuals[nodeType];
}

export function academicGraphRelationVisual(
  relation: AcademicGraphRelation
): AcademicGraphRelationVisual {
  if (evidenceRelations.has(relation)) {
    return {
      relation,
      family: 'evidence',
      lineStyle: relation === 'weakens' ? 'dashed' : 'solid',
      arrowShape: 'triangle',
      mark: '→'
    };
  }
  if (challengeRelations.has(relation)) {
    return {
      relation,
      family: 'challenge',
      lineStyle: 'dashed',
      arrowShape: 'tee',
      mark: '⊣'
    };
  }
  if (provenanceRelations.has(relation)) {
    return {
      relation,
      family: 'provenance',
      lineStyle: 'dotted',
      arrowShape: 'diamond',
      mark: '◇'
    };
  }
  if (structureRelations.has(relation)) {
    return {
      relation,
      family: 'structure',
      lineStyle: 'solid',
      arrowShape: 'square',
      mark: '▪'
    };
  }
  return {
    relation,
    family: 'development',
    lineStyle: 'dashed',
    arrowShape: 'triangle',
    mark: '↗'
  };
}

export function compactAcademicGraphLabel(label: string, maximumLength = 22): string {
  const normalized = label.replace(/\s+/g, ' ').trim();
  if (normalized.length <= maximumLength) return normalized;
  return `${normalized.slice(0, Math.max(1, maximumLength - 1)).trimEnd()}…`;
}
