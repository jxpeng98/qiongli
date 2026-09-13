// @ts-expect-error Vitest runs this source contract in Node; the Desktop
// production bundle intentionally does not depend on Node type declarations.
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';

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

  it('keeps the application shell solid and neutral like shadcn-svelte', () => {
    const layout = source('src/routes/+layout.svelte');
    const sidebar = source('src/lib/components/app/AppSidebar.svelte');
    const overview = source('src/routes/overview/+page.svelte');
    const projectBar = source(
      'src/lib/components/app/ProjectWorkspaceBar.svelte'
    );

    expect(layout).toContain('<AppSidebar');
    expect(layout).not.toContain('backdrop-filter');
    expect(sidebar).toContain('class="app-sidebar"');
    expect(sidebar).not.toContain('glass-material');
    expect(sidebar).not.toMatch(/\.mark\s*\{[^}]*box-shadow/s);
    expect(projectBar).toContain('class="project-context"');
    expect(projectBar).not.toContain('glass-material');
    expect(projectBar).toContain('background: var(--color-surface)');
    expect(projectBar).toContain('position: sticky');
    expect(overview).toContain('<Card.Root class="summary">');
    expect(overview).not.toContain('glass-material');
    expect(overview).not.toMatch(/\.summary\s*\{[^}]*border-left/s);
    expect(layout).not.toContain("nav a[aria-current='page']::before");

    for (const path of sourceTree('src')) {
      if (!path.endsWith('.svelte') && !path.endsWith('.css')) continue;
      expect(source(path), path).not.toContain('backdrop-filter');
    }
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
    expect(integrations).toContain('class="workspace-tabs"');
    for (const section of ['agents', 'mcp', 'migration', 'zotero', 'skills']) {
      expect(integrations).toContain(`value="${section}"`);
    }
    expect(integrations).toContain(".workspace-tabs [data-slot='tabs-trigger'][data-state='active']");
    expect(integrations).toContain(".integration-tabs [data-slot='tabs-trigger'][data-state='active']");
    expect(integrations).toContain('background: var(--color-surface) !important;');
    expect(integrations).toContain(':global(.integration-tabs) { display: grid; width: 100%;');
    expect(integrations).toContain('.integration-tab-copy strong { color: inherit;');
    expect(dialog).toContain('<AlertDialog.Root open');
    expect(dialog).not.toContain('handleDialogKeydown');
  });

  it('keeps transient feedback below the blocking confirmation boundary', () => {
    const tokens = source('src/app.css');
    const layout = source('src/routes/+layout.svelte');
    const projectBar = source('src/lib/components/app/ProjectWorkspaceBar.svelte');
    const dialogOverlay = source(
      'src/lib/components/ui/alert-dialog/alert-dialog-overlay.svelte'
    );
    const dialogContent = source(
      'src/lib/components/ui/alert-dialog/alert-dialog-content.svelte'
    );

    expect(layout).toContain('z-index: var(--z-banner)');
    expect(layout).toContain('z-index: var(--z-sticky)');
    expect(projectBar).toContain('z-index: var(--z-sticky-context)');
    expect(dialogOverlay).toContain('z-[var(--z-dialog-scrim)]');
    expect(dialogContent).toContain('z-[var(--z-dialog)]');
    expect(zIndex(tokens, '--z-sticky')).toBeLessThan(zIndex(tokens, '--z-sticky-context'));
    expect(zIndex(tokens, '--z-sticky-context')).toBeLessThan(zIndex(tokens, '--z-banner'));
    expect(zIndex(tokens, '--z-banner')).toBeLessThan(zIndex(tokens, '--z-dialog-scrim'));
    expect(zIndex(tokens, '--z-dialog-scrim')).toBeLessThan(zIndex(tokens, '--z-dialog'));
  });

  it('loads the heavyweight confirmation surface only when an operation needs it', () => {
    const layout = source('src/routes/+layout.svelte');
    const appUi = source('src/lib/components/app/index.ts');

    expect(layout).toContain(
      "{#await import('$lib/components/app/ConfirmationDialog.svelte')}"
    );
    expect(layout).toContain('class="dialog-loading-scrim"');
    expect(appUi).not.toContain('ConfirmationDialog');
  });

  it('loads only the preferred locale before rendering and keeps catalogs out of the shared shell', () => {
    const layoutLoad = source('src/routes/+layout.ts');
    const layout = source('src/routes/+layout.svelte');
    const sidebar = source('src/lib/components/app/AppSidebar.svelte');
    const runtime = source('src/lib/i18n.svelte.ts');
    const testSetup = source('src/tests/setup.ts');

    expect(layoutLoad).toContain('await i18n.initialize()');
    expect(layout).not.toContain('i18n.initialize()');
    expect(sidebar).toContain('disabled={i18n.loading}');
    expect(sidebar).toContain('i18n.loadFailed');
    expect(runtime).not.toContain("import enCatalog from './i18n/locales/en'");
    expect(runtime).toContain("import('./i18n/locales/en')");
    expect(runtime).toContain("import('./i18n/locales/zh-CN')");
    expect(runtime).toContain('bootstrapCatalog');
    expect(testSetup).toContain("await i18n.setLocale('en')");
  });

  it('defers the complete validation client without weakening the native boundary', () => {
    const state = source('src/lib/app-state.svelte.ts');
    const deferredClient = source('src/lib/deferred-app-client.ts');
    const validatedClient = source('src/lib/validated-app-client.ts');

    expect(state).not.toContain('new QiongliAppClient()');
    expect(state).toContain('deferredAppClient()');
    expect(deferredClient).toContain("import('./validated-app-client')");
    expect(validatedClient).toContain('new QiongliAppClient(transport)');
  });

  it('moves long update and Zotero evidence behind explicit disclosure controls', () => {
    const about = source('src/routes/about/+page.svelte');
    const integrations = source('src/routes/client-integrations/+page.svelte');

    expect(about).toContain('<details class="update-technical">');
    expect(integrations).toContain('<details class="zotero-technical">');
    expect(integrations).not.toContain('<div class="zotero-digest">');
  });

  it('keeps status capsules whole without wrapping inside responsive rows', () => {
    const badge = source('src/lib/components/app/StatusBadge.svelte');

    expect(badge).toContain('flex: none');
    expect(badge).toContain('white-space: nowrap');
    expect(badge).not.toContain('text-overflow: ellipsis');
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
      'DescriptionTip',
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

  it('keeps compact cards content-led and icon rows on an explicit alignment axis', () => {
    const overview = source('src/routes/overview/+page.svelte');
    const contentGrid = source('src/lib/components/app/ContentGrid.svelte');
    const metricCard = source('src/lib/components/app/MetricCard.svelte');
    const statePanel = source('src/lib/components/app/StatePanel.svelte');

    expect(overview).toContain("'icon title'");
    expect(overview).toContain("'footer footer'");
    expect(overview).toContain('<DescriptionTip text=');
    expect(overview).toContain('<ContentGrid columns={3} collapse="sm" lastSpan={2}');
    expect(overview).toContain('<IconFrame>');
    expect(contentGrid).toContain('align-items: start');
    expect(contentGrid).toContain("data-last-span='2'");
    expect(overview).toContain('min-height: 0');
    expect(overview).not.toContain('min-height: 136px');
    expect(metricCard).toMatch(/\.metric-card\)[\s\S]*?flex-direction: row/);
    expect(statePanel).toMatch(/\.state-panel\)[\s\S]*?flex-direction: row/);
    expect(statePanel).toMatch(/\.state-panel\.centered\)[\s\S]*?flex-direction: column/);
  });

  it('provides persistent light and dark modes from the application shell', () => {
    const appHtml = source('src/app.html');
    const layout = source('src/routes/+layout.svelte');
    const sidebar = source('src/lib/components/app/AppSidebar.svelte');

    expect(appHtml).toContain('content="light dark"');
    expect(appHtml).toContain('document.documentElement.dataset.theme = theme');
    expect(appHtml).not.toContain("classList.toggle('dark'");
    expect(layout).toContain("const THEME_STORAGE_KEY = 'qiongli.theme'");
    expect(layout).toContain("document.documentElement.dataset.theme = nextTheme");
    expect(layout).not.toContain("classList.toggle('dark'");
    expect(layout).toContain('onToggleTheme={toggleTheme}');
    expect(sidebar).toContain('aria-pressed={theme === \'dark\'}');
    expect(layout).toContain("window.matchMedia('(prefers-color-scheme: dark)')");
    expect(source('src/app.css')).toContain(
      "@custom-variant dark (&:where([data-theme='dark'], [data-theme='dark'] *));"
    );
  });

  it('keeps horizontal scrolling out of application pages', () => {
    const offenders = sourceTree('src').filter((path) => {
      if (!path.endsWith('.svelte') && !path.endsWith('.css')) return false;
      return /overflow-x:\s*(?:auto|scroll)/.test(source(path));
    });

    expect(offenders).toEqual([]);
  });

  it('prevents route-local legacy UI classes from returning', () => {
    const offenders = sourceTree('src').filter((path) => {
      if (!path.endsWith('.svelte')) return false;
      const component = source(path);
      const classNames = [...component.matchAll(/class=["']([^"']*)["']/g)]
        .flatMap((match) => match[1].split(/\s+/));
      return /button-(?:primary|secondary|quiet|danger)/.test(component)
        || classNames.includes('surface');
    });

    expect(offenders).toEqual([]);
    expect(source('src/app.css')).not.toMatch(/\.button-(?:primary|secondary|quiet|danger)\b/);
    expect(source('src/app.css')).not.toMatch(/(?:^|\n)\.surface\s*[,\{]/);
  });

  it('removes the compatibility boundary and unused generated groups', () => {
    expect(existsSync('src/lib/shared/ui/index.ts')).toBe(false);
    expect(existsSync('src/lib/shared/ui/styles.ts')).toBe(false);

    for (const group of [
      'collapsible',
      'empty',
      'popover',
      'scroll-area',
      'select',
      'spinner',
      'switch',
      'table',
      'textarea'
    ]) {
      expect(existsSync(`src/lib/components/ui/${group}/index.ts`), group).toBe(false);
    }
  });

  it('keeps hardcoded presentation colors inside the graph visual language', () => {
    const hardcodedColor = /(?:#[0-9a-f]{3,8}\b|(?:rgb|hsl)a?\()/i;
    const offenders = sourceTree('src').filter((path) => {
      if (!path.endsWith('.svelte')) return false;
      if (path.includes('/academic-graph/') || path.includes('routes/academic-graph/')) return false;
      const styles = [...source(path).matchAll(/<style(?:\s[^>]*)?>([\s\S]*?)<\/style>/g)]
        .map((match) => match[1])
        .join('\n');
      return hardcodedColor.test(styles);
    });

    expect(offenders).toEqual([]);
  });

  it('uses responsive semantic records for the Academic Graph tables', () => {
    const graph = source('src/routes/academic-graph/+page.svelte');
    const responsiveView = source('src/lib/components/app/ResponsiveDataView.svelte');

    expect(graph).toContain('<ResponsiveDataView');
    expect(graph).toContain('<table>');
    expect(graph).toContain('<ol class="node-cards">');
    expect(responsiveView).toContain('@media (max-width: 720px)');
    expect(responsiveView).toContain('.desktop-view { display: none; }');
    expect(responsiveView).toContain('.mobile-view { display: grid;');
  });

  it('collapses dense overview and library controls before the sidebar crowds content', () => {
    const overview = source('src/routes/overview/+page.svelte');
    const library = source('src/routes/research-library/+page.svelte');
    const timeline = source('src/lib/features/timeline/TimelineControls.svelte');

    expect(overview).toMatch(
      /@media \(max-width: 1200px\)[\s\S]*?\.client-list \{ grid-template-columns: 1fr; \}/
    );
    expect(library).toMatch(
      /@media \(max-width: 1040px\)[\s\S]*?\.controls \{ grid-template-columns: 1fr 1fr; \}/
    );
    expect(timeline).toMatch(
      /@media \(max-width: 860px\)[\s\S]*?form \{ grid-template-columns: 1fr; \}/
    );
  });

  it('keeps the research-library topology available without competing with project work', () => {
    const library = source('src/routes/research-library/+page.svelte');

    expect(library).toContain('<details class="portfolio-disclosure">');
    expect(library).toContain('showDetails={false}');
  });

  it('only references conditional popups while their controlled content is mounted', () => {
    const graph = source('src/routes/academic-graph/+page.svelte');
    const orchestrator = source('src/routes/orchestrator/+page.svelte');

    expect(graph).toContain("aria-controls={searchFocused && textFilter.trim().length > 0");
    expect(orchestrator).toContain('aria-controls={pendingCancelRunId === run.runId');
  });

  it('lets shared controls grow with translated labels instead of clipping text', () => {
    const button = source('src/lib/components/ui/button/button.svelte');
    const tabsList = source('src/lib/components/ui/tabs/tabs-list.svelte');
    const tabsTrigger = source('src/lib/components/ui/tabs/tabs-trigger.svelte');
    const pageHeader = source('src/lib/components/app/PageHeader.svelte');
    const sectionHeader = source('src/lib/components/app/SectionHeader.svelte');
    const descriptionTip = source('src/lib/components/app/DescriptionTip.svelte');
    const projectBar = source('src/lib/components/app/ProjectWorkspaceBar.svelte');

    expect(button).toContain('whitespace-normal');
    expect(button).toContain('[overflow-wrap:anywhere]');
    expect(button).toContain('rounded-[var(--radius-control)]');
    expect(button).toContain('h-auto min-h-8');
    expect(tabsList).toContain('group-data-horizontal/tabs:min-h-8');
    expect(tabsTrigger).toContain('whitespace-normal');
    expect(pageHeader).not.toContain('line-clamp: 3');
    expect(pageHeader).toContain('<DescriptionTip text={description} />');
    expect(sectionHeader).toContain('<DescriptionTip text={description} />');
    expect(sectionHeader).toContain('.identity { width: 100%; flex: 0 1 auto; }');
    expect(descriptionTip).toContain('aria-label={i18n.t(\'common.moreInformation\')}');
    expect(descriptionTip).toContain('class="description-sr sr-only"');
    expect(projectBar).toContain('-webkit-line-clamp: 2');
    expect(projectBar).toContain('.project-identity > div { min-width: 0; }');
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
