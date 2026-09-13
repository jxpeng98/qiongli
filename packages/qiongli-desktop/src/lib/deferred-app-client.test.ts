import type { AppTransport } from '@qiongli/app-api';
import { describe, expect, it, vi } from 'vitest';

import { sourceFixtureTransport } from './dev-transport';
import { deferredAppClient } from './deferred-app-client';

describe('deferred validated app client', () => {
  it('loads the validated client on first use and preserves the transport contract', async () => {
    const client = deferredAppClient(sourceFixtureTransport());

    const snapshot = await client.snapshot();
    const event = await client.execute({
      action: 'refresh-integration-discovery'
    });

    expect(snapshot.product.trust.mode).toBe('source-read-only');
    expect(event.type).toBe('snapshot');
  });

  it('rejects invalid intents before invoking the native transport', async () => {
    const invoke = vi.fn();
    const transport: AppTransport = {
      invoke
    };
    const client = deferredAppClient(transport);

    await expect(client.execute({ action: 'invalid-action' } as never))
      .rejects.toBeDefined();
    expect(invoke).not.toHaveBeenCalled();
  });
});
