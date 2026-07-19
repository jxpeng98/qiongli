import { describe, expect, it } from 'vitest';

import type { ArticleProjectSummary } from '@qiongli/app-api';
import { filterProjects, projectStatus } from './index';

const baseProject: ArticleProjectSummary = {
  projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051',
  displayName: 'Evidence graph article',
  projectKind: 'article',
  stage: 'writing',
  lifecycle: 'active',
  semanticRevision: 3,
  registeredAtUnix: 10,
  lastOpenedAtUnix: null,
  academicallyUpdatedAtUnix: 30,
  health: 'ready',
  nextAction: 'open',
  rootLabel: 'evidence-graph',
  overview: {
    focalQuestion: 'How should claims connect to literature?',
    thesis: 'Typed scholarly relations make the connection inspectable.',
    evidencePosition: null,
    unresolvedRiskCount: 1,
    claimEvidenceCoveragePercent: 72,
    nextPriorities: ['Resolve the remaining evidence gap']
  }
};

describe('Research Library presentation model', () => {
  it('searches academic overview text without relying on host paths', () => {
    const match = filterProjects([baseProject], 'scholarly relations', 'all', 'name');
    expect(match.map((project) => project.projectId)).toEqual([baseProject.projectId]);
    expect(filterProjects([baseProject], '/private/research', 'all', 'name')).toEqual([]);
  });

  it('keeps attention and lifecycle filters distinct', () => {
    const drifted = {
      ...baseProject,
      projectId: 'prj_118f4d5a3b2c71008a9b0c1d2e3f4052',
      health: 'revision-drift' as const,
      lifecycle: 'archived' as const,
      academicallyUpdatedAtUnix: 40
    };
    expect(filterProjects([baseProject, drifted], '', 'attention', 'academically-updated'))
      .toEqual([drifted]);
    expect(filterProjects([baseProject, drifted], '', 'active', 'academically-updated'))
      .toEqual([baseProject]);
    expect(projectStatus(drifted)).toBe('drifted');
  });
});
