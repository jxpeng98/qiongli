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
  apply: z.boolean()
});

export const appSnapshotSchema = z.object({
  schemaVersion: z.literal(APP_API_SCHEMA_VERSION),
  product: productSchema,
  content: contentSchema,
  mcp: mcpSchema,
  configuration: configurationSchema,
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

export const appIntentSchema = z.discriminatedUnion('action', [
  z.object({ action: z.literal('refresh') }).strict(),
  z.object({ action: z.literal('refresh-research-library') }).strict(),
  z.object({ action: z.literal('select-project-directory') }).strict(),
  z.object({
    action: z.literal('preview-project-register'),
    directoryToken: z.string().regex(/^[0-9a-f]{32}$/)
  }).strict(),
  z.object({ action: z.literal('preview-project-archive'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('preview-project-restore'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('preview-project-refresh'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('preview-project-unregister'), projectId: projectIdSchema }).strict(),
  z.object({ action: z.literal('refresh-integration-discovery') }).strict(),
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
  z.object({
    type: z.literal('project-directory-selected'),
    token: z.string().regex(/^[0-9a-f]{32}$/),
    rootLabel: z.string().min(1).max(160)
  }).strict(),
  z.object({ type: z.literal('completed'), code: z.string().min(1).max(128), snapshot: appSnapshotSchema }).strict(),
  z.object({ type: z.literal('cancelled'), code: z.string().min(1).max(128) }).strict(),
  z.object({ type: z.literal('validation-failed'), code: z.string().min(1).max(128) }).strict(),
  z.object({ type: z.literal('failed'), code: z.string().min(1).max(128) }).strict()
]);

export type AppEvent = z.infer<typeof appEventSchema>;
