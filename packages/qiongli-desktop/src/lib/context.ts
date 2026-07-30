import { getContext, setContext } from 'svelte';
import { QiongliAppClient } from '@qiongli/app-api';

import { AppState } from './app-state.svelte';
import { sourceFixtureTransport } from './dev-transport';
import { ProjectWorkspaceState } from './project-workspace.svelte';

const APP_STATE = Symbol('qiongli-app-state');
const PROJECT_WORKSPACE_STATE = Symbol('qiongli-project-workspace-state');

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

export function provideProjectWorkspace(
  state = new ProjectWorkspaceState()
): ProjectWorkspaceState {
  setContext(PROJECT_WORKSPACE_STATE, state);
  return state;
}

export function useProjectWorkspace(): ProjectWorkspaceState {
  return getContext<ProjectWorkspaceState>(PROJECT_WORKSPACE_STATE);
}
