import { invoke } from '@tauri-apps/api/core';

import {
  appEventSchema,
  appIntentSchema,
  appSnapshotSchema,
  type AppEvent,
  type AppIntent,
  type AppSnapshot
} from './schema';

export interface AppTransport {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

const tauriTransport: AppTransport = {
  invoke: (command, args) => invoke(command, args)
};

export class QiongliAppClient {
  constructor(private readonly transport: AppTransport = tauriTransport) {}

  async snapshot(): Promise<AppSnapshot> {
    const value = await this.transport.invoke<unknown>('qiongli_snapshot');
    return appSnapshotSchema.parse(value);
  }

  async execute(intent: AppIntent): Promise<AppEvent> {
    const boundedIntent = appIntentSchema.parse(intent);
    const value = await this.transport.invoke<unknown>('qiongli_execute', {
      intent: boundedIntent
    });
    return appEventSchema.parse(value);
  }
}
