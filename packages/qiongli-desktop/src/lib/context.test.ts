import {
  type AppEvent,
  type AppSnapshot
} from '@qiongli/app-api';
import { describe, expect, it } from 'vitest';

import { deferredSourceFixtureTransport } from './context';

describe('deferred development fixture transport', () => {
  it('loads the source fixture only when the development transport is invoked', async () => {
    const transport = deferredSourceFixtureTransport();
    const snapshot = await transport.invoke<AppSnapshot>('qiongli_snapshot');
    const event = await transport.invoke<AppEvent>('qiongli_execute', {
      intent: { action: 'refresh-integration-discovery' }
    });

    expect(snapshot.product.trust.mode).toBe('source-read-only');
    expect(event.type).toBe('snapshot');
  });
});
