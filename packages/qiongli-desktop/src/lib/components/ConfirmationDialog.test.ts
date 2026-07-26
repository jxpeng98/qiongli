import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type {
  CaptureConsolidationPreview,
  CaptureResolutionPreview,
  OperationPreview
} from '@qiongli/app-api';

import { i18n } from '$lib/i18n.svelte';
import ConfirmationDialog from './ConfirmationDialog.svelte';

const blockedPreview = {
  token: '00000000000000000000000000000001',
  kind: 'activation',
  title: 'Qiongli plugin preview',
  summary: 'Review the selected Qiongli content before applying it.',
  displayTarget: null,
  planDigestSha256: null,
  approvalsRequired: [],
  canConfirm: false,
  blockedReason: 'source-build-read-only'
} satisfies OperationPreview;

const conflictedConsolidation = {
  schemaVersion: 1,
  planDigest: '1'.repeat(64),
  captureId: `cap_${'a'.repeat(64)}`,
  projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
  disposition: 'contradiction',
  outcome: 'conflicted',
  expectedLibraryRevision: 7,
  expectedProjectRevision: 12,
  nextProjectRevision: null,
  projectStage: 'writing',
  reviewedAtUnix: 12,
  conflicts: [{
    kind: 'contradiction-requires-resolution',
    artifact: null,
    resolution: 'resolve-contradiction-before-consolidation'
  }],
  artifactDeltas: [],
  receiptEntry: 'history/consolidations/capture.json',
  approvalsRequired: []
} satisfies CaptureConsolidationPreview;

const resolutionPreview = {
  schemaVersion: 1,
  planDigest: '6'.repeat(64),
  assignmentReceiptId: `car_${'5'.repeat(64)}`,
  sourceEnvelopeId: `env_${'1'.repeat(64)}`,
  sourceCaptureId: `cap_${'2'.repeat(64)}`,
  derivedCaptureId: `cap_${'3'.repeat(64)}`,
  childEnvelopeId: `env_${'4'.repeat(64)}`,
  targetProjectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
  expectedLibraryRevision: 7,
  expectedProjectRevision: 12,
  nextProjectRevision: 13,
  reviewedAtUnix: 1784563300,
  items: [{
    itemId: `cri_${'6'.repeat(64)}`,
    kind: 'semantic-change',
    counterpartState: 'exact-identity-divergent',
    allowedDispositions: ['accept-current', 'accept-capture'],
    unavailableDispositions: ['retain-both', 'reject-capture'],
    sourceSummary: 'Use the accepted capture wording.',
    currentSummary: 'Keep the current project wording.',
    explanation: 'The same academic identity has divergent reviewed content.'
  }],
  approvalsRequired: ['academic-review', 'filesystem-write'],
  exactReplay: false
} satisfies CaptureResolutionPreview;

describe('ConfirmationDialog', () => {
  it('names the source authority block and disables confirmation', () => {
    render(ConfirmationDialog, {
      preview: blockedPreview,
      busy: false,
      onConfirm: vi.fn(),
      onCancel: vi.fn()
    });

    expect(screen.getByRole('dialog')).toHaveAccessibleName('Qiongli plugin preview');
    expect(screen.getByRole('alert')).toHaveTextContent('source-build-read-only');
    expect(screen.getByRole('button', { name: 'Confirm changes' })).toBeDisabled();
  });

  it('exposes a keyboard-addressable cancel action', async () => {
    const onCancel = vi.fn();
    render(ConfirmationDialog, {
      preview: blockedPreview,
      busy: false,
      onConfirm: vi.fn(),
      onCancel
    });

    const cancel = screen.getByRole('button', { name: 'Cancel' });
    cancel.focus();
    expect(cancel).toHaveFocus();
    await fireEvent.click(cancel);
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('cannot be dismissed while native confirmation is in progress', async () => {
    const onCancel = vi.fn();
    render(ConfirmationDialog, {
      preview: blockedPreview,
      busy: true,
      onConfirm: vi.fn(),
      onCancel
    });

    expect(screen.getByRole('dialog')).toHaveAttribute('aria-busy', 'true');
    expect(screen.getByRole('button', { name: 'Cancel operation' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();

    await fireEvent.keyDown(document, { key: 'Escape' });
    const overlay = document.querySelector('.overlay');
    expect(overlay).not.toBeNull();
    if (overlay) {
      await fireEvent.pointerDown(overlay);
      await fireEvent.pointerUp(overlay);
    }

    expect(onCancel).not.toHaveBeenCalled();
  });

  it('does not present a legacy orchestration preview as model execution', () => {
    render(ConfirmationDialog, {
      preview: {
        ...blockedPreview,
        kind: 'orchestration-continue',
        title: 'Continue orchestration run',
        canConfirm: true,
        blockedReason: null,
        approvalsRequired: ['network-request']
      },
      busy: false,
      onConfirm: vi.fn(),
      onCancel: vi.fn()
    });

    expect(screen.getByRole('button', { name: 'Confirm changes' })).toBeEnabled();
    expect(screen.queryByRole('button', { name: 'Confirm and run' })).not.toBeInTheDocument();
  });

  it('shows academic conflicts inside the confirmation boundary', () => {
    render(ConfirmationDialog, {
      preview: {
        ...blockedPreview,
        kind: 'capture-consolidation',
        title: 'Consolidate reviewed capture',
        blockedReason: 'academic-review-conflict'
      },
      consolidation: conflictedConsolidation,
      busy: false,
      onConfirm: vi.fn(),
      onCancel: vi.fn()
    });

    expect(screen.getByRole('region', { name: 'Academic consolidation review' }))
      .toHaveTextContent('contradiction-requires-resolution');
    expect(screen.getByRole('alert')).toHaveTextContent('academic-review-conflict');
    expect(screen.getByRole('button', { name: 'Confirm changes' })).toBeDisabled();
  });

  it('localizes structured project migration facts without losing counts', () => {
    i18n.locale = 'zh-CN';
    try {
      render(ConfirmationDialog, {
        preview: {
          ...blockedPreview,
          kind: 'project-migration',
          title: 'Migrate Qiongli 1.x article project',
          summary: 'Native fallback summary',
          canConfirm: true,
          blockedReason: null,
          approvalsRequired: ['filesystem-write'],
          migration: {
            mode: 'copy',
            copiedFileCount: 12,
            copiedBytes: 48_320,
            excludedEntryCount: 3,
            sourceRetained: true,
            copiesFiles: true,
            graphRebuildPasses: 2
          }
        },
        busy: false,
        onConfirm: vi.fn(),
        onCancel: vi.fn()
      });

      expect(screen.getByRole('dialog')).toHaveAccessibleName('迁移穷理 1.x 项目');
      expect(screen.getByRole('dialog')).toHaveTextContent('12 个已验证学术文件');
      expect(screen.getByRole('dialog')).toHaveTextContent('48,320 字节');
      expect(screen.getByRole('dialog')).toHaveTextContent('写入文件系统');
    } finally {
      i18n.locale = 'en';
    }
  });

  it('shows item-scoped rollback reconciliation and a localized drift block', () => {
    i18n.locale = 'zh-CN';
    try {
      render(ConfirmationDialog, {
        preview: {
          ...blockedPreview,
          kind: 'project-migration-rollback',
          title: 'Native fallback title',
          summary: 'Native fallback summary',
          blockedReason: 'project-migration-rollback-destination-drift',
          migrationRollback: {
            registrationState: 'registered',
            markerState: 'ready',
            reconciliation: {
              status: 'drifted',
              matchedArtifactCount: 4,
              driftedArtifactCount: 1,
              continuityGapCount: 2,
              artifacts: [{
                category: 'research-state',
                relativePath: 'context/research_state.md',
                state: 'changed'
              }]
            },
            sourceRetained: true,
            destinationRemoval: 'migration-owned-destination',
            canRollback: false
          }
        },
        busy: false,
        onConfirm: vi.fn(),
        onCancel: vi.fn()
      });

      expect(screen.getByRole('dialog')).toHaveAccessibleName('回滚迁移后的穷理 2 副本');
      expect(screen.getByRole('region', { name: '迁移回滚对账' }))
        .toHaveTextContent('context/research_state.md');
      expect(screen.getByRole('alert')).toHaveTextContent('请先导出或明确处理目标目录');
      expect(screen.getByRole('button', { name: '确认变更' })).toBeDisabled();
    } finally {
      i18n.locale = 'en';
    }
  });

  it('shows every selected academic disposition inside confirmation', () => {
    render(ConfirmationDialog, {
      preview: {
        ...blockedPreview,
        kind: 'capture-resolution',
        title: 'Resolve capture items',
        planDigestSha256: resolutionPreview.planDigest,
        canConfirm: true,
        blockedReason: null,
        approvalsRequired: ['academic-review', 'filesystem-write']
      },
      resolution: resolutionPreview,
      resolutionSelections: [{
        itemId: resolutionPreview.items[0].itemId,
        disposition: 'accept-capture'
      }],
      busy: false,
      onConfirm: vi.fn(),
      onCancel: vi.fn()
    });

    const review = screen.getByRole('region', { name: 'Academic resolution review' });
    expect(review).toHaveTextContent('Use the accepted capture wording.');
    expect(review).toHaveTextContent('Keep the current project wording.');
    expect(review).toHaveTextContent('Accept capture');
    expect(screen.getByRole('button', { name: 'Confirm changes' })).toBeEnabled();
  });
});
