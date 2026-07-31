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

  it('keeps the application shell quiet with bounded liquid glass', () => {
    const layout = source('src/routes/+layout.svelte');
    const sidebar = source('src/lib/components/app/AppSidebar.svelte');
    const overview = source('src/routes/overview/+page.svelte');
    const projectBar = source(
      'src/lib/components/app/ProjectWorkspaceBar.svelte'
    );

    expect(layout).toContain('<AppSidebar');
    expect(layout).not.toContain('backdrop-filter');
    expect(sidebar).toContain('class="app-sidebar glass-material"');
    expect(sidebar).not.toMatch(/\.mark\s*\{[^}]*box-shadow/s);
    expect(projectBar).toContain('class="project-context glass-material"');
    expect(projectBar).toContain('position: sticky');
    expect(overview).toContain('<Card.Root class="summary">');
    expect(overview).not.toContain('glass-material');
    expect(overview).not.toMatch(/\.summary\s*\{[^}]*border-left/s);
    expect(layout).not.toContain("nav a[aria-current='page']::before");
  });

  it('keeps Bits UI behind one styled Qiongli UI boundary', () => {
    const captureTabs = source(
      'src/lib/features/captures/CaptureWorkspaceTabs.svelte'
    );
    const integrations = source('src/routes/client-integrations/+page.svelte');
    const dialog = source('src/lib/components/app/ConfirmationDialog.svelte');
    const directBitsImports = sourceTree('src').filter((path) =>
      /from ['"]bits-ui['"]/.test(source(path))
    );

    expect(
      directBitsImports.filter((path) => !path.startsWith('src/lib/components/ui/'))
    ).toEqual([]);
    expect(captureTabs).toContain('<TabsRoot');
    expect(captureTabs).not.toContain('moveFocus');
    expect(integrations).toContain('<TabsContent');
    expect(integrations).not.toContain('handleTabKey');
    expect(dialog).toContain('<AlertDialog.Root open');
    expect(dialog).not.toContain('handleDialogKeydown');
  });

  it('keeps transient feedback below the blocking confirmation boundary', () => {
    const tokens = source('src/app.css');
    const layout = source('src/routes/+layout.svelte');
    const dialog = source('src/lib/components/app/ConfirmationDialog.svelte');

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
    const badge = source('src/lib/components/app/StatusBadge.svelte');

    expect(badge).toContain('flex: 0 1 auto');
    expect(badge).toContain('text-overflow: ellipsis');
    expect(badge).toContain('white-space: nowrap');
    expect(badge).not.toMatch(/\.status\s*\{[^}]*flex:\s*none/s);
  });

  it('keeps global navigation separate from the shared project workspace', () => {
    const layout = source('src/routes/+layout.svelte');
    const projectBar = source(
      'src/lib/components/app/ProjectWorkspaceBar.svelte'
    );

    expect(layout).toContain('<ProjectWorkspaceBar />');
    expect(layout).not.toContain("{ href: '/academic-graph', label: 'nav.graph'");
    expect(layout).not.toContain("{ href: '/captures', label: 'nav.captures'");
    expect(projectBar).toContain('projectWorkspaceNavigation');
    expect(projectBar).toContain('repeat(auto-fit, minmax(min(112px, 100%), 1fr))');
    expect(projectBar).not.toContain('overflow-x: auto');
    expect(projectBar).not.toContain('scrollIntoView');
  });

  it('assembles recurring page states from one shared UI system', () => {
    const appUi = source('src/lib/components/app/index.ts');

    for (const component of [
      'SectionHeader',
      'StatePanel',
      'MetricGrid',
      'MetricCard',
      'TabsRoot',
      'TabsList',
      'TabsTrigger',
      'TabsContent'
    ]) {
      expect(appUi).toContain(`export { default as ${component} }`);
    }

    for (const route of [
      'overview',
      'about',
      'client-integrations',
      'research-library',
      'artifacts',
      'captures',
      'academic-graph',
      'portfolio',
      'timeline',
      'orchestrator'
    ]) {
      const page = source(`src/routes/${route}/+page.svelte`);
      expect(page, route).toContain('StatePanel');
      expect(page, route).not.toMatch(/\.(?:state-panel|state-message|empty-state|load-failed|blocked-state|loading)\s*\{/);
    }
  });

  it('provides persistent light and dark modes from the application shell', () => {
    const appHtml = source('src/app.html');
    const layout = source('src/routes/+layout.svelte');
    const sidebar = source('src/lib/components/app/AppSidebar.svelte');

    expect(appHtml).toContain('content="light dark"');
    expect(appHtml).toContain("document.documentElement.classList.toggle('dark'");
    expect(layout).toContain("const THEME_STORAGE_KEY = 'qiongli.theme'");
    expect(layout).toContain("document.documentElement.dataset.theme = nextTheme");
    expect(layout).toContain("document.documentElement.classList.toggle('dark'");
    expect(layout).toContain('onToggleTheme={toggleTheme}');
    expect(sidebar).toContain('aria-pressed={theme === \'dark\'}');
    expect(layout).toContain("window.matchMedia('(prefers-color-scheme: dark)')");
  });

  it('keeps horizontal scrolling scoped to the academic graph data table', () => {
    const offenders = sourceTree('src').filter((path) => {
      if (!path.endsWith('.svelte') && !path.endsWith('.css')) return false;
      return /overflow-x:\s*(?:auto|scroll)/.test(source(path));
    });

    expect(offenders).toEqual(['src/routes/academic-graph/+page.svelte']);
    expect(source(offenders[0])).toMatch(/\.table-scroll\s*\{\s*overflow-x:\s*auto/);
  });

  it('collapses dense overview and library controls before the sidebar breakpoint', () => {
    const overview = source('src/routes/overview/+page.svelte');
    const library = source('src/routes/research-library/+page.svelte');

    expect(overview).toMatch(
      /@media \(max-width: 900px\)[\s\S]*?\.client-list \{ grid-template-columns: 1fr; \}/
    );
    expect(library).toMatch(
      /@media \(max-width: 900px\)[\s\S]*?\.controls \{ grid-template-columns: 1fr 1fr; \}/
    );
  });

  it('uses one project context across every project-scoped route', () => {
    for (const route of [
      'research-library',
      'artifacts',
      'captures',
      'academic-graph',
      'timeline',
      'orchestrator'
    ]) {
      const page = source(`src/routes/${route}/+page.svelte`);
      expect(page, route).toContain('useProjectWorkspace');
      expect(page, route).not.toMatch(
        /let selectedProjectId = \$state(?:<[^>]+>)?\(/
      );
    }
  });

  it('presents bundled Skills as evidence inside one host integration', () => {
    const integrations = source('src/routes/client-integrations/+page.svelte');
    const standalone = source(
      'src/lib/features/client-integrations/WorkflowContentPanel.svelte'
    );

    expect(integrations).toContain('class="host-package"');
    expect(integrations).toContain('class="package-components"');
    expect(integrations).toContain("integrations.component.skills");
    expect(integrations).toContain("integrations.hostPackageBoundary");
    expect(integrations).not.toContain('class="content-grid"');
    expect(standalone).toContain("content.advancedTitle");
    expect(standalone).toContain("content.advancedDescription");
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
