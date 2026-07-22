import { z } from 'zod';

export const APP_API_SCHEMA_VERSION = 1 as const;

export const statusCodeSchema = z.enum([
  'ready',
  'attention',
  'missing',
  'unavailable',
  'disabled',
  'blocked',
  'recovery-required',
  'conflict',
  'drifted',
  'invalid',
  'future-schema',
  'insecure',
  'busy',
  'write-unsupported'
]);

export type StatusCode = z.infer<typeof statusCodeSchema>;

const productSchema = z.object({
  version: z.string().min(1).max(128),
  build: z.string().min(1).max(128),
  operatingSystem: z.string().min(1).max(64),
  architecture: z.string().min(1).max(64),
  trust: z.object({
    mode: z.enum(['source-read-only', 'local-installable', 'packaged-product']),
    label: z.string().min(1).max(128),
    canApply: z.boolean(),
    reasonCode: z.string().min(1).max(128)
  })
});

const profileSchema = z.object({
  id: z.enum(['skill-only', 'marketplace-lite', 'full']),
  label: z.string().min(1).max(64),
  description: z.string().min(1).max(256),
  includedResourceKinds: z.number().int().min(1).max(32)
});

const contentSchema = z.object({
  status: statusCodeSchema,
  packId: z.string().min(1).max(128),
  contentVersion: z.string().min(1).max(128),
  entryCount: z.number().int().min(1).max(100_000),
  profiles: z.array(profileSchema).length(3)
});

const mcpSchema = z.object({
  status: statusCodeSchema,
  profile: z.enum(['skill-only', 'marketplace-lite', 'full']),
  publicToolCount: z.number().int().min(1).max(256)
});

const configurationSchema = z.object({
  status: statusCodeSchema,
  revision: z.number().int().min(0).nullable(),
  cleanupRequired: z.boolean()
});

export const updateStreamSchema = z.enum(['stable', 'beta']);
export const updatePhaseSchema = z.enum([
  'unavailable',
  'idle',
  'checking',
  'current',
  'available',
  'downloading',
  'verifying',
  'staging',
  'ready-to-install',
  'installing',
  'awaiting-restart',
  'cancelling',
  'cancelled',
  'recovery-required',
  'failed'
]);
export const updateRemediationSchema = z.enum([
  'none',
  'retry-update-check',
  'retry-update-preparation',
  'cancel-update-and-retry',
  'restart-qiongli',
  'move-qiongli-to-applications',
  'reinstall-qiongli',
  'install-trusted-qiongli-release',
  'use-supported-update-platform'
]);
export const updateViewSchema = z.object({
  status: statusCodeSchema,
  selectedStream: updateStreamSchema,
  phase: updatePhaseSchema,
  availableVersion: z.string().min(1).max(128).nullable(),
  archiveSizeBytes: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  progress: z.object({
    completedSteps: z.number().int().min(0).max(255),
    totalSteps: z.number().int().min(1).max(255),
    label: z.string().min(1).max(128),
    indeterminate: z.boolean()
  }).strict().nullable(),
  reasonCode: z.string().min(1).max(128),
  remediation: updateRemediationSchema,
  canSelectStream: z.boolean(),
  canCheck: z.boolean(),
  canPrepare: z.boolean(),
  canInstall: z.boolean(),
  canCancel: z.boolean()
}).strict();

export type UpdateView = z.infer<typeof updateViewSchema>;

export const projectIdSchema = z.string().regex(/^prj_[0-9a-f]{32}$/);
export const projectKindSchema = z.enum([
  'article',
  'review',
  'dissertation-article',
  'manuscript'
]);
export const projectStageSchema = z.enum([
  'idea',
  'framing',
  'literature',
  'design',
  'analysis',
  'writing',
  'review',
  'submission'
]);
export const projectLifecycleSchema = z.enum(['active', 'archived']);
export const projectHealthSchema = z.enum([
  'ready',
  'missing-root',
  'missing-manifest',
  'manifest-conflict',
  'revision-drift',
  'inspection-blocked'
]);
export const projectNextActionSchema = z.enum([
  'open',
  'refresh',
  'relocate',
  'repair-manifest',
  'inspect-permissions',
  'restore'
]);

const projectOverviewSchema = z.object({
  focalQuestion: z.string().min(1).max(500).nullable(),
  thesis: z.string().min(1).max(500).nullable(),
  evidencePosition: z.string().min(1).max(500).nullable(),
  unresolvedRiskCount: z.number().int().min(0),
  claimEvidenceCoveragePercent: z.number().int().min(0).max(100).nullable(),
  nextPriorities: z.array(z.string().min(1).max(500)).max(8)
}).strict();

export const articleProjectSummarySchema = z.object({
  projectId: projectIdSchema,
  displayName: z.string().min(1).max(160),
  projectKind: projectKindSchema,
  stage: projectStageSchema,
  lifecycle: projectLifecycleSchema,
  semanticRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  registeredAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  lastOpenedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER).nullable(),
  academicallyUpdatedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  health: projectHealthSchema,
  nextAction: projectNextActionSchema,
  rootLabel: z.string().min(1).max(160),
  overview: projectOverviewSchema
}).strict();

export const researchLibrarySnapshotSchema = z.object({
  schemaVersion: z.literal(1),
  revision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  health: z.enum(['ready', 'empty', 'recovery-required', 'inspection-blocked']),
  projects: z.array(articleProjectSummarySchema).max(512)
}).strict();

export type ArticleProjectSummary = z.infer<typeof articleProjectSummarySchema>;
export type ResearchLibrarySnapshot = z.infer<typeof researchLibrarySnapshotSchema>;

const sha256Schema = z.string().regex(/^[0-9a-f]{64}$/);
export const academicGraphProjectionIdSchema = z.string().regex(/^grp_[0-9a-f]{64}$/);
export const academicGraphIndexIdSchema = z.string().regex(/^gix_[0-9a-f]{64}$/);
export const academicGraphComparisonIdSchema = z.string().regex(/^gcp_[0-9a-f]{64}$/);
export const academicGraphNodeIdSchema = z.string().regex(/^nod_[0-9a-f]{64}$/);
export const academicGraphEdgeIdSchema = z.string().regex(/^edg_[0-9a-f]{64}$/);
export const academicGraphEntityReferenceSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('node'), id: academicGraphNodeIdSchema }).strict(),
  z.object({ kind: z.literal('edge'), id: academicGraphEdgeIdSchema }).strict()
]);
export const academicGraphNodeTypeSchema = z.enum([
  'project',
  'research-question',
  'idea',
  'contribution',
  'concept',
  'literature-cluster',
  'paper',
  'claim',
  'evidence',
  'decision',
  'gap',
  'method',
  'manuscript-section',
  'artifact',
  'task'
]);
export const academicGraphRelationSchema = z.enum([
  'contains',
  'cites',
  'cited-by',
  'supports',
  'weakens',
  'contradicts',
  'extends',
  'defines',
  'operationalizes',
  'uses-method',
  'belongs-to-cluster',
  'complements',
  'competes-with',
  'combines-with',
  'motivates',
  'informs',
  'addresses-gap',
  'appears-in-section',
  'derived-from',
  'supersedes',
  'bounded-by',
  'shares-source',
  'shares-concept',
  'forked-from',
  'extends-project'
]);
export const academicGraphLayerSchema = z.enum([
  'portfolio',
  'literature',
  'idea-decision',
  'argument',
  'manuscript',
  'combined'
]);
export const academicGraphDirectionSchema = z.enum(['incoming', 'outgoing', 'both']);
export const academicInferenceStrengthSchema = z.enum([
  'direct_evidence',
  'reasonable_inference',
  'unsupported_gap'
]);
export const academicGraphConfidenceSchema = z.enum(['high', 'medium', 'low', 'unknown']);
export const academicGraphEdgeStatusSchema = z.enum(['observed', 'proposed', 'reviewed', 'rejected']);
export const academicGraphIdentityScopeSchema = z.enum(['project', 'global']);
export const academicGraphSourceKindSchema = z.enum([
  'project-manifest',
  'registered-artifact',
  'semantic-links'
]);
export const academicGraphDiagnosticCodeSchema = z.enum([
  'missing-stable-id',
  'ambiguous-relation',
  'unsupported-relation',
  'dangling-node',
  'conflicting-identity'
]);

export const academicGraphArtifactPathSchema = z.enum([
  'context/project_manifest.json',
  'context/research_state.md',
  'context/decision_log.md',
  'context/stage_handoff.md',
  'context/boundary_review.md',
  'context/idea_funnel.md',
  'literature/literature_map.md',
  'evidence/claim-evidence-ledger.csv',
  'manuscript/claims_evidence_map.md',
  'graph/semantic_links.jsonl'
]);

export const academicGraphSourceSchema = z.object({
  sourceKind: academicGraphSourceKindSchema,
  artifactPath: academicGraphArtifactPathSchema,
  present: z.boolean(),
  contentDigest: sha256Schema.nullable(),
  sizeBytes: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER)
}).strict();

export const academicGraphNodeSchema = z.object({
  nodeId: academicGraphNodeIdSchema,
  nodeType: academicGraphNodeTypeSchema,
  identityScope: academicGraphIdentityScopeSchema,
  canonicalId: z.string().min(1).max(512),
  label: z.string().min(1).max(1_024),
  layers: z.array(academicGraphLayerSchema).min(1).max(6),
  artifactPath: academicGraphArtifactPathSchema,
  sourceAnchor: z.string().min(1).max(512)
}).strict();

export const academicGraphEdgeSchema = z.object({
  edgeId: academicGraphEdgeIdSchema,
  sourceNodeId: academicGraphNodeIdSchema,
  relation: academicGraphRelationSchema,
  targetNodeId: academicGraphNodeIdSchema,
  layers: z.array(academicGraphLayerSchema).min(1).max(6),
  rationale: z.string().min(1).max(4_096),
  artifactPath: academicGraphArtifactPathSchema,
  sourceAnchor: z.string().min(1).max(512),
  evidenceLimit: z.string().min(1).max(2_048),
  inferenceStrength: academicInferenceStrengthSchema,
  confidence: academicGraphConfidenceSchema,
  status: academicGraphEdgeStatusSchema,
  createdFromCapture: z.string().regex(/^cap_[0-9a-f]{64}$/).nullable()
}).strict();

export const academicGraphDiagnosticSchema = z.object({
  code: academicGraphDiagnosticCodeSchema,
  artifactPath: academicGraphArtifactPathSchema,
  sourceAnchor: z.string().min(1).max(512).nullable(),
  relatedId: z.string().min(1).max(512).nullable()
}).strict();

export const academicGraphSnapshotSchema = z.object({
  schemaVersion: z.literal(1),
  documentKind: z.literal('qiongli-academic-graph'),
  projectionId: academicGraphProjectionIdSchema,
  projectionDigest: sha256Schema,
  projectId: projectIdSchema,
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  projectStage: projectStageSchema,
  projectLifecycle: projectLifecycleSchema,
  projectManifestDigest: sha256Schema,
  projectSemanticDigest: sha256Schema,
  graphSourceDigest: sha256Schema,
  sourceCount: z.number().int().min(1).max(16),
  presentSourceCount: z.number().int().min(1).max(16),
  nodeCount: z.number().int().min(1).max(4_096),
  edgeCount: z.number().int().min(0).max(4_096),
  diagnosticCount: z.number().int().min(0).max(4_096),
  sources: z.array(academicGraphSourceSchema).min(1).max(16),
  nodes: z.array(academicGraphNodeSchema).min(1).max(4_096),
  edges: z.array(academicGraphEdgeSchema).max(4_096),
  diagnostics: z.array(academicGraphDiagnosticSchema).max(4_096)
}).strict().superRefine((graph, context) => {
  const nodeIds = new Set(graph.nodes.map((node) => node.nodeId));
  const edgeIds = new Set(graph.edges.map((edge) => edge.edgeId));
  if (graph.sourceCount !== graph.sources.length
    || graph.presentSourceCount !== graph.sources.filter((source) => source.present).length
    || graph.nodeCount !== graph.nodes.length
    || graph.edgeCount !== graph.edges.length
    || graph.diagnosticCount !== graph.diagnostics.length
    || nodeIds.size !== graph.nodes.length
    || edgeIds.size !== graph.edges.length
    || graph.nodes.some((node) => !sortedUnique(node.layers, academicGraphLayerSchema.options))
    || graph.edges.some((edge) => !sortedUnique(edge.layers, academicGraphLayerSchema.options))
    || graph.edges.some((edge) => !nodeIds.has(edge.sourceNodeId) || !nodeIds.has(edge.targetNodeId))) {
    context.addIssue({ code: 'custom', message: 'academic graph snapshot counts are inconsistent' });
  }
});

export const academicGraphChangeKindSchema = z.enum(['added', 'removed', 'modified']);
export const academicGraphRevisionActionSchema = z.enum([
  'inspect-new-contradictions',
  'fill-new-gaps',
  'verify-low-confidence-evidence',
  'review-rejected-relations',
  'reconnect-removed-evidence',
  'inspect-modified-relations'
]);

const academicGraphRiskSignalsSchema = z.object({
  contradictionCount: z.number().int().min(0).max(4_096),
  gapCount: z.number().int().min(0).max(8_192),
  rejectedRelationCount: z.number().int().min(0).max(4_096),
  lowConfidenceCount: z.number().int().min(0).max(4_096),
  totalSignalCount: z.number().int().min(0).max(20_480)
}).strict().superRefine((risks, context) => {
  if (risks.totalSignalCount !== risks.contradictionCount + risks.gapCount
    + risks.rejectedRelationCount + risks.lowConfidenceCount) {
    context.addIssue({ code: 'custom', message: 'academic graph risk counts are inconsistent' });
  }
});

const academicGraphRiskDeltaSchema = z.object({
  contradictionCount: z.number().int().min(-4_096).max(4_096),
  gapCount: z.number().int().min(-8_192).max(8_192),
  rejectedRelationCount: z.number().int().min(-4_096).max(4_096),
  lowConfidenceCount: z.number().int().min(-4_096).max(4_096),
  totalSignalCount: z.number().int().min(-20_480).max(20_480)
}).strict();

function validChangeSides<T extends { changeKind: 'added' | 'removed' | 'modified'; before: unknown; after: unknown }>(
  change: T
): boolean {
  return (change.changeKind === 'added' && change.before === null && change.after !== null)
    || (change.changeKind === 'removed' && change.before !== null && change.after === null)
    || (change.changeKind === 'modified' && change.before !== null && change.after !== null);
}

export const academicGraphSourceChangeSchema = z.object({
  changeKind: academicGraphChangeKindSchema,
  artifactPath: academicGraphArtifactPathSchema,
  before: academicGraphSourceSchema.nullable(),
  after: academicGraphSourceSchema.nullable()
}).strict().superRefine((change, context) => {
  if (!validChangeSides(change)
    || change.before?.artifactPath !== change.artifactPath && change.before !== null
    || change.after?.artifactPath !== change.artifactPath && change.after !== null
    || change.changeKind === 'modified' && change.before === change.after) {
    context.addIssue({ code: 'custom', message: 'academic graph source change is inconsistent' });
  }
});

export const academicGraphNodeChangeSchema = z.object({
  changeKind: academicGraphChangeKindSchema,
  nodeId: academicGraphNodeIdSchema,
  before: academicGraphNodeSchema.nullable(),
  after: academicGraphNodeSchema.nullable()
}).strict().superRefine((change, context) => {
  if (!validChangeSides(change)
    || change.before?.nodeId !== change.nodeId && change.before !== null
    || change.after?.nodeId !== change.nodeId && change.after !== null) {
    context.addIssue({ code: 'custom', message: 'academic graph node change is inconsistent' });
  }
});

export const academicGraphEdgeChangeSchema = z.object({
  changeKind: academicGraphChangeKindSchema,
  edgeId: academicGraphEdgeIdSchema,
  before: academicGraphEdgeSchema.nullable(),
  after: academicGraphEdgeSchema.nullable()
}).strict().superRefine((change, context) => {
  if (!validChangeSides(change)
    || change.before?.edgeId !== change.edgeId && change.before !== null
    || change.after?.edgeId !== change.edgeId && change.after !== null) {
    context.addIssue({ code: 'custom', message: 'academic graph edge change is inconsistent' });
  }
});

export const academicGraphRevisionComparisonSchema = z.object({
  schemaVersion: z.literal(1),
  documentKind: z.literal('qiongli-academic-graph-revision-comparison'),
  comparisonId: academicGraphComparisonIdSchema,
  projectId: projectIdSchema,
  beforeProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  afterProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  beforeProjectionId: academicGraphProjectionIdSchema,
  afterProjectionId: academicGraphProjectionIdSchema,
  sourceChangeCount: z.number().int().min(0).max(16),
  nodeChangeCount: z.number().int().min(0).max(8_192),
  edgeChangeCount: z.number().int().min(0).max(8_192),
  hasChanges: z.boolean(),
  beforeRisks: academicGraphRiskSignalsSchema,
  afterRisks: academicGraphRiskSignalsSchema,
  riskDelta: academicGraphRiskDeltaSchema,
  sourceChanges: z.array(academicGraphSourceChangeSchema).max(16),
  nodeChanges: z.array(academicGraphNodeChangeSchema).max(8_192),
  edgeChanges: z.array(academicGraphEdgeChangeSchema).max(8_192),
  nextActions: z.array(academicGraphRevisionActionSchema).max(6)
}).strict().superRefine((comparison, context) => {
  const actionOrder = academicGraphRevisionActionSchema.options;
  const idsUnique = new Set([
    ...comparison.sourceChanges.map((change) => `source:${change.artifactPath}`),
    ...comparison.nodeChanges.map((change) => `node:${change.nodeId}`),
    ...comparison.edgeChanges.map((change) => `edge:${change.edgeId}`)
  ]).size === comparison.sourceChanges.length + comparison.nodeChanges.length
    + comparison.edgeChanges.length;
  if (comparison.beforeProjectRevision > comparison.afterProjectRevision
    || comparison.sourceChangeCount !== comparison.sourceChanges.length
    || comparison.nodeChangeCount !== comparison.nodeChanges.length
    || comparison.edgeChangeCount !== comparison.edgeChanges.length
    || comparison.hasChanges !== (comparison.sourceChangeCount
      + comparison.nodeChangeCount + comparison.edgeChangeCount > 0)
    || comparison.riskDelta.contradictionCount
      !== comparison.afterRisks.contradictionCount - comparison.beforeRisks.contradictionCount
    || comparison.riskDelta.gapCount
      !== comparison.afterRisks.gapCount - comparison.beforeRisks.gapCount
    || comparison.riskDelta.rejectedRelationCount
      !== comparison.afterRisks.rejectedRelationCount
        - comparison.beforeRisks.rejectedRelationCount
    || comparison.riskDelta.lowConfidenceCount
      !== comparison.afterRisks.lowConfidenceCount - comparison.beforeRisks.lowConfidenceCount
    || comparison.riskDelta.totalSignalCount
      !== comparison.afterRisks.totalSignalCount - comparison.beforeRisks.totalSignalCount
    || !idsUnique
    || !sortedUnique(comparison.nextActions, actionOrder)) {
    context.addIssue({ code: 'custom', message: 'academic graph comparison is inconsistent' });
  }
});

function sortedUnique<T extends string>(values: T[], order: readonly T[]): boolean {
  return values.every((value, index) => index === 0
    || order.indexOf(values[index - 1]!) < order.indexOf(value));
}

export const academicGraphQuerySchema = z.object({
  expectedProjectionId: academicGraphProjectionIdSchema,
  focusNodeId: academicGraphNodeIdSchema.nullable(),
  direction: academicGraphDirectionSchema,
  nodeTypes: z.array(academicGraphNodeTypeSchema).max(15),
  relations: z.array(academicGraphRelationSchema).max(25),
  layers: z.array(academicGraphLayerSchema).max(6),
  canonicalId: z.string().min(1).max(256).nullable(),
  text: z.string().min(1).max(256).nullable(),
  maxNodes: z.number().int().min(1).max(256),
  maxEdges: z.number().int().min(1).max(512)
}).strict().superRefine((query, context) => {
  if (!sortedUnique(query.nodeTypes, academicGraphNodeTypeSchema.options)
    || !sortedUnique(query.relations, academicGraphRelationSchema.options)
    || !sortedUnique(query.layers, academicGraphLayerSchema.options)) {
    context.addIssue({ code: 'custom', message: 'academic graph filters must be sorted and unique' });
  }
  for (const value of [query.canonicalId, query.text]) {
    if (value !== null && (value.trim() !== value || /[\u0000-\u001f\u007f]/.test(value))) {
      context.addIssue({ code: 'custom', message: 'academic graph text filters are invalid' });
    }
  }
});

export const academicGraphQueryResultSchema = z.object({
  schemaVersion: z.literal(1),
  documentKind: z.literal('qiongli-academic-graph-query-result'),
  indexId: academicGraphIndexIdSchema,
  projectionId: academicGraphProjectionIdSchema,
  projectId: projectIdSchema,
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  matchedNodeCount: z.number().int().min(0).max(4_096),
  matchedEdgeCount: z.number().int().min(0).max(4_096),
  nodesTruncated: z.boolean(),
  edgesTruncated: z.boolean(),
  nodes: z.array(academicGraphNodeSchema).max(256),
  edges: z.array(academicGraphEdgeSchema).max(512)
}).strict().superRefine((result, context) => {
  const nodeIds = new Set(result.nodes.map((node) => node.nodeId));
  const edgeIds = new Set(result.edges.map((edge) => edge.edgeId));
  if (result.matchedNodeCount < result.nodes.length
    || result.matchedEdgeCount < result.edges.length
    || result.nodesTruncated !== (result.matchedNodeCount > result.nodes.length)
    || result.edgesTruncated !== (result.matchedEdgeCount > result.edges.length)
    || nodeIds.size !== result.nodes.length
    || edgeIds.size !== result.edges.length
    || result.nodes.some((node) => !sortedUnique(node.layers, academicGraphLayerSchema.options))
    || result.edges.some((edge) => !sortedUnique(edge.layers, academicGraphLayerSchema.options))
    || result.edges.some((edge) => !nodeIds.has(edge.sourceNodeId) || !nodeIds.has(edge.targetNodeId))) {
    context.addIssue({ code: 'custom', message: 'academic graph query counts are inconsistent' });
  }
});

export const academicGraphPathStatusSchema = z.enum(['found', 'not-found']);
export const academicGraphPathTraversalSchema = z.enum(['forward', 'reverse']);

export const academicGraphPathQuerySchema = z.object({
  expectedProjectionId: academicGraphProjectionIdSchema,
  sourceNodeId: academicGraphNodeIdSchema,
  targetNodeId: academicGraphNodeIdSchema,
  maxHops: z.number().int().min(1).max(12)
}).strict();

export const academicGraphPathStepSchema = z.object({
  sequence: z.number().int().min(1).max(12),
  fromNodeId: academicGraphNodeIdSchema,
  edgeId: academicGraphEdgeIdSchema,
  toNodeId: academicGraphNodeIdSchema,
  traversal: academicGraphPathTraversalSchema
}).strict();

export const academicGraphPathResultSchema = z.object({
  schemaVersion: z.literal(1),
  documentKind: z.literal('qiongli-academic-graph-explanatory-path'),
  indexId: academicGraphIndexIdSchema,
  projectionId: academicGraphProjectionIdSchema,
  projectId: projectIdSchema,
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  sourceNodeId: academicGraphNodeIdSchema,
  targetNodeId: academicGraphNodeIdSchema,
  maxHops: z.number().int().min(1).max(12),
  status: academicGraphPathStatusSchema,
  hopCount: z.number().int().min(0).max(12),
  nodes: z.array(academicGraphNodeSchema).max(13),
  edges: z.array(academicGraphEdgeSchema).max(12),
  steps: z.array(academicGraphPathStepSchema).max(12)
}).strict().superRefine((result, context) => {
  if (result.status === 'not-found') {
    if (result.hopCount !== 0
      || result.nodes.length !== 0
      || result.edges.length !== 0
      || result.steps.length !== 0) {
      context.addIssue({ code: 'custom', message: 'missing graph paths must have no path records' });
    }
    return;
  }

  const nodeIds = result.nodes.map((node) => node.nodeId);
  const edgeIds = result.edges.map((edge) => edge.edgeId);
  const structurallyValid = result.hopCount === result.edges.length
    && result.hopCount === result.steps.length
    && result.nodes.length === result.hopCount + 1
    && result.hopCount <= result.maxHops
    && nodeIds[0] === result.sourceNodeId
    && nodeIds.at(-1) === result.targetNodeId
    && new Set(nodeIds).size === nodeIds.length
    && new Set(edgeIds).size === edgeIds.length
    && result.nodes.every((node) => sortedUnique(node.layers, academicGraphLayerSchema.options))
    && result.edges.every((edge) => sortedUnique(edge.layers, academicGraphLayerSchema.options));
  const stepsValid = structurallyValid && result.steps.every((step, index) => {
    const edge = result.edges[index];
    const fromNodeId = nodeIds[index];
    const toNodeId = nodeIds[index + 1];
    if (!edge || !fromNodeId || !toNodeId) return false;
    const forward = edge.sourceNodeId === fromNodeId && edge.targetNodeId === toNodeId;
    const reverse = edge.targetNodeId === fromNodeId && edge.sourceNodeId === toNodeId;
    return step.sequence === index + 1
      && step.fromNodeId === fromNodeId
      && step.edgeId === edge.edgeId
      && step.toNodeId === toNodeId
      && ((step.traversal === 'forward' && forward)
        || (step.traversal === 'reverse' && reverse));
  });
  if (!stepsValid) {
    context.addIssue({ code: 'custom', message: 'academic graph path records are inconsistent' });
  }
});

export type AcademicGraphDirection = z.infer<typeof academicGraphDirectionSchema>;
export type AcademicGraphChangeKind = z.infer<typeof academicGraphChangeKindSchema>;
export type AcademicGraphEdge = z.infer<typeof academicGraphEdgeSchema>;
export type AcademicGraphEdgeChange = z.infer<typeof academicGraphEdgeChangeSchema>;
export type AcademicGraphEntityReference = z.infer<typeof academicGraphEntityReferenceSchema>;
export type AcademicGraphLayer = z.infer<typeof academicGraphLayerSchema>;
export type AcademicGraphNode = z.infer<typeof academicGraphNodeSchema>;
export type AcademicGraphNodeChange = z.infer<typeof academicGraphNodeChangeSchema>;
export type AcademicGraphNodeType = z.infer<typeof academicGraphNodeTypeSchema>;
export type AcademicGraphPathQuery = z.infer<typeof academicGraphPathQuerySchema>;
export type AcademicGraphPathResult = z.infer<typeof academicGraphPathResultSchema>;
export type AcademicGraphPathStatus = z.infer<typeof academicGraphPathStatusSchema>;
export type AcademicGraphPathStep = z.infer<typeof academicGraphPathStepSchema>;
export type AcademicGraphPathTraversal = z.infer<typeof academicGraphPathTraversalSchema>;
export type AcademicGraphQuery = z.infer<typeof academicGraphQuerySchema>;
export type AcademicGraphQueryResult = z.infer<typeof academicGraphQueryResultSchema>;
export type AcademicGraphRelation = z.infer<typeof academicGraphRelationSchema>;
export type AcademicGraphRevisionAction = z.infer<typeof academicGraphRevisionActionSchema>;
export type AcademicGraphRevisionComparison = z.infer<typeof academicGraphRevisionComparisonSchema>;
export type AcademicGraphSnapshot = z.infer<typeof academicGraphSnapshotSchema>;
export type AcademicGraphSourceChange = z.infer<typeof academicGraphSourceChangeSchema>;

export const captureIdSchema = z.string().regex(/^cap_[0-9a-f]{64}$/);
export const captureInboxStateSchema = z.enum([
  'pending-review',
  'stale',
  'conflicted',
  'applied'
]);
export const captureDispositionSchema = z.enum([
  'duplicate',
  'refinement',
  'contradiction',
  'supersession',
  'unresolved-candidate',
  'unsupported-gap'
]);
export const captureSourceSchema = z.enum([
  'codex',
  'claude-code',
  'chat-gpt',
  'cli',
  'manual',
  'repository',
  'portable-file'
]);
export const captureDeliverySchema = z.enum([
  'connected',
  'repository-backed',
  'portable',
  'manual'
]);
export const capturePolicySchema = z.enum(['review-required', 'history-only']);
export const captureAreaSchema = z.enum([
  'research-question',
  'thesis',
  'literature',
  'method',
  'evidence',
  'analysis',
  'manuscript',
  'scope'
]);

export const captureInboxEntrySchema = z.object({
  captureId: captureIdSchema,
  state: captureInboxStateSchema,
  disposition: captureDispositionSchema,
  source: captureSourceSchema,
  delivery: captureDeliverySchema,
  capturedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  baseRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  boundStage: projectStageSchema,
  task: z.string().min(1).max(300),
  capturePolicy: capturePolicySchema,
  summary: z.string().min(1).max(2_000),
  changeCount: z.number().int().min(0).max(16),
  decisionCount: z.number().int().min(0).max(16),
  evidenceCount: z.number().int().min(0).max(16),
  contradictionCount: z.number().int().min(0).max(16),
  nextActionCount: z.number().int().min(0).max(16),
  historyEntry: z.string().min(1).max(256)
}).strict();

export const captureInboxSnapshotSchema = z.object({
  schemaVersion: z.literal(1),
  projectId: projectIdSchema,
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  projectStage: projectStageSchema,
  pendingReviewCount: z.number().int().min(0).max(1_024),
  staleCount: z.number().int().min(0).max(1_024),
  conflictedCount: z.number().int().min(0).max(1_024),
  appliedCount: z.number().int().min(0).max(1_024),
  entries: z.array(captureInboxEntrySchema).max(1_024)
}).strict();

export const captureCoverageStateSchema = z.enum([
  'pending-review',
  'current',
  'stale',
  'conflicted',
  'unbound',
  'unknown'
]);
export const captureCoverageDeliverySchema = z.enum([
  'connected',
  'repository-backed',
  'portable',
  'manual',
  'unknown'
]);
export const captureSourceCoverageSchema = z.object({
  source: captureSourceSchema,
  state: captureCoverageStateSchema,
  delivery: captureCoverageDeliverySchema,
  captureCount: z.number().int().min(0).max(2_048),
  pendingReviewCount: z.number().int().min(0).max(2_048),
  currentCount: z.number().int().min(0).max(2_048),
  staleCount: z.number().int().min(0).max(2_048),
  conflictedCount: z.number().int().min(0).max(2_048),
  unboundCount: z.number().int().min(0).max(2_048),
  latestCaptureId: captureIdSchema.nullable(),
  lastCapturedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER).nullable()
}).strict();
export const captureCoverageSnapshotSchema = z.object({
  schemaVersion: z.literal(1),
  projectId: projectIdSchema,
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  projectStage: projectStageSchema,
  captureCount: z.number().int().min(0).max(2_048),
  connectedCount: z.number().int().min(0).max(2_048),
  repositoryBackedCount: z.number().int().min(0).max(2_048),
  portableCount: z.number().int().min(0).max(2_048),
  manualCount: z.number().int().min(0).max(2_048),
  pendingReviewCount: z.number().int().min(0).max(2_048),
  currentCount: z.number().int().min(0).max(2_048),
  staleCount: z.number().int().min(0).max(2_048),
  conflictedCount: z.number().int().min(0).max(2_048),
  unboundCount: z.number().int().min(0).max(2_048),
  unknownSourceCount: z.number().int().min(0).max(7),
  sources: z.array(captureSourceCoverageSchema).length(7)
}).strict();

export const registeredArtifactSchema = z.enum([
  'research-state',
  'decision-log',
  'stage-handoff',
  'boundary-review',
  'idea-funnel',
  'literature-map',
  'claim-evidence-ledger',
  'manuscript-claim-map'
]);
const registeredArtifactPathSchema = z.enum([
  'context/research_state.md',
  'context/decision_log.md',
  'context/stage_handoff.md',
  'context/boundary_review.md',
  'context/idea_funnel.md',
  'literature/literature_map.md',
  'evidence/claim-evidence-ledger.csv',
  'manuscript/claims_evidence_map.md'
]);
export const registeredArtifactObservationSchema = z.object({
  artifact: registeredArtifactSchema,
  relativePath: registeredArtifactPathSchema,
  present: z.boolean()
}).strict();
export const registeredArtifactChangeSchema = z.object({
  changeId: z.string().regex(/^chg_[0-9a-f]{64}$/),
  state: z.literal('unattributed'),
  detection: z.enum(['exact', 'aggregate']),
  effect: z.enum(['created', 'changed-set']),
  baseRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  relativePaths: z.array(registeredArtifactPathSchema).max(8),
  reason: z.literal('no-accepted-capture-lineage')
}).strict().superRefine((change, context) => {
  const exactCreated = change.detection === 'exact'
    && change.effect === 'created'
    && change.relativePaths.length > 0;
  const aggregateSet = change.detection === 'aggregate'
    && change.effect === 'changed-set'
    && change.relativePaths.length === 0;
  if (!exactCreated && !aggregateSet) {
    context.addIssue({ code: 'custom', message: 'artifact change evidence is inconsistent' });
  }
});
export const artifactChangeSnapshotSchema = z.object({
  schemaVersion: z.literal(1),
  projectId: projectIdSchema,
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  projectStage: projectStageSchema,
  state: z.enum(['current', 'unattributed']),
  registeredArtifactCount: z.literal(8),
  presentArtifactCount: z.number().int().min(0).max(8),
  changeCount: z.number().int().min(0).max(1),
  unattributedCount: z.number().int().min(0).max(1),
  changes: z.array(registeredArtifactChangeSchema).max(1),
  artifacts: z.array(registeredArtifactObservationSchema).length(8)
}).strict().superRefine((snapshot, context) => {
  const expectedCount = snapshot.state === 'current' ? 0 : 1;
  if (snapshot.changeCount !== expectedCount
    || snapshot.unattributedCount !== expectedCount
    || snapshot.changes.length !== expectedCount
    || snapshot.presentArtifactCount !== snapshot.artifacts.filter((artifact) => artifact.present).length) {
    context.addIssue({ code: 'custom', message: 'artifact change snapshot counts are inconsistent' });
  }
});

const captureBindingSchema = z.object({
  schemaVersion: z.literal(1),
  projectId: projectIdSchema,
  baseRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  stage: projectStageSchema,
  task: z.string().min(1).max(300),
  capturePolicy: capturePolicySchema
}).strict();

const semanticChangeSchema = z.object({
  area: captureAreaSchema,
  summary: z.string().min(1).max(1_000)
}).strict();

const decisionCandidateSchema = z.object({
  relation: z.enum(['candidate', 'refinement', 'challenge', 'supersession']),
  statement: z.string().min(1).max(1_000),
  rationale: z.string().min(1).max(1_000),
  target: z.string().min(1).max(1_000).nullable()
}).strict();

const evidenceReferenceSchema = z.object({
  locatorKind: z.enum(['doi', 'citation-key', 'https-url', 'artifact-anchor']),
  locator: z.string().min(1).max(500),
  relevance: z.string().min(1).max(1_000),
  limitation: z.string().min(1).max(1_000).nullable()
}).strict();

const contradictionSchema = z.object({
  statement: z.string().min(1).max(1_000),
  conflictsWith: z.string().min(1).max(1_000),
  consequence: z.string().min(1).max(1_000)
}).strict();

export const researchCaptureSchema = z.object({
  schemaVersion: z.literal(1),
  captureId: captureIdSchema,
  binding: captureBindingSchema,
  source: captureSourceSchema,
  delivery: captureDeliverySchema,
  capturedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  summary: z.string().min(1).max(2_000),
  changes: z.array(semanticChangeSchema).max(16),
  decisions: z.array(decisionCandidateSchema).max(16),
  evidence: z.array(evidenceReferenceSchema).max(16),
  contradictions: z.array(contradictionSchema).max(16),
  nextActions: z.array(z.string().min(1).max(1_000)).max(16)
}).strict();

export const captureIntakePreviewSchema = z.object({
  schemaVersion: z.literal(1),
  planDigest: z.string().regex(/^[0-9a-f]{64}$/),
  captureId: captureIdSchema,
  projectId: projectIdSchema,
  disposition: captureDispositionSchema,
  effect: z.enum(['append-pending-history', 'no-change']),
  source: captureSourceSchema,
  delivery: captureDeliverySchema,
  expectedLibraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  changeCount: z.number().int().min(0).max(16),
  decisionCount: z.number().int().min(0).max(16),
  evidenceCount: z.number().int().min(0).max(16),
  contradictionCount: z.number().int().min(0).max(16),
  nextActionCount: z.number().int().min(0).max(16),
  historyEntry: z.string().min(1).max(256),
  approvalsRequired: z.array(z.literal('filesystem-write')).max(1)
}).strict();

const consolidationConflictSchema = z.object({
  kind: z.enum([
    'project-archived',
    'stale-project-revision',
    'stage-changed',
    'history-only-policy',
    'scope-boundary-change',
    'locked-decision-guard',
    'contradiction-requires-resolution',
    'unsupported-evidence',
    'artifact-not-utf8',
    'artifact-lineage-conflict'
  ]),
  artifact: z.enum(['research-state', 'decision-log']).nullable(),
  resolution: z.string().min(1).max(256)
}).strict();

const consolidationArtifactDeltaSchema = z.object({
  artifact: z.enum(['research-state', 'decision-log']),
  relativePath: z.enum(['context/research_state.md', 'context/decision_log.md']),
  effect: z.enum(['create', 'update']),
  previousDigest: z.string().regex(/^[0-9a-f]{64}$/).nullable(),
  nextDigest: z.string().regex(/^[0-9a-f]{64}$/),
  previousBytes: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  nextBytes: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER)
}).strict();

export const captureConsolidationPreviewSchema = z.object({
  schemaVersion: z.literal(1),
  planDigest: z.string().regex(/^[0-9a-f]{64}$/),
  captureId: captureIdSchema,
  projectId: projectIdSchema,
  disposition: captureDispositionSchema,
  outcome: z.enum(['ready', 'conflicted', 'already-consolidated']),
  expectedLibraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  nextProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  projectStage: projectStageSchema,
  reviewedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  conflicts: z.array(consolidationConflictSchema).max(16),
  artifactDeltas: z.array(consolidationArtifactDeltaSchema).max(2),
  receiptEntry: z.string().min(1).max(256),
  approvalsRequired: z.array(z.enum(['academic-consolidation', 'filesystem-write'])).max(2)
}).strict();

export type CaptureInboxEntry = z.infer<typeof captureInboxEntrySchema>;
export type CaptureInboxSnapshot = z.infer<typeof captureInboxSnapshotSchema>;
export type CaptureSourceCoverage = z.infer<typeof captureSourceCoverageSchema>;
export type CaptureCoverageSnapshot = z.infer<typeof captureCoverageSnapshotSchema>;
export type ArtifactChangeSnapshot = z.infer<typeof artifactChangeSnapshotSchema>;
export type RegisteredArtifactChange = z.infer<typeof registeredArtifactChangeSchema>;
export type RegisteredArtifactObservation = z.infer<typeof registeredArtifactObservationSchema>;
export type ResearchCapture = z.infer<typeof researchCaptureSchema>;
export type CaptureIntakePreview = z.infer<typeof captureIntakePreviewSchema>;
export type CaptureConsolidationPreview = z.infer<typeof captureConsolidationPreviewSchema>;

const integrationPathSchema = z.object({
  surface: z.string().min(1).max(64),
  scope: z.string().min(1).max(64),
  source: z.string().min(1).max(64),
  state: statusCodeSchema,
  management: z.string().min(1).max(64),
  selected: z.boolean(),
  symbolicPath: z.string().min(1).max(128)
});

export const integrationTargetSchema = z.enum(['codex', 'claude-code']);
export type IntegrationTarget = z.infer<typeof integrationTargetSchema>;

export const connectionStateSchema = z.enum([
  'client-not-detected',
  'detected-not-connected',
  'connected',
  'needs-repair',
  'inspection-blocked',
  'unsupported-client-version'
]);

export const observationSchema = z.enum([
  'observed',
  'client-action-required',
  'not-observable',
  'missing',
  'inspection-blocked'
]);

const integrationSchema = z.object({
  target: integrationTargetSchema,
  label: z.string().min(1).max(64),
  connection: z.object({
    state: connectionStateSchema,
    label: z.string().min(1).max(64),
    reasonCode: z.string().min(1).max(128)
  }).strict(),
  client: z.object({
    detected: z.boolean(),
    status: statusCodeSchema,
    version: z.string().min(1).max(64).nullable(),
    compatibility: z.enum(['supported', 'unsupported', 'not-evaluated']),
    minimumSupportedVersion: z.string().min(1).max(64)
  }).strict(),
  plugin: z.object({
    installedVersion: z.string().min(1).max(64).nullable(),
    availableVersion: z.string().min(1).max(64)
  }).strict(),
  discovery: z.string().min(1).max(128),
  candidateRequired: z.boolean(),
  legacyDetected: z.boolean(),
  overall: statusCodeSchema,
  managedContent: z.object({
    source: statusCodeSchema,
    skills: statusCodeSchema,
    marketplace: statusCodeSchema,
    directPackage: statusCodeSchema.nullable(),
    registration: statusCodeSchema,
    activation: statusCodeSchema,
    activationObservation: observationSchema,
    mcpAttachment: statusCodeSchema,
    mcpAttachmentObservation: observationSchema
  }).strict(),
  symbolicLocation: z.string().min(1).max(128),
  activationPolicy: z.string().min(1).max(128),
  ownership: z.string().min(1).max(128),
  nextAction: z.string().min(1).max(128),
  evidenceCode: z.string().min(1).max(128),
  paths: z.array(integrationPathSchema).max(9)
}).strict();

const capabilitiesSchema = z.object({
  refresh: z.boolean(),
  skillsMaterialize: z.boolean(),
  integrationDiscovery: z.boolean(),
  integrationPreview: z.boolean(),
  projectLibrary: z.boolean(),
  projectMutation: z.boolean(),
  captureInbox: z.boolean(),
  captureMutation: z.boolean(),
  academicGraph: z.boolean(),
  apply: z.boolean()
});

export const appSnapshotSchema = z.object({
  schemaVersion: z.literal(APP_API_SCHEMA_VERSION),
  product: productSchema,
  content: contentSchema,
  mcp: mcpSchema,
  configuration: configurationSchema,
  update: updateViewSchema,
  researchLibrary: researchLibrarySnapshotSchema,
  integrations: z.array(integrationSchema).length(2),
  capabilities: capabilitiesSchema
}).strict();

export type AppSnapshot = z.infer<typeof appSnapshotSchema>;

export const integrationSelectionSchema = z.object({
  codex: z.boolean(),
  claudeCode: z.boolean()
}).strict();

export type IntegrationSelection = z.infer<typeof integrationSelectionSchema>;

const profileIdSchema = z.enum(['skill-only', 'marketplace-lite', 'full']);
const skillsPresetSchema = z.enum([
  'qiongli-managed',
  'detected-codex',
  'detected-claude-code',
  'current-project'
]);
const projectDialogNameSchema = z.string().min(1).max(160).regex(/^[^/\\\u0000-\u001f\u007f]+$/);

export const appIntentSchema = z.discriminatedUnion('action', [
  z.object({ action: z.literal('refresh') }).strict(),
  z.object({ action: z.literal('refresh-research-library') }).strict(),
  z.object({ action: z.literal('select-project-directory') }).strict(),
  z.object({
    action: z.literal('select-project-create-destination'),
    suggestedName: projectDialogNameSchema
  }).strict(),
  z.object({
    action: z.literal('preview-project-create'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/),
    displayName: z.string().min(1).max(160),
    projectKind: projectKindSchema,
    stage: projectStageSchema
  }).strict(),
  z.object({
    action: z.literal('preview-project-register'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/)
  }).strict(),
  z.object({ action: z.literal('open-project'), projectId: projectIdSchema }).strict(),
  z.object({
    action: z.literal('select-project-export-destination'),
    projectId: projectIdSchema
  }).strict(),
  z.object({
    action: z.literal('preview-project-export'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/)
  }).strict(),
  z.object({
    action: z.literal('select-project-import-locations'),
    suggestedName: projectDialogNameSchema
  }).strict(),
  z.object({
    action: z.literal('preview-project-import'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/)
  }).strict(),
  z.object({
    action: z.literal('preview-project-repair-manifest'),
    projectId: projectIdSchema
  }).strict(),
  z.object({ action: z.literal('preview-project-archive'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('preview-project-restore'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('preview-project-refresh'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('preview-project-unregister'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('load-capture-inbox'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('load-capture-coverage'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('load-artifact-changes'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('load-academic-graph'), projectId: projectIdSchema }).strict(),
  z.object({
    action: z.literal('query-academic-graph'),
    projectId: projectIdSchema,
    query: academicGraphQuerySchema
  }).strict(),
  z.object({
    action: z.literal('query-academic-graph-path'),
    projectId: projectIdSchema,
    query: academicGraphPathQuerySchema
  }).strict(),
  z.object({
    action: z.literal('open-academic-graph-artifact'),
    projectId: projectIdSchema,
    expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    expectedProjectionId: academicGraphProjectionIdSchema,
    entity: academicGraphEntityReferenceSchema
  }).strict(),
  z.object({
    action: z.literal('read-capture'),
    projectId: projectIdSchema,
    captureId: captureIdSchema
  }).strict(),
  z.object({ action: z.literal('select-capture-file'), projectId: projectIdSchema }).strict(),
  z.object({
    action: z.literal('preview-capture-intake'),
    fileToken: z.string().regex(/^[0-9a-f]{32}$/)
  }).strict(),
  z.object({
    action: z.literal('preview-capture-consolidation'),
    projectId: projectIdSchema,
    captureId: captureIdSchema
  }).strict(),
  z.object({ action: z.literal('refresh-integration-discovery') }).strict(),
  z.object({ action: z.literal('select-update-stream'), stream: updateStreamSchema }).strict(),
  z.object({ action: z.literal('check-for-updates') }).strict(),
  z.object({ action: z.literal('prepare-update') }).strict(),
  z.object({ action: z.literal('poll-update') }).strict(),
  z.object({ action: z.literal('cancel-update') }).strict(),
  z.object({ action: z.literal('preview-update-install') }).strict(),
  z.object({ action: z.literal('preview-install-recommended') }).strict(),
  z.object({ action: z.literal('preview-install-selected'), selection: integrationSelectionSchema }).strict(),
  z.object({ action: z.literal('verify-integrations'), selection: integrationSelectionSchema }).strict(),
  z.object({ action: z.literal('preview-repair-all') }).strict(),
  z.object({ action: z.literal('preview-update-integrations'), selection: integrationSelectionSchema }).strict(),
  z.object({ action: z.literal('preview-remove-integrations'), selection: integrationSelectionSchema }).strict(),
  z.object({ action: z.literal('preview-skills-preset-materialization'), profile: profileIdSchema, preset: skillsPresetSchema }).strict(),
  z.object({ action: z.literal('verify-skills-preset'), preset: skillsPresetSchema }).strict(),
  z.object({ action: z.literal('preview-skills-preset-removal'), preset: skillsPresetSchema }).strict(),
  z.object({ action: z.literal('confirm-operation'), token: z.string().regex(/^[0-9a-f]{32}$/) }).strict(),
  z.object({ action: z.literal('cancel-operation'), token: z.string().regex(/^[0-9a-f]{32}$/) }).strict()
]);

export type AppIntent = z.infer<typeof appIntentSchema>;

export const operationPreviewSchema = z.object({
  token: z.string().regex(/^[0-9a-f]{32}$/),
  kind: z.string().min(1).max(64),
  title: z.string().min(1).max(128),
  summary: z.string().min(1).max(512),
  displayTarget: z.string().min(1).max(4096).nullable(),
  planDigestSha256: z.string().regex(/^[0-9a-f]{64}$/).nullable(),
  approvalsRequired: z.array(z.string().min(1).max(64)).max(4),
  canConfirm: z.boolean(),
  blockedReason: z.string().min(1).max(128).nullable()
});

export type OperationPreview = z.infer<typeof operationPreviewSchema>;

export const appEventSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('snapshot'), snapshot: appSnapshotSchema }).strict(),
  z.object({ type: z.literal('preview'), preview: operationPreviewSchema }).strict(),
  z.object({ type: z.literal('capture-inbox'), inbox: captureInboxSnapshotSchema }).strict(),
  z.object({ type: z.literal('capture-coverage'), coverage: captureCoverageSnapshotSchema }).strict(),
  z.object({ type: z.literal('artifact-changes'), changes: artifactChangeSnapshotSchema }).strict(),
  z.object({
    type: z.literal('academic-graph'),
    graph: academicGraphSnapshotSchema,
    comparison: academicGraphRevisionComparisonSchema.nullable()
  }).strict(),
  z.object({ type: z.literal('academic-graph-query'), result: academicGraphQueryResultSchema }).strict(),
  z.object({ type: z.literal('academic-graph-path'), result: academicGraphPathResultSchema }).strict(),
  z.object({
    type: z.literal('academic-graph-artifact-opened'),
    projectId: projectIdSchema,
    projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    projectionId: academicGraphProjectionIdSchema,
    entity: academicGraphEntityReferenceSchema
  }).strict(),
  z.object({ type: z.literal('capture-read'), capture: researchCaptureSchema }).strict(),
  z.object({
    type: z.literal('capture-file-selected'),
    token: z.string().regex(/^[0-9a-f]{32}$/),
    fileLabel: z.string().min(1).max(160)
  }).strict(),
  z.object({
    type: z.literal('capture-intake-preview'),
    intake: captureIntakePreviewSchema,
    preview: operationPreviewSchema
  }).strict(),
  z.object({
    type: z.literal('capture-consolidation-preview'),
    consolidation: captureConsolidationPreviewSchema,
    preview: operationPreviewSchema
  }).strict(),
  z.object({
    type: z.literal('project-directory-selected'),
    token: z.string().regex(/^[0-9a-f]{32}$/),
    rootLabel: z.string().min(1).max(160)
  }).strict(),
  z.object({
    type: z.literal('update-changed'),
    update: updateViewSchema,
    closeRequested: z.boolean()
  }).strict(),
  z.object({ type: z.literal('completed'), code: z.string().min(1).max(128), snapshot: appSnapshotSchema }).strict(),
  z.object({
    type: z.literal('capture-operation-completed'),
    code: z.string().min(1).max(128),
    snapshot: appSnapshotSchema,
    inbox: captureInboxSnapshotSchema,
    coverage: captureCoverageSnapshotSchema,
    changes: artifactChangeSnapshotSchema
  }).strict(),
  z.object({ type: z.literal('cancelled'), code: z.string().min(1).max(128) }).strict(),
  z.object({ type: z.literal('validation-failed'), code: z.string().min(1).max(128) }).strict(),
  z.object({ type: z.literal('failed'), code: z.string().min(1).max(128) }).strict()
]);

export type AppEvent = z.infer<typeof appEventSchema>;
