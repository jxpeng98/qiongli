import type { AppEvent, AppIntent, AppSnapshot, AppTransport } from '@qiongli/app-api';

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
    apply: false
  }
} satisfies AppSnapshot;

export function sourceFixtureTransport(): AppTransport {
  return {
    async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
      if (command === 'qiongli_snapshot') return sourceSnapshot as T;
      if (command !== 'qiongli_execute') throw new Error('dev-fixture-command-unsupported');
      const intent = args?.intent as AppIntent | undefined;
      if (!intent) throw new Error('dev-fixture-intent-missing');
      return fixtureEvent(intent) as T;
    }
  };
}

function fixtureEvent(intent: AppIntent): AppEvent {
  switch (intent.action) {
    case 'refresh':
    case 'refresh-research-library':
    case 'refresh-integration-discovery':
      return { type: 'snapshot', snapshot: sourceSnapshot };
    case 'verify-integrations':
    case 'verify-skills-preset':
      return { type: 'completed', code: 'fixture-verification-complete', snapshot: sourceSnapshot };
    case 'cancel-operation':
      return { type: 'cancelled', code: 'fixture-operation-cancelled' };
    case 'select-project-directory':
      return {
        type: 'project-directory-selected',
        token: '00000000000000000000000000000002',
        rootLabel: 'selected-article-project'
      };
    case 'confirm-operation':
      return { type: 'completed', code: 'fixture-project-operation-completed', snapshot: sourceSnapshot };
    case 'preview-project-register':
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
