import type {
  AppSnapshot,
  IntegrationSelection,
  IntegrationTarget,
  StatusCode
} from '@qiongli/app-api';
import type { FeatureDescriptor } from '../types';

type Integration = AppSnapshot['integrations'][number];
export type IntegrationSetupStage =
  | 'install'
  | 'repair'
  | 'blocked'
  | 'activate'
  | 'verify'
  | 'ready';

export const clientIntegrationsFeature: FeatureDescriptor = {
  id: 'client-integrations',
  label: 'Client Integrations',
  route: '/client-integrations',
  stage: 'available'
};

export function integrationForTarget(
  snapshot: AppSnapshot | null,
  target: IntegrationTarget
): Integration | null {
  return snapshot?.integrations.find((integration) => integration.target === target) ?? null;
}

export function integrationTabTarget(
  active: IntegrationTarget,
  key: string
): IntegrationTarget | null {
  if (key === 'Home') return 'codex';
  if (key === 'End') return 'claude-code';
  if (key === 'ArrowLeft' || key === 'ArrowRight') {
    return active === 'codex' ? 'claude-code' : 'codex';
  }
  return null;
}

export function integrationEligible(integration: Integration): boolean {
  if (!integration.client.detected
    || integration.connection.state === 'inspection-blocked'
    || integration.nextAction === 'resolve-conflict'
    || integration.nextAction === 'unavailable') {
    return false;
  }
  if (integration.nextAction === 'upgrade-client') {
    return integration.ownershipState === 'qiongli-managed';
  }
  return true;
}

export function integrationSelectionDisabled(
  integration: Integration,
  operationInProgress: boolean
): boolean {
  return operationInProgress || !integrationEligible(integration);
}

export function integrationBatchActions(
  snapshot: AppSnapshot | null,
  selection: IntegrationSelection
): {
  verify: boolean;
  install: boolean;
  reconcile: boolean;
  remove: boolean;
} {
  const integrations = snapshot?.integrations ?? [];
  const selectedIntegrations = integrations.filter((integration) =>
    integration.target === 'codex' ? selection.codex : selection.claudeCode
  );
  const selected = selectedIntegrations.length > 0;
  const canApply = snapshot?.capabilities.apply === true;
  const everySelected = (predicate: (integration: Integration) => boolean): boolean =>
    selected && selectedIntegrations.every(predicate);
  const anySelected = (predicate: (integration: Integration) => boolean): boolean =>
    selectedIntegrations.some(predicate);

  return {
    verify: everySelected(integrationEligible),
    install: canApply
      && everySelected((integration) =>
        ['install-ready', 'current', 'repair-ready'].includes(integration.nextAction)
      )
      && anySelected((integration) => integration.nextAction === 'install-ready'),
    reconcile: canApply
      && everySelected((integration) =>
        ['current', 'repair-ready'].includes(integration.nextAction)
      )
      && anySelected((integration) => integration.nextAction === 'repair-ready'),
    remove: canApply
      && everySelected((integration) =>
        integration.ownershipState === 'qiongli-managed'
        && ['current', 'repair-ready', 'upgrade-client'].includes(integration.nextAction)
      )
  };
}

export function connectionStatus(state: Integration['connection']['state']): StatusCode {
  switch (state) {
    case 'connected': return 'ready';
    case 'activated': return 'attention';
    case 'prepared': return 'attention';
    case 'installed-host-action-required': return 'attention';
    case 'detected-not-connected': return 'attention';
    case 'needs-repair': return 'drifted';
    case 'inspection-blocked': return 'unavailable';
    case 'unsupported-client-version': return 'blocked';
    case 'client-not-detected': return 'missing';
  }
}

export function hostIntegrationSkillsStatus(integration: Integration): StatusCode {
  const { source, skills } = integration.managedContent;
  if (source === 'ready') return skills;
  if (source === 'missing' && skills === 'ready') return 'attention';
  if (source === 'missing') return skills;
  return source;
}

export function hostIntegrationSkillsDetached(integration: Integration): boolean {
  return integration.managedContent.source === 'missing'
    && integration.managedContent.skills === 'ready';
}

export function integrationSetupStage(integration: Integration): IntegrationSetupStage {
  if (integration.nextAction === 'install-ready') return 'install';
  if (integration.nextAction === 'repair-ready') return 'repair';
  if (['inspect-only', 'upgrade-client', 'resolve-conflict', 'unavailable']
    .includes(integration.nextAction)) return 'blocked';
  if (integration.managedContent.activationObservation === 'client-action-required') {
    return 'activate';
  }
  if (integration.managedContent.activationObservation === 'observed') return 'ready';
  return 'verify';
}

export function integrationActivationCommand(target: IntegrationTarget): string {
  return target === 'codex'
    ? 'codex plugin add qiongli-next@personal'
    : 'claude plugin install qiongli-next@qiongli-local';
}
