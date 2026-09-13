import { goto } from '$app/navigation';
import { page } from '$app/state';
import type { ArticleProjectSummary } from '@qiongli/app-api';

import {
  projectWorkspaceHref,
  resolveProjectWorkspaceId
} from './features/project-workspace';

export class ProjectWorkspaceState {
  projectId = $state<string | null>(null);

  reconcile(
    projects: readonly ArticleProjectSummary[],
    requestedProjectId: string | null
  ): void {
    this.projectId = resolveProjectWorkspaceId(
      projects,
      requestedProjectId,
      this.projectId
    );
  }

  href(
    pathname: string,
    projectId = this.projectId,
    currentSearch = page.url.search
  ): string {
    return projectId
      ? projectWorkspaceHref(pathname, projectId, currentSearch)
      : pathname;
  }

  async selectProject(
    projectId: string,
    pathname = page.url.pathname
  ): Promise<void> {
    if (projectId === this.projectId
      && page.url.searchParams.get('project') === projectId) return;
    this.projectId = projectId;
    await goto(this.href(pathname, projectId), {
      keepFocus: true,
      noScroll: true,
      replaceState: true
    });
  }
}
