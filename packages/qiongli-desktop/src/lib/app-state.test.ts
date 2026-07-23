import { describe, expect, it } from 'vitest';

import { QiongliAppClient, type AppEvent, type AppTransport } from '@qiongli/app-api';

import { AppState } from './app-state.svelte';

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
    availableVersion: '2.0.0-alpha.2',
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

  it('stores a bounded agent result and clears its confirmation preview', async () => {
    const runPreview: AppEvent = {
      type: 'preview',
      preview: {
        ...preview.preview,
        kind: 'agent-run',
        title: 'Run project query with OpenAI',
        approvalsRequired: ['Send prompt and redacted project data to OpenAI']
      }
    };
    const completed: AppEvent = {
      type: 'agent-run-completed',
      result: {
        schemaVersion: 1,
        runId: `run_${'1'.repeat(32)}`,
        backendId: 'openai-responses',
        model: 'gpt-5.6-sol',
        finishReason: 'stop',
        content: 'The evidence position remains provisional.',
        inputTokens: 20,
        outputTokens: 6,
        cachedInputTokens: 0,
        modelTurns: 2,
        toolCalls: 1,
        networkRequests: 2,
        auditedToolCalls: 1
      }
    };
    const events = [runPreview, completed];
    const transport: AppTransport = {
      invoke: async <T>() => events.shift() as T
    };
    const state = new AppState(new QiongliAppClient(transport));

    await state.execute({ action: 'refresh-integration-discovery' });
    await state.execute({ action: 'confirm-operation', token: runPreview.preview.token });

    expect(state.preview).toBeNull();
    expect(state.agentRun?.content).toContain('provisional');
    expect(state.notice?.title).toBe('Project query completed');
  });

  it('stores orchestration checkpoints, output, and control updates', async () => {
    const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
    const run = {
      runId: `run_${'2'.repeat(32)}`,
      profileId: 'openai-solo-v1',
      executionMode: 'solo' as const,
      status: 'running' as const,
      generation: 3,
      documentSha256: '3'.repeat(64),
      completedTaskCount: 1,
      totalTaskCount: 76 as const,
      nextTaskId: 'A1_5',
      activeTaskId: null,
      recoveryRequired: false,
      canContinue: true,
      canPause: true,
      canResume: false,
      canRecover: false,
      canCancel: true
    };
    const doctor = {
      schemaVersion: 1 as const,
      projectId,
      expectedProjectRevision: 12,
      workflowContractStatus: 'ready' as const,
      backendReadiness: 'ready' as const,
      runCount: 1,
      activeRunCount: 1,
      recoveryRequiredCount: 0,
      runnable: false,
      reasonCodes: ['orchestration-active-run-exists']
    };
    const runs = {
      schemaVersion: 1 as const,
      projectId,
      expectedProjectRevision: 12,
      runs: [run]
    };
    const events: AppEvent[] = [
      { type: 'orchestration-loaded', doctor, runs },
      {
        type: 'preview',
        preview: {
          ...preview.preview,
          kind: 'orchestration-continue',
          title: 'Continue orchestration run',
          approvalsRequired: ['network-request']
        }
      },
      {
        type: 'orchestration-executed',
        doctor,
        runs,
        execution: {
          schemaVersion: 1,
          outcome: 'task-completed',
          taskId: 'A1',
          run,
          roleOutputs: [{
            taskId: 'A1',
            role: 'primary',
            outputSha256: '4'.repeat(64),
            model: 'gpt-5.6-sol',
            finishReason: 'stop',
            content: 'Bounded orchestration output.',
            modelTurns: 1,
            toolCalls: 0,
            networkRequests: 1
          }]
        }
      },
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
        doctor: {
          ...doctor,
          runnable: false
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
    await state.execute({ action: 'confirm-operation', token: preview.preview.token });
    await state.execute({ action: 'refresh-integration-discovery' });

    expect(state.preview).toBeNull();
    expect(state.orchestrationExecution?.roleOutputs[0]?.content).toContain('Bounded');
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
});
