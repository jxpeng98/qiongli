import type { AppSnapshot } from '@qiongli/app-api';
import type { FeatureDescriptor } from '../types';

export const overviewFeature: FeatureDescriptor = {
  id: 'overview',
  label: 'Overview',
  route: '/overview',
  stage: 'available'
};

export function readyAreaCount(snapshot: AppSnapshot): number {
  const core = [snapshot.content.status, snapshot.configuration.status, snapshot.mcp.status]
    .filter((status) => status === 'ready').length;
  return core + snapshot.integrations.filter((integration) => integration.connection.state === 'connected').length;
}
