import { describe, expect, it } from 'vitest';

import { academicGraphNodeTypes, academicGraphRelations } from '.';
import {
  academicGraphNodeVisual,
  academicGraphRelationFamilies,
  academicGraphRelationVisual,
  compactAcademicGraphLabel
} from './visual-language';

describe('Academic Graph visual language', () => {
  it('assigns every node type a deterministic shape and visible type mark', () => {
    expect(academicGraphNodeTypes.map((nodeType) => academicGraphNodeVisual(nodeType)))
      .toHaveLength(15);
    expect(academicGraphNodeVisual('research-question')).toMatchObject({
      mark: 'RQ',
      shape: 'diamond'
    });
    expect(academicGraphNodeVisual('evidence')).toMatchObject({
      mark: 'EV',
      shape: 'barrel'
    });
    expect(academicGraphNodeVisual('manuscript-section')).toMatchObject({
      mark: '§',
      shape: 'rectangle'
    });
  });

  it('assigns every relation to one accessible line and arrow family', () => {
    const visuals = academicGraphRelations.map((relation) =>
      academicGraphRelationVisual(relation));
    expect(visuals).toHaveLength(25);
    expect(new Set(visuals.map((visual) => visual.family)))
      .toEqual(new Set(academicGraphRelationFamilies));
    expect(academicGraphRelationVisual('supports')).toMatchObject({
      family: 'evidence',
      lineStyle: 'solid',
      arrowShape: 'triangle'
    });
    expect(academicGraphRelationVisual('contradicts')).toMatchObject({
      family: 'challenge',
      lineStyle: 'dashed',
      arrowShape: 'tee'
    });
    expect(academicGraphRelationVisual('derived-from')).toMatchObject({
      family: 'provenance',
      lineStyle: 'dotted',
      arrowShape: 'diamond'
    });
  });

  it('produces stable single-line labels for reduced zoom levels', () => {
    expect(compactAcademicGraphLabel('  A   compact   label  ')).toBe('A compact label');
    expect(compactAcademicGraphLabel('An intentionally long scholarly graph node'))
      .toBe('An intentionally long…');
  });
});
