import { describe, expect, it } from 'vitest';

import {
  isProjectWorkspaceRoute,
  projectWorkspaceHref,
  resolveProjectWorkspaceId,
  type ProjectWorkspaceProject
} from '.';

const projects: ProjectWorkspaceProject[] = [
  {
    projectId: `prj_${'1'.repeat(32)}`,
    lifecycle: 'archived',
    health: 'ready'
  },
  {
    projectId: `prj_${'2'.repeat(32)}`,
    lifecycle: 'active',
    health: 'revision-drift'
  },
  {
    projectId: `prj_${'3'.repeat(32)}`,
    lifecycle: 'active',
    health: 'ready'
  }
];

describe('project workspace context', () => {
  it('prefers an exact deep link, then the current valid context', () => {
    expect(resolveProjectWorkspaceId(
      projects,
      projects[0]!.projectId,
      projects[2]!.projectId
    )).toBe(projects[0]!.projectId);
    expect(resolveProjectWorkspaceId(
      projects,
      null,
      projects[2]!.projectId
    )).toBe(projects[2]!.projectId);
  });

  it('falls back to the first active usable project without inventing an id', () => {
    expect(resolveProjectWorkspaceId(projects, 'prj_missing', null))
      .toBe(projects[1]!.projectId);
    expect(resolveProjectWorkspaceId(
      projects.slice(0, 2),
      projects[2]!.projectId,
      projects[2]!.projectId
    )).toBe(projects[1]!.projectId);
    expect(resolveProjectWorkspaceId([], null, projects[0]!.projectId)).toBeNull();
  });

  it('preserves shared fixture state without leaking route-local deep links', () => {
    expect(projectWorkspaceHref(
      '/academic-graph/',
      projects[2]!.projectId,
      '?fixture=source-read-only&project=stale&capture=cap_stale&entity=nod_stale'
    )).toBe(
      `/academic-graph?fixture=source-read-only&project=${projects[2]!.projectId}`
    );
  });

  it('recognizes only project-scoped workspace routes', () => {
    expect(isProjectWorkspaceRoute('/research-library')).toBe(true);
    expect(isProjectWorkspaceRoute('/artifacts')).toBe(true);
    expect(isProjectWorkspaceRoute('/academic-graph/')).toBe(true);
    expect(isProjectWorkspaceRoute('/client-integrations')).toBe(false);
    expect(isProjectWorkspaceRoute('/portfolio')).toBe(false);
  });
});
