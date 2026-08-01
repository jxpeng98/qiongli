import { describe, expect, it } from 'vitest';

import { QiongliAppClient, type AppEvent, type AppTransport } from '@qiongli/app-api';

import { AppState } from './app-state.svelte';
import { developmentSnapshotFixture } from './dev-transport';
import { i18n } from './i18n.svelte';

const preview: AppEvent = {
  type: 'preview',
  preview: {
    token: '0'.repeat(32),
    kind: 'project-import',
    title: 'Import portable article project',
    summary: 'Review the import before applying it.',
    displayTarget: 'imported-project',
    planDigestSha256: '1'.repeat(64),
    approvalsRequired: ['filesystem-write'],
    canConfirm: true,
    blockedReason: null
  }
};

const updateInstalling: AppEvent = {
  type: 'update-changed',
  update: {
    status: 'attention',
    selectedStream: 'stable',
    phase: 'installing',
    availableVersion: '2.0.0-alpha.3',
    archiveSizeBytes: 1024,
    progress: {
      completedSteps: 4,
      totalSteps: 5,
      label: 'Handing off to the native update helper',
      indeterminate: true
    },
    reasonCode: 'update-installing',
    remediation: 'restart-qiongli',
    canSelectStream: false,
    canCheck: false,
    canPrepare: false,
    canInstall: false,
    canCancel: false
  },
  closeRequested: false
};

const updatePreview: AppEvent = {
  type: 'preview',
  preview: {
    ...preview.preview,
    kind: 'update-install',
    title: 'Install Qiongli update',
    summary: 'Close Qiongli and activate the verified update.'
  }
};

describe('AppState confirmation recovery', () => {
  it('keeps a custom Skills folder opaque across read-only native probes', async () => {
    const targetId = `skills-target-${'3'.repeat(64)}`;
    const events: AppEvent[] = [
      {
        type: 'skills-destination-selected',
        targetId,
        symbolicPath: '<custom-folder>'
      },
      {
        type: 'snapshot',
        snapshot: developmentSnapshotFixture()
      }
    ];
    const transport: AppTransport = {
      invoke: async <T>() => events.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'select-skills-destination' });

    expect(state.selectedCustomSkillsTargetId).toBe(targetId);
    expect(state.notice?.detail).not.toContain('/Users/');
    await state.execute({ action: 'refresh-integration-discovery' });
    expect(state.selectedCustomSkillsTargetId).toBe(targetId);
  });

  it('clears a process-local custom Skills selection at the true reconnect boundary', async () => {
    const targetId = `skills-target-${'4'.repeat(64)}`;
    const responses: unknown[] = [
      {
        type: 'skills-destination-selected',
        targetId,
        symbolicPath: '<custom-folder>'
      } satisfies AppEvent,
      developmentSnapshotFixture()
    ];
    const transport: AppTransport = {
      invoke: async <T>() => responses.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'select-skills-destination' });
    expect(state.selectedCustomSkillsTargetId).toBe(targetId);

    await state.refresh();
    expect(state.selectedCustomSkillsTargetId).toBeNull();
  });

  it('keeps a selected custom Skills target after a local completion while it remains registered', async () => {
    const targetId = `skills-target-${'2'.repeat(64)}`;
    const events: AppEvent[] = [
      {
        type: 'skills-destination-selected',
        targetId,
        symbolicPath: '<custom-folder>'
      },
      {
        type: 'completed',
        code: 'managed-skills-target-verified',
        snapshot: developmentSnapshotFixture()
      }
    ];
    const transport: AppTransport = {
      invoke: async <T>() => events.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'select-skills-destination' });
    await state.execute({ action: 'verify-managed-skills-target', targetId });

    expect(state.selectedCustomSkillsTargetId).toBe(targetId);
  });

  it.each(['failed', 'validation-failed'] as const)(
    'closes an invalidated preview after a %s result',
    async (type) => {
      const events: AppEvent[] = [
        preview,
        { type, code: 'project-recovery-required' }
      ];
      const transport: AppTransport = {
        invoke: async <T>() => events.shift() as T
      };
      const state = new AppState(new QiongliAppClient(transport));

      await state.execute({ action: 'refresh-integration-discovery' });
      expect(state.preview?.kind).toBe('project-import');

      await state.execute({
        action: 'confirm-operation',
        token: preview.preview.token
      });

      expect(state.preview).toBeNull();
      expect(state.notice?.detail).toContain('choose Add existing project');
    }
  );

  it('closes an invalidated preview when native confirmation rejects', async () => {
    const transport: AppTransport = {
      invoke: async <T>(_command: string, args?: Record<string, unknown>) => {
        const intent = args?.intent as { action?: string } | undefined;
        if (intent?.action === 'confirm-operation') throw 'project-recovery-required';
        return preview as T;
      }
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'refresh-integration-discovery' });
    expect(state.preview?.kind).toBe('project-import');

    await state.execute({
      action: 'confirm-operation',
      token: preview.preview.token
    });

    expect(state.preview).toBeNull();
    expect(state.notice?.detail).toContain('choose Add existing project');
  });

  it('closes an accepted update preview while installation continues', async () => {
    const events: AppEvent[] = [updatePreview, updateInstalling];
    const transport: AppTransport = {
      invoke: async <T>() => events.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'preview-update-install' });
    expect(state.preview?.kind).toBe('update-install');

    await state.execute({
      action: 'confirm-operation',
      token: updatePreview.preview.token
    });

    expect(state.preview).toBeNull();
  });

  it.each([
    {
      status: 'verified' as const,
      projectionId: `grp_${'7'.repeat(64)}`,
      indexId: `gix_${'8'.repeat(64)}`,
      deterministicRebuild: true,
      reasonCode: null,
      expectedTone: 'success'
    },
    {
      status: 'rebuild-required' as const,
      projectionId: null,
      indexId: null,
      deterministicRebuild: false,
      reasonCode: 'project-migration-graph-rebuild-nondeterministic',
      expectedTone: 'warning'
    }
  ])('reports project migration graph qualification as $status', async (qualification) => {
    const snapshot = developmentSnapshotFixture();
    const event: AppEvent = {
      type: 'project-migration-completed',
      code: 'project-migration-completed',
      snapshot,
      qualification: {
        projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
        status: qualification.status,
        projectionId: qualification.projectionId,
        indexId: qualification.indexId,
        deterministicRebuild: qualification.deterministicRebuild,
        reasonCode: qualification.reasonCode
      }
    };
    const transport: AppTransport = {
      invoke: async <T>() => event as T
    };
    const state = new AppState(new QiongliAppClient(transport));
    state.preview = preview.preview;

    await state.execute({ action: 'refresh-integration-discovery' });

    expect(state.preview).toBeNull();
    expect(state.notice?.tone).toBe(qualification.expectedTone);
  });

  it('reports migration rollback without exposing the native completion code', async () => {
    await i18n.setLocale('zh-CN');
    try {
      const event: AppEvent = {
        type: 'completed',
        code: 'project-migration-rolled-back',
        snapshot: developmentSnapshotFixture()
      };
      const transport: AppTransport = {
        invoke: async <T>() => event as T
      };
      const state = new AppState(new QiongliAppClient(transport));
      state.preview = preview.preview;

      await state.execute({ action: 'refresh-integration-discovery' });

      expect(state.preview).toBeNull();
      expect(state.notice).toEqual({
        tone: 'success',
        title: '已回滚迁移副本',
        detail: '已取消注册并删除完全一致且未更改的穷理 2 目标目录；穷理 1.x 源目录保持不变。'
      });
    } finally {
      await i18n.setLocale('en');
    }
  });

  it('stores host orchestration checkpoints and control updates without model output', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const run = {
      runId: `run_${'2'.repeat(32)}`,
      profileId: `host-solo-${'a'.repeat(24)}`,
      executionMode: 'solo' as const,
      status: 'running' as const,
      generation: 3,
      documentSha256: '3'.repeat(64),
      completedTaskCount: 1,
      totalTaskCount: 76 as const,
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
    };
    const runs = {
      schemaVersion: 1 as const,
      projectId,
      expectedProjectRevision: 12,
      runs: [run]
    };
    const events: AppEvent[] = [
      { type: 'orchestration-loaded', runs },
      {
        type: 'orchestration-run-updated',
        run: {
          ...run,
          status: 'paused',
          generation: 4,
          documentSha256: '5'.repeat(64),
          canContinue: false,
          canPause: false,
          canResume: true
        },
        runs: {
          ...runs,
          runs: [{
            ...run,
            status: 'paused',
            generation: 4,
            documentSha256: '5'.repeat(64),
            canContinue: false,
            canPause: false,
            canResume: true
          }]
        }
      }
    ];
    const transport: AppTransport = {
      invoke: async <T>() => events.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'refresh-integration-discovery' });
    await state.execute({ action: 'refresh-integration-discovery' });

    expect(state.orchestrationRuns?.runs[0]?.status).toBe('paused');
    expect(state.orchestrationRuns?.runs[0]?.generation).toBe(4);
    expect(state.notice?.title).toBe('Orchestration run updated');
  });

  it('preserves the active graph after opening its exact source artifact', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const projectionId = `grp_${'a'.repeat(64)}`;
    const entity = { kind: 'node' as const, id: `nod_${'b'.repeat(64)}` };
    const opened: AppEvent = {
      type: 'academic-graph-artifact-opened',
      projectId,
      projectRevision: 12,
      projectionId,
      entity
    };
    const transport: AppTransport = {
      invoke: async <T>() => opened as T
    };
    const state = new AppState(new QiongliAppClient(transport));
    const graph = { projectionId } as NonNullable<AppState['academicGraph']>;
    const query = { projectionId } as NonNullable<AppState['academicGraphQuery']>;
    state.academicGraph = graph;
    state.academicGraphQuery = query;

    await state.execute({
      action: 'open-academic-graph-artifact',
      projectId,
      expectedProjectRevision: 12,
      expectedProjectionId: projectionId,
      entity
    });

    expect(state.academicGraph).toStrictEqual(graph);
    expect(state.academicGraphQuery).toStrictEqual(query);
  });

  it('stores a path-redacted artifact preview without replacing graph state', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const projectionId = `grp_${'a'.repeat(64)}`;
    const entity = { kind: 'node' as const, id: `nod_${'b'.repeat(64)}` };
    const artifact = {
      schemaVersion: 1 as const,
      documentKind: 'qiongli-project-artifact-view' as const,
      projectId,
      projectRevision: 12,
      projectionId,
      entityKind: entity.kind,
      entityId: entity.id,
      artifactPath: 'context/research_state.md' as const,
      sourceAnchor: 'Claim',
      format: 'markdown' as const,
      contentDigest: '7'.repeat(64),
      sourceSizeBytes: 8,
      content: '# Claim\n',
      contentSizeBytes: 8,
      startLine: 1,
      endLine: 2,
      anchorLine: 1,
      anchorMatched: true,
      truncatedBefore: false,
      truncatedAfter: false
    };
    const transport: AppTransport = {
      invoke: async <T>() => ({
        type: 'project-artifact-read',
        artifact
      }) as T
    };
    const state = new AppState(new QiongliAppClient(transport));
    const graph = { projectionId } as NonNullable<AppState['academicGraph']>;
    state.academicGraph = graph;

    await state.execute({
      action: 'read-project-artifact',
      projectId,
      expectedProjectRevision: 12,
      reference: {
        kind: 'academic-graph-entity',
        expectedProjectionId: projectionId,
        entity
      },
      maxBytes: 1_024
    });

    expect(state.projectArtifact).toEqual(artifact);
    expect(state.academicGraph).toStrictEqual(graph);
    expect(JSON.stringify(state.projectArtifact)).not.toContain('/private/');
  });

  it('stores strict continuity pages and clears a stale reviewed preview', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const envelopeId = `env_${'a'.repeat(64)}`;
    const delivery = {
      schemaVersion: 1 as const,
      envelopeId,
      captureId: `cap_${'b'.repeat(64)}`,
      source: 'codex' as const,
      delivery: 'connected' as const,
      destination: { projectId, expectedProjectRevision: 12 },
      state: 'delivered' as const,
      generation: 2,
      attemptCount: 1,
      retryCount: 0,
      createdAtUnix: 10,
      updatedAtUnix: 11,
      lastReason: 'delivery-accepted' as const,
      envelopeSha256: 'c'.repeat(64),
      recordSha256: 'd'.repeat(64),
      acknowledgement: null,
      capabilities: { canRetry: false, canCancel: false, canAcknowledge: true }
    };
    const events: AppEvent[] = [
      {
        type: 'capture-deliveries',
        page: {
          schemaVersion: 1,
          snapshotId: `dls_${'e'.repeat(64)}`,
          projectId,
          entries: [delivery],
          truncated: false,
          nextCursor: null
        }
      },
      {
        type: 'capture-delivery-acknowledgement-preview',
        acknowledgement: {
          schemaVersion: 1,
          planDigest: 'f'.repeat(64),
          envelopeId,
          destinationProjectId: projectId,
          acceptedCaptureId: delivery.captureId,
          expectedProjectRevision: 12,
          resultingProjectRevision: 12,
          acknowledgedAtUnix: 12,
          expectedGeneration: 2,
          expectedRecordSha256: delivery.recordSha256,
          approvalsRequired: ['delivery-acknowledgement']
        },
        preview: {
          token: '1'.repeat(32),
          kind: 'capture-delivery-acknowledgement',
          title: 'Acknowledge delivered capture',
          summary: 'Bind exact delivery evidence.',
          displayTarget: projectId,
          planDigestSha256: 'f'.repeat(64),
          approvalsRequired: ['delivery-acknowledgement'],
          canConfirm: true,
          blockedReason: null
        }
      },
      { type: 'validation-failed', code: 'capture-delivery-revision-conflict' }
    ];
    const transport: AppTransport = {
      invoke: async <T>() => events.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({
      action: 'load-capture-deliveries',
      request: { projectId, limit: 128 }
    });
    expect(state.captureDeliveries?.entries[0]).toEqual(delivery);

    await state.execute({
      action: 'preview-capture-delivery-acknowledgement',
      envelopeId,
      destinationProjectId: projectId,
      acceptedCaptureId: delivery.captureId,
      expectedProjectRevision: 12,
      resultingProjectRevision: 12,
      acknowledgedAtUnix: 12,
      expectedGeneration: 2,
      expectedRecordSha256: delivery.recordSha256
    });
    expect(state.captureDeliveryAcknowledgementPreview?.envelopeId).toBe(envelopeId);
    expect(state.preview?.kind).toBe('capture-delivery-acknowledgement');

    await state.execute({
      action: 'confirm-operation',
      token: '1'.repeat(32)
    });
    expect(state.preview).toBeNull();
    expect(state.captureDeliveryAcknowledgementPreview).toBeNull();
    expect(state.captureDeliveries?.entries[0]).toEqual(delivery);
  });

  it('invalidates continuity pages after a mutation but retains the exact affected record', async () => {
    const snapshot = developmentSnapshotFixture();
    const inbox = {
      schemaVersion: 1,
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
      projectRevision: 12
    } as NonNullable<AppState['captureInbox']>;
    const delivery = {
      envelopeId: `env_${'2'.repeat(64)}`
    } as NonNullable<AppState['captureDelivery']>;
    const event = {
      type: 'capture-operation-completed',
      code: 'capture-delivery-cancelled',
      snapshot,
      inbox,
      coverage: { projectId: inbox.projectId } as NonNullable<AppState['captureCoverage']>,
      changes: { projectId: inbox.projectId } as NonNullable<AppState['artifactChanges']>,
      delivery,
      assignment: null,
      resolution: null
    } as AppEvent;
    const state = new AppState();
    state.captureDeliveries = { entries: [{}] } as NonNullable<AppState['captureDeliveries']>;
    state.captureAssignments = { entries: [{}] } as NonNullable<AppState['captureAssignments']>;
    state.captureResolutions = { entries: [{}] } as NonNullable<AppState['captureResolutions']>;
    state.captureResolutionPlan = { items: [{}] } as NonNullable<AppState['captureResolutionPlan']>;

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent(event);

    expect(state.captureDeliveries).toBeNull();
    expect(state.captureAssignments).toBeNull();
    expect(state.captureResolutions).toBeNull();
    expect(state.captureResolutionPlan).toBeNull();
    expect(state.captureDelivery).toEqual(delivery);
    expect(state.notice?.tone).toBe('success');
  });

  it('stores a read-only resolution plan without opening confirmation', () => {
    const resolution = {
      planDigest: '6'.repeat(64),
      items: [{ itemId: `cri_${'7'.repeat(64)}` }]
    } as NonNullable<AppState['captureResolutionPlan']>;
    const state = new AppState();

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'capture-resolution-plan',
      resolution
    } as AppEvent);

    expect(state.captureResolutionPlan).toEqual(resolution);
    expect(state.captureResolutionPreview).toBeNull();
    expect(state.preview).toBeNull();
  });

  it('invalidates catalog-bound views when native portfolio identity changes', () => {
    const state = new AppState();
    const current = {
      schemaVersion: 1,
      state: 'current',
      libraryRevision: 7,
      catalogId: `pca_${'1'.repeat(64)}`,
      catalogGeneration: 2,
      portfolioId: `gpf_${'2'.repeat(64)}`,
      contributionCount: 1,
      projectCount: 1,
      nodeCount: 0,
      edgeCount: 0,
      reasonCode: 'portfolio-current',
      capabilities: {
        canQuery: true,
        canReconcile: true,
        canRebuild: true,
        canDeleteDerivedState: true
      }
    } as NonNullable<AppState['portfolioStatus']>;

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'portfolio-status',
      portfolio: current
    } as AppEvent);
    state.portfolioQuery = {
      catalogId: current.catalogId,
      queryId: `pqy_${'3'.repeat(64)}`
    } as NonNullable<AppState['portfolioQuery']>;
    state.semanticTimeline = {
      catalogId: current.catalogId,
      queryId: `pty_${'4'.repeat(64)}`
    } as NonNullable<AppState['semanticTimeline']>;
    state.portfolioMaintenancePreview = {
      expectedCatalogId: current.catalogId
    } as NonNullable<AppState['portfolioMaintenancePreview']>;
    state.preview = preview.preview;

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'portfolio-status',
      portfolio: { ...current, catalogGeneration: 3 }
    } as AppEvent);

    expect(state.portfolioQuery).toBeNull();
    expect(state.semanticTimeline).toBeNull();
    expect(state.portfolioMaintenancePreview).toBeNull();
    expect(state.preview).toBeNull();
  });

  it('retains timeline results only for the current catalog identity', () => {
    const state = new AppState();
    const catalogId = `pca_${'1'.repeat(64)}`;
    state.portfolioStatus = {
      schemaVersion: 1,
      state: 'current',
      libraryRevision: 7,
      catalogId,
      catalogGeneration: 2,
      portfolioId: `gpf_${'2'.repeat(64)}`,
      contributionCount: 1,
      projectCount: 1,
      nodeCount: 0,
      edgeCount: 0,
      reasonCode: 'portfolio-current',
      capabilities: {
        canQuery: true,
        canReconcile: true,
        canRebuild: true,
        canDeleteDerivedState: true
      }
    };
    const result = {
      catalogId,
      queryId: `pty_${'3'.repeat(64)}`
    } as NonNullable<AppState['semanticTimeline']>;

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'semantic-timeline',
      result
    } as AppEvent);
    expect(state.semanticTimeline).toEqual(result);

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'semantic-timeline',
      result: {
        ...result,
        catalogId: `pca_${'4'.repeat(64)}`
      }
    } as AppEvent);
    expect(state.semanticTimeline).toBeNull();
  });

  it('retains portfolio query and doctor results only for current authority', () => {
    const state = new AppState();
    const catalogId = `pca_${'1'.repeat(64)}`;
    state.portfolioStatus = {
      schemaVersion: 1,
      state: 'current',
      libraryRevision: 7,
      catalogId,
      catalogGeneration: 2,
      portfolioId: `gpf_${'2'.repeat(64)}`,
      contributionCount: 1,
      projectCount: 1,
      nodeCount: 0,
      edgeCount: 0,
      reasonCode: 'portfolio-current',
      capabilities: {
        canQuery: true,
        canReconcile: true,
        canRebuild: true,
        canDeleteDerivedState: true
      }
    };

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'portfolio-query',
      result: {
        catalogId: `pca_${'3'.repeat(64)}`
      }
    } as AppEvent);
    expect(state.portfolioQuery).toBeNull();

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'portfolio-doctor',
      doctor: {
        libraryRevision: 8
      }
    } as AppEvent);
    expect(state.portfolioDoctor).toBeNull();
  });

  it('retains native maintenance progress and clears derived views after completion', () => {
    const state = new AppState();
    const operationId = `cop_${'5'.repeat(64)}`;
    state.portfolioStatus = ({ catalogId: `pca_${'6'.repeat(64)}` } as
      NonNullable<AppState['portfolioStatus']>);
    state.portfolioQuery = ({ queryId: `pqy_${'7'.repeat(64)}` } as
      NonNullable<AppState['portfolioQuery']>);

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'continuity-operation-progress',
      progress: {
        schemaVersion: 1,
        operationId,
        operation: 'reconcile',
        phase: 'running',
        completedUnits: 1,
        totalUnits: 2,
        catalogId: `pca_${'6'.repeat(64)}`,
        cancellable: true,
        reasonCode: 'portfolio-operation-running'
      }
    } as AppEvent);
    expect(state.continuityOperationProgress?.operationId).toBe(operationId);

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'portfolio-maintenance-completed',
      result: {
        schemaVersion: 1,
        operationId,
        operation: 'reconcile',
        libraryRevision: 7,
        catalogId: `pca_${'8'.repeat(64)}`,
        portfolioId: `gpf_${'9'.repeat(64)}`,
        catalogChanged: true,
        rebuiltProjectCount: 1,
        reusedProjectCount: 1,
        removedProjectCount: 0,
        removedContributionCount: 0,
        derivedStateOnly: true
      }
    } as AppEvent);

    expect(state.continuityOperationProgress).toBeNull();
    expect(state.portfolioStatus).toBeNull();
    expect(state.portfolioQuery).toBeNull();
    expect(state.portfolioMaintenanceResult?.operationId).toBe(operationId);
    expect(state.notice?.tone).toBe('success');
  });

  it('invalidates process-bound state on every authoritative snapshot event', () => {
    const snapshot = developmentSnapshotFixture();
    const state = new AppState();
    state.snapshot = snapshot;
    state.preview = preview.preview;
    state.captureResolutionPlan = {
      planDigest: 'a'.repeat(64)
    } as NonNullable<AppState['captureResolutionPlan']>;
    state.portfolioStatus = {
      libraryRevision: snapshot.researchLibrary.revision
    } as NonNullable<AppState['portfolioStatus']>;
    state.portfolioQuery = {
      queryId: `pqy_${'b'.repeat(64)}`
    } as NonNullable<AppState['portfolioQuery']>;
    state.continuityOperationProgress = {
      operationId: `cop_${'c'.repeat(64)}`
    } as NonNullable<AppState['continuityOperationProgress']>;

    (state as unknown as { applyEvent(event: AppEvent): void }).applyEvent({
      type: 'snapshot',
      snapshot: { ...snapshot }
    });

    expect(state.preview).toBeNull();
    expect(state.captureResolutionPlan).toBeNull();
    expect(state.portfolioStatus).toBeNull();
    expect(state.portfolioQuery).toBeNull();
    expect(state.continuityOperationProgress).toBeNull();
  });

  it('treats a successful refresh as a possible native-process restart', async () => {
    const snapshot = developmentSnapshotFixture();
    const transport: AppTransport = {
      invoke: async <T>() => snapshot as T
    };
    const state = new AppState(new QiongliAppClient(transport));
    state.snapshot = snapshot;
    state.preview = preview.preview;
    state.portfolioStatus = {
      libraryRevision: snapshot.researchLibrary.revision
    } as NonNullable<AppState['portfolioStatus']>;
    state.semanticTimeline = {
      queryId: `pty_${'d'.repeat(64)}`
    } as NonNullable<AppState['semanticTimeline']>;

    await state.refresh();

    expect(state.snapshot).toEqual(snapshot);
    expect(state.preview).toBeNull();
    expect(state.portfolioStatus).toBeNull();
    expect(state.semanticTimeline).toBeNull();
  });

  it('rejects stale read events before applying them and tracks concurrent loading', async () => {
    const resolvers: Array<(event: AppEvent) => void> = [];
    const transport: AppTransport = {
      invoke: <T>() => new Promise<T>((resolve) => {
        resolvers.push((event) => resolve(event as T));
      })
    };
    const state = new AppState(new QiongliAppClient(transport));
    const first = state.execute(
      { action: 'refresh-integration-discovery' },
      () => false
    );
    const second = state.execute(
      { action: 'refresh-integration-discovery' },
      () => true
    );

    expect(state.loading).toBe(true);
    resolvers[1]!(preview);
    await second;
    expect(state.preview).toEqual(preview.preview);
    expect(state.loading).toBe(true);

    resolvers[0]!(preview);
    expect(await first).toBeNull();
    expect(state.preview).toEqual(preview.preview);
    expect(state.loading).toBe(false);
  });
});
