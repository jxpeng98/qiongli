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
});
