import type {
  AppEvent,
  AppIntent,
  AppSnapshot,
  AppTransport,
  CaptureCoverageSnapshot,
  CaptureInboxSnapshot,
  OperationPreview,
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
    entryCount: 420,
    profiles: [
      { id: 'skill-only', label: 'Skills', description: 'Portable Skills and workflow guidance.', includedResourceKinds: 4 },
      { id: 'marketplace-lite', label: 'Plugin Lite', description: 'Skills plus the native Lite MCP adapter.', includedResourceKinds: 7 },
      { id: 'full', label: 'Full workflow', description: 'Complete workflow and future Full runtime declarations.', includedResourceKinds: 11 }
    ]
  },
  mcp: { status: 'ready', profile: 'marketplace-lite', publicToolCount: 12 },
  configuration: { status: 'ready', revision: 3, cleanupRequired: false },
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
    apply: false
  }
} satisfies AppSnapshot;

const fixtureCaptureId = `cap_${'a'.repeat(64)}`;
const fixtureProjectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';

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

export function sourceFixtureTransport(): AppTransport {
  let pendingCaptureOperation = false;
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
          coverage: captureCoverage
        } as T;
      }
      const event = fixtureEvent(intent);
      pendingCaptureOperation = event.type === 'capture-intake-preview'
        || event.type === 'capture-consolidation-preview';
      if (intent.action === 'cancel-operation') pendingCaptureOperation = false;
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
    case 'load-capture-inbox':
      return { type: 'capture-inbox', inbox: captureInbox };
    case 'load-capture-coverage':
      return { type: 'capture-coverage', coverage: captureCoverage };
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
