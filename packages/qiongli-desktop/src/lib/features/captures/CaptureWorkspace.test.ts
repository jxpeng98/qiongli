import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import type {
  CaptureAssignmentView,
  CaptureDeliveryView,
  CaptureResolutionPreview
} from '@qiongli/app-api';
import { describe, expect, it, vi } from 'vitest';

import CaptureConflicts from './CaptureConflicts.svelte';
import CaptureOutbox from './CaptureOutbox.svelte';
import CaptureWorkspaceTabs from './CaptureWorkspaceTabs.svelte';

const projectId = 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051';
const envelopeId = `env_${'1'.repeat(64)}`;
const captureId = `cap_${'2'.repeat(64)}`;
const assignmentReceiptId = `car_${'3'.repeat(64)}`;
const resolutionItemId = `cri_${'4'.repeat(64)}`;

const delivery = {
  schemaVersion: 1,
  envelopeId,
  captureId,
  source: 'codex',
  delivery: 'connected',
  destination: null,
  state: 'conflicted',
  generation: 2,
  attemptCount: 1,
  retryCount: 0,
  createdAtUnix: 1_784_476_800,
  updatedAtUnix: 1_784_563_100,
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

const assignment = {
  schemaVersion: 1,
  state: 'completed',
  intentId: `cai_${'5'.repeat(64)}`,
  sourceEnvelopeId: envelopeId,
  sourceCaptureId: captureId,
  targetProjectId: projectId,
  targetProjectRevision: 12,
  outcome: 'assigned',
  receiptId: assignmentReceiptId,
  derivedCaptureId: `cap_${'6'.repeat(64)}`,
  childEnvelopeId: `env_${'7'.repeat(64)}`,
  createdAtUnix: 1_784_563_100,
  decidedAtUnix: 1_784_563_200,
  canResolve: true
} satisfies CaptureAssignmentView;

const plan = {
  schemaVersion: 1,
  planDigest: '8'.repeat(64),
  assignmentReceiptId,
  sourceEnvelopeId: envelopeId,
  sourceCaptureId: captureId,
  derivedCaptureId: assignment.derivedCaptureId,
  childEnvelopeId: assignment.childEnvelopeId,
  targetProjectId: projectId,
  expectedLibraryRevision: 7,
  expectedProjectRevision: 12,
  nextProjectRevision: 13,
  reviewedAtUnix: 1_784_563_300,
  items: [{
    itemId: resolutionItemId,
    kind: 'semantic-change',
    counterpartState: 'exact-identity-divergent',
    allowedDispositions: ['accept-current', 'accept-capture'],
    unavailableDispositions: ['retain-both', 'reject-capture'],
    sourceSummary: 'Use the reviewed capture meaning.',
    currentSummary: 'Keep the current project meaning.',
    explanation: 'The exact academic identity differs.'
  }],
  approvalsRequired: ['academic-review', 'filesystem-write'],
  exactReplay: false
} satisfies CaptureResolutionPreview;

describe('Capture workspace controls', () => {
  it('supports roving keyboard navigation across semantic tabs', async () => {
    const onChange = vi.fn();
    render(CaptureWorkspaceTabs, {
      mode: 'inbox',
      counts: { inbox: 1, outbox: 2, conflicts: 3, coverage: 4 },
      onChange
    });

    const inbox = screen.getByRole('tab', { name: /Inbox/ });
    const outbox = screen.getByRole('tab', { name: /Outbox/ });
    inbox.focus();
    await fireEvent.keyDown(inbox, { key: 'ArrowRight' });

    expect(onChange).toHaveBeenCalledWith('outbox');
    expect(outbox).toHaveFocus();
  });

  it('requires an explicit retry cause and a second cancellation action', async () => {
    const onRetry = vi.fn();
    const onCancel = vi.fn();
    render(CaptureOutbox, {
      entries: [delivery],
      currentProjectRevision: 12,
      selectedEnvelopeId: null,
      loading: false,
      truncated: false,
      onInspect: vi.fn(),
      onRetry,
      onCancel,
      onAcknowledge: vi.fn(),
      onLoadMore: vi.fn()
    });

    const retry = screen.getByRole('button', { name: 'Retry delivery' });
    expect(retry).toBeDisabled();
    await fireEvent.change(screen.getByLabelText('Retry cause'), {
      target: { value: 'transport-unavailable' }
    });
    expect(retry).toBeEnabled();
    await fireEvent.click(retry);
    expect(onRetry).toHaveBeenCalledWith(delivery, 'transport-unavailable');

    const cancelTrigger = screen.getByRole('button', { name: 'Cancel delivery' });
    await fireEvent.click(cancelTrigger);
    expect(onCancel).not.toHaveBeenCalled();
    let confirmation = screen.getByRole('alertdialog', {
      name: 'Cancel this exact delivery generation?'
    });
    await waitFor(() => expect(confirmation).toHaveFocus());
    await fireEvent.keyDown(confirmation, { key: 'Escape' });
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Cancel delivery' })).toHaveFocus();
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Cancel delivery' }));
    confirmation = screen.getByRole('alertdialog', {
      name: 'Cancel this exact delivery generation?'
    });
    await fireEvent.click(within(confirmation).getByRole('button', { name: 'Cancel delivery' }));
    expect(onCancel).toHaveBeenCalledWith(delivery);
  });

  it('requires an explicit project and one allowed choice for every resolution item', async () => {
    const onPreviewAssignment = vi.fn();
    const onPreviewResolution = vi.fn();
    render(CaptureConflicts, {
      deliveries: [delivery],
      assignments: [assignment],
      resolutions: [],
      projects: [{ projectId, displayName: 'Trustworthy research agents' }],
      plan,
      loading: false,
      assignmentsTruncated: false,
      resolutionsTruncated: false,
      onPreviewAssignment,
      onInspectAssignment: vi.fn(),
      onLoadResolutionPlan: vi.fn(),
      onPreviewResolution,
      onInspectResolution: vi.fn(),
      onLoadMoreAssignments: vi.fn(),
      onLoadMoreResolutions: vi.fn()
    });

    const reviewAssignment = screen.getByRole('button', { name: 'Review assignment' });
    expect(reviewAssignment).toBeDisabled();
    await fireEvent.change(screen.getByLabelText('Target project'), {
      target: { value: projectId }
    });
    await fireEvent.click(reviewAssignment);
    expect(onPreviewAssignment).toHaveBeenCalledWith(delivery, projectId, 'assign');

    await fireEvent.click(screen.getByRole('button', { name: 'Resolve academic meaning' }));
    const reviewResolution = screen.getByRole('button', {
      name: 'Review complete resolution'
    });
    expect(reviewResolution).toBeDisabled();
    await fireEvent.change(screen.getByLabelText('Disposition'), {
      target: { value: 'accept-capture' }
    });
    expect(reviewResolution).toBeEnabled();
    await fireEvent.click(reviewResolution);
    expect(onPreviewResolution).toHaveBeenCalledWith(assignment, [{
      itemId: resolutionItemId,
      disposition: 'accept-capture'
    }]);
  });
});
