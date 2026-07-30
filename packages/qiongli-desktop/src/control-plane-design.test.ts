// @ts-expect-error Vitest runs this source contract in Node; the Desktop
// production bundle intentionally does not depend on Node type declarations.
import { readdirSync, readFileSync, statSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

const source = (relativePath: string) =>
  readFileSync(relativePath, 'utf8');

describe('control-plane design contract', () => {
  it('keeps Desktop surfaces readable instead of shrinking dense copy', () => {
    for (const path of sourceTree('src')) {
      if (!path.endsWith('.svelte') && !path.endsWith('.css')) continue;
      expect(source(path), path).not.toMatch(/font-size:\s*[1-9]px/);
    }
  });

  it('keeps transient feedback below the blocking confirmation boundary', () => {
    const tokens = source('src/app.css');
    const layout = source('src/routes/+layout.svelte');
    const dialog = source('src/lib/components/ConfirmationDialog.svelte');

    expect(layout).toContain('z-index: var(--z-banner)');
    expect(dialog).toContain('z-index: var(--z-dialog-scrim)');
    expect(dialog).toContain('z-index: var(--z-dialog)');
    expect(zIndex(tokens, '--z-banner')).toBeLessThan(zIndex(tokens, '--z-dialog-scrim'));
    expect(zIndex(tokens, '--z-dialog-scrim')).toBeLessThan(zIndex(tokens, '--z-dialog'));
  });

  it('moves long update and Zotero evidence behind explicit disclosure controls', () => {
    const about = source('src/routes/about/+page.svelte');
    const integrations = source('src/routes/client-integrations/+page.svelte');

    expect(about).toContain('<details class="update-technical">');
    expect(integrations).toContain('<details class="zotero-technical">');
    expect(integrations).not.toContain('<div class="zotero-digest">');
  });

  it('allows status capsules to shrink without wrapping inside narrow rows', () => {
    const badge = source('src/lib/components/StatusBadge.svelte');

    expect(badge).toContain('flex: 0 1 auto');
    expect(badge).toContain('text-overflow: ellipsis');
    expect(badge).toContain('white-space: nowrap');
    expect(badge).not.toMatch(/\.status\s*\{[^}]*flex:\s*none/s);
  });
});

function zIndex(css: string, token: string): number {
  const match = css.match(new RegExp(`${token}:\\s*(\\d+)`));
  if (!match) throw new Error(`missing z-index token ${token}`);
  return Number(match[1]);
}

function sourceTree(directory: string): string[] {
  return readdirSync(directory).flatMap((entry: string) => {
    const path = `${directory}/${entry}`;
    return statSync(path).isDirectory() ? sourceTree(path) : [path];
  });
}
