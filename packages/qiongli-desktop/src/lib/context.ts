import { getContext, setContext } from 'svelte';
import { QiongliAppClient } from '@qiongli/app-api';

import { AppState } from './app-state.svelte';
import { sourceFixtureTransport } from './dev-transport';

const APP_STATE = Symbol('qiongli-app-state');

export function provideAppState(state = defaultAppState()): AppState {
  setContext(APP_STATE, state);
  return state;
}

function defaultAppState(): AppState {
  const fixtureRequested = import.meta.env.DEV
    && typeof window !== 'undefined'
    && new URLSearchParams(window.location.search).get('fixture') === 'source-read-only';
  return fixtureRequested
    ? new AppState(new QiongliAppClient(sourceFixtureTransport()))
    : new AppState();
}

export function useAppState(): AppState {
  return getContext<AppState>(APP_STATE);
}
