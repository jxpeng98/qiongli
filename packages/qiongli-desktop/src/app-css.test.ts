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

describe('global visual system', () => {
  it('uses quiet semantic surfaces instead of decorative depth effects', () => {
    expect(appCss).toContain('--color-canvas: #f4f3ef');
    expect(appCss).toContain('--color-accent: #2f7168');
    expect(appCss).toContain('--radius-card: 7px');
    expect(appCss).toContain('--shadow-card: 0 1px 1px');
    expect(appCss).not.toContain('radial-gradient');
  });

  it('limits liquid glass to an accessible material utility', () => {
    expect(appCss).toContain('--glass-surface: rgb(250 250 247 / 0.66)');
    expect(appCss).toContain('--glass-filter: blur(24px) saturate(1.1)');
    expect(appCss).toContain('.glass-material');
    expect(appCss).toContain('@media (prefers-reduced-transparency: reduce)');
  });

  it('keeps full touch targets while allowing pointer-dense controls', () => {
    expect(appCss).toMatch(
      /\.button-primary,[\s\S]*?min-height:\s*44px/
    );
    expect(appCss).toMatch(
      /@media \(pointer: fine\)[\s\S]*?min-height:\s*38px/
    );
  });
});
