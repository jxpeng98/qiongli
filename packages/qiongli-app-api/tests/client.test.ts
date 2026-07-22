import { describe, expect, it } from 'vitest';

import {
  QiongliAppClient,
  type AppTransport,
  appEventSchema,
  appIntentSchema,
  appSnapshotSchema,
  artifactChangeSnapshotSchema,
  articleProjectSummarySchema,
  captureCoverageSnapshotSchema
} from '../src';

const captureId = `cap_${'a'.repeat(64)}`;

const snapshot = {
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
    entryCount: 42,
    profiles: [
      { id: 'skill-only', label: 'Skills', description: 'Skills only', includedResourceKinds: 4 },
      { id: 'marketplace-lite', label: 'Plugin Lite', description: 'Skills and Lite MCP', includedResourceKinds: 7 },
      { id: 'full', label: 'Full workflow', description: 'Complete workflow', includedResourceKinds: 11 }
    ]
  },
  mcp: { status: 'ready', profile: 'marketplace-lite', publicToolCount: 12 },
  configuration: { status: 'ready', revision: 3, cleanupRequired: false },
  update: {
    status: 'unavailable',
    selectedStream: 'stable',
    phase: 'unavailable',
    availableVersion: null,
    archiveSizeBytes: null,
    progress: null,
    reasonCode: 'source-build-update-unavailable',
    remediation: 'install-trusted-qiongli-release',
    canSelectStream: false,
    canCheck: false,
    canPrepare: false,
    canInstall: false,
    canCancel: false
  },
  researchLibrary: {
    schemaVersion: 1,
    revision: 0,
    health: 'empty',
    projects: []
  },
  integrations: [
    {
      target: 'codex',
      label: 'Codex',
      connection: { state: 'detected-not-connected', label: 'Detected, not connected', reasonCode: 'client-discovered-content-missing' },
      client: { detected: true, status: 'ready', version: '1.2.3', compatibility: 'supported', minimumSupportedVersion: '0.144.1' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.1' },
      discovery: 'Discovered but unmanaged',
      candidateRequired: false,
      legacyDetected: false,
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'missing', marketplace: 'missing', directPackage: null,
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Codex personal marketplace',
      activationPolicy: 'Client action required',
      ownership: 'Not installed',
      nextAction: 'Install available',
      evidenceCode: 'client-discovered-content-missing',
      paths: []
    },
    {
      target: 'claude-code',
      label: 'Claude Code',
      connection: { state: 'client-not-detected', label: 'Client not detected', reasonCode: 'client-not-detected' },
      client: { detected: false, status: 'missing', version: null, compatibility: 'not-evaluated', minimumSupportedVersion: '2.1.206' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.1' },
      discovery: 'Client not discovered',
      candidateRequired: false,
      legacyDetected: false,
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'missing', marketplace: 'missing', directPackage: 'missing',
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Claude Code marketplace',
      activationPolicy: 'Reload or client action required',
      ownership: 'Not installed',
      nextAction: 'Inspect only',
      evidenceCode: 'client-not-detected',
      paths: []
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
    apply: false
  }
} as const;

describe('QiongliAppClient', () => {
  it('validates a bounded snapshot returned by the native bridge', async () => {
    const transport: AppTransport = { invoke: async <T>() => snapshot as T };
    await expect(new QiongliAppClient(transport).snapshot()).resolves.toEqual(snapshot);
  });

  it('rejects a frontend/native schema drift', () => {
    expect(() => appSnapshotSchema.parse({ ...snapshot, schemaVersion: 2 })).toThrow();
  });

  it('rejects unknown commands before crossing IPC', async () => {
    const transport: AppTransport = {
      invoke: async <T>() => ({ type: 'failed', code: 'unexpected' }) as T
    };
    await expect(
      new QiongliAppClient(transport).execute({ action: 'arbitrary-shell' } as never)
    ).rejects.toThrow();
  });

  it('accepts bounded update actions and state changes', () => {
    expect(appIntentSchema.parse({
      action: 'select-update-stream',
      stream: 'beta'
    }).action).toBe('select-update-stream');
    expect(appIntentSchema.parse({ action: 'check-for-updates' }).action)
      .toBe('check-for-updates');
    expect(appEventSchema.parse({
      type: 'update-changed',
      update: snapshot.update,
      closeRequested: false
    }).type).toBe('update-changed');
    expect(() => appEventSchema.parse({
      type: 'update-changed',
      update: snapshot.update,
      closeRequested: false,
      archivePath: '/private/update.zip'
    })).toThrow();
  });

  it('accepts only opaque native directory selections', () => {
    expect(appEventSchema.parse({
      type: 'project-directory-selected',
      token: '0000000000000000000000000000002a',
      rootLabel: 'article-project'
    })).toEqual({
      type: 'project-directory-selected',
      token: '0000000000000000000000000000002a',
      rootLabel: 'article-project'
    });
    expect(() => appEventSchema.parse({
      type: 'project-directory-selected',
      token: '0000000000000000000000000000002a',
      rootLabel: 'article-project',
      rootPath: '/private/research/article-project'
    })).toThrow();
  });

  it('accepts bounded project mobility intents without accepting paths', () => {
    expect(appIntentSchema.parse({
      action: 'preview-project-create',
      directoryToken: '0000000000000000000000000000002a',
      displayName: 'Portable paper',
      projectKind: 'article',
      stage: 'idea'
    }).action).toBe('preview-project-create');
    expect(appIntentSchema.parse({
      action: 'select-project-import-locations',
      suggestedName: 'imported-paper'
    }).action).toBe('select-project-import-locations');
    expect(() => appIntentSchema.parse({
      action: 'preview-project-import',
      directoryToken: '0000000000000000000000000000002a',
      sourcePath: '/private/session.json'
    })).toThrow();
  });

  it('accepts versioned Capture Inbox intents and rejects injected paths', () => {
    expect(appIntentSchema.parse({
      action: 'load-capture-coverage',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051'
    }).action).toBe('load-capture-coverage');
    expect(appIntentSchema.parse({
      action: 'load-artifact-changes',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051'
    }).action).toBe('load-artifact-changes');
    expect(appIntentSchema.parse({
      action: 'read-capture',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      captureId
    }).action).toBe('read-capture');
    expect(appIntentSchema.parse({
      action: 'preview-capture-intake',
      fileToken: '0000000000000000000000000000002a'
    }).action).toBe('preview-capture-intake');
    expect(() => appIntentSchema.parse({
      action: 'preview-capture-intake',
      fileToken: '0000000000000000000000000000002a',
      filePath: '/private/research/capture.json'
    })).toThrow();
  });

  it('accepts bounded academic graph queries and rejects unbounded filters', () => {
    const projectionId = `grp_${'a'.repeat(64)}`;
    const query = {
      expectedProjectionId: projectionId,
      focusNodeId: null,
      direction: 'both' as const,
      nodeTypes: ['claim'] as const,
      relations: ['supports'] as const,
      layers: ['argument'] as const,
      canonicalId: null,
      text: 'evidence provenance',
      maxNodes: 100,
      maxEdges: 200
    };

    expect(appIntentSchema.parse({
      action: 'query-academic-graph',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      query
    }).action).toBe('query-academic-graph');
    expect(() => appIntentSchema.parse({
      action: 'query-academic-graph',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      query: { ...query, maxNodes: 10_000 }
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'query-academic-graph',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      query: { ...query, projectPath: '/private/research/article' }
    })).toThrow();

    const pathQuery = {
      expectedProjectionId: projectionId,
      sourceNodeId: `nod_${'b'.repeat(64)}`,
      targetNodeId: `nod_${'c'.repeat(64)}`,
      maxHops: 6
    };
    expect(appIntentSchema.parse({
      action: 'query-academic-graph-path',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      query: pathQuery
    }).action).toBe('query-academic-graph-path');
    expect(() => appIntentSchema.parse({
      action: 'query-academic-graph-path',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      query: { ...pathQuery, maxHops: 13, projectPath: '/private/research/article' }
    })).toThrow();

    expect(appIntentSchema.parse({
      action: 'open-academic-graph-artifact',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      expectedProjectRevision: 12,
      expectedProjectionId: projectionId,
      entity: { kind: 'node', id: `nod_${'b'.repeat(64)}` }
    }).action).toBe('open-academic-graph-artifact');
    expect(() => appIntentSchema.parse({
      action: 'open-academic-graph-artifact',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      expectedProjectRevision: 12,
      expectedProjectionId: projectionId,
      entity: { kind: 'edge', id: `nod_${'b'.repeat(64)}` },
      artifactPath: '/private/research/article/context/research_state.md'
    })).toThrow();
  });

  it('accepts revision-bound unattributed artifact drift without private paths', () => {
    const observations = [
      ['research-state', 'context/research_state.md', true],
      ['decision-log', 'context/decision_log.md', false],
      ['stage-handoff', 'context/stage_handoff.md', false],
      ['boundary-review', 'context/boundary_review.md', false],
      ['idea-funnel', 'context/idea_funnel.md', false],
      ['literature-map', 'literature/literature_map.md', false],
      ['claim-evidence-ledger', 'evidence/claim-evidence-ledger.csv', false],
      ['manuscript-claim-map', 'manuscript/claims_evidence_map.md', false]
    ].map(([artifact, relativePath, present]) => ({ artifact, relativePath, present }));
    const changes = {
      schemaVersion: 1,
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      projectRevision: 1,
      projectStage: 'writing',
      state: 'unattributed',
      registeredArtifactCount: 8,
      presentArtifactCount: 1,
      changeCount: 1,
      unattributedCount: 1,
      changes: [{
        changeId: `chg_${'a'.repeat(64)}`,
        state: 'unattributed',
        detection: 'exact',
        effect: 'created',
        baseRevision: 1,
        relativePaths: ['context/research_state.md'],
        reason: 'no-accepted-capture-lineage'
      }],
      artifacts: observations
    };

    expect(artifactChangeSnapshotSchema.parse(changes)).toEqual(changes);
    expect(appEventSchema.parse({ type: 'artifact-changes', changes }).type)
      .toBe('artifact-changes');
    expect(() => artifactChangeSnapshotSchema.parse({
      ...changes,
      changes: [{ ...changes.changes[0], source: 'codex' }]
    })).toThrow();
    expect(() => appEventSchema.parse({
      type: 'artifact-changes',
      changes,
      rootPath: '/private/research/paper'
    })).toThrow();
  });

  it('keeps every capture source visible and unknown without host evidence', () => {
    const sources = [
      'codex',
      'claude-code',
      'chat-gpt',
      'cli',
      'manual',
      'repository',
      'portable-file'
    ] as const;
    const coverage = {
      schemaVersion: 1,
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      projectRevision: 1,
      projectStage: 'writing',
      captureCount: 0,
      connectedCount: 0,
      repositoryBackedCount: 0,
      portableCount: 0,
      manualCount: 0,
      pendingReviewCount: 0,
      currentCount: 0,
      staleCount: 0,
      conflictedCount: 0,
      unboundCount: 0,
      unknownSourceCount: 7,
      sources: sources.map((source) => ({
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
    };

    expect(captureCoverageSnapshotSchema.parse(coverage)).toEqual(coverage);
    expect(appEventSchema.parse({ type: 'capture-coverage', coverage }).type)
      .toBe('capture-coverage');
    expect(() => captureCoverageSnapshotSchema.parse({
      ...coverage,
      sources: coverage.sources.slice(0, 6)
    })).toThrow();
    expect(() => appEventSchema.parse({
      type: 'capture-coverage',
      coverage,
      repositoryPath: '/private/repository'
    })).toThrow();
  });

  it('accepts only opaque capture file selections', () => {
    expect(appEventSchema.parse({
      type: 'capture-file-selected',
      token: '0000000000000000000000000000002a',
      fileLabel: 'capture.json'
    })).toEqual({
      type: 'capture-file-selected',
      token: '0000000000000000000000000000002a',
      fileLabel: 'capture.json'
    });
    expect(() => appEventSchema.parse({
      type: 'capture-file-selected',
      token: '0000000000000000000000000000002a',
      fileLabel: 'capture.json',
      filePath: '/private/research/capture.json'
    })).toThrow();
  });

  it('rejects an absolute path injected into a project summary', () => {
    expect(() => articleProjectSummarySchema.parse({
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      displayName: 'Example article',
      projectKind: 'article',
      stage: 'writing',
      lifecycle: 'active',
      semanticRevision: 2,
      registeredAtUnix: 1,
      lastOpenedAtUnix: null,
      academicallyUpdatedAtUnix: 2,
      health: 'ready',
      nextAction: 'open',
      rootLabel: 'example-article',
      rootPath: '/private/research/example-article',
      overview: {
        focalQuestion: null,
        thesis: null,
        evidencePosition: null,
        unresolvedRiskCount: 0,
        claimEvidenceCoveragePercent: null,
        nextPriorities: []
      }
    })).toThrow();
  });

  it('validates the mandatory canonical Rust snapshot and every event contract', async () => {
    const processLike = (globalThis as typeof globalThis & {
      process?: { env?: Record<string, string | undefined> };
    }).process;
    const fixtureModuleUrl = processLike?.env?.QIONGLI_APP_CONTRACT_MODULE;
    expect(
      fixtureModuleUrl,
      'Rust App API contract fixture is required; run this package through its test script'
    ).toBeTypeOf('string');

    const fixtureModule = await import(fixtureModuleUrl as string) as { default: unknown };
    const fixture = fixtureModule.default as Record<string, unknown>;
    expect(Object.keys(fixture).sort()).toEqual(['events', 'schemaVersion', 'snapshot']);
    expect(fixture.schemaVersion).toBe(1);

    const parsed = appSnapshotSchema.parse(fixture.snapshot);
    expect(parsed.schemaVersion).toBe(1);
    expect(parsed.integrations).toHaveLength(2);
    expect(parsed.researchLibrary.projects).toEqual([]);

    expect(Array.isArray(fixture.events)).toBe(true);
    const eventTypes = (fixture.events as unknown[]).map((event) => appEventSchema.parse(event).type);
    expect(eventTypes).toEqual([
      'snapshot',
      'preview',
      'capture-inbox',
      'capture-coverage',
      'artifact-changes',
      'academic-graph',
      'academic-graph-query',
      'academic-graph-path',
      'academic-graph-artifact-opened',
      'capture-read',
      'project-directory-selected',
      'capture-file-selected',
      'capture-intake-preview',
      'capture-consolidation-preview',
      'update-changed',
      'completed',
      'capture-operation-completed',
      'cancelled',
      'validation-failed',
      'failed'
    ]);

    const pathEvent = (fixture.events as Array<Record<string, unknown>>)
      .find((event) => event.type === 'academic-graph-path');
    expect(pathEvent).toBeDefined();
    const malformedPathEvent = JSON.parse(JSON.stringify(pathEvent)) as {
      result: { steps: Array<{ toNodeId: string }>; sourceNodeId: string };
    };
    malformedPathEvent.result.steps[0]!.toNodeId = malformedPathEvent.result.sourceNodeId;
    expect(() => appEventSchema.parse(malformedPathEvent)).toThrow();
  });
});
