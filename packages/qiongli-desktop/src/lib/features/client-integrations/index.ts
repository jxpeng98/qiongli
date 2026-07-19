import type { AppSnapshot, StatusCode } from '@qiongli/app-api';
import type { FeatureDescriptor } from '../types';

type Integration = AppSnapshot['integrations'][number];

export const clientIntegrationsFeature: FeatureDescriptor = {
  id: 'client-integrations',
  label: 'Client Integrations',
  route: '/client-integrations',
  stage: 'available'
};

export function integrationEligible(integration: Integration): boolean {
  return integration.client.detected
    && integration.client.compatibility !== 'unsupported'
    && integration.connection.state !== 'inspection-blocked';
}

export function connectionStatus(state: Integration['connection']['state']): StatusCode {
  switch (state) {
    case 'connected': return 'ready';
    case 'detected-not-connected': return 'attention';
    case 'needs-repair': return 'drifted';
    case 'inspection-blocked': return 'unavailable';
    case 'unsupported-client-version': return 'blocked';
    case 'client-not-detected': return 'missing';
  }
}
