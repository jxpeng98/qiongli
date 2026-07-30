import { describe, expect, it } from 'vitest';

import { AcademicGraphRequestSequence } from './request-sequence';

describe('AcademicGraphRequestSequence', () => {
  it('rejects late results after an A to B to A project switch', () => {
    const sequence = new AcademicGraphRequestSequence();
    const firstA = sequence.begin('project-a:12');
    sequence.begin('project-b:6');
    const secondA = sequence.begin('project-a:12');

    expect(sequence.isCurrent(firstA, 'project-a:12')).toBe(false);
    expect(sequence.isCurrent(secondA, 'project-a:12')).toBe(true);
  });

  it('invalidates a pending result when the view changes without a replacement request', () => {
    const sequence = new AcademicGraphRequestSequence();
    const pending = sequence.begin('project-a:12');
    sequence.invalidate();

    expect(sequence.isCurrent(pending, 'project-a:12')).toBe(false);
  });

  it('requires both request generation and exact active scope', () => {
    const sequence = new AcademicGraphRequestSequence();
    const pending = sequence.begin('project-a:12:projection-a');

    expect(sequence.isCurrent(pending, 'project-a:13:projection-b')).toBe(false);
  });
});
