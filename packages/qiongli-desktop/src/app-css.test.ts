// @ts-expect-error Vitest runs this source contract in Node; the Desktop
// production bundle intentionally does not depend on Node type declarations.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const appCss = readFileSync('src/app.css', 'utf8');

describe('global accessibility styles', () => {
  it('reduces motion without removing operation feedback', () => {
    const ruleStart = appCss.indexOf('@media (prefers-reduced-motion: reduce)');
    const reducedMotion = appCss.slice(ruleStart);

    expect(ruleStart).toBeGreaterThanOrEqual(0);
    expect(reducedMotion).toContain('scroll-behavior: auto !important');
    expect(reducedMotion).toContain('transition-duration: 0.01ms !important');
    expect(reducedMotion).toContain('animation-duration: 0.01ms !important');
    expect(reducedMotion).toContain('animation-iteration-count: 1 !important');
  });
});
