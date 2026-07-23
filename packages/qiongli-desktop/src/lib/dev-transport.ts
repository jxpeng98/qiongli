import type {
  AcademicGraphPathQuery,
  AcademicGraphPathResult,
  AcademicGraphPathTraversal,
  AcademicGraphPortfolioSnapshot,
  AcademicGraphQuery,
  AcademicGraphQueryResult,
  AcademicGraphRevisionComparison,
  AcademicGraphSnapshot,
  AppEvent,
  AppIntent,
  AppSnapshot,
  AppTransport,
  ArtifactChangeSnapshot,
  CaptureCoverageSnapshot,
  CaptureInboxSnapshot,
  OperationPreview,
  OrchestrationDoctor,
  OrchestrationRunList,
  OrchestrationRunSummary,
  ResearchCapture
} from '@qiongli/app-api';

const sourceSnapshot = {
  schemaVersion: 1,
  product: {
    version: '2.0.0-alpha.1',
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
    ]
  },
  mcp: { status: 'ready', profile: 'marketplace-lite', publicToolCount: 12 },
  configuration: {
    status: 'ready',
    revision: 3,
    openaiBackend: {
      backendId: 'openai-responses',
      model: 'gpt-5.6-sol',
      enabled: true,
      readiness: 'ready',
      secretReferencePresent: true,
      testAvailable: true
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
  integrations: [
    {
      target: 'codex',
      label: 'Codex',
      connection: { state: 'detected-not-connected', label: 'Detected, not connected', reasonCode: 'client-detected-install-ready' },
      client: { detected: true, status: 'ready', version: '0.144.4', compatibility: 'supported', minimumSupportedVersion: '0.144.1' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.1' },
      discovery: 'Discovered but unmanaged',
      candidateRequired: true,
      legacyDetected: true,
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'ready', marketplace: 'missing', directPackage: null,
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Codex personal marketplace',
      activationPolicy: 'Client action required',
      ownership: 'Unmanaged',
      nextAction: 'Install available',
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
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.1' },
      discovery: 'Discovered but unmanaged',
      candidateRequired: true,
      legacyDetected: false,
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'missing', marketplace: 'missing', directPackage: 'missing',
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Claude Code local marketplace',
      activationPolicy: 'Reload or client action required',
      ownership: 'Not installed',
      nextAction: 'Install available',
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
    academicGraph: true,
    agentBackendConfig: false,
    agentBackendTest: false,
    agentBackendRun: false,
    orchestration: false,
    apply: false
  }
} satisfies AppSnapshot;

const fixtureCaptureId = `cap_${'a'.repeat(64)}`;
const fixtureProjectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
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
  sourceCount: 3,
  presentSourceCount: 3,
  nodeCount: 6,
  edgeCount: 6,
  diagnosticCount: 0,
  sources: [
    { sourceKind: 'project-manifest', artifactPath: 'context/project_manifest.json', present: true, contentDigest: 'c'.repeat(64), sizeBytes: 512 },
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

const fixtureOrchestrationRun = {
  runId: `run_${'2'.repeat(32)}`,
  profileId: 'openai-solo-v1',
  executionMode: 'solo',
  status: 'running',
  generation: 3,
  documentSha256: '3'.repeat(64),
  completedTaskCount: 1,
  totalTaskCount: 76,
  nextTaskId: 'A1_5',
  activeTaskId: null,
  recoveryRequired: false,
  canContinue: true,
  canPause: true,
  canResume: false,
  canRecover: false,
  canCancel: true
} satisfies OrchestrationRunSummary;

const fixtureOrchestrationDoctor = {
  schemaVersion: 1,
  projectId: fixtureProjectId,
  expectedProjectRevision: 12,
  workflowContractStatus: 'ready',
  backendReadiness: 'ready',
  runCount: 1,
  activeRunCount: 1,
  recoveryRequiredCount: 0,
  runnable: false,
  reasonCodes: ['orchestration-active-run-exists']
} satisfies OrchestrationDoctor;

const fixtureOrchestrationRuns = {
  schemaVersion: 1,
  projectId: fixtureProjectId,
  expectedProjectRevision: 12,
  runs: [fixtureOrchestrationRun]
} satisfies OrchestrationRunList;

export function sourceFixtureTransport(): AppTransport {
  let pendingCaptureOperation = false;
  let pendingAgentRun = false;
  let pendingOrchestration = false;
  return {
    async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
      if (command === 'qiongli_snapshot') return sourceSnapshot as T;
      if (command !== 'qiongli_execute') throw new Error('dev-fixture-command-unsupported');
      const intent = args?.intent as AppIntent | undefined;
      if (!intent) throw new Error('dev-fixture-intent-missing');
      if (intent.action === 'confirm-operation' && pendingCaptureOperation) {
        pendingCaptureOperation = false;
        return {
          type: 'capture-operation-completed',
          code: 'fixture-capture-operation-completed',
          snapshot: sourceSnapshot,
          inbox: captureInbox,
          coverage: captureCoverage,
          changes: artifactChanges
        } as T;
      }
      if (intent.action === 'confirm-operation' && pendingAgentRun) {
        pendingAgentRun = false;
        return {
          type: 'agent-run-completed',
          result: {
            schemaVersion: 1,
            runId: `run_${'1'.repeat(32)}`,
            backendId: 'openai-responses',
            model: 'gpt-5.6-sol',
            finishReason: 'stop',
            content: 'The registered project keeps its research state as the durable authority, while client sessions remain execution surfaces.',
            inputTokens: 312,
            outputTokens: 23,
            cachedInputTokens: 0,
            modelTurns: 2,
            toolCalls: 1,
            networkRequests: 2,
            auditedToolCalls: 1
          }
        } as T;
      }
      if (intent.action === 'confirm-operation' && pendingOrchestration) {
        pendingOrchestration = false;
        return {
          type: 'orchestration-executed',
          execution: {
            schemaVersion: 1,
            outcome: 'task-completed',
            taskId: 'A1',
            run: fixtureOrchestrationRun,
            roleOutputs: [{
              taskId: 'A1',
              role: 'primary',
              outputSha256: '4'.repeat(64),
              model: 'gpt-5.6-sol',
              finishReason: 'stop',
              content: 'The fixture produced a revision-bound research-question candidate.',
              modelTurns: 1,
              toolCalls: 0,
              networkRequests: 1
            }]
          },
          doctor: fixtureOrchestrationDoctor,
          runs: fixtureOrchestrationRuns
        } as T;
      }
      const event = fixtureEvent(intent);
      pendingCaptureOperation = event.type === 'capture-intake-preview'
        || event.type === 'capture-consolidation-preview';
      pendingAgentRun = event.type === 'preview' && event.preview.kind === 'agent-run';
      pendingOrchestration = event.type === 'preview'
        && (event.preview.kind === 'orchestration-test'
          || event.preview.kind === 'orchestration-continue');
      if (intent.action === 'cancel-operation') {
        pendingCaptureOperation = false;
        pendingAgentRun = false;
        pendingOrchestration = false;
      }
      return event as T;
    }
  };
}

function fixtureEvent(intent: AppIntent): AppEvent {
  switch (intent.action) {
    case 'refresh':
    case 'refresh-research-library':
    case 'refresh-integration-discovery':
      return { type: 'snapshot', snapshot: sourceSnapshot };
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
          availableVersion: '2.0.0-alpha.2',
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
          availableVersion: '2.0.0-alpha.2',
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
          displayTarget: 'Qiongli 2.0.0-alpha.2',
          planDigestSha256: '5'.repeat(64),
          approvalsRequired: ['filesystem-write'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'preview-agent-run':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000006',
          kind: 'agent-run',
          title: 'Run project query with OpenAI',
          summary: 'Send this prompt and redacted read-only project tool results to OpenAI after explicit confirmation.',
          displayTarget: null,
          planDigestSha256: '6'.repeat(64),
          approvalsRequired: ['Send prompt and redacted project data to OpenAI'],
          canConfirm: true,
          blockedReason: null
        }
      };
    case 'load-orchestration':
      return {
        type: 'orchestration-loaded',
        doctor: fixtureOrchestrationDoctor,
        runs: fixtureOrchestrationRuns
      };
    case 'preview-orchestration-test':
    case 'preview-orchestration-continue':
      return {
        type: 'preview',
        preview: {
          token: '00000000000000000000000000000007',
          kind: intent.action === 'preview-orchestration-test'
            ? 'orchestration-test'
            : 'orchestration-continue',
          title: intent.action === 'preview-orchestration-test'
            ? 'Start orchestration test'
            : 'Continue orchestration run',
          summary: 'Send the next canonical task packet and project-scoped read evidence to the configured OpenAI backend.',
          displayTarget: fixtureOrchestrationRun.runId,
          planDigestSha256: intent.action === 'preview-orchestration-continue'
            ? fixtureOrchestrationRun.documentSha256
            : null,
          approvalsRequired: ['network-request'],
          canConfirm: true,
          blockedReason: null
        }
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
        doctor: {
          ...fixtureOrchestrationDoctor,
          activeRunCount: status === 'cancelled' ? 0 : 1,
          runnable: status === 'cancelled',
          reasonCodes: status === 'cancelled' ? [] : ['orchestration-active-run-exists']
        },
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
    case 'verify-integrations':
    case 'verify-skills-preset':
      return { type: 'completed', code: 'fixture-verification-complete', snapshot: sourceSnapshot };
    case 'cancel-operation':
      return { type: 'cancelled', code: 'fixture-operation-cancelled' };
    case 'select-project-directory':
    case 'select-project-create-destination':
    case 'select-project-export-destination':
    case 'select-project-import-locations':
      return {
        type: 'project-directory-selected',
        token: '00000000000000000000000000000002',
        rootLabel: 'selected-article-project'
      };
    case 'confirm-operation':
    case 'open-project':
      return { type: 'completed', code: 'fixture-project-operation-completed', snapshot: sourceSnapshot };
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
  const relatedEdges = academicGraph.edges.filter((edge) => {
    const focusMatches = query.focusNodeId === null
      || (query.direction !== 'incoming' && edge.sourceNodeId === query.focusNodeId)
      || (query.direction !== 'outgoing' && edge.targetNodeId === query.focusNodeId);
    return focusMatches
      && (query.relations.length === 0 || query.relations.includes(edge.relation))
      && layerMatches(edge.layers);
  });
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
