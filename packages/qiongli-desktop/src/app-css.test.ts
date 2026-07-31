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
    expect(appCss).toContain('--color-canvas: #ecebe6');
    expect(appCss).toContain('--color-accent: #2a6c63');
    expect(appCss).toContain('--radius-md: 10px');
    expect(appCss).toContain('--radius-card: var(--radius-md)');
    expect(appCss).toContain('--ui-panel-radius: var(--radius-card)');
    expect(appCss).toContain('--shadow-card: 0 1px 1px');
    expect(appCss).not.toContain('radial-gradient');
  });

  it('limits liquid glass to an accessible material utility', () => {
    expect(appCss).toContain('--glass-surface: rgb(255 255 253 / 0.78)');
    expect(appCss).toContain('--glass-filter: blur(30px) saturate(1.18) brightness(1.02)');
    expect(appCss).toContain('--glass-highlight: rgb(255 255 255 / 0.86)');
    expect(appCss).toContain('--glass-tint: rgb(73 136 125 / 0.11)');
    expect(appCss).toContain('.glass-material--strong');
    expect(appCss).toContain('.glass-material');
    expect(appCss).toContain('@media (prefers-reduced-transparency: reduce)');
    expect(appCss).toContain('@media (prefers-contrast: more)');
  });

  it('keeps dark glass flatter and more opaque than the light material', () => {
    const darkTheme = appCss.slice(appCss.indexOf(":root[data-theme='dark']"));

    expect(darkTheme).toContain('--glass-surface: rgb(28 33 30 / 0.92)');
    expect(darkTheme).toContain('--glass-highlight: rgb(255 255 255 / 0.09)');
    expect(darkTheme).toContain('--glass-tint: rgb(99 184 168 / 0.025)');
    expect(darkTheme).toContain('--glass-filter: blur(16px) saturate(1.02) brightness(0.99)');
    expect(darkTheme).not.toContain('blur(30px)');
  });

  it('defines semantic component tokens for shared panels, states, and metrics', () => {
    for (const tokenName of [
      '--ui-panel-background',
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
      expect(contrast(token(theme, '--color-ink'), surface)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(theme, '--color-muted'), surface)).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(theme, '--color-accent-strong'), surface)).toBeGreaterThanOrEqual(4.5);
    }
  });

  it('guards the document boundary without hiding local data-table scrolling', () => {
    expect(appCss).toMatch(/html\s*\{[\s\S]*?overflow-x:\s*clip/);
    expect(appCss).toMatch(/body\s*\{[\s\S]*?overflow-x:\s*clip/);
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
