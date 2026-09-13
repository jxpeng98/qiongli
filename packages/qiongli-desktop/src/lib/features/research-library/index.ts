import type { ArticleProjectSummary, StatusCode } from '@qiongli/app-api';
import type { FeatureDescriptor } from '../types';

export const researchLibraryFeature: FeatureDescriptor = {
  id: 'research-library',
  label: 'Research Library',
  route: '/research-library',
  stage: 'available'
};

export type ProjectSort = 'academically-updated' | 'name' | 'stage';
export type ProjectLifecycleFilter = 'all' | 'active' | 'archived' | 'attention';

export function projectStatus(project: ArticleProjectSummary): StatusCode {
  switch (project.health) {
    case 'ready':
      return 'ready';
    case 'revision-drift':
      return 'drifted';
    case 'manifest-conflict':
      return 'conflict';
    case 'inspection-blocked':
      return 'blocked';
    case 'missing-root':
    case 'missing-manifest':
      return 'missing';
  }
}

export function filterProjects(
  projects: ArticleProjectSummary[],
  query: string,
  lifecycle: ProjectLifecycleFilter,
  sort: ProjectSort
): ArticleProjectSummary[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const filtered = projects.filter((project) => {
    const matchesQuery = normalizedQuery.length === 0 || [
      project.displayName,
      project.rootLabel,
      project.overview.focalQuestion ?? '',
      project.overview.thesis ?? ''
    ].some((value) => value.toLocaleLowerCase().includes(normalizedQuery));
    const matchesLifecycle = lifecycle === 'all'
      || (lifecycle === 'attention' ? project.health !== 'ready' : project.lifecycle === lifecycle);
    return matchesQuery && matchesLifecycle;
  });

  return [...filtered].sort((left, right) => {
    if (sort === 'name') return left.displayName.localeCompare(right.displayName);
    if (sort === 'stage') return left.stage.localeCompare(right.stage)
      || right.academicallyUpdatedAtUnix - left.academicallyUpdatedAtUnix;
    return right.academicallyUpdatedAtUnix - left.academicallyUpdatedAtUnix;
  });
}
