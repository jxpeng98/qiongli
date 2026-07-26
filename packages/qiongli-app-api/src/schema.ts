import { z } from 'zod';

export const APP_API_SCHEMA_VERSION = 5 as const;

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
  legacyCredential: z.object({
    referencePresent: z.boolean(),
    cleanupAvailable: z.boolean()
  }).strict(),
  cleanupRequired: z.boolean()
}).strict();

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
export const academicGraphPortfolioIdSchema = z.string().regex(/^gpf_[0-9a-f]{64}$/);
export const academicGraphPortfolioNodeIdSchema = z.string().regex(/^pnd_[0-9a-f]{64}$/);
export const academicGraphPortfolioEdgeIdSchema = z.string().regex(/^ped_[0-9a-f]{64}$/);
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

export const academicGraphPortfolioProjectSchema = z.object({
  projectId: projectIdSchema,
  displayName: z.string().min(1).max(160),
  lifecycle: projectLifecycleSchema,
  health: projectHealthSchema,
  included: z.boolean(),
  projectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  projectionId: academicGraphProjectionIdSchema.nullable()
}).strict().superRefine((project, context) => {
  if (project.included !== (project.projectRevision !== null && project.projectionId !== null)
    || project.included && project.health !== 'ready') {
    context.addIssue({ code: 'custom', message: 'portfolio project inclusion is inconsistent' });
  }
});

export const academicGraphPortfolioOccurrenceSchema = z.object({
  projectId: projectIdSchema,
  projectionId: academicGraphProjectionIdSchema,
  graphNodeId: academicGraphNodeIdSchema,
  label: z.string().min(1).max(1_024),
  artifactPath: academicGraphArtifactPathSchema,
  sourceAnchor: z.string().min(1).max(512)
}).strict();

export const academicGraphPortfolioNodeSchema = z.object({
  nodeId: academicGraphPortfolioNodeIdSchema,
  nodeType: academicGraphNodeTypeSchema,
  identityScope: academicGraphIdentityScopeSchema,
  canonicalId: z.string().min(1).max(512),
  label: z.string().min(1).max(1_024),
  projectIds: z.array(projectIdSchema).min(1).max(512),
  occurrences: z.array(academicGraphPortfolioOccurrenceSchema).min(1).max(65_536)
}).strict().superRefine((node, context) => {
  const occurrenceProjects = [...new Set(node.occurrences.map((value) => value.projectId))].sort();
  if (!sortedUnique(node.projectIds, [...node.projectIds].sort())
    || occurrenceProjects.join('\0') !== node.projectIds.join('\0')
    || node.identityScope === 'project' && (node.nodeType !== 'project' || node.projectIds.length !== 1)
    || node.identityScope === 'global' && (!['paper', 'concept', 'method'].includes(node.nodeType)
      || node.projectIds.length < 2)) {
    context.addIssue({ code: 'custom', message: 'portfolio node identity is inconsistent' });
  }
});

export const academicGraphPortfolioEdgeOriginSchema = z.object({
  projectId: projectIdSchema,
  projectionId: academicGraphProjectionIdSchema,
  graphEdgeId: academicGraphEdgeIdSchema.nullable(),
  artifactPath: academicGraphArtifactPathSchema,
  sourceAnchor: z.string().min(1).max(512)
}).strict();

export const academicGraphPortfolioEdgeSchema = z.object({
  edgeId: academicGraphPortfolioEdgeIdSchema,
  sourceNodeId: academicGraphPortfolioNodeIdSchema,
  relation: z.enum(['shares-source', 'shares-concept', 'uses-method', 'forked-from', 'extends-project']),
  targetNodeId: academicGraphPortfolioNodeIdSchema,
  sharedCanonicalId: z.string().min(1).max(512).nullable(),
  rationale: z.string().min(1).max(4_096),
  evidenceLimit: z.string().min(1).max(2_048),
  inferenceStrength: academicInferenceStrengthSchema,
  confidence: academicGraphConfidenceSchema,
  status: academicGraphEdgeStatusSchema,
  origins: z.array(academicGraphPortfolioEdgeOriginSchema).min(1).max(65_536)
}).strict().superRefine((edge, context) => {
  const shared = ['shares-source', 'shares-concept', 'uses-method'].includes(edge.relation);
  if (edge.sourceNodeId === edge.targetNodeId
    || shared !== (edge.sharedCanonicalId !== null)
    || shared && edge.origins.some((origin) => origin.graphEdgeId !== null)
    || !shared && edge.origins.some((origin) => origin.graphEdgeId === null)) {
    context.addIssue({ code: 'custom', message: 'portfolio edge provenance is inconsistent' });
  }
});

export const academicGraphPortfolioSnapshotSchema = z.object({
  schemaVersion: z.literal(1),
  documentKind: z.literal('qiongli-academic-graph-portfolio'),
  portfolioId: academicGraphPortfolioIdSchema,
  libraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  projectCount: z.number().int().min(0).max(512),
  includedProjectCount: z.number().int().min(0).max(512),
  skippedProjectCount: z.number().int().min(0).max(512),
  nodeCount: z.number().int().min(0).max(16_384),
  edgeCount: z.number().int().min(0).max(32_768),
  projects: z.array(academicGraphPortfolioProjectSchema).max(512),
  nodes: z.array(academicGraphPortfolioNodeSchema).max(16_384),
  edges: z.array(academicGraphPortfolioEdgeSchema).max(32_768)
}).strict().superRefine((portfolio, context) => {
  const projectIds = portfolio.projects.map((project) => project.projectId);
  const includedProjects = new Map(portfolio.projects
    .filter((project) => project.included)
    .map((project) => [project.projectId, project]));
  const nodes = new Map(portfolio.nodes.map((node) => [node.nodeId, node]));
  const nodeIds = portfolio.nodes.map((node) => node.nodeId);
  const edgeIds = portfolio.edges.map((edge) => edge.edgeId);
  const structurallyValid = portfolio.projectCount === portfolio.projects.length
    && portfolio.includedProjectCount === includedProjects.size
    && portfolio.skippedProjectCount === portfolio.projectCount - portfolio.includedProjectCount
    && portfolio.nodeCount === portfolio.nodes.length
    && portfolio.edgeCount === portfolio.edges.length
    && new Set(projectIds).size === projectIds.length
    && new Set(nodeIds).size === nodeIds.length
    && new Set(edgeIds).size === edgeIds.length
    && sortedUnique(projectIds, [...projectIds].sort())
    && portfolio.nodes.every((node) => node.projectIds.every((id) => includedProjects.has(id)))
    && portfolio.nodes.every((node) => node.occurrences.every((occurrence) => {
      const project = includedProjects.get(occurrence.projectId);
      return project?.projectionId === occurrence.projectionId;
    }))
    && portfolio.edges.every((edge) => {
      const source = nodes.get(edge.sourceNodeId);
      const target = nodes.get(edge.targetNodeId);
      if (!source || !target) return false;
      if (edge.relation === 'shares-source') return target.nodeType === 'paper'
        && target.identityScope === 'global' && target.canonicalId === edge.sharedCanonicalId;
      if (edge.relation === 'shares-concept') return target.nodeType === 'concept'
        && target.identityScope === 'global' && target.canonicalId === edge.sharedCanonicalId;
      if (edge.relation === 'uses-method') return target.nodeType === 'method'
        && target.identityScope === 'global' && target.canonicalId === edge.sharedCanonicalId;
      return source.nodeType === 'project' && target.nodeType === 'project'
        && source.identityScope === 'project' && target.identityScope === 'project';
    });
  if (!structurallyValid) {
    context.addIssue({ code: 'custom', message: 'academic graph portfolio is inconsistent' });
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
export type AcademicGraphPortfolioEdge = z.infer<typeof academicGraphPortfolioEdgeSchema>;
export type AcademicGraphPortfolioNode = z.infer<typeof academicGraphPortfolioNodeSchema>;
export type AcademicGraphPortfolioOccurrence = z.infer<typeof academicGraphPortfolioOccurrenceSchema>;
export type AcademicGraphPortfolioProject = z.infer<typeof academicGraphPortfolioProjectSchema>;
export type AcademicGraphPortfolioSnapshot = z.infer<typeof academicGraphPortfolioSnapshotSchema>;
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
  migration: z.object({
    state: z.enum(['not-detected', 'available', 'review-required', 'unavailable']),
    detectedItems: z.number().int().min(0).max(32),
    eligibleItems: z.number().int().min(0).max(32),
    reviewItems: z.number().int().min(0).max(32)
  }).strict(),
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
  paths: z.array(integrationPathSchema).max(10)
}).strict();

export const legacyMigrationStateSchema = z.enum([
  'not-detected',
  'available',
  'preview-ready',
  'staged',
  'awaiting-client-activation',
  'verification-required',
  'cleanup-ready',
  'complete',
  'recovery-required',
  'review-required',
  'unavailable'
]);

export const legacyMigrationActionSchema = z.enum([
  'none',
  'start',
  'apply',
  'confirm-host-activation',
  'cleanup',
  'finalize',
  'recover',
  'review'
]);

export const legacyMigrationSchema = z.object({
  state: legacyMigrationStateSchema,
  nextAction: legacyMigrationActionSchema,
  migrationId: z.string().regex(/^[A-Za-z0-9_-]{1,128}$/).nullable(),
  detectedItems: z.number().int().min(0).max(8),
  eligibleItems: z.number().int().min(0).max(8),
  reviewItems: z.number().int().min(0).max(8),
  reasonCode: z.string().min(1).max(128)
}).strict();

export type LegacyMigration = z.infer<typeof legacyMigrationSchema>;

const capabilitiesSchema = z.object({
  refresh: z.boolean(),
  skillsMaterialize: z.boolean(),
  integrationDiscovery: z.boolean(),
  integrationPreview: z.boolean(),
  projectLibrary: z.boolean(),
  projectMutation: z.boolean(),
  captureInbox: z.boolean(),
  captureMutation: z.boolean(),
  captureDelivery: z.boolean(),
  captureResolution: z.boolean(),
  academicGraph: z.boolean(),
  portfolio: z.boolean(),
  timeline: z.boolean(),
  orchestrationInspect: z.boolean(),
  orchestrationControl: z.boolean(),
  legacyCredentialCleanup: z.boolean(),
  apply: z.boolean()
}).strict();

export const appSnapshotSchema = z.object({
  schemaVersion: z.literal(APP_API_SCHEMA_VERSION),
  product: productSchema,
  content: contentSchema,
  mcp: mcpSchema,
  configuration: configurationSchema,
  update: updateViewSchema,
  researchLibrary: researchLibrarySnapshotSchema,
  legacyMigration: legacyMigrationSchema,
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
export const orchestrationExecutionModeSchema = z.enum(['solo', 'duo', 'triad']);
export const orchestrationRunStatusSchema = z.enum([
  'planned',
  'running',
  'paused',
  'completed',
  'failed',
  'cancelled'
]);
export const orchestrationRunSummarySchema = z.object({
  runId: z.string().regex(/^run_[0-9a-f]{32}$/),
  profileId: z.string().min(1).max(96),
  executionMode: orchestrationExecutionModeSchema,
  status: orchestrationRunStatusSchema,
  generation: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  documentSha256: sha256Schema,
  completedTaskCount: z.number().int().min(0).max(76),
  totalTaskCount: z.literal(76),
  nextTaskId: z.string().min(1).max(16).nullable(),
  activeTaskId: z.string().min(1).max(16).nullable(),
  activeRole: z.enum(['primary', 'reviewer', 'verifier']).nullable(),
  completedRoleCount: z.number().int().min(0).max(3),
  requiredRoleCount: z.number().int().min(1).max(3),
  hostDriven: z.boolean(),
  recoveryRequired: z.boolean(),
  canContinue: z.boolean(),
  canPause: z.boolean(),
  canResume: z.boolean(),
  canRecover: z.boolean(),
  canCancel: z.boolean()
}).strict();
export const orchestrationRunListSchema = z.object({
  schemaVersion: z.literal(1),
  projectId: projectIdSchema,
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  runs: z.array(orchestrationRunSummarySchema).max(128)
}).strict();
export type OrchestrationRunList = z.infer<typeof orchestrationRunListSchema>;
export type OrchestrationRunSummary = z.infer<typeof orchestrationRunSummarySchema>;

export const deliveryEnvelopeIdSchema = z.string().regex(/^env_[0-9a-f]{64}$/);
export const deliveryAcknowledgementIdSchema = z.string().regex(/^dack_[0-9a-f]{64}$/);
export const captureAssignmentIntentIdSchema = z.string().regex(/^cai_[0-9a-f]{64}$/);
export const captureAssignmentReceiptIdSchema = z.string().regex(/^car_[0-9a-f]{64}$/);
export const captureResolutionItemIdSchema = z.string().regex(/^cri_[0-9a-f]{64}$/);
export const captureResolutionReceiptIdSchema = z.string().regex(/^crr_[0-9a-f]{64}$/);
export const portfolioCatalogIdSchema = z.string().regex(/^pca_[0-9a-f]{64}$/);
export const portfolioQueryRequestIdSchema = z.string().regex(/^pqr_[0-9a-f]{64}$/);
export const portfolioQueryIdSchema = z.string().regex(/^pqy_[0-9a-f]{64}$/);
export const portfolioQueryCursorIdSchema = z.string().regex(/^pqc_[0-9a-f]{64}$/);
export const semanticTimelineRequestIdSchema = z.string().regex(/^ptr_[0-9a-f]{64}$/);
export const semanticTimelineQueryIdSchema = z.string().regex(/^pty_[0-9a-f]{64}$/);
export const semanticTimelineCursorIdSchema = z.string().regex(/^ptc_[0-9a-f]{64}$/);
export const semanticTimelineEventIdSchema = z.string().regex(/^pte_[0-9a-f]{64}$/);
export const continuityOperationIdSchema = z.string().regex(/^cop_[0-9a-f]{64}$/);

export const captureDeliveryStateSchema = z.enum([
  'queued',
  'delivering',
  'delivered',
  'acknowledged',
  'retry-required',
  'conflicted',
  'cancelled'
]);
export const captureDeliveryReasonSchema = z.enum([
  'delivery-enqueued',
  'delivery-attempt-started',
  'delivery-retry-started',
  'delivery-accepted',
  'delivery-process-interrupted',
  'delivery-transport-unavailable',
  'delivery-destination-unavailable',
  'delivery-destination-conflict',
  'delivery-revision-conflict',
  'delivery-retry-requested',
  'delivery-acknowledged',
  'delivery-cancelled',
  'delivery-recovery-required'
]);
export const captureDeliveryRetryCauseSchema = z.enum([
  'process-interrupted',
  'transport-unavailable',
  'destination-unavailable',
  'recovery-required',
  'conflict-resolved'
]);
export const captureAssignmentDecisionSchema = z.enum(['assign', 'reject']);
export const captureAssignmentOutcomeSchema = z.enum(['assigned', 'rejected']);
export const captureAssignmentStatusStateSchema = z.enum(['pending', 'completed']);
export const captureAssignmentPreviewOutcomeSchema = z.enum([
  'ready',
  'duplicate',
  'resolution-required',
  'rejected'
]);
export const captureAssignmentBindingEffectSchema = z.enum(['direct', 'rebound']);
export const captureResolutionItemKindSchema = z.enum([
  'semantic-change',
  'decision',
  'evidence',
  'contradiction',
  'next-action'
]);
export const captureResolutionCounterpartStateSchema = z.enum([
  'absent',
  'exact-match',
  'exact-identity-divergent'
]);
export const captureResolutionDispositionSchema = z.enum([
  'accept-current',
  'accept-capture',
  'retain-both',
  'reject-capture'
]);

export const continuityCursorKindSchema = z.enum([
  'deliveries',
  'assignments',
  'resolutions'
]);
export const continuityCursorSchema = z.object({
  schemaVersion: z.literal(1),
  cursorId: z.string().regex(/^apc_[0-9a-f]{64}$/),
  kind: continuityCursorKindSchema,
  snapshotId: z.string().regex(/^(dls|als|rls)_[0-9a-f]{64}$/),
  afterId: z.string().min(1).max(160)
}).strict().superRefine((cursor, context) => {
  const expectedPrefix = {
    deliveries: 'dls_',
    assignments: 'als_',
    resolutions: 'rls_'
  }[cursor.kind];
  if (!cursor.snapshotId.startsWith(expectedPrefix)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'continuity cursor kind and snapshot identity are inconsistent',
      path: ['snapshotId']
    });
  }
});

export const captureDeliveryListRequestSchema = z.object({
  projectId: projectIdSchema.optional(),
  states: z.array(captureDeliveryStateSchema).max(7).optional(),
  limit: z.number().int().min(1).max(256),
  cursor: continuityCursorSchema.optional()
}).strict().superRefine((request, context) => {
  if (request.cursor && request.cursor.kind !== 'deliveries') {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'capture delivery list requires a delivery cursor',
      path: ['cursor', 'kind']
    });
  }
});

export const captureAssignmentListRequestSchema = z.object({
  projectId: projectIdSchema.optional(),
  states: z.array(captureAssignmentStatusStateSchema).max(2).optional(),
  limit: z.number().int().min(1).max(256),
  cursor: continuityCursorSchema.optional()
}).strict().superRefine((request, context) => {
  if (request.cursor && request.cursor.kind !== 'assignments') {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'capture assignment list requires an assignment cursor',
      path: ['cursor', 'kind']
    });
  }
});

export const captureResolutionListRequestSchema = z.object({
  projectId: projectIdSchema,
  limit: z.number().int().min(1).max(128),
  cursor: continuityCursorSchema.optional()
}).strict().superRefine((request, context) => {
  if (request.cursor && request.cursor.kind !== 'resolutions') {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'capture resolution list requires a resolution cursor',
      path: ['cursor', 'kind']
    });
  }
});

export const captureDeliveryDestinationSchema = z.object({
  projectId: projectIdSchema,
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER)
}).strict();
export const captureDeliveryAcknowledgementSchema = z.object({
  acknowledgementId: deliveryAcknowledgementIdSchema,
  destinationProjectId: projectIdSchema,
  acceptedCaptureId: captureIdSchema,
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  resultingProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  acknowledgedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER)
}).strict().superRefine((acknowledgement, context) => {
  if (acknowledgement.resultingProjectRevision < acknowledgement.expectedProjectRevision) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'acknowledgement resulting revision precedes its expected revision',
      path: ['resultingProjectRevision']
    });
  }
});
export const captureDeliveryCapabilitiesSchema = z.object({
  canRetry: z.boolean(),
  canCancel: z.boolean(),
  canAcknowledge: z.boolean()
}).strict();
export const captureDeliveryViewSchema = z.object({
  schemaVersion: z.literal(1),
  envelopeId: deliveryEnvelopeIdSchema,
  captureId: captureIdSchema,
  source: captureSourceSchema,
  delivery: captureDeliverySchema,
  destination: captureDeliveryDestinationSchema.nullable(),
  state: captureDeliveryStateSchema,
  generation: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  attemptCount: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  retryCount: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  createdAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  updatedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  lastReason: captureDeliveryReasonSchema,
  envelopeSha256: sha256Schema,
  recordSha256: sha256Schema,
  acknowledgement: captureDeliveryAcknowledgementSchema.nullable(),
  capabilities: captureDeliveryCapabilitiesSchema
}).strict().superRefine((delivery, context) => {
  if ((delivery.state === 'acknowledged') !== (delivery.acknowledgement !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'acknowledged delivery state and acknowledgement evidence are inconsistent',
      path: ['acknowledgement']
    });
  }
  if (delivery.capabilities.canAcknowledge
    && (delivery.state !== 'delivered' || delivery.destination === null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'only a delivered, project-bound record can be acknowledged',
      path: ['capabilities', 'canAcknowledge']
    });
  }
});
export const captureDeliveryPageSchema = z.object({
  schemaVersion: z.literal(1),
  snapshotId: z.string().regex(/^dls_[0-9a-f]{64}$/),
  projectId: projectIdSchema.nullable(),
  entries: z.array(captureDeliveryViewSchema).max(256),
  truncated: z.boolean(),
  nextCursor: continuityCursorSchema.nullable()
}).strict().superRefine((page, context) => {
  if (page.truncated !== (page.nextCursor !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'delivery page truncation and next cursor are inconsistent',
      path: ['nextCursor']
    });
  }
  if (page.nextCursor
    && (page.nextCursor.kind !== 'deliveries'
      || page.nextCursor.snapshotId !== page.snapshotId)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'delivery next cursor does not bind this snapshot',
      path: ['nextCursor']
    });
  }
});
export const captureDeliveryAcknowledgementPreviewSchema = z.object({
  schemaVersion: z.literal(1),
  planDigest: sha256Schema,
  envelopeId: deliveryEnvelopeIdSchema,
  destinationProjectId: projectIdSchema,
  acceptedCaptureId: captureIdSchema,
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  resultingProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  acknowledgedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  expectedGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  expectedRecordSha256: sha256Schema,
  approvalsRequired: z.array(z.literal('delivery-acknowledgement')).length(1)
}).strict();

export const captureAssignmentViewSchema = z.object({
  schemaVersion: z.literal(1),
  state: captureAssignmentStatusStateSchema,
  intentId: captureAssignmentIntentIdSchema,
  sourceEnvelopeId: deliveryEnvelopeIdSchema,
  sourceCaptureId: captureIdSchema,
  targetProjectId: projectIdSchema,
  targetProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  outcome: captureAssignmentOutcomeSchema.nullable(),
  receiptId: captureAssignmentReceiptIdSchema.nullable(),
  derivedCaptureId: captureIdSchema.nullable(),
  childEnvelopeId: deliveryEnvelopeIdSchema.nullable(),
  createdAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  decidedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER).nullable(),
  canResolve: z.boolean()
}).strict().superRefine((assignment, context) => {
  const completed = assignment.state === 'completed';
  if (completed !== (assignment.outcome !== null
    && assignment.receiptId !== null
    && assignment.decidedAtUnix !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'assignment completion fields are inconsistent'
    });
  }
  const assigned = assignment.outcome === 'assigned';
  if (assigned !== (assignment.derivedCaptureId !== null
    && assignment.childEnvelopeId !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'assignment child lineage is inconsistent'
    });
  }
  if (assignment.canResolve && !assigned) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'only an assigned capture can enter academic resolution',
      path: ['canResolve']
    });
  }
});
export const captureAssignmentPageSchema = z.object({
  schemaVersion: z.literal(1),
  snapshotId: z.string().regex(/^als_[0-9a-f]{64}$/),
  projectId: projectIdSchema.nullable(),
  entries: z.array(captureAssignmentViewSchema).max(256),
  truncated: z.boolean(),
  nextCursor: continuityCursorSchema.nullable()
}).strict().superRefine((page, context) => {
  if (page.truncated !== (page.nextCursor !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'assignment page truncation and next cursor are inconsistent',
      path: ['nextCursor']
    });
  }
  if (page.nextCursor
    && (page.nextCursor.kind !== 'assignments'
      || page.nextCursor.snapshotId !== page.snapshotId)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'assignment next cursor does not bind this snapshot',
      path: ['nextCursor']
    });
  }
});
export const captureAssignmentPreviewSchema = z.object({
  schemaVersion: z.literal(1),
  planDigest: sha256Schema,
  intentId: captureAssignmentIntentIdSchema,
  decision: captureAssignmentDecisionSchema,
  outcome: captureAssignmentPreviewOutcomeSchema,
  bindingEffect: captureAssignmentBindingEffectSchema,
  sourceDisposition: captureDispositionSchema,
  sourceEnvelopeId: deliveryEnvelopeIdSchema,
  sourceCaptureId: captureIdSchema,
  sourceRecordState: captureDeliveryStateSchema,
  expectedSourceGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  targetProjectId: projectIdSchema,
  expectedLibraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  targetStage: projectStageSchema,
  derivedCaptureId: captureIdSchema.nullable(),
  childEnvelopeId: deliveryEnvelopeIdSchema.nullable(),
  resolutionRequired: z.boolean(),
  decidedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  explanation: z.string().min(1).max(1_024),
  approvalsRequired: z.array(z.string().min(1).max(64)).max(2)
}).strict().superRefine((assignment, context) => {
  const assigned = assignment.decision === 'assign';
  if (assigned !== (assignment.derivedCaptureId !== null
    && assignment.childEnvelopeId !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'assignment preview child lineage is inconsistent'
    });
  }
  if (assignment.resolutionRequired !== (assignment.outcome === 'resolution-required')) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'assignment resolution requirement is inconsistent',
      path: ['resolutionRequired']
    });
  }
});

export const captureResolutionSelectionSchema = z.object({
  itemId: captureResolutionItemIdSchema,
  disposition: captureResolutionDispositionSchema
}).strict();
export const captureResolutionDecisionSchema = z.object({
  itemId: captureResolutionItemIdSchema,
  kind: captureResolutionItemKindSchema,
  disposition: captureResolutionDispositionSchema
}).strict();
export const captureResolutionViewSchema = z.object({
  schemaVersion: z.literal(1),
  receiptId: captureResolutionReceiptIdSchema,
  assignmentReceiptId: captureAssignmentReceiptIdSchema,
  sourceEnvelopeId: deliveryEnvelopeIdSchema,
  sourceCaptureId: captureIdSchema,
  derivedCaptureId: captureIdSchema,
  childEnvelopeId: deliveryEnvelopeIdSchema,
  targetProjectId: projectIdSchema,
  fromProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  toProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  reviewedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  resolvedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  decisions: z.array(captureResolutionDecisionSchema).min(1).max(80)
}).strict().superRefine((resolution, context) => {
  if (resolution.toProjectRevision !== resolution.fromProjectRevision + 1) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution must advance exactly one project revision',
      path: ['toProjectRevision']
    });
  }
  if (resolution.resolvedAtUnix < resolution.reviewedAtUnix) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution timestamp precedes review',
      path: ['resolvedAtUnix']
    });
  }
});
export const captureResolutionPageSchema = z.object({
  schemaVersion: z.literal(1),
  snapshotId: z.string().regex(/^rls_[0-9a-f]{64}$/),
  projectId: projectIdSchema,
  entries: z.array(captureResolutionViewSchema).max(128),
  truncated: z.boolean(),
  nextCursor: continuityCursorSchema.nullable()
}).strict().superRefine((page, context) => {
  if (page.truncated !== (page.nextCursor !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution page truncation and next cursor are inconsistent',
      path: ['nextCursor']
    });
  }
  if (page.nextCursor
    && (page.nextCursor.kind !== 'resolutions'
      || page.nextCursor.snapshotId !== page.snapshotId)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution next cursor does not bind this snapshot',
      path: ['nextCursor']
    });
  }
});
export const captureResolutionItemPreviewSchema = z.object({
  itemId: captureResolutionItemIdSchema,
  kind: captureResolutionItemKindSchema,
  counterpartState: captureResolutionCounterpartStateSchema,
  allowedDispositions: z.array(captureResolutionDispositionSchema).min(1).max(4),
  unavailableDispositions: z.array(captureResolutionDispositionSchema).max(4),
  sourceSummary: z.string().min(1).max(4_096),
  currentSummary: z.string().min(1).max(4_096).nullable(),
  explanation: z.string().min(1).max(2_048)
}).strict().superRefine((item, context) => {
  const overlap = item.allowedDispositions
    .some((disposition) => item.unavailableDispositions.includes(disposition));
  if (overlap) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution disposition cannot be both allowed and unavailable'
    });
  }
});
export const captureResolutionPreviewSchema = z.object({
  schemaVersion: z.literal(1),
  planDigest: sha256Schema,
  assignmentReceiptId: captureAssignmentReceiptIdSchema,
  sourceEnvelopeId: deliveryEnvelopeIdSchema,
  sourceCaptureId: captureIdSchema,
  derivedCaptureId: captureIdSchema,
  childEnvelopeId: deliveryEnvelopeIdSchema,
  targetProjectId: projectIdSchema,
  expectedLibraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  nextProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  reviewedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  items: z.array(captureResolutionItemPreviewSchema).min(1).max(80),
  approvalsRequired: z.array(z.string().min(1).max(64)).max(2),
  exactReplay: z.boolean()
}).strict().superRefine((resolution, context) => {
  if (resolution.nextProjectRevision !== resolution.expectedProjectRevision + 1) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution preview must advance exactly one project revision',
      path: ['nextProjectRevision']
    });
  }
});

export const portfolioCatalogStateSchema = z.enum([
  'current',
  'missing',
  'stale',
  'recovery-required'
]);
export const portfolioCapabilitiesSchema = z.object({
  canQuery: z.boolean(),
  canReconcile: z.boolean(),
  canRebuild: z.boolean(),
  canDeleteDerivedState: z.boolean()
}).strict();
export const portfolioStatusSchema = z.object({
  schemaVersion: z.literal(1),
  state: portfolioCatalogStateSchema,
  libraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  catalogId: portfolioCatalogIdSchema.nullable(),
  catalogGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  portfolioId: academicGraphPortfolioIdSchema.nullable(),
  contributionCount: z.number().int().min(0).max(1_024),
  projectCount: z.number().int().min(0).max(1_024),
  nodeCount: z.number().int().min(0).max(65_536),
  edgeCount: z.number().int().min(0).max(131_072),
  reasonCode: z.string().min(1).max(128),
  capabilities: portfolioCapabilitiesSchema
}).strict().superRefine((portfolio, context) => {
  const current = portfolio.state === 'current';
  if (current !== (portfolio.catalogId !== null
    && portfolio.catalogGeneration !== null
    && portfolio.portfolioId !== null
    && portfolio.capabilities.canQuery)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'current portfolio identity and query capability are inconsistent'
    });
  }
  if (!current && portfolio.capabilities.canQuery) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'a non-current portfolio cannot be queried',
      path: ['capabilities', 'canQuery']
    });
  }
});

export const portfolioEvidenceSignalSchema = z.enum(['gap', 'contradiction']);
export const portfolioSharedIdentityFilterSchema = z.object({
  nodeType: z.enum(['paper', 'concept', 'method']),
  canonicalId: z.string().min(1).max(512)
}).strict();
export const portfolioQueryFiltersSchema = z.object({
  projectId: projectIdSchema.optional(),
  stage: projectStageSchema.optional(),
  evidenceSignal: portfolioEvidenceSignalSchema.optional(),
  manuscriptSection: z.string().min(1).max(512).optional(),
  sharedIdentity: portfolioSharedIdentityFilterSchema.optional(),
  captureSource: captureSourceSchema.optional(),
  captureDelivery: captureDeliverySchema.optional(),
  deliveryState: captureDeliveryStateSchema.optional(),
  assignmentOutcome: captureAssignmentOutcomeSchema.optional(),
  lineageId: z.string().min(1).max(160).optional(),
  text: z.string().min(1).max(256).optional()
}).strict();
export const portfolioQueryLimitsSchema = z.object({
  projects: z.number().int().min(1).max(128),
  nodes: z.number().int().min(1).max(256),
  edges: z.number().int().min(1).max(256),
  lineage: z.number().int().min(1).max(256),
  maxBytes: z.number().int().min(65_536).max(4 * 1_024 * 1_024)
}).strict();
export const portfolioQueryCursorSchema = z.object({
  cursorId: portfolioQueryCursorIdSchema,
  queryId: portfolioQueryIdSchema,
  projectAfter: z.string().min(1).max(512).optional(),
  nodeAfter: z.string().min(1).max(512).optional(),
  edgeAfter: z.string().min(1).max(512).optional(),
  lineageAfter: z.string().min(1).max(512).optional()
}).strict();
export const portfolioQueryRequestSchema = z.object({
  catalogId: portfolioCatalogIdSchema,
  filters: portfolioQueryFiltersSchema.optional(),
  limits: portfolioQueryLimitsSchema,
  cursor: portfolioQueryCursorSchema.optional()
}).strict();

export const portfolioQueryProjectSchema = z.object({
  resultId: z.string().min(1).max(256),
  projectId: projectIdSchema,
  displayName: z.string().min(1).max(160),
  stage: projectStageSchema,
  lifecycle: projectLifecycleSchema,
  health: projectHealthSchema,
  semanticRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
  projectionId: academicGraphProjectionIdSchema,
  nodeCount: z.number().int().min(0).max(65_536),
  edgeCount: z.number().int().min(0).max(131_072),
  lineageCount: z.number().int().min(0).max(65_536)
}).strict();
export const portfolioQueryNodeSchema = z.object({
  resultId: z.string().min(1).max(512),
  projectId: projectIdSchema,
  projectionId: academicGraphProjectionIdSchema,
  node: academicGraphNodeSchema
}).strict();
export const portfolioQueryEdgeSchema = z.object({
  resultId: z.string().min(1).max(512),
  projectId: projectIdSchema,
  projectionId: academicGraphProjectionIdSchema,
  edge: academicGraphEdgeSchema
}).strict();
export const portfolioLineageKindSchema = z.enum([
  'capture',
  'consolidation',
  'delivery',
  'assignment',
  'resolution'
]);
export const portfolioLineageSchema = z.object({
  lineageId: z.string().regex(/^lin_[0-9a-f]{64}$/),
  kind: portfolioLineageKindSchema,
  projectIds: z.array(projectIdSchema).min(1).max(2),
  relatedIds: z.array(z.string().min(1).max(160)).min(1).max(24),
  occurredAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  source: captureSourceSchema.nullable(),
  delivery: captureDeliverySchema.nullable(),
  deliveryState: captureDeliveryStateSchema.nullable(),
  assignmentOutcome: captureAssignmentOutcomeSchema.nullable(),
  fromProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  toProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable()
}).strict();
export const portfolioQueryResultSchema = z.object({
  schemaVersion: z.literal(1),
  requestId: portfolioQueryRequestIdSchema,
  queryId: portfolioQueryIdSchema,
  catalogId: portfolioCatalogIdSchema,
  portfolioId: academicGraphPortfolioIdSchema,
  lineageDigest: z.string().regex(/^plg_[0-9a-f]{64}$/),
  matchedProjectCount: z.number().int().min(0).max(1_024),
  matchedNodeCount: z.number().int().min(0).max(65_536),
  matchedEdgeCount: z.number().int().min(0).max(131_072),
  matchedLineageCount: z.number().int().min(0).max(65_536),
  projectsTruncated: z.boolean(),
  nodesTruncated: z.boolean(),
  edgesTruncated: z.boolean(),
  lineageTruncated: z.boolean(),
  projects: z.array(portfolioQueryProjectSchema).max(128),
  nodes: z.array(portfolioQueryNodeSchema).max(256),
  edges: z.array(portfolioQueryEdgeSchema).max(256),
  lineage: z.array(portfolioLineageSchema).max(256),
  nextCursor: portfolioQueryCursorSchema.nullable()
}).strict().superRefine((result, context) => {
  const anyTruncated = result.projectsTruncated
    || result.nodesTruncated
    || result.edgesTruncated
    || result.lineageTruncated;
  if (anyTruncated !== (result.nextCursor !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'portfolio truncation and next cursor are inconsistent',
      path: ['nextCursor']
    });
  }
  if (result.nextCursor && result.nextCursor.queryId !== result.queryId) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'portfolio cursor does not bind this query',
      path: ['nextCursor', 'queryId']
    });
  }
});

export const semanticTimelineViewSchema = z.enum([
  'activity',
  'revision-history',
  'merge-resolution-history'
]);
export const semanticActivityKindSchema = z.enum([
  'project-registered',
  'project-revision-observed',
  'project-lifecycle-observed',
  'capture-accepted',
  'capture-consolidated',
  'delivery-queued',
  'delivery-started',
  'delivery-delivered',
  'delivery-acknowledged',
  'delivery-retry-required',
  'delivery-conflicted',
  'delivery-cancelled',
  'assignment-created',
  'capture-assigned',
  'capture-assignment-rejected',
  'resolution-reviewed',
  'resolution-item-resolved',
  'resolution-completed'
]);
export const semanticActivityTimestampSourceSchema = z.enum([
  'project-registered-at',
  'project-academically-updated-at',
  'capture-captured-at',
  'consolidation-consolidated-at',
  'delivery-transitioned-at',
  'assignment-created-at',
  'assignment-decided-at',
  'resolution-reviewed-at',
  'resolution-resolved-at'
]);
export const semanticTimelineCursorSchema = z.object({
  cursorId: semanticTimelineCursorIdSchema,
  queryId: semanticTimelineQueryIdSchema,
  afterOccurredAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  afterEventId: semanticTimelineEventIdSchema
}).strict();
export const semanticTimelineRequestSchema = z.object({
  catalogId: portfolioCatalogIdSchema,
  projectId: projectIdSchema.optional(),
  view: semanticTimelineViewSchema,
  limit: z.number().int().min(1).max(512),
  maxBytes: z.number().int().min(65_536).max(4 * 1_024 * 1_024),
  cursor: semanticTimelineCursorSchema.optional()
}).strict();
export const semanticActivitySchema = z.object({
  eventId: semanticTimelineEventIdSchema,
  kind: semanticActivityKindSchema,
  occurredAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  timestampSource: semanticActivityTimestampSourceSchema,
  projectIds: z.array(projectIdSchema).min(1).max(2),
  relatedIds: z.array(z.string().min(1).max(160)).min(1).max(24),
  fromProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  toProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  lifecycle: projectLifecycleSchema.nullable(),
  source: captureSourceSchema.nullable(),
  delivery: captureDeliverySchema.nullable(),
  deliveryState: captureDeliveryStateSchema.nullable(),
  deliveryReason: captureDeliveryReasonSchema.nullable(),
  deliveryGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  assignmentOutcome: captureAssignmentOutcomeSchema.nullable(),
  resolutionItemId: captureResolutionItemIdSchema.nullable(),
  resolutionItemKind: captureResolutionItemKindSchema.nullable(),
  resolutionDisposition: captureResolutionDispositionSchema.nullable()
}).strict().superRefine((event, context) => {
  const itemResolution = event.kind === 'resolution-item-resolved';
  if (itemResolution !== (event.resolutionItemId !== null
    && event.resolutionItemKind !== null
    && event.resolutionDisposition !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'resolution item event details are inconsistent'
    });
  }
});
export const semanticTimelineResultSchema = z.object({
  schemaVersion: z.literal(1),
  requestId: semanticTimelineRequestIdSchema,
  queryId: semanticTimelineQueryIdSchema,
  catalogId: portfolioCatalogIdSchema,
  portfolioId: academicGraphPortfolioIdSchema,
  timelineDigest: z.string().regex(/^ptl_[0-9a-f]{64}$/),
  projectId: projectIdSchema.nullable(),
  view: semanticTimelineViewSchema,
  matchedEventCount: z.number().int().min(0).max(65_536),
  truncated: z.boolean(),
  events: z.array(semanticActivitySchema).max(512),
  nextCursor: semanticTimelineCursorSchema.nullable()
}).strict().superRefine((result, context) => {
  if (result.truncated !== (result.nextCursor !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'timeline truncation and next cursor are inconsistent',
      path: ['nextCursor']
    });
  }
  if (result.nextCursor && result.nextCursor.queryId !== result.queryId) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'timeline cursor does not bind this query',
      path: ['nextCursor', 'queryId']
    });
  }
});

export const portfolioDoctorStatusSchema = z.enum(['missing', 'equivalent', 'divergent']);
export const portfolioDoctorSchema = z.object({
  schemaVersion: z.literal(1),
  status: portfolioDoctorStatusSchema,
  libraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  catalogId: portfolioCatalogIdSchema.nullable(),
  incrementalPortfolioId: academicGraphPortfolioIdSchema.nullable(),
  cleanPortfolioId: academicGraphPortfolioIdSchema,
  byteEquivalent: z.boolean(),
  contributionCount: z.number().int().min(0).max(1_024)
}).strict().superRefine((doctor, context) => {
  const equivalent = doctor.status === 'equivalent';
  if (equivalent !== doctor.byteEquivalent) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'portfolio doctor equivalence state is inconsistent',
      path: ['byteEquivalent']
    });
  }
  if (doctor.status === 'missing'
    && (doctor.catalogId !== null || doctor.incrementalPortfolioId !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'missing portfolio doctor result cannot expose current identities'
    });
  }
});
export const portfolioMaintenanceOperationSchema = z.enum([
  'reconcile',
  'full-rebuild',
  'delete-derived-state'
]);
export const portfolioMaintenancePreviewSchema = z.object({
  schemaVersion: z.literal(1),
  planDigest: sha256Schema,
  operation: portfolioMaintenanceOperationSchema,
  expectedLibraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  expectedCatalogId: portfolioCatalogIdSchema.nullable(),
  expectedCatalogGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER).nullable(),
  currentContributionCount: z.number().int().min(0).max(1_024),
  derivedStateOnly: z.literal(true),
  explanation: z.string().min(1).max(1_024),
  approvalsRequired: z.array(z.literal('derived-state-write')).length(1)
}).strict().superRefine((maintenance, context) => {
  if ((maintenance.expectedCatalogId === null)
    !== (maintenance.expectedCatalogGeneration === null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'portfolio maintenance catalog identity is incomplete'
    });
  }
});
export const continuityOperationPhaseSchema = z.enum([
  'queued',
  'running',
  'completed',
  'cancelled',
  'recovery-required',
  'failed'
]);
export const continuityOperationProgressSchema = z.object({
  schemaVersion: z.literal(1),
  operationId: continuityOperationIdSchema,
  operation: portfolioMaintenanceOperationSchema,
  phase: continuityOperationPhaseSchema,
  completedUnits: z.number().int().min(0).max(1_024),
  totalUnits: z.number().int().min(1).max(1_024),
  catalogId: portfolioCatalogIdSchema.nullable(),
  cancellable: z.boolean(),
  reasonCode: z.string().min(1).max(128)
}).strict().superRefine((progress, context) => {
  if (progress.completedUnits > progress.totalUnits) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'continuity progress exceeds its total',
      path: ['completedUnits']
    });
  }
  const active = progress.phase === 'queued' || progress.phase === 'running';
  if (progress.cancellable && !active) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'terminal continuity progress cannot be cancellable',
      path: ['cancellable']
    });
  }
});
export const portfolioMaintenanceResultSchema = z.object({
  schemaVersion: z.literal(1),
  operationId: continuityOperationIdSchema,
  operation: portfolioMaintenanceOperationSchema,
  libraryRevision: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
  catalogId: portfolioCatalogIdSchema.nullable(),
  portfolioId: academicGraphPortfolioIdSchema.nullable(),
  catalogChanged: z.boolean(),
  rebuiltProjectCount: z.number().int().min(0).max(1_024),
  reusedProjectCount: z.number().int().min(0).max(1_024),
  removedProjectCount: z.number().int().min(0).max(1_024),
  removedContributionCount: z.number().int().min(0).max(1_024),
  derivedStateOnly: z.literal(true)
}).strict().superRefine((result, context) => {
  if (result.operation === 'delete-derived-state'
    && (result.catalogId !== null || result.portfolioId !== null)) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'derived-state deletion cannot return a current catalog'
    });
  }
});

export type ContinuityCursor = z.infer<typeof continuityCursorSchema>;
export type CaptureDeliveryListRequest = z.infer<typeof captureDeliveryListRequestSchema>;
export type CaptureDeliveryView = z.infer<typeof captureDeliveryViewSchema>;
export type CaptureDeliveryPage = z.infer<typeof captureDeliveryPageSchema>;
export type CaptureDeliveryAcknowledgementPreview = z.infer<
  typeof captureDeliveryAcknowledgementPreviewSchema
>;
export type CaptureAssignmentListRequest = z.infer<typeof captureAssignmentListRequestSchema>;
export type CaptureAssignmentView = z.infer<typeof captureAssignmentViewSchema>;
export type CaptureAssignmentPage = z.infer<typeof captureAssignmentPageSchema>;
export type CaptureAssignmentPreview = z.infer<typeof captureAssignmentPreviewSchema>;
export type CaptureResolutionListRequest = z.infer<typeof captureResolutionListRequestSchema>;
export type CaptureResolutionSelection = z.infer<typeof captureResolutionSelectionSchema>;
export type CaptureResolutionView = z.infer<typeof captureResolutionViewSchema>;
export type CaptureResolutionPage = z.infer<typeof captureResolutionPageSchema>;
export type CaptureResolutionPreview = z.infer<typeof captureResolutionPreviewSchema>;
export type PortfolioStatus = z.infer<typeof portfolioStatusSchema>;
export type PortfolioQueryFilters = z.infer<typeof portfolioQueryFiltersSchema>;
export type PortfolioQueryRequest = z.infer<typeof portfolioQueryRequestSchema>;
export type PortfolioQueryResult = z.infer<typeof portfolioQueryResultSchema>;
export type SemanticTimelineRequest = z.infer<typeof semanticTimelineRequestSchema>;
export type SemanticTimelineResult = z.infer<typeof semanticTimelineResultSchema>;
export type PortfolioDoctor = z.infer<typeof portfolioDoctorSchema>;
export type PortfolioMaintenancePreview = z.infer<typeof portfolioMaintenancePreviewSchema>;
export type ContinuityOperationProgress = z.infer<typeof continuityOperationProgressSchema>;
export type PortfolioMaintenanceResult = z.infer<typeof portfolioMaintenanceResultSchema>;

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
    action: z.literal('select-project-migration-locations'),
    suggestedName: projectDialogNameSchema
  }).strict(),
  z.object({
    action: z.literal('preview-project-migration'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/),
    displayName: z.string().min(1).max(160),
    projectKind: projectKindSchema,
    stage: projectStageSchema
  }).strict(),
  z.object({ action: z.literal('select-project-migration-recovery-locations') }).strict(),
  z.object({
    action: z.literal('preview-project-migration-recovery'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/)
  }).strict(),
  z.object({ action: z.literal('select-project-migration-rollback-locations') }).strict(),
  z.object({
    action: z.literal('preview-project-migration-rollback'),
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
  z.object({ action: z.literal('load-academic-graph-portfolio') }).strict(),
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
  z.object({
    action: z.literal('load-capture-deliveries'),
    request: captureDeliveryListRequestSchema
  }).strict(),
  z.object({
    action: z.literal('inspect-capture-delivery'),
    envelopeId: deliveryEnvelopeIdSchema
  }).strict(),
  z.object({
    action: z.literal('retry-capture-delivery'),
    envelopeId: deliveryEnvelopeIdSchema,
    expectedGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    expectedRecordSha256: sha256Schema,
    retriedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
    cause: captureDeliveryRetryCauseSchema
  }).strict(),
  z.object({
    action: z.literal('cancel-capture-delivery'),
    envelopeId: deliveryEnvelopeIdSchema,
    expectedGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    expectedRecordSha256: sha256Schema,
    cancelledAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER)
  }).strict(),
  z.object({
    action: z.literal('preview-capture-delivery-acknowledgement'),
    envelopeId: deliveryEnvelopeIdSchema,
    destinationProjectId: projectIdSchema,
    acceptedCaptureId: captureIdSchema,
    expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    resultingProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    acknowledgedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
    expectedGeneration: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    expectedRecordSha256: sha256Schema
  }).strict(),
  z.object({
    action: z.literal('load-capture-assignments'),
    request: captureAssignmentListRequestSchema
  }).strict(),
  z.object({
    action: z.literal('inspect-capture-assignment'),
    intentId: captureAssignmentIntentIdSchema
  }).strict(),
  z.object({
    action: z.literal('preview-capture-assignment'),
    sourceEnvelopeId: deliveryEnvelopeIdSchema,
    targetProjectId: projectIdSchema,
    decision: captureAssignmentDecisionSchema,
    decidedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER)
  }).strict(),
  z.object({
    action: z.literal('load-capture-resolutions'),
    request: captureResolutionListRequestSchema
  }).strict(),
  z.object({
    action: z.literal('inspect-capture-resolution'),
    projectId: projectIdSchema,
    receiptId: captureResolutionReceiptIdSchema
  }).strict(),
  z.object({
    action: z.literal('preview-capture-resolution'),
    assignmentReceiptId: captureAssignmentReceiptIdSchema,
    reviewedAtUnix: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
    selections: z.array(captureResolutionSelectionSchema).max(80).optional()
  }).strict(),
  z.object({ action: z.literal('load-portfolio-status') }).strict(),
  z.object({
    action: z.literal('query-portfolio'),
    request: portfolioQueryRequestSchema
  }).strict(),
  z.object({
    action: z.literal('load-semantic-timeline'),
    request: semanticTimelineRequestSchema
  }).strict(),
  z.object({ action: z.literal('load-portfolio-doctor') }).strict(),
  z.object({
    action: z.literal('preview-portfolio-maintenance'),
    operation: portfolioMaintenanceOperationSchema
  }).strict(),
  z.object({
    action: z.literal('poll-continuity-operation'),
    operationId: continuityOperationIdSchema
  }).strict(),
  z.object({
    action: z.literal('cancel-continuity-operation'),
    operationId: continuityOperationIdSchema
  }).strict(),
  z.object({ action: z.literal('refresh-integration-discovery') }).strict(),
  z.object({ action: z.literal('prepare-legacy-migration') }).strict(),
  z.object({ action: z.literal('preview-legacy-migration-next') }).strict(),
  z.object({ action: z.literal('select-update-stream'), stream: updateStreamSchema }).strict(),
  z.object({ action: z.literal('check-for-updates') }).strict(),
  z.object({ action: z.literal('prepare-update') }).strict(),
  z.object({ action: z.literal('poll-update') }).strict(),
  z.object({ action: z.literal('cancel-update') }).strict(),
  z.object({ action: z.literal('preview-update-install') }).strict(),
  z.object({ action: z.literal('preview-remove-agent-backend-credential') }).strict(),
  z.object({
    action: z.literal('load-orchestration'),
    projectId: projectIdSchema,
    expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER)
  }).strict(),
  z.object({
    action: z.literal('control-orchestration'),
    projectId: projectIdSchema,
    expectedProjectRevision: z.number().int().min(1).max(Number.MAX_SAFE_INTEGER),
    runId: z.string().regex(/^run_[0-9a-f]{32}$/),
    expectedGeneration: z.number().int().min(0).max(Number.MAX_SAFE_INTEGER),
    expectedDocumentSha256: sha256Schema,
    actionName: z.enum(['pause', 'recover', 'resume', 'cancel'])
  }).strict(),
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

export const projectMigrationPreviewSchema = z.object({
  mode: z.enum(['copy', 'recovery']),
  copiedFileCount: z.number().int().nonnegative(),
  copiedBytes: z.number().int().nonnegative(),
  excludedEntryCount: z.number().int().nonnegative(),
  sourceRetained: z.literal(true),
  copiesFiles: z.boolean(),
  graphRebuildPasses: z.literal(2)
}).strict().superRefine((preview, context) => {
  if ((preview.mode === 'copy') !== preview.copiesFiles) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'migration preview mode and copy behavior are inconsistent',
      path: ['copiesFiles']
    });
  }
});

export type ProjectMigrationPreview = z.infer<typeof projectMigrationPreviewSchema>;

export const projectMigrationArtifactCategorySchema = z.enum([
  'research-state',
  'decisions',
  'evidence',
  'captures',
  'semantic-links',
  'continuity',
  'other'
]);

export const projectMigrationArtifactStateSchema = z.enum([
  'matched',
  'not-present',
  'missing-at-destination',
  'destination-only',
  'changed'
]);

export const projectMigrationReconciliationSchema = z.object({
  status: z.enum(['matched', 'matched-with-gaps', 'drifted']),
  matchedArtifactCount: z.number().int().nonnegative(),
  driftedArtifactCount: z.number().int().nonnegative(),
  continuityGapCount: z.number().int().nonnegative(),
  artifacts: z.array(z.object({
    category: projectMigrationArtifactCategorySchema,
    relativePath: z.string().min(1).max(4_096),
    state: projectMigrationArtifactStateSchema
  }).strict()).max(4_096)
}).strict();

export type ProjectMigrationReconciliation = z.infer<typeof projectMigrationReconciliationSchema>;

export const projectMigrationRollbackPreviewSchema = z.object({
  registrationState: z.enum(['registered', 'unregistered']),
  markerState: z.enum(['ready', 'missing', 'conflicting']),
  reconciliation: projectMigrationReconciliationSchema,
  sourceRetained: z.literal(true),
  destinationRemoval: z.string().min(1).max(128),
  canRollback: z.boolean()
}).strict();

export type ProjectMigrationRollbackPreview = z.infer<typeof projectMigrationRollbackPreviewSchema>;

export const operationPreviewSchema = z.object({
  token: z.string().regex(/^[0-9a-f]{32}$/),
  kind: z.string().min(1).max(64),
  title: z.string().min(1).max(128),
  summary: z.string().min(1).max(512),
  displayTarget: z.string().min(1).max(4096).nullable(),
  planDigestSha256: z.string().regex(/^[0-9a-f]{64}$/).nullable(),
  approvalsRequired: z.array(z.string().min(1).max(64)).max(4),
  canConfirm: z.boolean(),
  blockedReason: z.string().min(1).max(128).nullable(),
  migration: projectMigrationPreviewSchema.optional(),
  migrationRollback: projectMigrationRollbackPreviewSchema.optional()
}).strict().superRefine((preview, context) => {
  if (preview.migrationRollback && preview.migrationRollback.canRollback !== preview.canConfirm) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'migration rollback and operation confirmation states are inconsistent',
      path: ['migrationRollback', 'canRollback']
    });
  }
  if (!preview.canConfirm && preview.approvalsRequired.length !== 0) {
    context.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'blocked operation previews cannot request approvals',
      path: ['approvalsRequired']
    });
  }
});

export type OperationPreview = z.infer<typeof operationPreviewSchema>;

export const projectMigrationQualificationSchema = z.object({
  projectId: projectIdSchema,
  status: z.enum(['verified', 'rebuild-required']),
  projectionId: academicGraphProjectionIdSchema.nullable(),
  indexId: academicGraphIndexIdSchema.nullable(),
  deterministicRebuild: z.boolean(),
  reasonCode: z.string().min(1).max(128).nullable()
}).strict().superRefine((qualification, context) => {
  if (qualification.status === 'verified') {
    if (!qualification.deterministicRebuild
      || qualification.projectionId === null
      || qualification.indexId === null
      || qualification.reasonCode !== null) {
      context.addIssue({ code: 'custom', message: 'verified migration qualification is incomplete' });
    }
  } else if (qualification.deterministicRebuild
    || qualification.projectionId !== null
    || qualification.indexId !== null
    || qualification.reasonCode === null) {
    context.addIssue({ code: 'custom', message: 'rebuild-required migration qualification is inconsistent' });
  }
});
export type ProjectMigrationQualification = z.infer<typeof projectMigrationQualificationSchema>;

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
  z.object({
    type: z.literal('academic-graph-portfolio'),
    portfolio: academicGraphPortfolioSnapshotSchema
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
    type: z.literal('capture-deliveries'),
    page: captureDeliveryPageSchema
  }).strict(),
  z.object({
    type: z.literal('capture-delivery-inspected'),
    delivery: captureDeliveryViewSchema
  }).strict(),
  z.object({
    type: z.literal('capture-delivery-updated'),
    delivery: captureDeliveryViewSchema
  }).strict(),
  z.object({
    type: z.literal('capture-delivery-acknowledgement-preview'),
    acknowledgement: captureDeliveryAcknowledgementPreviewSchema,
    preview: operationPreviewSchema
  }).strict().superRefine((event, context) => {
    if (event.preview.canConfirm
      && event.preview.planDigestSha256 !== event.acknowledgement.planDigest) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'acknowledgement operation preview does not bind the native plan',
        path: ['preview', 'planDigestSha256']
      });
    }
  }),
  z.object({
    type: z.literal('capture-assignments'),
    page: captureAssignmentPageSchema
  }).strict(),
  z.object({
    type: z.literal('capture-assignment-inspected'),
    assignment: captureAssignmentViewSchema
  }).strict(),
  z.object({
    type: z.literal('capture-assignment-preview'),
    assignment: captureAssignmentPreviewSchema,
    preview: operationPreviewSchema
  }).strict().superRefine((event, context) => {
    if (event.preview.canConfirm
      && event.preview.planDigestSha256 !== event.assignment.planDigest) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'assignment operation preview does not bind the native plan',
        path: ['preview', 'planDigestSha256']
      });
    }
  }),
  z.object({
    type: z.literal('capture-resolutions'),
    page: captureResolutionPageSchema
  }).strict(),
  z.object({
    type: z.literal('capture-resolution-inspected'),
    resolution: captureResolutionViewSchema
  }).strict(),
  z.object({
    type: z.literal('capture-resolution-preview'),
    resolution: captureResolutionPreviewSchema,
    selections: z.array(captureResolutionSelectionSchema).max(80),
    preview: operationPreviewSchema
  }).strict().superRefine((event, context) => {
    const selected = new Map(event.selections.map((selection) => [
      selection.itemId,
      selection.disposition
    ]));
    if (selected.size !== event.selections.length
      || event.resolution.items.some((item) => {
        const disposition = selected.get(item.itemId);
        return disposition === undefined || !item.allowedDispositions.includes(disposition);
      })
      || event.selections.length !== event.resolution.items.length) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'resolution preview requires one allowed selection for every item',
        path: ['selections']
      });
    }
    if (event.preview.canConfirm
      && event.preview.planDigestSha256 !== event.resolution.planDigest) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'resolution operation preview does not bind the native plan',
        path: ['preview', 'planDigestSha256']
      });
    }
  }),
  z.object({
    type: z.literal('portfolio-status'),
    portfolio: portfolioStatusSchema
  }).strict(),
  z.object({
    type: z.literal('portfolio-query'),
    result: portfolioQueryResultSchema
  }).strict(),
  z.object({
    type: z.literal('semantic-timeline'),
    result: semanticTimelineResultSchema
  }).strict(),
  z.object({
    type: z.literal('portfolio-doctor'),
    doctor: portfolioDoctorSchema
  }).strict(),
  z.object({
    type: z.literal('portfolio-maintenance-preview'),
    maintenance: portfolioMaintenancePreviewSchema,
    preview: operationPreviewSchema
  }).strict().superRefine((event, context) => {
    if (event.preview.canConfirm
      && event.preview.planDigestSha256 !== event.maintenance.planDigest) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'portfolio operation preview does not bind the native plan',
        path: ['preview', 'planDigestSha256']
      });
    }
  }),
  z.object({
    type: z.literal('continuity-operation-progress'),
    progress: continuityOperationProgressSchema
  }).strict(),
  z.object({
    type: z.literal('portfolio-maintenance-completed'),
    result: portfolioMaintenanceResultSchema
  }).strict(),
  z.object({
    type: z.literal('project-directory-selected'),
    token: z.string().regex(/^[0-9a-f]{32}$/),
    rootLabel: z.string().min(1).max(160)
  }).strict(),
  z.object({
    type: z.literal('project-migration-completed'),
    code: z.string().min(1).max(128),
    snapshot: appSnapshotSchema,
    qualification: projectMigrationQualificationSchema
  }).strict(),
  z.object({
    type: z.literal('update-changed'),
    update: updateViewSchema,
    closeRequested: z.boolean()
  }).strict(),
  z.object({
    type: z.literal('orchestration-loaded'),
    runs: orchestrationRunListSchema
  }).strict(),
  z.object({
    type: z.literal('orchestration-run-updated'),
    run: orchestrationRunSummarySchema,
    runs: orchestrationRunListSchema
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
