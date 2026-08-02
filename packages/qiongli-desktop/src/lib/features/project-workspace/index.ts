import type { ArticleProjectSummary } from '@qiongli/app-api';

export type ProjectWorkspaceDestination =
  | 'overview'
  | 'artifacts'
  | 'captures'
  | 'academic-graph'
  | 'timeline'
  | 'run-in-client';

export interface ProjectWorkspaceNavigationItem {
  id: ProjectWorkspaceDestination;
  href: string;
  labelKey: string;
}

export type ProjectWorkspaceProject = Pick<
  ArticleProjectSummary,
  'projectId' | 'lifecycle' | 'health'
>;

export const projectWorkspaceNavigation: readonly ProjectWorkspaceNavigationItem[] = [
  { id: 'overview', href: '/research-library', labelKey: 'projectWorkspace.overview' },
  { id: 'artifacts', href: '/artifacts', labelKey: 'projectWorkspace.artifacts' },
  { id: 'captures', href: '/captures', labelKey: 'projectWorkspace.captures' },
  { id: 'academic-graph', href: '/academic-graph', labelKey: 'projectWorkspace.graph' },
  { id: 'timeline', href: '/timeline', labelKey: 'projectWorkspace.timeline' },
  { id: 'run-in-client', href: '/orchestrator', labelKey: 'projectWorkspace.run' }
] as const;

const projectWorkspaceRoutes = new Set(
  projectWorkspaceNavigation.map((item) => item.href)
);

const projectWorkspaceSharedSearchParameters = new Set(['fixture']);

export function isProjectWorkspaceRoute(pathname: string): boolean {
  return projectWorkspaceRoutes.has(normalizePathname(pathname));
}

export function projectWorkspaceHref(
  pathname: string,
  projectId: string,
  currentSearch = ''
): string {
  const current = new URLSearchParams(
    currentSearch.startsWith('?') ? currentSearch.slice(1) : currentSearch
  );
  const params = new URLSearchParams();
  for (const [key, value] of current) {
    if (projectWorkspaceSharedSearchParameters.has(key)) params.append(key, value);
  }
  params.set('project', projectId);
  const query = params.toString();
  return `${normalizePathname(pathname)}${query ? `?${query}` : ''}`;
}

export function resolveProjectWorkspaceId(
  projects: readonly ProjectWorkspaceProject[],
  requestedProjectId: string | null,
  currentProjectId: string | null
): string | null {
  const known = new Set(projects.map((project) => project.projectId));
  if (requestedProjectId && known.has(requestedProjectId)) return requestedProjectId;
  if (currentProjectId && known.has(currentProjectId)) return currentProjectId;

  return projects.find((project) =>
    project.lifecycle === 'active'
    && (project.health === 'ready' || project.health === 'revision-drift')
  )?.projectId
    ?? projects.find((project) => project.lifecycle === 'active')?.projectId
    ?? projects[0]?.projectId
    ?? null;
}

function normalizePathname(pathname: string): string {
  if (pathname === '/') return pathname;
  return pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
}
