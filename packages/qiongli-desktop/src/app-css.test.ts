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
  it('uses the shadcn-svelte homepage neutral visual language', () => {
    expect(appCss).toContain('--color-canvas: #ffffff');
    expect(appCss).toContain('--color-accent: #171717');
    expect(appCss).toContain('--radius-md: 10px');
    expect(appCss).toContain('--radius-card: 1.25rem');
    expect(appCss).toContain('--shadow-card: 0 1px 3px');
    expect(appCss).toContain('--background-canvas: var(--color-canvas)');
    expect(appCss).not.toContain('radial-gradient');
    expect(appCss).not.toContain('linear-gradient');
  });

  it('uses solid shadcn surfaces instead of liquid glass', () => {
    expect(appCss).not.toContain('--glass-');
    expect(appCss).not.toContain('--radius-glass');
    expect(appCss).not.toContain('--shadow-glass');
    expect(appCss).not.toContain('.glass-material');
    expect(appCss).not.toContain('backdrop-filter');
    expect(appCss).toContain('@media (prefers-contrast: more)');
  });

  it('matches the homepage dark mode with near-black solid surfaces', () => {
    const darkTheme = appCss.slice(appCss.indexOf(":root[data-theme='dark']"));

    expect(darkTheme).toContain('--color-canvas: #0a0a0a');
    expect(darkTheme).toContain('--color-surface: #171717');
    expect(darkTheme).toContain('--color-surface-subtle: #262626');
    expect(darkTheme).not.toContain('blur(');
  });

  it('loads the shadcn-svelte theme bridge without replacing Qiongli semantics', () => {
    expect(appCss).toContain("@import 'shadcn-svelte/tailwind.css'");
    expect(appCss).toContain("@import './lib/styles/shadcn.css'");
    expect(appCss).toContain('@custom-variant dark');
  });

  it('defines semantic component tokens for shared panels, states, and metrics', () => {
    for (const tokenName of [
      '--ui-section-title-size',
      '--ui-state-padding',
      '--ui-state-centered-min-height',
      '--ui-metric-grid-gap',
      '--ui-metric-value-size'
    ]) {
      expect(appCss).toContain(`${tokenName}:`);
    }
  });

  it('defines independent high-contrast light and dark theme tokens', () => {
    const lightTheme = appCss.slice(
      appCss.indexOf(':root {'),
      appCss.indexOf(":root[data-theme='dark']")
    );
    const darkTheme = appCss.slice(appCss.indexOf(":root[data-theme='dark']"));

    expect(darkTheme).toContain('color-scheme: dark');
    for (const theme of [lightTheme, darkTheme]) {
      const surface = token(theme, '--color-surface');
      const subtleSurface = token(theme, '--color-surface-subtle');
      expect(contrast(token(theme, '--color-ink'), surface)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(theme, '--color-muted'), surface)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(theme, '--color-accent-strong'), surface)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(theme, '--color-ink'), subtleSurface)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(theme, '--color-muted'), subtleSurface)).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(token(theme, '--color-warning-strong'), token(theme, '--color-warning-soft'))
      ).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(token(theme, '--color-danger'), token(theme, '--color-danger-soft'))
      ).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(token(theme, '--color-info'), token(theme, '--color-info-soft'))
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  it('guards the document boundary without hiding local data-table scrolling', () => {
    expect(appCss).toMatch(/html\s*\{[\s\S]*?overflow-x:\s*clip/);
    expect(appCss).toMatch(/body\s*\{[\s\S]*?overflow-x:\s*clip/);
  });

  it('keeps full touch targets for coarse pointers', () => {
    expect(appCss).toMatch(
      /@media \(pointer: coarse\)[\s\S]*?\[data-slot='button'\]\s*\{\s*min-height:\s*44px/
    );
  });
});

function token(theme: string, name: string): string {
  const match = theme.match(new RegExp(`${name}:\\s*(#[0-9a-f]{6})`, 'i'));
  if (!match) throw new Error(`missing ${name}`);
  return match[1];
}

function contrast(foreground: string, background: string): number {
  const luminance = (hex: string): number => {
    const channels = [1, 3, 5].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16) / 255);
    const [red, green, blue] = channels.map((channel) =>
      channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4
    );
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
  };
  const foregroundLuminance = luminance(foreground);
  const backgroundLuminance = luminance(background);
  return (Math.max(foregroundLuminance, backgroundLuminance) + 0.05)
    / (Math.min(foregroundLuminance, backgroundLuminance) + 0.05);
}
