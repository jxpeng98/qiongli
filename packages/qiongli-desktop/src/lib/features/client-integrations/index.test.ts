import { describe, expect, it } from 'vitest';

import { developmentSnapshotFixture } from '$lib/dev-transport';
import {
  connectionStatus,
  integrationBatchActions,
  integrationActivationCommands,
  integrationForTarget,
  integrationSetupStage,
  hostIntegrationSkillsDetached,
  hostIntegrationSkillsStatus,
  integrationSelectionDisabled,
  integrationTabTarget
} from './index';

describe('integrationBatchActions', () => {
  it('reserves the ready highlight for a positively observed connection', () => {
    expect(connectionStatus('connected')).toBe('ready');
    expect(connectionStatus('activated')).toBe('attention');
    expect(connectionStatus('prepared')).toBe('attention');
    expect(connectionStatus('installed-host-action-required')).toBe('attention');
    expect(connectionStatus('inspection-blocked')).toBe('unavailable');
  });

  it('resolves clients by stable target identity instead of array position', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.integrations = [...snapshot.integrations].reverse();

    expect(integrationForTarget(snapshot, 'codex')?.target).toBe('codex');
    expect(integrationForTarget(snapshot, 'claude-code')?.target).toBe('claude-code');
  });

  it('supports the complete horizontal tab keyboard contract', () => {
    expect(integrationTabTarget('codex', 'ArrowRight')).toBe('claude-code');
    expect(integrationTabTarget('claude-code', 'ArrowLeft')).toBe('codex');
    expect(integrationTabTarget('claude-code', 'Home')).toBe('codex');
    expect(integrationTabTarget('codex', 'End')).toBe('claude-code');
    expect(integrationTabTarget('codex', 'Enter')).toBeNull();
  });

  it('does not report detached or legacy Skills as an installed Host Integration component', () => {
    const snapshot = developmentSnapshotFixture();
    const codex = snapshot.integrations[0];
    const claude = snapshot.integrations[1];

    expect(codex.managedContent).toMatchObject({ source: 'missing', skills: 'ready' });
    expect(hostIntegrationSkillsStatus(codex)).toBe('attention');
    expect(hostIntegrationSkillsDetached(codex)).toBe(true);
    expect(hostIntegrationSkillsStatus(claude)).toBe('missing');
    expect(hostIntegrationSkillsDetached(claude)).toBe(false);

    codex.managedContent.source = 'ready';
    expect(hostIntegrationSkillsStatus(codex)).toBe('ready');
    expect(hostIntegrationSkillsDetached(codex)).toBe(false);
  });

  it('locks the selected client scope while an operation is in progress', () => {
    const snapshot = developmentSnapshotFixture();
    const integration = snapshot.integrations[0];

    expect(integrationSelectionDisabled(integration, false)).toBe(false);
    expect(integrationSelectionDisabled(integration, true)).toBe(true);
  });

  it('keeps verification available but blocks client mutations in a read-only source build', () => {
    const snapshot = developmentSnapshotFixture();

    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: true })).toEqual({
      verify: true,
      install: false,
      reconcile: false,
      remove: false
    });
  });

  it('enables receipt-owned client mutations only with packaged apply authority', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    snapshot.product.trust = {
      mode: 'packaged-product',
      label: 'Verified packaged product',
      canApply: true,
      reasonCode: 'verified-packaged-product-control'
    };

    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: false })).toEqual({
      verify: true,
      install: true,
      reconcile: false,
      remove: false
    });
  });

  it('requires an explicit client selection for target-scoped actions', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;

    expect(integrationBatchActions(snapshot, { codex: false, claudeCode: false })).toEqual({
      verify: false,
      install: false,
      reconcile: false,
      remove: false
    });
  });

  it('exposes one scoped reconcile action and removal for receipt-owned repair states', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    Object.assign(snapshot.integrations[0], {
      nextAction: 'repair-ready',
      ownership: 'Qiongli managed',
      ownershipState: 'qiongli-managed'
    });

    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: false })).toEqual({
      verify: true,
      install: false,
      reconcile: true,
      remove: true
    });
  });

  it('does not offer generic mutations for a conflict that requires explicit resolution', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    Object.assign(snapshot.integrations[0], {
      nextAction: 'resolve-conflict',
      ownership: 'Mixed ownership',
      ownershipState: 'mixed'
    });

    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: false })).toEqual({
      verify: false,
      install: false,
      reconcile: false,
      remove: false
    });
  });

  it('keeps verify and receipt-owned removal available while an installed client needs upgrade', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    Object.assign(snapshot.integrations[0], {
      connection: {
        state: 'unsupported-client-version',
        label: 'Unsupported client version',
        reasonCode: 'client-version-below-supported-minimum'
      },
      client: {
        ...snapshot.integrations[0].client,
        compatibility: 'unsupported'
      },
      overall: 'blocked',
      ownership: 'Qiongli managed',
      ownershipState: 'qiongli-managed',
      nextAction: 'upgrade-client'
    });

    expect(integrationSelectionDisabled(snapshot.integrations[0], false)).toBe(false);
    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: false })).toEqual({
      verify: true,
      install: false,
      reconcile: false,
      remove: true
    });
  });

  it('blocks Qiongli mutations for an unsupported client with no managed install', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    Object.assign(snapshot.integrations[0], {
      connection: {
        state: 'unsupported-client-version',
        label: 'Unsupported client version',
        reasonCode: 'client-version-below-supported-minimum'
      },
      client: {
        ...snapshot.integrations[0].client,
        compatibility: 'unsupported'
      },
      overall: 'blocked',
      ownership: 'Not installed',
      ownershipState: 'not-installed',
      nextAction: 'upgrade-client'
    });

    expect(integrationSelectionDisabled(snapshot.integrations[0], false)).toBe(true);
    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: false })).toEqual({
      verify: false,
      install: false,
      reconcile: false,
      remove: false
    });
  });

  it('allows a batch install to preserve already-current selected targets', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    Object.assign(snapshot.integrations[1], {
      nextAction: 'current',
      ownership: 'Qiongli managed',
      ownershipState: 'qiongli-managed'
    });

    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: true })).toEqual({
      verify: true,
      install: true,
      reconcile: false,
      remove: false
    });
  });

  it('turns native registration evidence into an explicit client activation checklist', () => {
    const snapshot = developmentSnapshotFixture();
    const codex = snapshot.integrations[0];

    expect(integrationSetupStage(codex)).toBe('install');
    expect(integrationActivationCommands(codex))
      .toEqual(['codex plugin add --json qiongli-next@personal']);
    expect(integrationActivationCommands(snapshot.integrations[1]))
      .toEqual([
        'claude plugin marketplace add $HOME/.qiongli/plugins/claude-code/qiongli-local --scope user',
        'claude plugin install qiongli-next@qiongli-local --scope user'
      ]);

    Object.assign(codex, { nextAction: 'current' });
    Object.assign(codex.managedContent, {
      registration: 'ready',
      activation: 'attention',
      activationObservation: 'client-action-required'
    });
    expect(integrationSetupStage(codex)).toBe('activate');

    codex.managedContent.activationObservation = 'not-observable';
    expect(integrationSetupStage(codex)).toBe('verify');

    codex.managedContent.activationObservation = 'observed';
    expect(integrationSetupStage(codex)).toBe('ready');

    codex.nextAction = 'repair-ready';
    expect(integrationSetupStage(codex)).toBe('repair');

    codex.nextAction = 'upgrade-client';
    expect(integrationSetupStage(codex)).toBe('blocked');
  });

  it('keeps a mixed missing and repair batch in the native install transaction', () => {
    const snapshot = developmentSnapshotFixture();
    snapshot.capabilities.apply = true;
    Object.assign(snapshot.integrations[1], {
      nextAction: 'repair-ready',
      ownership: 'Qiongli managed',
      ownershipState: 'qiongli-managed'
    });

    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: false }).reconcile)
      .toBe(false);
    expect(integrationBatchActions(snapshot, { codex: true, claudeCode: true })).toMatchObject({
      install: true,
      reconcile: false
    });
    expect(integrationBatchActions(snapshot, { codex: false, claudeCode: true }).reconcile)
      .toBe(true);
  });
});
