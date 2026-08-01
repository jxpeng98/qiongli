import type {
  AppEvent,
  AppIntent,
  AppSnapshot,
  AppTransport
} from '@qiongli/app-api';

export interface AppClient {
  snapshot(): Promise<AppSnapshot>;
  execute(intent: AppIntent): Promise<AppEvent>;
}

export function deferredAppClient(transport?: AppTransport): AppClient {
  let client: Promise<AppClient> | null = null;
  const load = (): Promise<AppClient> => {
    client ??= import('./validated-app-client')
      .then(({ createValidatedAppClient }) => createValidatedAppClient(transport));
    return client;
  };

  return {
    async snapshot(): Promise<AppSnapshot> {
      return (await load()).snapshot();
    },
    async execute(intent: AppIntent): Promise<AppEvent> {
      return (await load()).execute(intent);
    }
  };
}
