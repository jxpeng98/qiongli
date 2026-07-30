import { describe, expect, it } from 'vitest';

import {
  QiongliAppClient,
  type AppTransport,
  appEventSchema,
  appIntentSchema,
  appSnapshotSchema,
  artifactChangeSnapshotSchema,
  articleProjectSummarySchema,
  captureCoverageSnapshotSchema,
  managedOperationPlanV1Schema,
  operationPreviewSchema,
  orchestrationRunSummarySchema
} from '../src';

const captureId = `cap_${'a'.repeat(64)}`;

const snapshot = {
  schemaVersion: 14,
  product: {
    version: '2.0.0-alpha.2',
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
    ],
    managedSkills: {
      status: 'ready',
      destinations: [{
        targetId: `skills-target-${'1'.repeat(64)}`,
        preset: 'qiongli-managed',
        symbolicPath: '<user-home>/.qiongli-skills',
        state: 'missing',
        status: 'missing',
        profile: null,
        productVersion: null,
        projectId: null
      }]
    }
  },
  mcp: { status: 'ready', profile: 'marketplace-lite', publicToolCount: 12 },
  cli: {
    status: 'missing',
    state: 'missing',
    installedVersion: null,
    availableVersion: '2.0.0-alpha.2',
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
    legacyCredential: {
      referencePresent: false,
      cleanupAvailable: false
    },
    cleanupRequired: false
  },
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
  legacyMigration: {
    state: 'not-detected',
    nextAction: 'none',
    migrationId: null,
    detectedItems: 0,
    eligibleItems: 0,
    reviewItems: 0,
    reasonCode: 'legacy-migration-not-detected',
    providerConflicts: []
  },
  integrations: [
    {
      target: 'codex',
      label: 'Codex',
      connection: { state: 'detected-not-connected', label: 'Detected, not connected', reasonCode: 'client-discovered-content-missing' },
      client: { detected: true, status: 'ready', version: '1.2.3', compatibility: 'supported', minimumSupportedVersion: '0.144.1' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.2' },
      discovery: 'Discovered but unmanaged',
      candidateRequired: false,
      legacyDetected: false,
      migration: { state: 'not-detected', detectedItems: 0, eligibleItems: 0, reviewItems: 0 },
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'missing', marketplace: 'missing', directPackage: null,
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Codex personal marketplace',
      activationPolicy: 'Client action required',
      ownership: 'Not installed',
      ownershipState: 'not-installed',
      nextAction: 'install-ready',
      evidenceCode: 'client-discovered-content-missing',
      paths: []
    },
    {
      target: 'claude-code',
      label: 'Claude Code',
      connection: { state: 'client-not-detected', label: 'Client not detected', reasonCode: 'client-not-detected' },
      client: { detected: false, status: 'missing', version: null, compatibility: 'not-evaluated', minimumSupportedVersion: '2.1.206' },
      plugin: { installedVersion: null, availableVersion: '2.0.0-alpha.2' },
      discovery: 'Client not discovered',
      candidateRequired: false,
      legacyDetected: false,
      migration: { state: 'not-detected', detectedItems: 0, eligibleItems: 0, reviewItems: 0 },
      overall: 'missing',
      managedContent: {
        source: 'missing', skills: 'missing', marketplace: 'missing', directPackage: 'missing',
        registration: 'missing', activation: 'missing', activationObservation: 'missing',
        mcpAttachment: 'missing', mcpAttachmentObservation: 'missing'
      },
      symbolicLocation: 'Claude Code marketplace',
      activationPolicy: 'Reload or client action required',
      ownership: 'Not installed',
      ownershipState: 'not-installed',
      nextAction: 'inspect-only',
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
    captureDelivery: true,
    captureResolution: true,
    academicGraph: true,
    portfolio: true,
    timeline: true,
    orchestrationInspect: true,
    orchestrationControl: true,
    legacyCredentialCleanup: false,
    apply: false
  }
} as const;

describe('QiongliAppClient', () => {
  it('validates a bounded snapshot returned by the native bridge', async () => {
    const transport: AppTransport = { invoke: async <T>() => snapshot as T };
    await expect(new QiongliAppClient(transport).snapshot()).resolves.toEqual(snapshot);
  });

  it('rejects a frontend/native schema drift', () => {
    expect(() => appSnapshotSchema.parse({ ...snapshot, schemaVersion: 4 })).toThrow();
  });

  it('rejects duplicate or noncanonical integration target identities', () => {
    expect(appSnapshotSchema.safeParse({
      ...snapshot,
      integrations: [snapshot.integrations[1], snapshot.integrations[0]]
    }).success).toBe(false);
    expect(appSnapshotSchema.safeParse({
      ...snapshot,
      integrations: [snapshot.integrations[0], snapshot.integrations[0]]
    }).success).toBe(false);
  });

  it('rejects contradictory product trust and mutation authority', () => {
    expect(appSnapshotSchema.safeParse({
      ...snapshot,
      capabilities: {
        ...snapshot.capabilities,
        apply: true
      }
    }).success).toBe(false);
    expect(appSnapshotSchema.safeParse({
      ...snapshot,
      product: {
        ...snapshot.product,
        trust: {
          ...snapshot.product.trust,
          mode: 'packaged-product',
          canApply: true
        }
      }
    }).success).toBe(false);
  });

  it('keeps update and repair under one selection-bound Integration intent', () => {
    expect(appIntentSchema.parse({
      action: 'preview-reconcile-integrations',
      selection: { codex: false, claudeCode: true }
    })).toEqual({
      action: 'preview-reconcile-integrations',
      selection: { codex: false, claudeCode: true }
    });
    expect(() => appIntentSchema.parse({ action: 'preview-repair-all' })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'preview-update-integrations',
      selection: { codex: false, claudeCode: true }
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'preview-reconcile-integrations'
    })).toThrow();
  });

  it('requires one causal upgrade state for unsupported client versions', () => {
    const unsupported = {
      ...snapshot,
      integrations: snapshot.integrations.map((integration, index) =>
        index === 0
          ? {
              ...integration,
              connection: {
                state: 'unsupported-client-version' as const,
                label: 'Unsupported client version',
                reasonCode: 'client-version-below-supported-minimum'
              },
              client: {
                ...integration.client,
                version: '0.144.0',
                compatibility: 'unsupported' as const
              },
              overall: 'blocked' as const,
              nextAction: 'upgrade-client' as const,
              evidenceCode: 'client-version-below-supported-minimum'
            }
          : integration
      )
    };
    expect(appSnapshotSchema.parse(unsupported).integrations[0]?.nextAction)
      .toBe('upgrade-client');
    expect(() => appSnapshotSchema.parse({
      ...unsupported,
      integrations: unsupported.integrations.map((integration, index) =>
        index === 0
          ? { ...integration, nextAction: 'install-ready' as const }
          : integration
      )
    })).toThrow();
    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      integrations: snapshot.integrations.map((integration, index) =>
        index === 0
          ? { ...integration, nextAction: 'current' as const }
          : integration
      )
    })).toThrow();
  });

  it('allows PATH diagnosis for an outdated regular CLI but not a missing target', () => {
    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      cli: {
        ...snapshot.cli,
        canTest: true
      }
    })).toThrow();
    const outdated = {
      ...snapshot,
      cli: {
        ...snapshot.cli,
        status: 'attention',
        state: 'update-available',
        installedVersion: '1.19.0-beta.1',
        pathStatus: 'attention',
        pathState: 'version-mismatch',
        reasonCode: 'qiongli-cli-shell-version-mismatch',
        canInstall: true,
        canTest: true
      }
    } as const;
    expect(appSnapshotSchema.parse(outdated).cli).toEqual(outdated.cli);
  });

  it('rejects contradictory or path-leaking managed Skills evidence', () => {
    const destination = snapshot.content.managedSkills.destinations[0];
    const unmanaged = {
      ...snapshot,
      content: {
        ...snapshot.content,
        managedSkills: {
          status: 'conflict',
          destinations: snapshot.content.managedSkills.destinations.map((candidate, index) =>
            index === 0
              ? { ...candidate, state: 'unmanaged' as const, status: 'conflict' as const }
              : candidate
          )
        }
      }
    };
    expect(appSnapshotSchema.parse(unmanaged).content.managedSkills.status).toBe('conflict');
    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      content: {
        ...snapshot.content,
        managedSkills: {
          status: 'ready',
          destinations: [{
            ...destination,
            state: 'current',
            status: 'ready'
          }]
        }
      }
    })).toThrow();
    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      content: {
        ...snapshot.content,
        managedSkills: {
          status: 'ready',
          destinations: [{
            ...destination,
            symbolicPath: '/Users/researcher/.qiongli-skills'
          }]
        }
      }
    })).toThrow();
  });

  it('manages custom Skills destinations only through opaque native target ids', () => {
    const targetId = `skills-target-${'2'.repeat(64)}`;
    expect(appIntentSchema.parse({
      action: 'select-skills-destination'
    })).toEqual({
      action: 'select-skills-destination'
    });
    for (const action of [
      'verify-managed-skills-target',
      'preview-update-managed-skills-target',
      'preview-remove-managed-skills-target',
      'preview-detach-managed-skills-target'
    ] as const) {
      expect(appIntentSchema.parse({ action, targetId })).toEqual({ action, targetId });
      expect(() => appIntentSchema.parse({
        action,
        targetId,
        path: '/Users/researcher/private-skills'
      })).toThrow();
      expect(() => appIntentSchema.parse({
        action,
        targetId: 'skills-target-not-a-digest'
      })).toThrow();
    }
    expect(appEventSchema.parse({
      type: 'skills-destination-selected',
      targetId,
      symbolicPath: '<custom-folder>'
    })).toEqual({
      type: 'skills-destination-selected',
      targetId,
      symbolicPath: '<custom-folder>'
    });
    expect(() => appEventSchema.parse({
      type: 'skills-destination-selected',
      targetId,
      symbolicPath: '<custom-folder>',
      path: '/Users/researcher/private-skills'
    })).toThrow();
  });

  it('binds project Skills installation to a registered project id without accepting a path', () => {
    const projectId = `prj_${'a'.repeat(32)}`;
    expect(appIntentSchema.parse({
      action: 'preview-project-skills-materialization',
      profile: 'marketplace-lite',
      projectId
    })).toEqual({
      action: 'preview-project-skills-materialization',
      profile: 'marketplace-lite',
      projectId
    });
    expect(() => appIntentSchema.parse({
      action: 'preview-project-skills-materialization',
      profile: 'marketplace-lite',
      projectId,
      path: '/Users/researcher/private-project'
    })).toThrow();
    expect(appSnapshotSchema.parse({
      ...snapshot,
      content: {
        ...snapshot.content,
        managedSkills: {
          status: 'ready',
          destinations: [{
            ...snapshot.content.managedSkills.destinations[0],
            targetId: `skills-target-${'3'.repeat(64)}`,
            preset: 'current-project',
            symbolicPath: '<project>/.qiongli-skills',
            projectId
          }]
        }
      }
    }).content.managedSkills.destinations[0]?.projectId).toBe(projectId);
    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      content: {
        ...snapshot.content,
        managedSkills: {
          status: 'ready',
          destinations: [{
            ...snapshot.content.managedSkills.destinations[0],
            preset: 'current-project',
            symbolicPath: '<project>/.qiongli-skills'
          }]
        }
      }
    })).toThrow();
  });

  it('validates path-free, digest-bound managed operation plans', () => {
    const plan = {
      document_kind: 'qiongli-managed-operation-plan',
      schema_version: 1,
      product_version: '2.0.0-alpha.2',
      content_pack_sha256: '1'.repeat(64),
      content_root_sha256: '2'.repeat(64),
      created_at_unix: 1_000,
      expires_at_unix: 1_600,
      operation: {
        kind: 'skills-reconcile-preset',
        preset: 'qiongli-managed',
        target_id: `skills-target-${'3'.repeat(64)}`,
        profile: 'skill-only',
        expected_state: 'missing',
        expected_receipt_sha256: null
      },
      approvals_required: ['filesystem-write'],
      semantic_digest_sha256: '4'.repeat(64),
      plan_digest_sha256: '5'.repeat(64)
    } as const;
    expect(managedOperationPlanV1Schema.parse(plan)).toEqual(plan);
    expect(() => managedOperationPlanV1Schema.parse({
      ...plan,
      operation: {
        ...plan.operation,
        target_path: '/Users/researcher/private-skills'
      }
    })).toThrow();
    expect(() => managedOperationPlanV1Schema.parse({
      ...plan,
      approvals_required: ['filesystem-write', 'host-trust']
    })).toThrow();
    expect(() => managedOperationPlanV1Schema.parse({
      ...plan,
      expires_at_unix: 1_601
    })).toThrow();
    const detachPlan = {
      ...plan,
      operation: {
        kind: 'skills-detach-target',
        target_id: `skills-target-${'3'.repeat(64)}`,
        profile: 'skill-only',
        expected_state: 'drifted',
        expected_receipt_sha256: '6'.repeat(64)
      }
    } as const;
    expect(managedOperationPlanV1Schema.parse(detachPlan)).toEqual(detachPlan);
    expect(() => managedOperationPlanV1Schema.parse({
      ...detachPlan,
      operation: {
        ...detachPlan.operation,
        expected_state: 'current'
      }
    })).toThrow();

    const integrationPlan = {
      ...plan,
      operation: {
        kind: 'integrations-reconcile',
        mode: 'install',
        control_sha256: '6'.repeat(64),
        native_batch_plan_digest_sha256: '7'.repeat(64),
        installs: [{
          target: 'codex',
          effect: 'install',
          native_plan_digest_sha256: '8'.repeat(64)
        }]
      },
      approvals_required: ['filesystem-write', 'client-config-change', 'host-trust']
    } as const;
    expect(managedOperationPlanV1Schema.parse(integrationPlan)).toEqual(integrationPlan);
    expect(() => managedOperationPlanV1Schema.parse({
      ...integrationPlan,
      operation: {
        ...integrationPlan.operation,
        mode: 'repair'
      }
    })).toThrow();
    expect(managedOperationPlanV1Schema.parse({
      ...integrationPlan,
      operation: {
        ...integrationPlan.operation,
        mode: 'repair',
        installs: [{
          ...integrationPlan.operation.installs[0],
          effect: 'repair'
        }]
      }
    }).operation).toMatchObject({ mode: 'repair' });
  });

  it('rejects contradictory Zotero integration observations', () => {
    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      zotero: {
        ...snapshot.zotero,
        status: 'ready',
        state: 'ready',
        observation: 'observed'
      }
    })).toThrow();

    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      zotero: {
        ...snapshot.zotero,
        status: 'attention',
        state: 'companion-incompatible',
        observation: 'observed',
        connectorAvailable: true,
        companionAvailable: true,
        companionVersion: '0.3.0',
        endpointVersion: '2'
      }
    })).toThrow();

    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      zotero: {
        ...snapshot.zotero,
        status: 'attention',
        state: 'restart-required',
        observation: 'observed'
      }
    })).toThrow();

    expect(() => appSnapshotSchema.parse({
      ...snapshot,
      zotero: {
        ...snapshot.zotero,
        status: 'attention',
        state: 'zotero-incompatible',
        observation: 'observed',
        zoteroVersion: '7.0.15',
        canPrepareInstall: true
      }
    })).toThrow();
  });

  it('keeps Zotero actions path-free across IPC', () => {
    for (const action of [
      'refresh-zotero-integration',
      'preview-zotero-companion-stage',
      'reveal-zotero-companion',
      'open-zotero',
      'verify-zotero-integration'
    ] as const) {
      expect(appIntentSchema.parse({ action })).toEqual({ action });
      expect(() => appIntentSchema.parse({
        action,
        path: '/Users/researcher/Library/Application Support/Zotero'
      })).toThrow();
    }
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

  it('rejects retired direct-model intents and result events', () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    expect(() => appIntentSchema.parse({
      action: 'preview-agent-run',
      projectId,
      expectedProjectRevision: 12,
      prompt: 'Summarize the current evidence position.'
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'preview-agent-backend-credential',
      apiKey: 'private-provider-key'
    })).toThrow();
    expect(() => appIntentSchema.parse({ action: 'test-open-ai-backend' })).toThrow();
    expect(() => appEventSchema.parse({
      type: 'agent-run-completed',
      result: {
        schemaVersion: 1,
        runId: `run_${'1'.repeat(32)}`,
        backendId: 'openai-responses',
        model: 'gpt-5.6-sol',
        finishReason: 'stop',
        content: 'Bounded result.',
        inputTokens: 20,
        outputTokens: 4,
        cachedInputTokens: 0,
        modelTurns: 2,
        toolCalls: 1,
        networkRequests: 2,
        auditedToolCalls: 1
      }
    })).toThrow();
  });

  it('keeps host-owned Skills lifecycle out of standalone content intents', () => {
    for (const preset of ['detected-codex', 'detected-claude-code']) {
      expect(() => appIntentSchema.parse({
        action: 'preview-skills-preset-materialization',
        profile: 'marketplace-lite',
        preset
      })).toThrow();
      expect(() => appIntentSchema.parse({
        action: 'verify-skills-preset',
        preset
      })).toThrow();
      expect(() => appIntentSchema.parse({
        action: 'preview-skills-preset-removal',
        preset
      })).toThrow();
    }
  });

  it('requires managed operation previews to identify their symbolic destination', () => {
    const activation = {
      token: '00000000000000000000000000000001',
      kind: 'activation',
      title: 'Install selected integrations',
      summary: 'Install the selected managed Plugin and Skills content.',
      displayTarget:
        'Codex · <user-home>/.qiongli/plugins/codex/qiongli-next '
        + '→ <user-home>/.agents/plugins/marketplace.json',
      planDigestSha256: 'a'.repeat(64),
      approvalsRequired: ['filesystem-write', 'client-config-change', 'host-trust'],
      canConfirm: true,
      blockedReason: null
    };
    expect(operationPreviewSchema.parse(activation).displayTarget).toContain('qiongli-next');
    expect(() => operationPreviewSchema.parse({
      ...activation,
      displayTarget: null
    })).toThrow();

    const skills = {
      ...activation,
      kind: 'skills-materialization',
      title: 'Install standalone Skills',
      displayTarget: '<custom-folder>',
      approvalsRequired: ['filesystem-write']
    };
    expect(operationPreviewSchema.parse(skills).displayTarget).toBe('<custom-folder>');
    expect(() => operationPreviewSchema.parse({
      ...skills,
      displayTarget: '/Users/researcher/private-skills'
    })).toThrow();

    const cli = {
      ...skills,
      kind: 'cli-install',
      title: 'Install Qiongli CLI',
      displayTarget: '<user-home>/.local/bin/qiongli'
    };
    expect(operationPreviewSchema.parse(cli).displayTarget)
      .toBe('<user-home>/.local/bin/qiongli');
    expect(() => operationPreviewSchema.parse({
      ...cli,
      displayTarget: 'C:\\Users\\researcher\\.local\\bin\\qiongli.exe'
    })).toThrow();
  });

  it('closes orchestration controls to revision and checkpoint references', () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const runId = `run_${'2'.repeat(32)}`;
    const documentSha256 = '3'.repeat(64);
    const run = {
      runId,
      profileId: `host-solo-${'a'.repeat(24)}`,
      executionMode: 'solo',
      status: 'running',
      generation: 3,
      documentSha256,
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
    } as const;

    expect(() => appIntentSchema.parse({
      action: 'preview-orchestration-test',
      projectId,
      expectedProjectRevision: 12,
      executionMode: 'triad'
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'preview-orchestration-continue',
      projectId,
      expectedProjectRevision: 12,
      runId,
      expectedGeneration: 3,
      expectedDocumentSha256: documentSha256
    })).toThrow();
    expect(appIntentSchema.parse({
      action: 'load-orchestration',
      projectId,
      expectedProjectRevision: 12
    }).action).toBe('load-orchestration');
    expect(appIntentSchema.parse({
      action: 'control-orchestration',
      projectId,
      expectedProjectRevision: 12,
      runId,
      expectedGeneration: 3,
      expectedDocumentSha256: documentSha256,
      actionName: 'pause'
    }).action).toBe('control-orchestration');
    expect(() => appIntentSchema.parse({
      action: 'control-orchestration',
      projectId,
      expectedProjectRevision: 12,
      runId,
      expectedGeneration: 3,
      expectedDocumentSha256: documentSha256,
      actionName: 'pause',
      projectPath: '/private/research/article'
    })).toThrow();
    expect(() => orchestrationRunSummarySchema.parse({
      ...run,
      profileId: 'openai-solo-v1',
      hostDriven: false,
      canContinue: true
    })).toThrow();
    expect(orchestrationRunSummarySchema.parse({
      ...run,
      profileId: 'openai-solo-v1',
      hostDriven: false,
      recoveryRequired: true,
      canContinue: false,
      canPause: false,
      canResume: false,
      canRecover: false
    }).canCancel).toBe(true);

    expect(appEventSchema.parse({
      type: 'orchestration-run-updated',
      run,
      runs: {
        schemaVersion: 1,
        projectId,
        expectedProjectRevision: 12,
        runs: [run]
      }
    }).type).toBe('orchestration-run-updated');
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
    expect(appIntentSchema.parse({
      action: 'select-project-migration-locations',
      suggestedName: 'migrated-paper'
    }).action).toBe('select-project-migration-locations');
    expect(appIntentSchema.parse({
      action: 'preview-project-migration',
      directoryToken: '0000000000000000000000000000002a',
      displayName: 'Migrated paper',
      projectKind: 'article',
      stage: 'literature'
    }).action).toBe('preview-project-migration');
    expect(appIntentSchema.parse({
      action: 'select-project-migration-recovery-locations'
    }).action).toBe('select-project-migration-recovery-locations');
    expect(appIntentSchema.parse({
      action: 'preview-project-migration-recovery',
      directoryToken: '0000000000000000000000000000002a'
    }).action).toBe('preview-project-migration-recovery');
    expect(appIntentSchema.parse({
      action: 'select-project-migration-rollback-locations'
    }).action).toBe('select-project-migration-rollback-locations');
    expect(appIntentSchema.parse({
      action: 'preview-project-migration-rollback',
      directoryToken: '0000000000000000000000000000002a'
    }).action).toBe('preview-project-migration-rollback');
    expect(() => appIntentSchema.parse({
      action: 'preview-project-import',
      directoryToken: '0000000000000000000000000000002a',
      sourcePath: '/private/session.json'
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'preview-project-migration',
      directoryToken: '0000000000000000000000000000002a',
      displayName: 'Migrated paper',
      projectKind: 'article',
      stage: 'literature',
      sourcePath: '/private/legacy-paper'
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'preview-project-migration-rollback',
      directoryToken: '0000000000000000000000000000002a',
      destinationPath: '/private/migrated-paper'
    })).toThrow();
  });

  it('requires migration rollback reconciliation to match confirmation state', () => {
    const rollbackPreview = {
      token: '0000000000000000000000000000002a',
      kind: 'project-migration-rollback',
      title: 'Roll back migrated Qiongli 2 project',
      summary: 'Remove only the unchanged migration-owned destination and retain the source.',
      displayTarget: 'migrated-paper',
      planDigestSha256: '0'.repeat(64),
      approvalsRequired: ['filesystem-write'],
      canConfirm: true,
      blockedReason: null,
      migrationRollback: {
        registrationState: 'registered' as const,
        markerState: 'ready' as const,
        reconciliation: {
          status: 'matched-with-gaps' as const,
          matchedArtifactCount: 4,
          driftedArtifactCount: 0,
          continuityGapCount: 2,
          artifacts: [{
            category: 'research-state' as const,
            relativePath: 'context/research_state.md',
            state: 'matched' as const
          }]
        },
        sourceRetained: true as const,
        destinationRemoval: 'migration-owned-destination',
        canRollback: true
      }
    };
    expect(
      operationPreviewSchema.parse(rollbackPreview).migrationRollback?.reconciliation.status
    ).toBe('matched-with-gaps');
    expect(() => operationPreviewSchema.parse({
      ...rollbackPreview,
      canConfirm: false,
      blockedReason: 'project-migration-rollback-destination-drift'
    })).toThrow();
  });

  it('requires migration previews to describe copy behavior consistently', () => {
    const preview = {
      token: '0000000000000000000000000000002a',
      kind: 'project-migration',
      title: 'Migrate Qiongli 1.x article project',
      summary: 'Copy verified academic files into a new Qiongli 2 project.',
      displayTarget: 'migrated-paper',
      planDigestSha256: '0'.repeat(64),
      approvalsRequired: ['filesystem-write'],
      canConfirm: true,
      blockedReason: null,
      migration: {
        mode: 'copy' as const,
        copiedFileCount: 12,
        copiedBytes: 48_320,
        excludedEntryCount: 3,
        sourceRetained: true as const,
        copiesFiles: true,
        graphRebuildPasses: 2 as const
      }
    };
    expect(operationPreviewSchema.parse(preview).migration?.copiesFiles).toBe(true);
    expect(() => operationPreviewSchema.parse({
      ...preview,
      migration: {
        ...preview.migration,
        copiesFiles: false
      }
    })).toThrow();
  });

  it('requires internally consistent project migration qualification', () => {
    const qualified = {
      type: 'project-migration-completed' as const,
      code: 'project-migration-completed',
      snapshot,
      qualification: {
        projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
        status: 'verified' as const,
        projectionId: `grp_${'7'.repeat(64)}`,
        indexId: `gix_${'8'.repeat(64)}`,
        deterministicRebuild: true,
        reasonCode: null
      }
    };
    const parsed = appEventSchema.parse(qualified);
    expect(parsed.type).toBe('project-migration-completed');
    if (parsed.type !== 'project-migration-completed') throw new Error('unexpected event type');
    expect(parsed.qualification.status).toBe('verified');
    expect(() => appEventSchema.parse({
      ...qualified,
      qualification: {
        ...qualified.qualification,
        deterministicRebuild: false
      }
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
      maxDepth: 1,
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
      query: { ...query, maxDepth: 4 }
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'query-academic-graph',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      query: { ...query, maxDepth: 2 }
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

  it('closes continuity intents to bounded opaque identities and native cursors', () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const envelopeId = `env_${'1'.repeat(64)}`;
    const assignmentReceiptId = `car_${'2'.repeat(64)}`;
    const catalogId = `pca_${'3'.repeat(64)}`;
    expect(appIntentSchema.parse({
      action: 'load-capture-deliveries',
      request: {
        projectId,
        states: ['queued', 'retry-required'],
        limit: 64
      }
    }).action).toBe('load-capture-deliveries');
    expect(appIntentSchema.parse({
      action: 'retry-capture-delivery',
      envelopeId,
      expectedGeneration: 2,
      expectedRecordSha256: '4'.repeat(64),
      retriedAtUnix: 10,
      cause: 'transport-unavailable'
    }).action).toBe('retry-capture-delivery');
    expect(appIntentSchema.parse({
      action: 'preview-capture-resolution',
      assignmentReceiptId,
      reviewedAtUnix: 11,
      selections: [{
        itemId: `cri_${'5'.repeat(64)}`,
        disposition: 'accept-current'
      }]
    }).action).toBe('preview-capture-resolution');
    expect(appIntentSchema.parse({
      action: 'query-portfolio',
      request: {
        catalogId,
        filters: {
          projectId,
          evidenceSignal: 'contradiction',
          text: 'causal evidence'
        },
        limits: {
          projects: 32,
          nodes: 128,
          edges: 128,
          lineage: 128,
          maxBytes: 2 * 1_024 * 1_024
        }
      }
    }).action).toBe('query-portfolio');

    expect(() => appIntentSchema.parse({
      action: 'load-capture-deliveries',
      request: {
        limit: 64,
        cursor: {
          schemaVersion: 1,
          cursorId: `apc_${'6'.repeat(64)}`,
          kind: 'assignments',
          snapshotId: `als_${'7'.repeat(64)}`,
          afterId: `cai_${'8'.repeat(64)}`
        }
      }
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'query-portfolio',
      request: {
        catalogId,
        filters: { text: 'x'.repeat(257) },
        limits: {
          projects: 32,
          nodes: 128,
          edges: 128,
          lineage: 128,
          maxBytes: 2 * 1_024 * 1_024
        },
        projectRoot: '/private/research'
      }
    })).toThrow();
    expect(() => appIntentSchema.parse({
      action: 'load-capture-deliveries',
      request: {
        states: Array.from({ length: 8 }, () => 'queued'),
        limit: 64
      }
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
    expect(fixture.schemaVersion).toBe(14);

    const parsed = appSnapshotSchema.parse(fixture.snapshot);
    expect(parsed.schemaVersion).toBe(14);
    expect(parsed.integrations).toHaveLength(2);
    expect(parsed.researchLibrary.projects).toEqual([]);

    expect(Array.isArray(fixture.events)).toBe(true);
    const eventTypes = (fixture.events as unknown[]).map((event) => appEventSchema.parse(event).type);
    expect(eventTypes).toEqual([
      'snapshot',
      'preview',
      'skills-destination-selected',
      'capture-inbox',
      'capture-coverage',
      'artifact-changes',
      'academic-graph',
      'academic-graph-portfolio',
      'academic-graph-query',
      'academic-graph-path',
      'academic-graph-artifact-opened',
      'capture-read',
      'project-directory-selected',
      'project-migration-completed',
      'capture-file-selected',
      'capture-intake-preview',
      'capture-consolidation-preview',
      'capture-deliveries',
      'capture-delivery-inspected',
      'capture-delivery-updated',
      'capture-delivery-acknowledgement-preview',
      'capture-assignments',
      'capture-assignment-inspected',
      'capture-assignment-preview',
      'capture-resolutions',
      'capture-resolution-inspected',
      'capture-resolution-plan',
      'capture-resolution-preview',
      'portfolio-status',
      'portfolio-query',
      'semantic-timeline',
      'portfolio-doctor',
      'portfolio-maintenance-preview',
      'continuity-operation-progress',
      'portfolio-maintenance-completed',
      'update-changed',
      'orchestration-loaded',
      'orchestration-run-updated',
      'completed',
      'capture-operation-completed',
      'cancelled',
      'validation-failed',
      'failed'
    ]);

    const continuityEvents = fixture.events as Array<Record<string, unknown>>;
    const resolutionEvent = continuityEvents
      .find((event) => event.type === 'capture-resolution-preview');
    expect(resolutionEvent).toBeDefined();
    expect(() => appEventSchema.parse({
      ...resolutionEvent,
      selections: []
    })).toThrow();

    const deliveryPageEvent = continuityEvents
      .find((event) => event.type === 'capture-deliveries');
    expect(deliveryPageEvent).toBeDefined();
    const oversizedDeliveryPage = JSON.parse(JSON.stringify(deliveryPageEvent)) as {
      page: { entries: unknown[] };
    };
    oversizedDeliveryPage.page.entries = Array.from(
      { length: 257 },
      () => oversizedDeliveryPage.page.entries[0]
    );
    expect(() => appEventSchema.parse(oversizedDeliveryPage)).toThrow();

    const foreignDeliveryCursor = JSON.parse(JSON.stringify(deliveryPageEvent)) as {
      page: { nextCursor: { snapshotId: string } };
    };
    foreignDeliveryCursor.page.nextCursor.snapshotId = `dls_${'0'.repeat(64)}`;
    expect(() => appEventSchema.parse(foreignDeliveryCursor)).toThrow();

    expect(() => appEventSchema.parse({
      ...deliveryPageEvent,
      projectRoot: '/private/research'
    })).toThrow();

    const completionEvent = continuityEvents
      .find((event) => event.type === 'capture-operation-completed');
    const deliveryUpdatedEvent = continuityEvents
      .find((event) => event.type === 'capture-delivery-updated');
    const assignmentInspectedEvent = continuityEvents
      .find((event) => event.type === 'capture-assignment-inspected');
    expect(completionEvent).toBeDefined();
    expect(deliveryUpdatedEvent).toBeDefined();
    expect(assignmentInspectedEvent).toBeDefined();
    expect(completionEvent).toMatchObject({
      delivery: null,
      assignment: null,
      resolution: null
    });
    expect(appEventSchema.parse({
      ...completionEvent,
      delivery: deliveryUpdatedEvent?.delivery
    }).type).toBe('capture-operation-completed');
    expect(() => appEventSchema.parse({
      ...completionEvent,
      delivery: deliveryUpdatedEvent?.delivery,
      assignment: assignmentInspectedEvent?.assignment
    })).toThrow();

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
