import type {
  AcademicGraphPathQuery,
  AcademicGraphPathResult,
  AcademicGraphPathTraversal,
  AcademicGraphPortfolioSnapshot,
  AcademicGraphQuery,
  AcademicGraphQueryResult,
  AcademicGraphReadiness,
  AcademicGraphRevisionComparison,
  AcademicGraphSnapshot,
  AppEvent,
  AppIntent,
  AppSnapshot,
  AppTransport,
  ArtifactChangeSnapshot,
  CaptureAssignmentPage,
  CaptureAssignmentPreview,
  CaptureAssignmentView,
  CaptureCoverageSnapshot,
  CaptureDeliveryAcknowledgementPreview,
  CaptureDeliveryPage,
  CaptureDeliveryView,
  CaptureInboxSnapshot,
  CaptureResolutionPage,
  CaptureResolutionPreview,
  CaptureResolutionView,
  ContinuityOperationProgress,
  OperationPreview,
  OrchestrationRunList,
  OrchestrationRunSummary,
  PortfolioDoctor,
  PortfolioMaintenancePreview,
  PortfolioMaintenanceResult,
  PortfolioQueryResult,
  PortfolioStatus,
  ProjectArtifactView,
  ResearchCapture,
  SemanticTimelineRequest,
  SemanticTimelineResult
} from '@qiongli/app-api';

let sourceSnapshot: AppSnapshot = {
  schemaVersion: 16,
  product: {
    version: '2.0.0-alpha.3',
    build: 'source-build',
    operatingSystem: 'macOS',
    architecture: 'AArch64',
    trust: {
      mode: 'source-read-only',
      label: 'Source build — client changes inspect only',
      canApply: false,
      reasonCode: 'source-build-read-only'
    }
  },
  content: {
    status: 'ready',
    packId: 'qiongli-core',
    contentVersion: '1.19.0-beta.1',
    entryCount: 422,
    profiles: [
      { id: 'skill-only', label: 'Skills', description: 'Portable Skills and workflow guidance.', includedResourceKinds: 4 },
      { id: 'marketplace-lite', label: 'Plugin Lite', description: 'Skills plus the native Lite MCP adapter.', includedResourceKinds: 7 },
      { id: 'full', label: 'Full workflow', description: 'Complete workflow and future Full runtime declarations.', includedResourceKinds: 11 }
    ],
    managedSkills: {
      status: 'ready',
      destinations: [
        {
          targetId: `skills-target-${'1'.repeat(64)}`,
          preset: 'qiongli-managed',
          symbolicPath: '<user-home>/.qiongli-skills',
          state: 'missing',
          status: 'missing',
          profile: null,
          productVersion: null,
          projectId: null
        },
        {
          targetId: `skills-target-${'2'.repeat(64)}`,
          preset: 'custom-folder',
          symbolicPath: '<custom-folder>',
          state: 'current',
          status: 'ready',
          profile: 'skill-only',
          productVersion: '2.0.0-alpha.3',
          projectId: null
        },
        {
          targetId: `skills-target-${'4'.repeat(64)}`,
          preset: 'current-project',
          symbolicPath: '<project>/.qiongli-skills',
          state: 'missing',
          status: 'missing',
          profile: null,
          productVersion: null,
          projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051'
        }
      ]
    }
  },
  mcp: { status: 'ready', profile: 'marketplace-lite', publicToolCount: 12 },
  cli: {
    status: 'missing',
    state: 'missing',
    installedVersion: null,
    availableVersion: '2.0.0-alpha.3',
    symbolicTarget: '<user-home>/.local/bin/qiongli',
    pathStatus: 'attention',
    pathState: 'not-configured',
    reasonCode: 'qiongli-cli-not-installed',
    canInstall: false,
    canTest: false
  },
  zotero: {
    status: 'disabled',
    state: 'not-observed',
    observation: 'not-observed',
    zoteroVersion: null,
    connectorAvailable: false,
    companionAvailable: false,
    companionVersion: null,
    availableCompanionVersion: '0.3.0',
    availableCompanionSha256: 'a'.repeat(64),
    availableCompanionSizeBytes: 41_156,
    endpointVersion: null,
    supportedEndpointVersion: '2',
    supportedZoteroMinVersion: '8.0',
    supportedZoteroMaxVersion: '9.0.*',
    installationPrepared: false,
    fallbackImportAvailable: true,
    fallbackFormats: [
      'references.json',
      'references.ris',
      'bibliography.bib',
      'zotero-import-report.md'
    ],
    reasonCode: 'zotero-integration-not-observed',
    canPrepareInstall: true,
    canReveal: false,
    canOpenZotero: false,
    canVerify: true
  },
  configuration: {
    status: 'ready',
    revision: 3,
    secretStore: 'ready',
    providers: [
      { provider: 'openalex', enabled: true, readiness: 'needs-secret', publicSettingPresent: false, secretReferencePresent: false },
      { provider: 'semantic-scholar', enabled: true, readiness: 'ready', publicSettingPresent: false, secretReferencePresent: true },
      { provider: 'crossref', enabled: true, readiness: 'needs-public-setting', publicSettingPresent: false, secretReferencePresent: false },
      { provider: 'pubmed', enabled: false, readiness: 'disabled', publicSettingPresent: false, secretReferencePresent: false },
      { provider: 'arxiv', enabled: true, readiness: 'ready', publicSettingPresent: false, secretReferencePresent: false }
    ],
    legacyCredential: {
      referencePresent: true,
      cleanupAvailable: true
    },
    cleanupRequired: false
  },
  update: {
    status: 'ready',
    selectedStream: 'stable',
    phase: 'idle',
    availableVersion: null,
    archiveSizeBytes: null,
    progress: null,
    reasonCode: 'update-ready-to-check',
    remediation: 'none',
    canSelectStream: true,
    canCheck: true,
    canPrepare: false,
    canInstall: false,
    canCancel: false
  },
  researchLibrary: {
    schemaVersion: 1,
    revision: 7,
    health: 'ready',
    projects: [
      {
        projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
        displayName: 'Trustworthy research agents',
        projectKind: 'article',
        stage: 'writing',
        lifecycle: 'active',
        semanticRevision: 12,
        registeredAtUnix: 1784304000,
        lastOpenedAtUnix: 1784390400,
        academicallyUpdatedAtUnix: 1784390400,
        health: 'ready',
        nextAction: 'open',
        rootLabel: 'trustworthy-research-agents',
        overview: {
          focalQuestion: 'How can research agents preserve evidence provenance across heterogeneous clients?',
          thesis: 'A portable article state should be the authority while each client remains an execution surface.',
          evidencePosition: 'Core architecture claims are supported; evaluation evidence is still being collected.',
          unresolvedRiskCount: 2,
          claimEvidenceCoveragePercent: 68,
          nextPriorities: ['Complete the cross-client evaluation', 'Resolve two evidence gaps']
        }
      },
      {
        projectId: 'prj_118f4d5a3b2c71008a9b0c1d2e3f4052',
        displayName: 'Academic knowledge graphs review',
        projectKind: 'review',
        stage: 'literature',
        lifecycle: 'active',
        semanticRevision: 6,
        registeredAtUnix: 1783785600,
        lastOpenedAtUnix: null,
        academicallyUpdatedAtUnix: 1784217600,
        health: 'revision-drift',
        nextAction: 'refresh',
        rootLabel: 'knowledge-graphs-review',
        overview: {
          focalQuestion: 'Which graph structures best expose the logic connecting claims and literature?',
          thesis: null,
          evidencePosition: 'Screening is in progress.',
          unresolvedRiskCount: 4,
          claimEvidenceCoveragePercent: 42,
          nextPriorities: ['Refresh the project state', 'Finish abstract screening']
        }
      },
      {
        projectId: 'prj_218f4d5a3b2c71008a9b0c1d2e3f4053',
        displayName: 'Methods appendix revision',
        projectKind: 'manuscript',
        stage: 'review',
        lifecycle: 'archived',
        semanticRevision: 18,
        registeredAtUnix: 1781193600,
        lastOpenedAtUnix: 1783094400,
        academicallyUpdatedAtUnix: 1783094400,
        health: 'ready',
        nextAction: 'restore',
        rootLabel: 'methods-appendix',
        overview: {
          focalQuestion: null,
          thesis: 'The revision documents the full robustness workflow.',
          evidencePosition: 'All planned checks are complete.',
          unresolvedRiskCount: 0,
          claimEvidenceCoveragePercent: 100,
          nextPriorities: []
        }
      }
    ]
  },
  legacyMigration: {
    state: 'available',
    nextAction: 'start',
    migrationId: null,
    detectedItems: 4,
    eligibleItems: 4,
    reviewItems: 0,
    reasonCode: 'legacy-migration-available',
    providerConflicts: []
  },
  integrations: [
    {
      target: 'codex',
      label: 'Codex',
      connection: { state: 'detected-not-connected', label: 'Detected, not connected', reasonCode: 'client-detected-install-ready' },
      client: { detected: true, status: 'ready', version: '0.144.4', compatibility: 'supported', minimumSupportedVersion: '0.144.1' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.3' },
      discovery: 'Discovered but unmanaged',
      candidateRequired: true,
      legacyDetected: true,
      migration: { state: 'available', detectedItems: 1, eligibleItems: 1, reviewItems: 0 },
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'ready', marketplace: 'missing', directPackage: null,
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Codex personal marketplace',
      activationPolicy: 'Client action required',
      hostAction: {
        scope: 'personal', restartRequired: true,
        commands: [{ executable: 'codex', arguments: ['plugin', 'add', '--json', 'qiongli-next@personal'] }]
      },
      ownership: 'Unmanaged',
      ownershipState: 'unmanaged',
      nextAction: 'install-ready',
      evidenceCode: 'client-detected-install-ready',
      paths: [
        { surface: 'Client configuration', scope: 'User', source: 'Official default', state: 'ready', management: 'Supported', selected: true, symbolicPath: '<user-home>/.codex' },
        { surface: 'Skills package', scope: 'Legacy', source: 'Legacy observed', state: 'ready', management: 'Legacy only', selected: false, symbolicPath: '<codex-config>/skills/qiongli-workflow' }
      ]
    },
    {
      target: 'claude-code',
      label: 'Claude Code',
      connection: { state: 'detected-not-connected', label: 'Detected, not connected', reasonCode: 'client-detected-install-ready' },
      client: { detected: true, status: 'ready', version: '2.1.209', compatibility: 'supported', minimumSupportedVersion: '2.1.206' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.3' },
      discovery: 'Discovered but unmanaged',
      candidateRequired: true,
      legacyDetected: false,
      migration: { state: 'not-detected', detectedItems: 0, eligibleItems: 0, reviewItems: 0 },
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'missing', marketplace: 'missing', directPackage: 'missing',
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Claude Code local marketplace',
      activationPolicy: 'Reload or client action required',
      hostAction: {
        scope: 'user', restartRequired: true,
        commands: [
          { executable: 'claude', arguments: ['plugin', 'marketplace', 'add', '$HOME/.qiongli/plugins/claude-code/qiongli-local', '--scope', 'user'] },
          { executable: 'claude', arguments: ['plugin', 'install', 'qiongli-next@qiongli-local', '--scope', 'user'] }
        ]
      },
      ownership: 'Not installed',
      ownershipState: 'not-installed',
      nextAction: 'install-ready',
      evidenceCode: 'client-detected-install-ready',
      paths: [
        { surface: 'Client configuration', scope: 'User', source: 'Official default', state: 'ready', management: 'Supported', selected: true, symbolicPath: '<user-home>/.claude' }
      ]
    }
  ],
  capabilities: {
    refresh: true,
    skillsMaterialize: true,
    integrationDiscovery: true,
    integrationPreview: true,
    projectLibrary: true,
    projectMutation: true,
    captureInbox: true,
    captureMutation: true,
    captureDelivery: true,
    captureResolution: true,
    academicGraph: true,
    portfolio: true,
    timeline: true,
    orchestrationInspect: true,
    orchestrationControl: true,
    legacyCredentialCleanup: true,
    apply: false
  }
};

export function developmentSnapshotFixture(): AppSnapshot {
  return structuredClone(sourceSnapshot);
}

const fixtureCaptureId = `cap_${'a'.repeat(64)}`;
const fixtureProjectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const fixtureUnboundEnvelopeId = `env_${'1'.repeat(64)}`;
const fixtureDeliveredEnvelopeId = `env_${'2'.repeat(64)}`;
const fixtureChildEnvelopeId = `env_${'3'.repeat(64)}`;
const fixtureAssignmentIntentId = `cai_${'4'.repeat(64)}`;
const fixtureAssignmentReceiptId = `car_${'5'.repeat(64)}`;
const fixtureResolutionItemId = `cri_${'6'.repeat(64)}`;
const fixtureResolutionReceiptId = `crr_${'7'.repeat(64)}`;
const fixtureDerivedCaptureId = `cap_${'8'.repeat(64)}`;
const fixtureProjectionId = `grp_${'a'.repeat(64)}`;
const fixtureProjectNodeId = `nod_${'1'.repeat(64)}`;
const fixtureClaimNodeId = `nod_${'2'.repeat(64)}`;
const fixtureEvidenceNodeId = `nod_${'3'.repeat(64)}`;
const fixturePaperNodeId = `nod_${'4'.repeat(64)}`;
const fixtureSectionNodeId = `nod_${'5'.repeat(64)}`;
const fixtureGapNodeId = `nod_${'6'.repeat(64)}`;

const academicGraph: AcademicGraphSnapshot = {
  schemaVersion: 1,
  documentKind: 'qiongli-academic-graph',
  projectionId: fixtureProjectionId,
  projectionDigest: 'b'.repeat(64),
  projectId: fixtureProjectId,
  projectRevision: 12,
  projectStage: 'writing',
  projectLifecycle: 'active',
  projectManifestDigest: 'c'.repeat(64),
  projectSemanticDigest: 'd'.repeat(64),
  graphSourceDigest: 'e'.repeat(64),
  sourceCount: 5,
  presentSourceCount: 5,
  nodeCount: 6,
  edgeCount: 6,
  diagnosticCount: 0,
  sources: [
    { sourceKind: 'project-manifest', artifactPath: 'context/project_manifest.json', present: true, contentDigest: 'c'.repeat(64), sizeBytes: 512 },
    { sourceKind: 'registered-artifact', artifactPath: 'context/research_state.md', present: true, contentDigest: 'f'.repeat(64), sizeBytes: 1024 },
    { sourceKind: 'registered-artifact', artifactPath: 'evidence/claim-evidence-ledger.csv', present: true, contentDigest: 'a'.repeat(64), sizeBytes: 1536 },
    { sourceKind: 'registered-artifact', artifactPath: 'manuscript/claims_evidence_map.md', present: true, contentDigest: 'd'.repeat(64), sizeBytes: 2048 },
    { sourceKind: 'registered-artifact', artifactPath: 'literature/literature_map.md', present: true, contentDigest: 'e'.repeat(64), sizeBytes: 1536 }
  ],
  nodes: [
    {
      nodeId: fixtureProjectNodeId,
      nodeType: 'project',
      identityScope: 'project',
      canonicalId: fixtureProjectId,
      label: 'Trustworthy research agents',
      layers: ['portfolio', 'combined'],
      artifactPath: 'context/project_manifest.json',
      sourceAnchor: 'project'
    },
    {
      nodeId: fixtureClaimNodeId,
      nodeType: 'claim',
      identityScope: 'project',
      canonicalId: 'CLM-001',
      label: 'Portable article state preserves evidence provenance across clients',
      layers: ['argument', 'manuscript', 'combined'],
      artifactPath: 'manuscript/claims_evidence_map.md',
      sourceAnchor: 'CLM-001'
    },
    {
      nodeId: fixtureEvidenceNodeId,
      nodeType: 'evidence',
      identityScope: 'project',
      canonicalId: 'EVD-001',
      label: 'Cross-client restart acceptance preserves canonical artifacts',
      layers: ['argument', 'combined'],
      artifactPath: 'evidence/claim-evidence-ledger.csv',
      sourceAnchor: 'EVD-001'
    },
    {
      nodeId: fixturePaperNodeId,
      nodeType: 'paper',
      identityScope: 'global',
      canonicalId: 'doi:10.1000/qiongli-fixture',
      label: 'Portable provenance for research systems',
      layers: ['literature', 'combined'],
      artifactPath: 'literature/literature_map.md',
      sourceAnchor: 'PAPER-001'
    },
    {
      nodeId: fixtureSectionNodeId,
      nodeType: 'manuscript-section',
      identityScope: 'project',
      canonicalId: 'section:discussion',
      label: 'Discussion',
      layers: ['manuscript', 'combined'],
      artifactPath: 'manuscript/claims_evidence_map.md',
      sourceAnchor: 'section:discussion'
    },
    {
      nodeId: fixtureGapNodeId,
      nodeType: 'gap',
      identityScope: 'project',
      canonicalId: 'GAP-001',
      label: 'Independent replication remains unregistered',
      layers: ['argument', 'combined'],
      artifactPath: 'context/research_state.md',
      sourceAnchor: 'GAP-001'
    }
  ],
  edges: [
    {
      edgeId: `edg_${'1'.repeat(64)}`,
      sourceNodeId: fixtureProjectNodeId,
      relation: 'contains',
      targetNodeId: fixtureClaimNodeId,
      layers: ['combined'],
      rationale: 'The article project contains its canonical manuscript claims.',
      artifactPath: 'manuscript/claims_evidence_map.md',
      sourceAnchor: 'CLM-001',
      evidenceLimit: 'Project containment does not establish empirical support.',
      inferenceStrength: 'direct_evidence',
      confidence: 'high',
      status: 'observed',
      createdFromCapture: null
    },
    {
      edgeId: `edg_${'2'.repeat(64)}`,
      sourceNodeId: fixtureEvidenceNodeId,
      relation: 'supports',
      targetNodeId: fixtureClaimNodeId,
      layers: ['argument', 'combined'],
      rationale: 'The restart acceptance supplies implementation evidence for the portability claim.',
      artifactPath: 'evidence/claim-evidence-ledger.csv',
      sourceAnchor: 'EVD-001',
      evidenceLimit: 'The fixture does not substitute for an external empirical evaluation.',
      inferenceStrength: 'reasonable_inference',
      confidence: 'medium',
      status: 'reviewed',
      createdFromCapture: null
    },
    {
      edgeId: `edg_${'3'.repeat(64)}`,
      sourceNodeId: fixturePaperNodeId,
      relation: 'informs',
      targetNodeId: fixtureClaimNodeId,
      layers: ['literature', 'argument', 'combined'],
      rationale: 'The literature record informs the provenance architecture claim.',
      artifactPath: 'literature/literature_map.md',
      sourceAnchor: 'PAPER-001',
      evidenceLimit: 'The relation is conceptual and does not establish causality.',
      inferenceStrength: 'reasonable_inference',
      confidence: 'medium',
      status: 'proposed',
      createdFromCapture: null
    },
    {
      edgeId: `edg_${'4'.repeat(64)}`,
      sourceNodeId: fixtureClaimNodeId,
      relation: 'appears-in-section',
      targetNodeId: fixtureSectionNodeId,
      layers: ['argument', 'manuscript', 'combined'],
      rationale: 'The canonical claim map assigns this claim to the Discussion section.',
      artifactPath: 'manuscript/claims_evidence_map.md',
      sourceAnchor: 'CLM-001',
      evidenceLimit: 'This records manuscript placement, not empirical support.',
      inferenceStrength: 'direct_evidence',
      confidence: 'high',
      status: 'observed',
      createdFromCapture: null
    },
    {
      edgeId: `edg_${'5'.repeat(64)}`,
      sourceNodeId: fixturePaperNodeId,
      relation: 'contradicts',
      targetNodeId: fixtureClaimNodeId,
      layers: ['literature', 'argument', 'combined'],
      rationale: 'A bounded literature result reports a conflicting portability outcome.',
      artifactPath: 'literature/literature_map.md',
      sourceAnchor: 'PAPER-001:limitation',
      evidenceLimit: 'The conflicting study uses a different client and a small sample.',
      inferenceStrength: 'reasonable_inference',
      confidence: 'low',
      status: 'reviewed',
      createdFromCapture: null
    },
    {
      edgeId: `edg_${'6'.repeat(64)}`,
      sourceNodeId: fixtureGapNodeId,
      relation: 'weakens',
      targetNodeId: fixtureClaimNodeId,
      layers: ['argument', 'combined'],
      rationale: 'The claim has no registered independent replication.',
      artifactPath: 'context/research_state.md',
      sourceAnchor: 'GAP-001',
      evidenceLimit: 'Absence of registered replication is not evidence that replication failed.',
      inferenceStrength: 'unsupported_gap',
      confidence: 'unknown',
      status: 'rejected',
      createdFromCapture: null
    }
  ],
  diagnostics: []
};

const academicGraphReadiness: AcademicGraphReadiness = {
  schemaVersion: 1,
  documentKind: 'qiongli-academic-graph-readiness',
  projectionId: fixtureProjectionId,
  projectId: fixtureProjectId,
  projectRevision: 4,
  graphSourceDigest: '4'.repeat(64),
  lastSuccessfulBuild: {
    projectRevision: 4,
    projectionId: fixtureProjectionId,
    graphSourceDigest: '4'.repeat(64)
  },
  state: 'visualizable',
  reasonCode: 'academic-graph-visualizable',
  remediation: 'none',
  recognizedSourceCount: 5,
  presentSourceCount: 5,
  missingSourceCount: 0,
  invalidSourceCount: 0,
  unsupportedSourceCount: 0,
  staleSourceCount: 0,
  nodeCount: 6,
  semanticNodeCount: 5,
  connectedNodeCount: 6,
  isolatedNodeCount: 0,
  relationCount: 6,
  layerCounts: [
    { layer: 'portfolio', nodeCount: 1 },
    { layer: 'literature', nodeCount: 1 },
    { layer: 'argument', nodeCount: 3 },
    { layer: 'manuscript', nodeCount: 2 },
    { layer: 'combined', nodeCount: 6 }
  ],
  nodeTypeCounts: [
    { nodeType: 'project', nodeCount: 1 },
    { nodeType: 'paper', nodeCount: 1 },
    { nodeType: 'claim', nodeCount: 1 },
    { nodeType: 'evidence', nodeCount: 1 },
    { nodeType: 'gap', nodeCount: 1 },
    { nodeType: 'manuscript-section', nodeCount: 1 }
  ],
  relationCounts: [
    { relation: 'contains', edgeCount: 1 },
    { relation: 'supports', edgeCount: 1 },
    { relation: 'weakens', edgeCount: 1 },
    { relation: 'contradicts', edgeCount: 1 },
    { relation: 'informs', edgeCount: 1 },
    { relation: 'appears-in-section', edgeCount: 1 }
  ],
  sources: [
    { sourceKind: 'project-manifest', artifactPath: 'context/project_manifest.json', state: 'present', freshness: 'fresh', nodeCount: 1, edgeCount: 0, diagnosticCount: 0 },
    { sourceKind: 'registered-artifact', artifactPath: 'context/research_state.md', state: 'present', freshness: 'fresh', nodeCount: 1, edgeCount: 1, diagnosticCount: 0 },
    { sourceKind: 'registered-artifact', artifactPath: 'evidence/claim-evidence-ledger.csv', state: 'present', freshness: 'fresh', nodeCount: 1, edgeCount: 1, diagnosticCount: 0 },
    { sourceKind: 'registered-artifact', artifactPath: 'manuscript/claims_evidence_map.md', state: 'present', freshness: 'fresh', nodeCount: 2, edgeCount: 2, diagnosticCount: 0 },
    { sourceKind: 'registered-artifact', artifactPath: 'literature/literature_map.md', state: 'present', freshness: 'fresh', nodeCount: 1, edgeCount: 2, diagnosticCount: 0 }
  ]
};

function fixtureProjectArtifact(intent: Extract<AppIntent, { action: 'read-project-artifact' }>): ProjectArtifactView {
  const graphReference = intent.reference.kind === 'academic-graph-entity'
    ? intent.reference
    : null;
  const entity = graphReference?.entity ?? null;
  const source = entity?.kind === 'node'
    ? academicGraph.nodes.find((node) => node.nodeId === entity.id)
    : entity?.kind === 'edge'
      ? academicGraph.edges.find((edge) => edge.edgeId === entity.id)
      : null;
  const artifactPath = intent.reference.kind === 'registered-artifact'
    ? intent.reference.artifactPath
    : source?.artifactPath ?? 'context/research_state.md';
  const sourceAnchor = intent.reference.kind === 'registered-artifact'
    ? intent.reference.sourceAnchor
    : source?.sourceAnchor ?? null;
  const content = artifactPath.endsWith('.json')
    ? `{\n  "projectId": "${intent.projectId}",\n  "anchor": "${sourceAnchor ?? 'project'}"\n}\n`
    : artifactPath.endsWith('.csv')
      ? `id,summary\n${sourceAnchor ?? 'EVD-001'},Bounded fixture evidence\n`
      : artifactPath.endsWith('.jsonl')
        ? `{"anchor":"${sourceAnchor ?? 'semantic-link'}","relation":"supports"}\n`
        : `# Project artifact\n\n## ${sourceAnchor ?? 'Overview'}\n\nThis bounded source excerpt is rendered inside Qiongli.\n`;
  const contentSizeBytes = new TextEncoder().encode(content).length;
  const anchorOffset = sourceAnchor ? content.indexOf(sourceAnchor) : -1;
  const anchorLine = anchorOffset >= 0
    ? content.slice(0, anchorOffset).split('\n').length
    : null;
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-project-artifact-view',
    projectId: intent.projectId,
    projectRevision: intent.expectedProjectRevision,
    projectionId: graphReference?.expectedProjectionId ?? null,
    entityKind: entity?.kind ?? null,
    entityId: entity?.id ?? null,
    artifactPath,
    sourceAnchor,
    format: artifactPath.endsWith('.md')
      ? 'markdown'
      : artifactPath.endsWith('.csv')
        ? 'csv'
        : artifactPath.endsWith('.jsonl') ? 'json-lines' : 'json',
    contentDigest: '7'.repeat(64),
    sourceSizeBytes: contentSizeBytes,
    content,
    contentSizeBytes,
    startLine: 1,
    endLine: content.split('\n').length,
    anchorLine,
    anchorMatched: anchorLine !== null,
    truncatedBefore: false,
    truncatedAfter: false
  };
}

const portfolioProjectA = fixtureProjectId;
const portfolioProjectB = 'prj_118f4d5a3b2c71008a9b0c1d2e3f4052';
const portfolioProjectC = 'prj_218f4d5a3b2c71008a9b0c1d2e3f4053';
const portfolioProjectionC = `grp_${'c'.repeat(64)}`;
const portfolioNodeA = `pnd_${'1'.repeat(64)}`;
const portfolioNodeC = `pnd_${'2'.repeat(64)}`;
const portfolioPaperNode = `pnd_${'3'.repeat(64)}`;
const portfolioConceptNode = `pnd_${'4'.repeat(64)}`;

const academicGraphPortfolio: AcademicGraphPortfolioSnapshot = {
  schemaVersion: 1,
  documentKind: 'qiongli-academic-graph-portfolio',
  portfolioId: `gpf_${'5'.repeat(64)}`,
  libraryRevision: 7,
  projectCount: 3,
  includedProjectCount: 2,
  skippedProjectCount: 1,
  nodeCount: 4,
  edgeCount: 5,
  projects: [
    {
      projectId: portfolioProjectA,
      displayName: 'Trustworthy research agents',
      lifecycle: 'active',
      health: 'ready',
      included: true,
      projectRevision: 12,
      projectionId: fixtureProjectionId
    },
    {
      projectId: portfolioProjectB,
      displayName: 'Academic knowledge graphs review',
      lifecycle: 'active',
      health: 'revision-drift',
      included: false,
      projectRevision: null,
      projectionId: null
    },
    {
      projectId: portfolioProjectC,
      displayName: 'Methods appendix revision',
      lifecycle: 'archived',
      health: 'ready',
      included: true,
      projectRevision: 18,
      projectionId: portfolioProjectionC
    }
  ],
  nodes: [
    {
      nodeId: portfolioNodeA,
      nodeType: 'project',
      identityScope: 'project',
      canonicalId: portfolioProjectA,
      label: 'Trustworthy research agents',
      projectIds: [portfolioProjectA],
      occurrences: [{
        projectId: portfolioProjectA,
        projectionId: fixtureProjectionId,
        graphNodeId: fixtureProjectNodeId,
        label: 'Trustworthy research agents',
        artifactPath: 'context/project_manifest.json',
        sourceAnchor: 'project'
      }]
    },
    {
      nodeId: portfolioNodeC,
      nodeType: 'project',
      identityScope: 'project',
      canonicalId: portfolioProjectC,
      label: 'Methods appendix revision',
      projectIds: [portfolioProjectC],
      occurrences: [{
        projectId: portfolioProjectC,
        projectionId: portfolioProjectionC,
        graphNodeId: `nod_${'7'.repeat(64)}`,
        label: 'Methods appendix revision',
        artifactPath: 'context/project_manifest.json',
        sourceAnchor: 'project'
      }]
    },
    {
      nodeId: portfolioPaperNode,
      nodeType: 'paper',
      identityScope: 'global',
      canonicalId: 'doi:10.1000/qiongli-fixture',
      label: 'Portable provenance for research systems',
      projectIds: [portfolioProjectA, portfolioProjectC],
      occurrences: [
        {
          projectId: portfolioProjectA,
          projectionId: fixtureProjectionId,
          graphNodeId: fixturePaperNodeId,
          label: 'Portable provenance for research systems',
          artifactPath: 'literature/literature_map.md',
          sourceAnchor: 'PAPER-001'
        },
        {
          projectId: portfolioProjectC,
          projectionId: portfolioProjectionC,
          graphNodeId: `nod_${'8'.repeat(64)}`,
          label: 'Portable provenance — methods appendix citation',
          artifactPath: 'literature/literature_map.md',
          sourceAnchor: 'PAPER-METHODS-004'
        }
      ]
    },
    {
      nodeId: portfolioConceptNode,
      nodeType: 'concept',
      identityScope: 'global',
      canonicalId: 'concept:evidence-provenance',
      label: 'Evidence provenance',
      projectIds: [portfolioProjectA, portfolioProjectC],
      occurrences: [
        {
          projectId: portfolioProjectA,
          projectionId: fixtureProjectionId,
          graphNodeId: `nod_${'9'.repeat(64)}`,
          label: 'Evidence provenance',
          artifactPath: 'context/research_state.md',
          sourceAnchor: 'concept:evidence-provenance'
        },
        {
          projectId: portfolioProjectC,
          projectionId: portfolioProjectionC,
          graphNodeId: `nod_${'a'.repeat(64)}`,
          label: 'Evidence provenance',
          artifactPath: 'manuscript/claims_evidence_map.md',
          sourceAnchor: 'concept:evidence-provenance'
        }
      ]
    }
  ],
  edges: [
    portfolioSharedEdge('1', portfolioNodeA, 'shares-source', portfolioPaperNode, 'doi:10.1000/qiongli-fixture', portfolioProjectA, fixtureProjectionId, 'literature/literature_map.md', 'PAPER-001'),
    portfolioSharedEdge('2', portfolioNodeC, 'shares-source', portfolioPaperNode, 'doi:10.1000/qiongli-fixture', portfolioProjectC, portfolioProjectionC, 'literature/literature_map.md', 'PAPER-METHODS-004'),
    portfolioSharedEdge('3', portfolioNodeA, 'shares-concept', portfolioConceptNode, 'concept:evidence-provenance', portfolioProjectA, fixtureProjectionId, 'context/research_state.md', 'concept:evidence-provenance'),
    portfolioSharedEdge('4', portfolioNodeC, 'shares-concept', portfolioConceptNode, 'concept:evidence-provenance', portfolioProjectC, portfolioProjectionC, 'manuscript/claims_evidence_map.md', 'concept:evidence-provenance'),
    {
      edgeId: `ped_${'5'.repeat(64)}`,
      sourceNodeId: portfolioNodeC,
      relation: 'forked-from',
      targetNodeId: portfolioNodeA,
      sharedCanonicalId: null,
      rationale: 'The reviewed semantic link records that the methods revision was forked from the research-agent article.',
      evidenceLimit: 'Lineage does not imply identical claims or evidence strength.',
      inferenceStrength: 'direct_evidence',
      confidence: 'high',
      status: 'reviewed',
      origins: [{
        projectId: portfolioProjectC,
        projectionId: portfolioProjectionC,
        graphEdgeId: `edg_${'9'.repeat(64)}`,
        artifactPath: 'graph/semantic_links.jsonl',
        sourceAnchor: 'line:1'
      }]
    }
  ]
};

function portfolioSharedEdge(
  id: string,
  sourceNodeId: string,
  relation: 'shares-source' | 'shares-concept',
  targetNodeId: string,
  sharedCanonicalId: string,
  projectId: string,
  projectionId: string,
  artifactPath: 'context/research_state.md' | 'literature/literature_map.md' | 'manuscript/claims_evidence_map.md',
  sourceAnchor: string
): AcademicGraphPortfolioSnapshot['edges'][number] {
  return {
    edgeId: `ped_${id.repeat(64)}`,
    sourceNodeId,
    relation,
    targetNodeId,
    sharedCanonicalId,
    rationale: `The registered projections reuse the exact ${relation === 'shares-source' ? 'source' : 'concept'} identifier.`,
    evidenceLimit: 'Exact identifier reuse does not imply identical conclusions or evidence strength.',
    inferenceStrength: 'direct_evidence',
    confidence: 'high',
    status: 'observed',
    origins: [{
      projectId,
      projectionId,
      graphEdgeId: null,
      artifactPath,
      sourceAnchor
    }]
  };
}

const fixtureCatalogId = `pca_${'6'.repeat(64)}`;
const fixturePortfolioQueryId = `pqy_${'7'.repeat(64)}`;
const fixturePortfolioOperationId = `cop_${'8'.repeat(64)}`;
const fixtureTimelineDigest = `ptl_${'e'.repeat(64)}`;

const fixturePortfolioStatus = {
  schemaVersion: 1,
  state: 'current',
  libraryRevision: 7,
  catalogId: fixtureCatalogId,
  catalogGeneration: 3,
  portfolioId: academicGraphPortfolio.portfolioId,
  contributionCount: 2,
  projectCount: 2,
  nodeCount: 4,
  edgeCount: 5,
  reasonCode: 'portfolio-current',
  capabilities: {
    canQuery: true,
    canReconcile: true,
    canRebuild: true,
    canDeleteDerivedState: true
  }
} satisfies PortfolioStatus;

const fixturePortfolioDoctor = {
  schemaVersion: 1,
  status: 'equivalent',
  libraryRevision: 7,
  catalogId: fixtureCatalogId,
  incrementalPortfolioId: academicGraphPortfolio.portfolioId,
  cleanPortfolioId: academicGraphPortfolio.portfolioId,
  byteEquivalent: true,
  contributionCount: 2
} satisfies PortfolioDoctor;

function fixturePortfolioQuery(cursor: boolean): PortfolioQueryResult {
  const project = cursor
    ? {
        resultId: `project:${portfolioProjectC}`,
        projectId: portfolioProjectC,
        displayName: 'Methods appendix revision',
        stage: 'review' as const,
        lifecycle: 'archived' as const,
        health: 'ready' as const,
        semanticRevision: 18,
        projectionId: portfolioProjectionC,
        nodeCount: 2,
        edgeCount: 3,
        lineageCount: 1
      }
    : {
        resultId: `project:${portfolioProjectA}`,
        projectId: portfolioProjectA,
        displayName: 'Trustworthy research agents',
        stage: 'writing' as const,
        lifecycle: 'active' as const,
        health: 'ready' as const,
        semanticRevision: 12,
        projectionId: fixtureProjectionId,
        nodeCount: 2,
        edgeCount: 2,
        lineageCount: 1
      };
  const lineage = cursor
    ? {
        lineageId: `lin_${'b'.repeat(64)}`,
        kind: 'delivery' as const,
        projectIds: [portfolioProjectA, portfolioProjectC],
        relatedIds: [fixtureDeliveredEnvelopeId, fixtureCaptureId],
        occurredAtUnix: 1_784_563_300,
        source: 'codex' as const,
        delivery: 'connected' as const,
        deliveryState: 'acknowledged' as const,
        assignmentOutcome: null,
        fromProjectRevision: 12,
        toProjectRevision: 13
      }
    : {
        lineageId: `lin_${'a'.repeat(64)}`,
        kind: 'capture' as const,
        projectIds: [portfolioProjectA],
        relatedIds: [fixtureCaptureId],
        occurredAtUnix: 1_784_476_800,
        source: 'codex' as const,
        delivery: 'portable' as const,
        deliveryState: null,
        assignmentOutcome: null,
        fromProjectRevision: null,
        toProjectRevision: null
      };
  return {
    schemaVersion: 1,
    requestId: `pqr_${(cursor ? '2' : '1').repeat(64)}`,
    queryId: fixturePortfolioQueryId,
    catalogId: fixtureCatalogId,
    portfolioId: academicGraphPortfolio.portfolioId,
    lineageDigest: `plg_${'9'.repeat(64)}`,
    matchedProjectCount: 2,
    matchedNodeCount: 2,
    matchedEdgeCount: 1,
    matchedLineageCount: 2,
    projectsTruncated: !cursor,
    nodesTruncated: !cursor,
    edgesTruncated: false,
    lineageTruncated: !cursor,
    projects: [project],
    nodes: [{
      resultId: `node:${cursor ? portfolioProjectC : portfolioProjectA}:${academicGraph.nodes[cursor ? 1 : 0].nodeId}`,
      projectId: cursor ? portfolioProjectC : portfolioProjectA,
      projectionId: cursor ? portfolioProjectionC : fixtureProjectionId,
      node: academicGraph.nodes[cursor ? 1 : 0]
    }],
    edges: cursor ? [] : [{
      resultId: `edge:${portfolioProjectA}:${academicGraph.edges[0].edgeId}`,
      projectId: portfolioProjectA,
      projectionId: fixtureProjectionId,
      edge: academicGraph.edges[0]
    }],
    lineage: [lineage],
    nextCursor: cursor ? null : {
      cursorId: `pqc_${'a'.repeat(64)}`,
      queryId: fixturePortfolioQueryId,
      projectAfter: portfolioProjectA,
      nodeAfter: academicGraph.nodes[0].nodeId,
      edgeAfter: academicGraph.edges[0].edgeId,
      lineageAfter: `lin_${'a'.repeat(64)}`
    }
  };
}

type FixtureTimelineEvent = SemanticTimelineResult['events'][number];

function fixtureTimelineEvent(
  identity: string,
  kind: FixtureTimelineEvent['kind'],
  occurredAtUnix: number,
  timestampSource: FixtureTimelineEvent['timestampSource'],
  relatedIds: string[],
  details: Partial<Omit<
    FixtureTimelineEvent,
    'eventId' | 'kind' | 'occurredAtUnix' | 'timestampSource' | 'relatedIds'
  >> = {}
): FixtureTimelineEvent {
  return {
    eventId: `pte_${identity.repeat(64)}`,
    kind,
    occurredAtUnix,
    timestampSource,
    projectIds: [portfolioProjectA],
    relatedIds: [...relatedIds].sort(),
    fromProjectRevision: null,
    toProjectRevision: null,
    lifecycle: null,
    source: null,
    delivery: null,
    deliveryState: null,
    deliveryReason: null,
    deliveryGeneration: null,
    assignmentOutcome: null,
    resolutionItemId: null,
    resolutionItemKind: null,
    resolutionDisposition: null,
    ...details
  };
}

const fixtureTimelineEvents = [
  fixtureTimelineEvent(
    '1',
    'project-registered',
    1_784_304_000,
    'project-registered-at',
    [portfolioProjectA]
  ),
  fixtureTimelineEvent(
    '2',
    'capture-accepted',
    1_784_476_800,
    'capture-captured-at',
    [fixtureCaptureId],
    {
      fromProjectRevision: 12,
      source: 'codex',
      delivery: 'portable'
    }
  ),
  fixtureTimelineEvent(
    '3',
    'delivery-acknowledged',
    1_784_563_100,
    'delivery-transitioned-at',
    [fixtureDeliveredEnvelopeId, fixtureCaptureId, `dack_${'3'.repeat(64)}`],
    {
      projectIds: [portfolioProjectA, portfolioProjectC],
      fromProjectRevision: 12,
      toProjectRevision: 13,
      source: 'codex',
      delivery: 'connected',
      deliveryState: 'acknowledged',
      deliveryReason: 'delivery-acknowledged',
      deliveryGeneration: 2
    }
  ),
  fixtureTimelineEvent(
    '4',
    'assignment-created',
    1_784_563_150,
    'assignment-created-at',
    [fixtureAssignmentIntentId, fixtureUnboundEnvelopeId, fixtureCaptureId],
    {
      projectIds: [portfolioProjectA, portfolioProjectC],
      fromProjectRevision: 12
    }
  ),
  fixtureTimelineEvent(
    '5',
    'capture-assigned',
    1_784_563_200,
    'assignment-decided-at',
    [
      fixtureAssignmentIntentId,
      fixtureAssignmentReceiptId,
      fixtureUnboundEnvelopeId,
      fixtureCaptureId,
      fixtureDerivedCaptureId,
      fixtureChildEnvelopeId
    ],
    {
      projectIds: [portfolioProjectA, portfolioProjectC],
      fromProjectRevision: 12,
      assignmentOutcome: 'assigned'
    }
  ),
  fixtureTimelineEvent(
    '6',
    'capture-consolidated',
    1_784_563_250,
    'consolidation-consolidated-at',
    [fixtureCaptureId, 'fixture-consolidation-acknowledgement'],
    {
      fromProjectRevision: 12,
      toProjectRevision: 13,
      source: 'codex',
      delivery: 'portable'
    }
  ),
  fixtureTimelineEvent(
    '7',
    'resolution-reviewed',
    1_784_563_300,
    'resolution-reviewed-at',
    [
      fixtureResolutionReceiptId,
      fixtureAssignmentReceiptId,
      fixtureUnboundEnvelopeId,
      fixtureCaptureId,
      fixtureDerivedCaptureId,
      fixtureChildEnvelopeId
    ],
    { fromProjectRevision: 12 }
  ),
  fixtureTimelineEvent(
    '8',
    'resolution-item-resolved',
    1_784_563_400,
    'resolution-resolved-at',
    [
      fixtureResolutionReceiptId,
      fixtureResolutionItemId,
      fixtureAssignmentReceiptId,
      fixtureCaptureId
    ],
    {
      fromProjectRevision: 12,
      toProjectRevision: 13,
      resolutionItemId: fixtureResolutionItemId,
      resolutionItemKind: 'semantic-change',
      resolutionDisposition: 'accept-capture'
    }
  ),
  fixtureTimelineEvent(
    '9',
    'resolution-completed',
    1_784_563_400,
    'resolution-resolved-at',
    [fixtureResolutionReceiptId, fixtureAssignmentReceiptId, fixtureCaptureId],
    {
      fromProjectRevision: 12,
      toProjectRevision: 13
    }
  ),
  fixtureTimelineEvent(
    'a',
    'project-revision-observed',
    1_784_563_500,
    'project-academically-updated-at',
    [portfolioProjectA, fixtureProjectionId],
    { toProjectRevision: 13 }
  )
] satisfies SemanticTimelineResult['events'];

function fixtureTimelineResult(request: SemanticTimelineRequest): SemanticTimelineResult {
  const revisionKinds = new Set<FixtureTimelineEvent['kind']>([
    'project-registered',
    'project-revision-observed',
    'project-lifecycle-observed',
    'capture-consolidated',
    'delivery-acknowledged',
    'resolution-completed'
  ]);
  const resolutionKinds = new Set<FixtureTimelineEvent['kind']>([
    'capture-consolidated',
    'resolution-reviewed',
    'resolution-item-resolved',
    'resolution-completed'
  ]);
  const events = fixtureTimelineEvents.filter((event) =>
    (!request.projectId || event.projectIds.includes(request.projectId))
    && (
      request.view === 'activity'
      || (request.view === 'revision-history' && revisionKinds.has(event.kind))
      || (
        request.view === 'merge-resolution-history'
        && resolutionKinds.has(event.kind)
      )
    )
  );
  const identity = request.view === 'activity'
    ? request.projectId ? 'b' : 'a'
    : request.view === 'revision-history'
      ? request.projectId ? 'd' : 'c'
      : request.projectId ? 'f' : 'e';
  const queryId = `pty_${identity.repeat(64)}`;
  const offset = request.cursor ? 2 : 0;
  const selected = request.cursor ? events.slice(offset) : events.slice(0, 2);
  const truncated = offset + selected.length < events.length;
  const last = selected.at(-1);
  return {
    schemaVersion: 1,
    requestId: `ptr_${identity.repeat(64)}`,
    queryId,
    catalogId: fixtureCatalogId,
    portfolioId: academicGraphPortfolio.portfolioId,
    timelineDigest: fixtureTimelineDigest,
    projectId: request.projectId ?? null,
    view: request.view,
    matchedEventCount: events.length,
    truncated,
    events: selected,
    nextCursor: truncated && last ? {
      cursorId: `ptc_${identity.repeat(64)}`,
      queryId,
      afterOccurredAtUnix: last.occurredAtUnix,
      afterEventId: last.eventId
    } : null
  };
}

function fixtureMaintenancePreview(
  operation: PortfolioMaintenancePreview['operation'],
  catalogPresent = true
): PortfolioMaintenancePreview {
  const explanations = {
    reconcile: 'Reconcile only changed or missing derived project contributions against the current Research Library. Canonical academic artifacts are retained.',
    'full-rebuild': 'Rebuild every derived project contribution from the current registered canonical artifacts. Canonical academic artifacts are retained.',
    'delete-derived-state': 'Delete only the private rebuildable portfolio catalog and contributions. Registered projects and canonical academic artifacts are retained.'
  };
  return {
    schemaVersion: 1,
    planDigest: 'b'.repeat(64),
    operation,
    expectedLibraryRevision: 7,
    expectedCatalogId: catalogPresent ? fixtureCatalogId : null,
    expectedCatalogGeneration: catalogPresent ? 3 : null,
    currentContributionCount: catalogPresent ? 2 : 0,
    derivedStateOnly: true,
    explanation: explanations[operation],
    approvalsRequired: ['derived-state-write']
  };
}

function fixtureOperationProgress(
  operation: PortfolioMaintenancePreview['operation'],
  phase: ContinuityOperationProgress['phase'],
  completedUnits: number,
  cancellable: boolean,
  reasonCode: string
): ContinuityOperationProgress {
  return {
    schemaVersion: 1,
    operationId: fixturePortfolioOperationId,
    operation,
    phase,
    completedUnits,
    totalUnits: 2,
    catalogId: fixtureCatalogId,
    cancellable,
    reasonCode
  };
}

function fixtureAcademicGraphComparison(): AcademicGraphRevisionComparison {
  const gap = academicGraph.nodes.find((node) => node.nodeId === fixtureGapNodeId)!;
  const changedEdges = academicGraph.edges.filter((edge) =>
    edge.edgeId === `edg_${'5'.repeat(64)}` || edge.edgeId === `edg_${'6'.repeat(64)}`
  );
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-revision-comparison',
    comparisonId: `gcp_${'7'.repeat(64)}`,
    projectId: fixtureProjectId,
    beforeProjectRevision: 11,
    afterProjectRevision: 12,
    beforeProjectionId: `grp_${'9'.repeat(64)}`,
    afterProjectionId: fixtureProjectionId,
    sourceChangeCount: 0,
    nodeChangeCount: 1,
    edgeChangeCount: 2,
    hasChanges: true,
    beforeRisks: {
      contradictionCount: 0,
      gapCount: 0,
      rejectedRelationCount: 0,
      lowConfidenceCount: 0,
      totalSignalCount: 0
    },
    afterRisks: {
      contradictionCount: 1,
      gapCount: 2,
      rejectedRelationCount: 1,
      lowConfidenceCount: 2,
      totalSignalCount: 6
    },
    riskDelta: {
      contradictionCount: 1,
      gapCount: 2,
      rejectedRelationCount: 1,
      lowConfidenceCount: 2,
      totalSignalCount: 6
    },
    sourceChanges: [],
    nodeChanges: [{
      changeKind: 'added',
      nodeId: gap.nodeId,
      before: null,
      after: gap
    }],
    edgeChanges: changedEdges.map((edge) => ({
      changeKind: 'added' as const,
      edgeId: edge.edgeId,
      before: null,
      after: edge
    })),
    nextActions: [
      'inspect-new-contradictions',
      'fill-new-gaps',
      'verify-low-confidence-evidence',
      'review-rejected-relations'
    ]
  };
}

const captureInbox = {
  schemaVersion: 1,
  projectId: fixtureProjectId,
  projectRevision: 12,
  projectStage: 'writing',
  pendingReviewCount: 1,
  staleCount: 0,
  conflictedCount: 0,
  appliedCount: 0,
  entries: [{
    captureId: fixtureCaptureId,
    state: 'pending-review',
    disposition: 'refinement',
    source: 'codex',
    delivery: 'portable',
    capturedAtUnix: 1784476800,
    baseRevision: 12,
    boundStage: 'writing',
    task: 'Preserve evidence provenance across clients',
    capturePolicy: 'review-required',
    summary: 'Clarify why the article project, rather than a runtime session, is the durable research unit.',
    changeCount: 1,
    decisionCount: 1,
    evidenceCount: 1,
    contradictionCount: 0,
    nextActionCount: 1,
    historyEntry: `history/captures/${fixtureCaptureId}.json`
  }]
} satisfies CaptureInboxSnapshot;

const captureCoverage = {
  schemaVersion: 1,
  projectId: fixtureProjectId,
  projectRevision: 12,
  projectStage: 'writing',
  captureCount: 1,
  connectedCount: 0,
  repositoryBackedCount: 0,
  portableCount: 1,
  manualCount: 0,
  pendingReviewCount: 1,
  currentCount: 0,
  staleCount: 0,
  conflictedCount: 0,
  unboundCount: 0,
  unknownSourceCount: 6,
  sources: [
    {
      source: 'codex',
      state: 'pending-review',
      delivery: 'portable',
      captureCount: 1,
      pendingReviewCount: 1,
      currentCount: 0,
      staleCount: 0,
      conflictedCount: 0,
      unboundCount: 0,
      latestCaptureId: fixtureCaptureId,
      lastCapturedAtUnix: 1784476800
    },
    ...(['claude-code', 'chat-gpt', 'cli', 'manual', 'repository', 'portable-file'] as const)
      .map((source) => ({
        source,
        state: 'unknown' as const,
        delivery: 'unknown' as const,
        captureCount: 0,
        pendingReviewCount: 0,
        currentCount: 0,
        staleCount: 0,
        conflictedCount: 0,
        unboundCount: 0,
        latestCaptureId: null,
        lastCapturedAtUnix: null
      }))
  ]
} satisfies CaptureCoverageSnapshot;

const artifactChanges = {
  schemaVersion: 1,
  projectId: fixtureProjectId,
  projectRevision: 12,
  projectStage: 'writing',
  state: 'unattributed',
  registeredArtifactCount: 8,
  presentArtifactCount: 3,
  changeCount: 1,
  unattributedCount: 1,
  changes: [{
    changeId: `chg_${'b'.repeat(64)}`,
    state: 'unattributed',
    detection: 'aggregate',
    effect: 'changed-set',
    baseRevision: 12,
    relativePaths: [],
    reason: 'no-accepted-capture-lineage'
  }],
  artifacts: [
    { artifact: 'research-state', relativePath: 'context/research_state.md', present: true },
    { artifact: 'decision-log', relativePath: 'context/decision_log.md', present: true },
    { artifact: 'stage-handoff', relativePath: 'context/stage_handoff.md', present: false },
    { artifact: 'boundary-review', relativePath: 'context/boundary_review.md', present: false },
    { artifact: 'idea-funnel', relativePath: 'context/idea_funnel.md', present: false },
    { artifact: 'literature-map', relativePath: 'literature/literature_map.md', present: true },
    { artifact: 'claim-evidence-ledger', relativePath: 'evidence/claim-evidence-ledger.csv', present: false },
    { artifact: 'manuscript-claim-map', relativePath: 'manuscript/claims_evidence_map.md', present: false }
  ]
} satisfies ArtifactChangeSnapshot;

const fixtureCapture = {
  schemaVersion: 1,
  captureId: fixtureCaptureId,
  binding: {
    schemaVersion: 1,
    projectId: fixtureProjectId,
    baseRevision: 12,
    stage: 'writing',
    task: 'Preserve evidence provenance across clients',
    capturePolicy: 'review-required'
  },
  source: 'codex',
  delivery: 'portable',
  capturedAtUnix: 1784476800,
  summary: 'Clarify why the article project, rather than a runtime session, is the durable research unit.',
  changes: [{ area: 'literature', summary: 'Group continuity evidence by academic claim rather than client session.' }],
  decisions: [{
    relation: 'candidate',
    statement: 'Keep the article project as the durable authority.',
    rationale: 'Execution surfaces should not become canonical research memory.',
    target: null
  }],
  evidence: [{
    locatorKind: 'doi',
    locator: '10.1000/qiongli-fixture',
    relevance: 'Provides an example evidence anchor for the capture review.',
    limitation: 'Fixture evidence only.'
  }],
  contradictions: [],
  nextActions: ['Review the refinement before consolidating it.']
} satisfies ResearchCapture;

const fixtureUnboundDelivery = {
  schemaVersion: 1,
  envelopeId: fixtureUnboundEnvelopeId,
  captureId: fixtureCaptureId,
  source: 'claude-code',
  delivery: 'connected',
  destination: null,
  state: 'conflicted',
  generation: 3,
  attemptCount: 2,
  retryCount: 1,
  createdAtUnix: 1784476800,
  updatedAtUnix: 1784563000,
  lastReason: 'delivery-destination-conflict',
  envelopeSha256: '1'.repeat(64),
  recordSha256: '2'.repeat(64),
  acknowledgement: null,
  capabilities: {
    canRetry: true,
    canCancel: true,
    canAcknowledge: false
  }
} satisfies CaptureDeliveryView;

const fixtureDeliveredDelivery = {
  schemaVersion: 1,
  envelopeId: fixtureDeliveredEnvelopeId,
  captureId: fixtureCaptureId,
  source: 'codex',
  delivery: 'connected',
  destination: {
    projectId: fixtureProjectId,
    expectedProjectRevision: 12
  },
  state: 'delivered',
  generation: 2,
  attemptCount: 1,
  retryCount: 0,
  createdAtUnix: 1784476800,
  updatedAtUnix: 1784563100,
  lastReason: 'delivery-accepted',
  envelopeSha256: '3'.repeat(64),
  recordSha256: '4'.repeat(64),
  acknowledgement: null,
  capabilities: {
    canRetry: false,
    canCancel: false,
    canAcknowledge: true
  }
} satisfies CaptureDeliveryView;

const fixtureDeliveryPage = {
  schemaVersion: 1,
  snapshotId: `dls_${'1'.repeat(64)}`,
  projectId: fixtureProjectId,
  entries: [fixtureUnboundDelivery, fixtureDeliveredDelivery],
  truncated: false,
  nextCursor: null
} satisfies CaptureDeliveryPage;

const fixtureAssignment = {
  schemaVersion: 1,
  state: 'completed',
  intentId: fixtureAssignmentIntentId,
  sourceEnvelopeId: fixtureUnboundEnvelopeId,
  sourceCaptureId: fixtureCaptureId,
  targetProjectId: fixtureProjectId,
  targetProjectRevision: 12,
  outcome: 'assigned',
  receiptId: fixtureAssignmentReceiptId,
  derivedCaptureId: fixtureDerivedCaptureId,
  childEnvelopeId: fixtureChildEnvelopeId,
  createdAtUnix: 1784563100,
  decidedAtUnix: 1784563200,
  canResolve: true
} satisfies CaptureAssignmentView;

const fixtureAssignmentPage = {
  schemaVersion: 1,
  snapshotId: `als_${'2'.repeat(64)}`,
  projectId: fixtureProjectId,
  entries: [fixtureAssignment],
  truncated: false,
  nextCursor: null
} satisfies CaptureAssignmentPage;

const fixtureResolution = {
  schemaVersion: 1,
  receiptId: fixtureResolutionReceiptId,
  assignmentReceiptId: fixtureAssignmentReceiptId,
  sourceEnvelopeId: fixtureUnboundEnvelopeId,
  sourceCaptureId: fixtureCaptureId,
  derivedCaptureId: fixtureDerivedCaptureId,
  childEnvelopeId: fixtureChildEnvelopeId,
  targetProjectId: fixtureProjectId,
  fromProjectRevision: 12,
  toProjectRevision: 13,
  reviewedAtUnix: 1784563300,
  resolvedAtUnix: 1784563400,
  decisions: [{
    itemId: fixtureResolutionItemId,
    kind: 'semantic-change',
    disposition: 'accept-capture'
  }]
} satisfies CaptureResolutionView;

const fixtureResolutionPage = {
  schemaVersion: 1,
  snapshotId: `rls_${'3'.repeat(64)}`,
  projectId: fixtureProjectId,
  entries: [],
  truncated: false,
  nextCursor: null
} satisfies CaptureResolutionPage;

const fixtureResolutionPlan = {
  schemaVersion: 1,
  planDigest: '6'.repeat(64),
  assignmentReceiptId: fixtureAssignmentReceiptId,
  sourceEnvelopeId: fixtureUnboundEnvelopeId,
  sourceCaptureId: fixtureCaptureId,
  derivedCaptureId: fixtureDerivedCaptureId,
  childEnvelopeId: fixtureChildEnvelopeId,
  targetProjectId: fixtureProjectId,
  expectedLibraryRevision: 7,
  expectedProjectRevision: 12,
  nextProjectRevision: 13,
  reviewedAtUnix: 1784563300,
  items: [{
    itemId: fixtureResolutionItemId,
    kind: 'semantic-change',
    counterpartState: 'exact-identity-divergent',
    allowedDispositions: ['accept-current', 'accept-capture', 'reject-capture'],
    unavailableDispositions: ['retain-both'],
    sourceSummary: 'Use the accepted capture wording for the project-level provenance claim.',
    currentSummary: 'Keep the current project wording for the provenance claim.',
    explanation: 'The same semantic identity contains divergent reviewed content.'
  }],
  approvalsRequired: ['academic-review', 'filesystem-write'],
  exactReplay: false
} satisfies CaptureResolutionPreview;

const fixtureAcknowledgementPreview = {
  schemaVersion: 1,
  planDigest: '7'.repeat(64),
  envelopeId: fixtureDeliveredEnvelopeId,
  destinationProjectId: fixtureProjectId,
  acceptedCaptureId: fixtureCaptureId,
  expectedProjectRevision: 12,
  resultingProjectRevision: 12,
  acknowledgedAtUnix: 1784563500,
  expectedGeneration: 2,
  expectedRecordSha256: '4'.repeat(64),
  approvalsRequired: ['delivery-acknowledgement']
} satisfies CaptureDeliveryAcknowledgementPreview;

const fixtureAssignmentPreview = {
  schemaVersion: 1,
  planDigest: '8'.repeat(64),
  intentId: fixtureAssignmentIntentId,
  decision: 'assign',
  outcome: 'resolution-required',
  bindingEffect: 'direct',
  sourceDisposition: 'refinement',
  sourceEnvelopeId: fixtureUnboundEnvelopeId,
  sourceCaptureId: fixtureCaptureId,
  sourceRecordState: 'conflicted',
  expectedSourceGeneration: 3,
  targetProjectId: fixtureProjectId,
  expectedLibraryRevision: 7,
  expectedProjectRevision: 12,
  targetStage: 'writing',
  derivedCaptureId: fixtureDerivedCaptureId,
  childEnvelopeId: fixtureChildEnvelopeId,
  resolutionRequired: true,
  decidedAtUnix: 1784563200,
  explanation: 'The capture can be assigned, but divergent academic meaning remains explicit.',
  approvalsRequired: ['assignment-write']
} satisfies CaptureAssignmentPreview;

const fixtureOrchestrationRun = {
  runId: `run_${'2'.repeat(32)}`,
  profileId: `host-solo-${'a'.repeat(24)}`,
  executionMode: 'solo',
  status: 'running',
  generation: 3,
  documentSha256: '3'.repeat(64),
  completedTaskCount: 1,
  totalTaskCount: 76,
  nextTaskId: 'A1_5',
  activeTaskId: null,
  activeRole: null,
  completedRoleCount: 0,
  requiredRoleCount: 1,
  hostDriven: true,
  recoveryRequired: false,
  canContinue: true,
  canPause: true,
  canResume: false,
  canRecover: false,
  canCancel: true
} satisfies OrchestrationRunSummary;

const fixtureOrchestrationRuns = {
  schemaVersion: 1,
  projectId: fixtureProjectId,
  expectedProjectRevision: 12,
  runs: [fixtureOrchestrationRun]
} satisfies OrchestrationRunList;

export function sourceFixtureTransport(): AppTransport {
  let pendingCaptureOperation: AppEvent['type'] | null = null;
  let pendingPortfolioOperation: PortfolioMaintenancePreview['operation'] | null = null;
  let activePortfolioOperation: PortfolioMaintenancePreview['operation'] | null = null;
  let portfolioPollCount = 0;
  let portfolioDerivedStateDeleted = false;
  return {
    async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
      if (command === 'qiongli_snapshot') return sourceSnapshot as T;
      if (command !== 'qiongli_execute') throw new Error('dev-fixture-command-unsupported');
      const intent = args?.intent as AppIntent | undefined;
      if (!intent) throw new Error('dev-fixture-intent-missing');
      if (intent.action === 'load-portfolio-status' && portfolioDerivedStateDeleted) {
        return {
          type: 'portfolio-status',
          portfolio: {
            ...fixturePortfolioStatus,
            state: 'missing',
            catalogId: null,
            catalogGeneration: null,
            portfolioId: null,
            contributionCount: 0,
            projectCount: 0,
            nodeCount: 0,
            edgeCount: 0,
            reasonCode: 'portfolio-missing',
            capabilities: {
              canQuery: false,
              canReconcile: true,
              canRebuild: true,
              canDeleteDerivedState: false
            }
          }
        } as T;
      }
      if (intent.action === 'confirm-operation' && pendingPortfolioOperation) {
        activePortfolioOperation = pendingPortfolioOperation;
        pendingPortfolioOperation = null;
        portfolioPollCount = 0;
        return {
          type: 'continuity-operation-progress',
          progress: fixtureOperationProgress(
            activePortfolioOperation,
            'queued',
            0,
            true,
            'portfolio-operation-queued'
          )
        } as T;
      }
      if (intent.action === 'poll-continuity-operation' && activePortfolioOperation) {
        portfolioPollCount += 1;
        if (portfolioPollCount < 2) {
          return {
            type: 'continuity-operation-progress',
            progress: fixtureOperationProgress(
              activePortfolioOperation,
              'running',
              1,
              true,
              'portfolio-operation-running'
            )
          } as T;
        }
        const operation = activePortfolioOperation;
        activePortfolioOperation = null;
        portfolioDerivedStateDeleted = operation === 'delete-derived-state';
        const result = {
          schemaVersion: 1,
          operationId: fixturePortfolioOperationId,
          operation,
          libraryRevision: 7,
          catalogId: operation === 'delete-derived-state' ? null : fixtureCatalogId,
          portfolioId: operation === 'delete-derived-state'
            ? null
            : academicGraphPortfolio.portfolioId,
          catalogChanged: true,
          rebuiltProjectCount: operation === 'full-rebuild'
            ? 2
            : operation === 'reconcile' ? 1 : 0,
          reusedProjectCount: operation === 'reconcile' ? 1 : 0,
          removedProjectCount: 0,
          removedContributionCount: operation === 'delete-derived-state' ? 2 : 0,
          derivedStateOnly: true
        } satisfies PortfolioMaintenanceResult;
        return { type: 'portfolio-maintenance-completed', result } as T;
      }
      if (intent.action === 'cancel-continuity-operation' && activePortfolioOperation) {
        const operation = activePortfolioOperation;
        activePortfolioOperation = null;
        return {
          type: 'continuity-operation-progress',
          progress: fixtureOperationProgress(
            operation,
            'cancelled',
            portfolioPollCount > 0 ? 1 : 0,
            false,
            'portfolio-operation-cancelled'
          )
        } as T;
      }
      if (intent.action === 'confirm-operation' && pendingCaptureOperation) {
        const completed = {
          type: 'capture-operation-completed',
          code: 'fixture-capture-operation-completed',
          snapshot: sourceSnapshot,
          inbox: captureInbox,
          coverage: captureCoverage,
          changes: artifactChanges,
          delivery: pendingCaptureOperation === 'capture-delivery-acknowledgement-preview'
            ? fixtureDeliveredDelivery
            : null,
          assignment: pendingCaptureOperation === 'capture-assignment-preview'
            ? fixtureAssignment
            : null,
          resolution: pendingCaptureOperation === 'capture-resolution-preview'
            ? fixtureResolution
            : null
        } satisfies AppEvent;
        pendingCaptureOperation = null;
        return completed as T;
      }
      const event = fixtureEvent(intent, !portfolioDerivedStateDeleted);
      if (
        event.type === 'capture-intake-preview'
        || event.type === 'capture-consolidation-preview'
        || event.type === 'capture-delivery-acknowledgement-preview'
        || event.type === 'capture-assignment-preview'
        || event.type === 'capture-resolution-preview'
      ) pendingCaptureOperation = event.type;
      if (event.type === 'portfolio-maintenance-preview') {
        pendingPortfolioOperation = event.maintenance.operation;
        pendingCaptureOperation = null;
      }
      if (intent.action === 'cancel-operation') {
        pendingCaptureOperation = null;
        pendingPortfolioOperation = null;
      }
      return event as T;
    }
  };
}

function fixtureEvent(intent: AppIntent, portfolioCatalogPresent = true): AppEvent {
  switch (intent.action) {
    case 'refresh':
    case 'refresh-research-library':
    case 'refresh-integration-discovery':
      return { type: 'snapshot', snapshot: sourceSnapshot };
    case 'select-skills-destination':
      return {
        type: 'skills-destination-selected',
        targetId: `skills-target-${'3'.repeat(64)}`,
        symbolicPath: '<custom-folder>'
      };
    case 'refresh-zotero-integration':
      sourceSnapshot.zotero = {
        ...sourceSnapshot.zotero,
        status: 'attention',
        state: 'zotero-not-running',
        observation: 'observed',
        zoteroVersion: '9.0.4',
        reasonCode: 'zotero-not-running',
        canOpenZotero: true
      };
      return { type: 'snapshot', snapshot: sourceSnapshot };
    case 'prepare-legacy-migration':
      sourceSnapshot.legacyMigration = {
        ...sourceSnapshot.legacyMigration,
        state: 'preview-ready',
        nextAction: 'apply',
        migrationId: 'migration-1784563200-fixture',
        reasonCode: 'legacy-migration-preview-ready'
      };
      return {
        type: 'completed',
        code: 'legacy-migration-preview-ready',
        snapshot: sourceSnapshot
      };
    case 'preview-legacy-migration-next':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000006',
          kind: 'legacy-migration-stage',
          title: 'Install Qiongli 2.x before migration',
          summary: 'Install exact 2.x managed content while preserving recognized 1.x content.',
          displayTarget: null,
          planDigestSha256: '6'.repeat(64),
          approvalsRequired: ['Filesystem write', 'Client configuration change'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'select-update-stream':
      return {
        type: 'update-changed',
        update: { ...sourceSnapshot.update, selectedStream: intent.stream },
        closeRequested: false
      };
    case 'check-for-updates':
      return {
        type: 'update-changed',
        update: {
          ...sourceSnapshot.update,
          phase: 'available',
          availableVersion: '2.0.0-alpha.3',
          archiveSizeBytes: 24_600_000,
          reasonCode: 'trusted-update-available',
          canCheck: true,
          canPrepare: true
        },
        closeRequested: false
      };
    case 'prepare-update':
    case 'poll-update':
      return {
        type: 'update-changed',
        update: {
          ...sourceSnapshot.update,
          phase: 'ready-to-install',
          availableVersion: '2.0.0-alpha.3',
          archiveSizeBytes: 24_600_000,
          reasonCode: 'verified-update-ready-to-install',
          canCheck: false,
          canPrepare: false,
          canInstall: true
        },
        closeRequested: false
      };
    case 'cancel-update':
      return {
        type: 'update-changed',
        update: {
          ...sourceSnapshot.update,
          phase: 'cancelled',
          reasonCode: 'update-cancelled',
          canCheck: true
        },
        closeRequested: false
      };
    case 'preview-update-install':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000005',
          kind: 'update-install',
          title: 'Install prepared Qiongli update',
          summary: 'Replace the packaged application with the verified staged release after explicit confirmation.',
          displayTarget: 'Qiongli 2.0.0-alpha.3',
          planDigestSha256: '5'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'preview-cli-install':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000007',
          kind: 'cli-install',
          title: 'Install Qiongli CLI',
          summary: 'Install the exact native CLI bundled with this App into the user CLI directory.',
          displayTarget: '<user-home>/.local/bin/qiongli',
          planDigestSha256: '7'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'preview-cli-remove':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000009',
          kind: 'cli-remove',
          title: 'Remove Qiongli CLI',
          summary: 'Remove the exact receipt-owned native CLI or restore its verified predecessor.',
          displayTarget: '<user-home>/.local/bin/qiongli',
          planDigestSha256: '9'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'preview-cli-path-configure':
      return {
        type: 'preview',
        preview: {
          token: '0000000000000000000000000000000a',
          kind: 'cli-path-configure',
          title: 'Configure Qiongli CLI PATH',
          summary: 'Add one receipt-bound Qiongli marker to the supported login-shell profile.',
          displayTarget: '<user-home>/.zprofile',
          planDigestSha256: 'a'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'test-cli-command':
      return {
        type: 'snapshot',
        snapshot: {
          ...sourceSnapshot,
          cli: {
            ...sourceSnapshot.cli,
            pathStatus: 'ready',
            pathState: 'active',
            reasonCode: 'qiongli-cli-shell-command-active',
            canTest: true
          }
        }
      };
    case 'preview-provider-settings':
      return {
        type: 'preview',
        preview: {
          token: '0000000000000000000000000000000b',
          kind: 'provider-settings',
          title: 'Literature provider settings preview',
          summary: 'Update provider enablement without exposing credential values.',
          displayTarget: null,
          planDigestSha256: 'b'.repeat(64),
          approvalsRequired: ['client-config-change'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'preview-provider-secret-change':
      return {
        type: 'preview',
        preview: {
          token: '0000000000000000000000000000000c',
          kind: 'provider-secret',
          title: 'Provider credential preview',
          summary: 'Save or remove the selected API key in the operating-system credential store.',
          displayTarget: null,
          planDigestSha256: 'c'.repeat(64),
          approvalsRequired: ['secret-store-write', 'client-config-change'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'test-literature-provider':
      return {
        type: 'completed',
        code: 'literature-provider-ready',
        snapshot: sourceSnapshot
      };
    case 'preview-zotero-companion-stage':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000008',
          kind: 'zotero-companion-stage',
          title: 'Prepare Zotero Companion installation',
          summary: 'Copy the verified XPI into Qiongli-owned state for a Zotero-confirmed handoff.',
          displayTarget: '<qiongli-state>/zotero/companion/0.3.0-aaaaaaaaaaaaaaaa',
          planDigestSha256: '8'.repeat(64),
          approvalsRequired: ['Filesystem write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'load-orchestration':
      return {
        type: 'orchestration-loaded',
        runs: fixtureOrchestrationRuns
      };
    case 'control-orchestration': {
      const status: OrchestrationRunSummary['status'] = intent.actionName === 'cancel'
        ? 'cancelled'
        : intent.actionName === 'resume'
          ? 'running'
          : 'paused';
      const run = {
        ...fixtureOrchestrationRun,
        status,
        generation: fixtureOrchestrationRun.generation + 1,
        documentSha256: '5'.repeat(64),
        recoveryRequired: false,
        canContinue: status === 'running',
        canPause: status === 'running',
        canResume: status === 'paused',
        canRecover: false,
        canCancel: status !== 'cancelled'
      };
      return {
        type: 'orchestration-run-updated',
        run,
        runs: {
          ...fixtureOrchestrationRuns,
          runs: [run]
        }
      };
    }
    case 'load-capture-inbox':
      return { type: 'capture-inbox', inbox: captureInbox };
    case 'load-capture-coverage':
      return { type: 'capture-coverage', coverage: captureCoverage };
    case 'load-artifact-changes':
      return { type: 'artifact-changes', changes: artifactChanges };
    case 'load-academic-graph':
      return {
        type: 'academic-graph',
        graph: academicGraph,
        readiness: academicGraphReadiness,
        comparison: fixtureAcademicGraphComparison()
      };
    case 'load-academic-graph-portfolio':
      return { type: 'academic-graph-portfolio', portfolio: academicGraphPortfolio };
    case 'query-academic-graph':
      return { type: 'academic-graph-query', result: fixtureGraphQuery(intent.query) };
    case 'query-academic-graph-path':
      return { type: 'academic-graph-path', result: fixtureGraphPath(intent.query) };
    case 'open-academic-graph-artifact':
      return {
        type: 'academic-graph-artifact-opened',
        projectId: intent.projectId,
        projectRevision: intent.expectedProjectRevision,
        projectionId: intent.expectedProjectionId,
        entity: intent.entity
      };
    case 'read-project-artifact':
      return {
        type: 'project-artifact-read',
        artifact: fixtureProjectArtifact(intent)
      };
    case 'read-capture':
      return { type: 'capture-read', capture: fixtureCapture };
    case 'select-capture-file':
      return {
        type: 'capture-file-selected',
        token: '00000000000000000000000000000004',
        fileLabel: 'portable-research-capture.json'
      };
    case 'preview-capture-intake':
      return {
        type: 'capture-intake-preview',
        intake: {
          schemaVersion: 1,
          planDigest: '1'.repeat(64),
          captureId: fixtureCaptureId,
          projectId: fixtureProjectId,
          disposition: 'refinement',
          effect: 'append-pending-history',
          source: 'codex',
          delivery: 'portable',
          expectedLibraryRevision: 7,
          expectedProjectRevision: 12,
          changeCount: 1,
          decisionCount: 1,
          evidenceCount: 1,
          contradictionCount: 0,
          nextActionCount: 1,
          historyEntry: `history/captures/${fixtureCaptureId}.json`,
          approvalsRequired: ['filesystem-write']
        },
        preview: capturePreview('capture-intake', 'Import research capture', ['filesystem-write'])
      };
    case 'preview-capture-consolidation':
      return {
        type: 'capture-consolidation-preview',
        consolidation: {
          schemaVersion: 1,
          planDigest: '2'.repeat(64),
          captureId: fixtureCaptureId,
          projectId: fixtureProjectId,
          disposition: 'refinement',
          outcome: 'ready',
          expectedLibraryRevision: 7,
          expectedProjectRevision: 12,
          nextProjectRevision: 13,
          projectStage: 'writing',
          reviewedAtUnix: 1784563200,
          conflicts: [],
          artifactDeltas: [{
            artifact: 'research-state',
            relativePath: 'context/research_state.md',
            effect: 'update',
            previousDigest: '3'.repeat(64),
            nextDigest: '4'.repeat(64),
            previousBytes: 1200,
            nextBytes: 1580
          }],
          receiptEntry: `history/consolidations/${fixtureCaptureId}.json`,
          approvalsRequired: ['academic-consolidation', 'filesystem-write']
        },
        preview: capturePreview(
          'capture-consolidation',
          'Consolidate reviewed capture',
          ['academic-consolidation', 'filesystem-write']
        )
      };
    case 'load-capture-deliveries':
      return { type: 'capture-deliveries', page: fixtureDeliveryPage };
    case 'inspect-capture-delivery': {
      const delivery = fixtureDeliveryPage.entries.find((entry) =>
        entry.envelopeId === intent.envelopeId
      );
      if (!delivery) throw new Error('capture-delivery-not-found');
      return { type: 'capture-delivery-inspected', delivery };
    }
    case 'retry-capture-delivery':
      return {
        type: 'capture-delivery-updated',
        delivery: {
          ...fixtureUnboundDelivery,
          state: 'queued',
          generation: intent.expectedGeneration + 1,
          retryCount: fixtureUnboundDelivery.retryCount + 1,
          updatedAtUnix: intent.retriedAtUnix,
          lastReason: 'delivery-retry-requested',
          recordSha256: '9'.repeat(64),
          capabilities: {
            canRetry: false,
            canCancel: true,
            canAcknowledge: false
          }
        }
      };
    case 'cancel-capture-delivery':
      return {
        type: 'capture-delivery-updated',
        delivery: {
          ...fixtureUnboundDelivery,
          state: 'cancelled',
          generation: intent.expectedGeneration + 1,
          updatedAtUnix: intent.cancelledAtUnix,
          lastReason: 'delivery-cancelled',
          recordSha256: 'a'.repeat(64),
          capabilities: {
            canRetry: false,
            canCancel: false,
            canAcknowledge: false
          }
        }
      };
    case 'preview-capture-delivery-acknowledgement':
      return {
        type: 'capture-delivery-acknowledgement-preview',
        acknowledgement: {
          ...fixtureAcknowledgementPreview,
          destinationProjectId: intent.destinationProjectId,
          acceptedCaptureId: intent.acceptedCaptureId,
          expectedProjectRevision: intent.expectedProjectRevision,
          resultingProjectRevision: intent.resultingProjectRevision,
          acknowledgedAtUnix: intent.acknowledgedAtUnix,
          expectedGeneration: intent.expectedGeneration,
          expectedRecordSha256: intent.expectedRecordSha256
        },
        preview: {
          ...capturePreview(
            'capture-delivery-acknowledgement',
            'Acknowledge delivered capture',
            ['delivery-acknowledgement']
          ),
          planDigestSha256: fixtureAcknowledgementPreview.planDigest
        }
      };
    case 'load-capture-assignments':
      return { type: 'capture-assignments', page: fixtureAssignmentPage };
    case 'inspect-capture-assignment':
      return { type: 'capture-assignment-inspected', assignment: fixtureAssignment };
    case 'preview-capture-assignment': {
      const assigned = intent.decision === 'assign';
      const assignment = {
        ...fixtureAssignmentPreview,
        decision: intent.decision,
        outcome: assigned ? 'resolution-required' : 'rejected',
        targetProjectId: intent.targetProjectId,
        derivedCaptureId: assigned ? fixtureDerivedCaptureId : null,
        childEnvelopeId: assigned ? fixtureChildEnvelopeId : null,
        resolutionRequired: assigned,
        decidedAtUnix: intent.decidedAtUnix,
        explanation: assigned
          ? fixtureAssignmentPreview.explanation
          : 'The capture remains unassigned and its source delivery lineage is preserved.'
      } satisfies CaptureAssignmentPreview;
      return {
        type: 'capture-assignment-preview',
        assignment,
        preview: {
          ...capturePreview(
            'capture-assignment',
            assigned ? 'Assign capture' : 'Reject capture assignment',
            ['assignment-write']
          ),
          planDigestSha256: assignment.planDigest
        }
      };
    }
    case 'load-capture-resolutions':
      return { type: 'capture-resolutions', page: fixtureResolutionPage };
    case 'inspect-capture-resolution':
      return { type: 'capture-resolution-inspected', resolution: fixtureResolution };
    case 'preview-capture-resolution': {
      const resolution = {
        ...fixtureResolutionPlan,
        assignmentReceiptId: intent.assignmentReceiptId,
        reviewedAtUnix: intent.reviewedAtUnix
      };
      if (!intent.selections) {
        return { type: 'capture-resolution-plan', resolution };
      }
      return {
        type: 'capture-resolution-preview',
        resolution,
        selections: intent.selections,
        preview: {
          ...capturePreview(
            'capture-resolution',
            'Resolve capture items',
            ['academic-review', 'filesystem-write']
          ),
          planDigestSha256: resolution.planDigest
        }
      };
    }
    case 'load-portfolio-status':
      return { type: 'portfolio-status', portfolio: fixturePortfolioStatus };
    case 'query-portfolio':
      return {
        type: 'portfolio-query',
        result: fixturePortfolioQuery(intent.request.cursor !== undefined)
      };
    case 'load-semantic-timeline':
      return {
        type: 'semantic-timeline',
        result: fixtureTimelineResult(intent.request)
      };
    case 'load-portfolio-doctor':
      return { type: 'portfolio-doctor', doctor: fixturePortfolioDoctor };
    case 'preview-portfolio-maintenance': {
      const maintenance = fixtureMaintenancePreview(intent.operation, portfolioCatalogPresent);
      return {
        type: 'portfolio-maintenance-preview',
        maintenance,
        preview: {
          token: 'c'.repeat(32),
          kind: `portfolio-${intent.operation}`,
          title: 'Review portfolio maintenance',
          summary: maintenance.explanation,
          displayTarget: maintenance.expectedCatalogId,
          planDigestSha256: maintenance.planDigest,
          approvalsRequired: ['derived-state-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    }
    case 'verify-integrations':
    case 'verify-skills-preset':
    case 'verify-managed-skills-target':
    case 'reveal-zotero-companion':
    case 'open-zotero':
    case 'verify-zotero-integration':
      return { type: 'completed', code: 'fixture-verification-complete', snapshot: sourceSnapshot };
    case 'preview-detach-managed-skills-target':
      return {
        type: 'preview',
        preview: {
          token: 'd'.repeat(32),
          kind: 'skills-detach',
          title: 'Preserve and detach managed Skills',
          summary: 'Remove only Qiongli ownership and retain every target file unchanged.',
          displayTarget: '<managed-skills-destination>',
          planDigestSha256: 'd'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'cancel-operation':
      return { type: 'cancelled', code: 'fixture-operation-cancelled' };
    case 'select-project-directory':
    case 'select-project-create-destination':
    case 'select-project-export-destination':
    case 'select-project-import-locations':
    case 'select-project-migration-locations':
    case 'select-project-migration-recovery-locations':
    case 'select-project-migration-rollback-locations':
      return {
        type: 'project-directory-selected',
        token: '00000000000000000000000000000002',
        rootLabel: 'selected-article-project'
      };
    case 'confirm-operation':
    case 'open-project':
      return { type: 'completed', code: 'fixture-project-operation-completed', snapshot: sourceSnapshot };
    case 'preview-project-migration':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000003',
          kind: 'project-migration',
          title: 'Migrate Qiongli 1.x article project',
          summary: 'Copy verified academic files into a new Qiongli 2 project.',
          displayTarget: 'selected-article-project',
          planDigestSha256: '0'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null,
          migration: {
            mode: 'copy',
            copiedFileCount: 12,
            copiedBytes: 48_320,
            excludedEntryCount: 3,
            sourceRetained: true,
            copiesFiles: true,
            graphRebuildPasses: 2
          }
        }
      };
    case 'preview-project-migration-recovery':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000003',
          kind: 'project-migration-recovery',
          title: 'Resume interrupted project migration',
          summary: 'Verify the committed Qiongli 2 copy and complete registration without copying again.',
          displayTarget: 'selected-article-project',
          planDigestSha256: '0'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null,
          migration: {
            mode: 'recovery',
            copiedFileCount: 12,
            copiedBytes: 48_320,
            excludedEntryCount: 3,
            sourceRetained: true,
            copiesFiles: false,
            graphRebuildPasses: 2
          }
        }
      };
    case 'preview-project-migration-rollback':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000003',
          kind: 'project-migration-rollback',
          title: 'Roll back migrated Qiongli 2 project',
          summary: 'Remove only the unchanged migration-owned Qiongli 2 copy and retain the Qiongli 1.x source.',
          displayTarget: 'selected-article-project',
          planDigestSha256: '0'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null,
          migrationRollback: {
            registrationState: 'registered',
            markerState: 'ready',
            reconciliation: {
              status: 'matched-with-gaps',
              matchedArtifactCount: 5,
              driftedArtifactCount: 0,
              continuityGapCount: 2,
              artifacts: [
                {
                  category: 'research-state',
                  relativePath: 'context/research_state.md',
                  state: 'matched'
                },
                {
                  category: 'continuity',
                  relativePath: 'context/stage_handoff.md',
                  state: 'not-present'
                }
              ]
            },
            sourceRetained: true,
            destinationRemoval: 'migration-owned-destination',
            canRollback: true
          }
        }
      };
    case 'preview-project-create':
    case 'preview-project-register':
    case 'preview-project-export':
    case 'preview-project-import':
    case 'preview-project-repair-manifest':
    case 'preview-project-archive':
    case 'preview-project-restore':
    case 'preview-project-refresh':
    case 'preview-project-unregister':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000003',
          kind: intent.action.replace('preview-', ''),
          title: 'Article project operation',
          summary: 'This fixture previews the requested Research Library change without exposing a project path.',
          displayTarget: 'selected-article-project',
          planDigestSha256: '0'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    default:
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000001',
          kind: 'activation',
          title: 'Qiongli content preview',
          summary: 'The selected Qiongli content was inspected. No client application will be installed or modified.',
          displayTarget: null,
          planDigestSha256: null,
          approvalsRequired: [],
          canConfirm: false,
          blockedReason: 'source-build-read-only'
        }
      };
  }
}

function fixtureGraphQuery(query: AcademicGraphQuery): AcademicGraphQueryResult {
  if (query.expectedProjectionId !== academicGraph.projectionId) {
    throw new Error('project-revision-conflict');
  }
  const layerMatches = (values: AcademicGraphSnapshot['nodes'][number]['layers']) =>
    query.layers.length === 0 || query.layers.some((value) => values.includes(value));
  const edgeMatches = (edge: AcademicGraphSnapshot['edges'][number]) =>
    (query.relations.length === 0 || query.relations.includes(edge.relation))
      && layerMatches(edge.layers);
  let relatedEdges: AcademicGraphSnapshot['edges'];
  if (query.focusNodeId === null) {
    relatedEdges = academicGraph.edges.filter(edgeMatches);
  } else {
    const edgeIds = new Set<string>();
    const visited = new Set([query.focusNodeId]);
    let frontier = new Set([query.focusNodeId]);
    for (let depth = 0; depth < query.maxDepth && frontier.size > 0; depth += 1) {
      const next = new Set<string>();
      for (const edge of academicGraph.edges) {
        if (!edgeMatches(edge)) continue;
        if (
          query.direction !== 'outgoing'
          && frontier.has(edge.targetNodeId)
        ) {
          edgeIds.add(edge.edgeId);
          next.add(edge.sourceNodeId);
        }
        if (
          query.direction !== 'incoming'
          && frontier.has(edge.sourceNodeId)
        ) {
          edgeIds.add(edge.edgeId);
          next.add(edge.targetNodeId);
        }
      }
      for (const nodeId of visited) next.delete(nodeId);
      for (const nodeId of next) visited.add(nodeId);
      frontier = next;
    }
    relatedEdges = academicGraph.edges.filter((edge) => edgeIds.has(edge.edgeId));
  }
  const candidateIds = query.focusNodeId !== null || query.relations.length > 0
    ? new Set([
        ...(query.focusNodeId ? [query.focusNodeId] : []),
        ...relatedEdges.flatMap((edge) => [edge.sourceNodeId, edge.targetNodeId])
      ])
    : new Set(academicGraph.nodes.map((node) => node.nodeId));
  const normalizedText = query.text?.toLocaleLowerCase() ?? null;
  const matchingNodes = academicGraph.nodes.filter((node) =>
    candidateIds.has(node.nodeId)
    && (query.nodeTypes.length === 0 || query.nodeTypes.includes(node.nodeType))
    && layerMatches(node.layers)
    && (query.canonicalId === null || query.canonicalId === node.canonicalId)
    && (normalizedText === null
      || node.label.toLocaleLowerCase().includes(normalizedText)
      || node.canonicalId.toLocaleLowerCase().includes(normalizedText))
  );
  const nodes = matchingNodes.slice(0, query.maxNodes);
  const selectedIds = new Set(nodes.map((node) => node.nodeId));
  const matchingEdges = relatedEdges.filter((edge) =>
    selectedIds.has(edge.sourceNodeId) && selectedIds.has(edge.targetNodeId)
  );
  const edges = matchingEdges.slice(0, query.maxEdges);
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-query-result',
    indexId: `gix_${'f'.repeat(64)}`,
    projectionId: fixtureProjectionId,
    projectId: fixtureProjectId,
    projectRevision: 12,
    matchedNodeCount: matchingNodes.length,
    matchedEdgeCount: matchingEdges.length,
    nodesTruncated: matchingNodes.length > nodes.length,
    edgesTruncated: matchingEdges.length > edges.length,
    nodes,
    edges
  };
}

function fixtureGraphPath(query: AcademicGraphPathQuery): AcademicGraphPathResult {
  if (query.expectedProjectionId !== academicGraph.projectionId) {
    throw new Error('project-revision-conflict');
  }
  const nodeById = new Map(academicGraph.nodes.map((node) => [node.nodeId, node]));
  if (!nodeById.has(query.sourceNodeId) || !nodeById.has(query.targetNodeId)) {
    throw new Error('academic-graph-entity-not-found');
  }
  if (query.sourceNodeId === query.targetNodeId) {
    return fixturePathResult(query, [query.sourceNodeId], [], []);
  }

  type Candidate = {
    nodeIds: string[];
    edgeIds: string[];
    traversals: AcademicGraphPathTraversal[];
  };
  const queue: Candidate[] = [{
    nodeIds: [query.sourceNodeId],
    edgeIds: [],
    traversals: []
  }];
  const visited = new Set([query.sourceNodeId]);
  while (queue.length > 0) {
    const candidate = queue.shift()!;
    if (candidate.edgeIds.length >= query.maxHops) continue;
    const current = candidate.nodeIds.at(-1)!;
    const adjacent: Array<{
      edgeId: string;
      nodeId: string;
      traversal: AcademicGraphPathTraversal;
    }> = [];
    for (const edge of academicGraph.edges) {
      if (edge.sourceNodeId === current) {
        adjacent.push({ edgeId: edge.edgeId, nodeId: edge.targetNodeId, traversal: 'forward' });
      }
      if (edge.targetNodeId === current) {
        adjacent.push({ edgeId: edge.edgeId, nodeId: edge.sourceNodeId, traversal: 'reverse' });
      }
    }
    adjacent.sort((left, right) => left.edgeId.localeCompare(right.edgeId));
    for (const next of adjacent) {
      if (visited.has(next.nodeId)) continue;
      visited.add(next.nodeId);
      const path = {
        nodeIds: [...candidate.nodeIds, next.nodeId],
        edgeIds: [...candidate.edgeIds, next.edgeId],
        traversals: [...candidate.traversals, next.traversal]
      };
      if (next.nodeId === query.targetNodeId) {
        return fixturePathResult(query, path.nodeIds, path.edgeIds, path.traversals);
      }
      queue.push(path);
    }
  }
  return fixturePathResult(query, [], [], []);
}

function fixturePathResult(
  query: AcademicGraphPathQuery,
  nodeIds: string[],
  edgeIds: string[],
  traversals: AcademicGraphPathTraversal[]
): AcademicGraphPathResult {
  const nodeById = new Map(academicGraph.nodes.map((node) => [node.nodeId, node]));
  const edgeById = new Map(academicGraph.edges.map((edge) => [edge.edgeId, edge]));
  return {
    schemaVersion: 1,
    documentKind: 'qiongli-academic-graph-explanatory-path',
    indexId: `gix_${'f'.repeat(64)}`,
    projectionId: fixtureProjectionId,
    projectId: fixtureProjectId,
    projectRevision: 12,
    sourceNodeId: query.sourceNodeId,
    targetNodeId: query.targetNodeId,
    maxHops: query.maxHops,
    status: nodeIds.length > 0 ? 'found' : 'not-found',
    hopCount: edgeIds.length,
    nodes: nodeIds.map((nodeId) => nodeById.get(nodeId)!),
    edges: edgeIds.map((edgeId) => edgeById.get(edgeId)!),
    steps: edgeIds.map((edgeId, index) => ({
      sequence: index + 1,
      fromNodeId: nodeIds[index]!,
      edgeId,
      toNodeId: nodeIds[index + 1]!,
      traversal: traversals[index]!
    }))
  };
}

function capturePreview(
  kind: string,
  title: string,
  approvalsRequired: string[]
): OperationPreview {
  return {
    token: '00000000000000000000000000000005',
    kind,
    title,
    summary: 'This fixture preserves the typed native preview and confirmation boundary.',
    displayTarget: 'portable-research-capture.json',
    planDigestSha256: kind === 'capture-intake' ? '1'.repeat(64) : '2'.repeat(64),
    approvalsRequired,
    canConfirm: true,
    blockedReason: null
  };
}
