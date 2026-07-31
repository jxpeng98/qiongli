<script lang="ts">
  import { BookOpenText, Cable, Database, Info, Languages, LayoutDashboard, Moon, Network, RefreshCw, Sun } from '@lucide/svelte';
  import { page } from '$app/state';
  import { onMount } from 'svelte';

  import '../app.css';
  import { ConfirmationDialog, FeedbackBanner, materialClass } from '$lib/shared/ui';
  import ProjectWorkspaceBar from '$lib/features/project-workspace/ProjectWorkspaceBar.svelte';
  import { isProjectWorkspaceRoute } from '$lib/features/project-workspace';
  import { provideAppState, provideProjectWorkspace } from '$lib/context';
  import { i18n, type Locale } from '$lib/i18n.svelte';

  let { children } = $props();
  const app = provideAppState();
  const projectWorkspace = provideProjectWorkspace();
  let previewFocusTarget = $state<HTMLElement | null>(null);
  let theme = $state<'light' | 'dark'>('light');

  const THEME_STORAGE_KEY = 'qiongli.theme';

  const navigation = [
    { href: '/overview', label: 'nav.overview', icon: LayoutDashboard },
    { href: '/research-library', label: 'nav.library', icon: BookOpenText },
    { href: '/portfolio', label: 'nav.portfolio', icon: Database },
    { href: '/client-integrations', label: 'nav.integrations', icon: Cable },
    { href: '/about', label: 'nav.about', icon: Info }
  ];

  onMount(() => {
    i18n.initialize();
    const savedTheme = window.localStorage.getItem(THEME_STORAGE_KEY);
    applyTheme(
      savedTheme === 'light' || savedTheme === 'dark'
        ? savedTheme
        : window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
      false
    );
    void app.refresh();
    document.addEventListener('pointerdown', rememberInteractionTarget, true);
    document.addEventListener('click', rememberInteractionTarget, true);
    document.addEventListener('focusin', rememberInteractionTarget, true);
    return () => {
      document.removeEventListener('pointerdown', rememberInteractionTarget, true);
      document.removeEventListener('click', rememberInteractionTarget, true);
      document.removeEventListener('focusin', rememberInteractionTarget, true);
    };
  });

  $effect(() => {
    projectWorkspace.reconcile(
      app.snapshot?.researchLibrary.projects ?? [],
      page.url.searchParams.get('project')
    );
  });

  $effect(() => {
    const projectId = projectWorkspace.projectId;
    if (
      projectId
      && isProjectWorkspaceRoute(page.url.pathname)
      && page.url.searchParams.get('project') !== projectId
    ) {
      void projectWorkspace.selectProject(projectId);
    }
  });

  function changeLanguage(event: Event): void {
    const locale = (event.currentTarget as HTMLSelectElement).value as Locale;
    if (locale === i18n.locale) return;
    app.dismissNotice();
    i18n.setLocale(locale);
  }

  function applyTheme(nextTheme: 'light' | 'dark', persist = true): void {
    theme = nextTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = nextTheme;
      document.documentElement.classList.toggle('dark', nextTheme === 'dark');
      document.querySelector<HTMLMetaElement>('meta[name="theme-color"]')
        ?.setAttribute('content', nextTheme === 'dark' ? '#151815' : '#ecebe6');
    }
    if (persist && typeof window !== 'undefined') {
      window.localStorage.setItem(THEME_STORAGE_KEY, nextTheme);
    }
  }

  function toggleTheme(): void {
    applyTheme(theme === 'dark' ? 'light' : 'dark');
  }

  function rememberInteractionTarget(event: Event): void {
    const target = event.target instanceof Element
      ? event.target.closest<HTMLElement>('button, a, input, select, textarea, [tabindex]')
      : null;
    if (target && !target.closest('[role="dialog"]')) previewFocusTarget = target;
  }

  async function confirmOperation(): Promise<void> {
    if (!app.preview) return;
    await app.execute({ action: 'confirm-operation', token: app.preview.token });
  }

  async function cancelOperation(): Promise<void> {
    if (!app.preview) return;
    if (app.preview.canConfirm) {
      await app.execute({ action: 'cancel-operation', token: app.preview.token });
    } else {
      app.closePreview();
    }
  }
</script>

<svelte:head>
  <title>Qiongli 2</title>
  <meta name="description" content="Qiongli academic research workflow" />
</svelte:head>

<div class="shell">
  <a class="skip-link" href="#main-content">{i18n.t('nav.skip')}</a>
  <aside class={materialClass('glass')}>
    <div class="brand">
      <div class="mark" aria-hidden="true"><Network size={23} strokeWidth={1.9} /></div>
      <div>
        <strong>Qiongli</strong>
        <span>{i18n.t('app.subtitle')}</span>
      </div>
    </div>

    <nav aria-label={i18n.t('nav.primary')}>
      <p>{i18n.t('nav.global')}</p>
      {#each navigation as item}
        <a href={item.href} aria-current={page.url.pathname === item.href ? 'page' : undefined}>
          <item.icon size={18} strokeWidth={1.9} aria-hidden="true" />
          {i18n.t(item.label)}
        </a>
      {/each}
    </nav>

    <div class="sidebar-footer">
      <label class="language-control">
        <Languages size={15} aria-hidden="true" />
        <span>{i18n.t('language.label')}</span>
        <select value={i18n.locale} onchange={changeLanguage}>
          <option value="en">{i18n.t('language.en')}</option>
          <option value="zh-CN">{i18n.t('language.zh-CN')}</option>
        </select>
      </label>
      <div class="runtime">
        <span class:online={app.bridgeReady} class="runtime-dot" aria-hidden="true"></span>
        <div>
          <strong>{app.bridgeReady ? i18n.t('sidebar.native') : i18n.t('sidebar.unavailable')}</strong>
          <span>{app.snapshot?.product.version ?? i18n.t('sidebar.connecting')}</span>
        </div>
      </div>
      <div class="utility-controls">
        <button
          class="theme-toggle"
          type="button"
          aria-label={i18n.t(theme === 'dark' ? 'theme.useLight' : 'theme.useDark')}
          aria-pressed={theme === 'dark'}
          title={i18n.t(theme === 'dark' ? 'theme.useLight' : 'theme.useDark')}
          onclick={toggleTheme}
        >
          {#if theme === 'dark'}
            <Sun size={16} aria-hidden="true" />
          {:else}
            <Moon size={16} aria-hidden="true" />
          {/if}
          <span>{i18n.t(theme === 'dark' ? 'theme.light' : 'theme.dark')}</span>
        </button>
        <button
          class="refresh"
          type="button"
          aria-label={i18n.t('sidebar.refresh')}
          disabled={app.loading}
          onclick={() => app.refresh()}
        >
          <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
          <span class="refresh-label">{i18n.t('sidebar.refresh')}</span>
        </button>
      </div>
    </div>
  </aside>

  <main id="main-content" tabindex="-1">
    <ProjectWorkspaceBar />
    {@render children()}
  </main>
</div>

{#if app.notice}
  <div class="notice-layer">
    {#key app.notice}
      <FeedbackBanner notice={app.notice} onDismiss={() => app.dismissNotice()} />
    {/key}
  </div>
{/if}

{#if app.preview}
  <ConfirmationDialog
    preview={app.preview}
    intake={app.captureIntakePreview}
    consolidation={app.captureConsolidationPreview}
    acknowledgement={app.captureDeliveryAcknowledgementPreview}
    assignment={app.captureAssignmentPreview}
    resolution={app.captureResolutionPreview}
    resolutionSelections={app.captureResolutionSelections}
    portfolioMaintenance={app.portfolioMaintenancePreview}
    returnFocusTarget={previewFocusTarget}
    busy={app.loading}
    onConfirm={confirmOperation}
    onCancel={cancelOperation}
  />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: 224px minmax(0, 1fr);
    min-height: 100vh;
  }

  .skip-link {
    position: fixed;
    top: 8px;
    left: 8px;
    z-index: 100;
    transform: translateY(-160%);
    border-radius: 8px;
    padding: 8px 12px;
    color: var(--color-on-accent);
    background: var(--color-ink-strong);
    font-size: 12px;
    font-weight: 750;
    text-decoration: none;
  }

  .skip-link:focus { transform: translateY(0); }

  aside {
    --glass-base: var(--color-sidebar);
    --glass-highlight-angle: 118deg;
    --glass-highlight-stop: 42%;
    --glass-tint-angle: 300deg;
    --glass-tint-stop: 58%;

    position: sticky;
    top: 0;
    display: flex;
    height: 100vh;
    overflow-y: auto;
    flex-direction: column;
    border-right: 1px solid var(--glass-border);
    padding: 18px 14px 14px;
    box-shadow: var(--shadow-glass);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 7px 20px;
  }

  .mark {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border-radius: 6px;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .brand strong,
  .brand span {
    display: block;
  }

  .brand strong {
    color: var(--color-ink-strong);
    font-size: 16px;
    font-weight: 680;
    letter-spacing: -0.015em;
  }

  .brand span {
    margin-top: 1px;
    color: var(--color-muted);
    font-size: 11px;
    font-weight: 500;
  }

  nav p {
    margin: 0 9px 7px;
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  nav a {
    position: relative;
    display: flex;
    min-height: 40px;
    align-items: center;
    gap: 10px;
    margin-bottom: 2px;
    border: 1px solid transparent;
    border-radius: 10px;
    padding: 9px 10px;
    color: var(--color-muted);
    font-size: 13px;
    font-weight: 560;
    text-decoration: none;
    transition: background-color 140ms ease, color 140ms ease;
  }

  nav a:hover {
    color: var(--color-ink-strong);
    background: var(--glass-control-background-hover);
  }

  nav a[aria-current='page'] {
    color: var(--color-accent-strong);
    border-color: var(--glass-border);
    background: var(--glass-control-background-hover);
    box-shadow:
      0 0 0 0.5px var(--glass-outline),
      inset 0 1px 0 var(--glass-highlight-soft),
      inset 0 -1px 0 var(--glass-shade);
    font-weight: 650;
  }

  .sidebar-footer {
    margin-top: auto;
    border-top: 1px solid var(--color-border);
    padding-top: 12px;
  }

  .language-control {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 4px 7px;
    margin-bottom: 8px;
    padding: 5px 7px 8px;
    color: var(--color-muted);
  }

  .language-control span {
    font-size: 10px;
    font-weight: 600;
  }

  .language-control select {
    grid-column: 1 / -1;
    width: 100%;
    min-height: 36px;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    padding: 3px 8px;
    color: var(--color-ink);
    background: var(--glass-control-background);
    box-shadow:
      0 0 0 0.5px var(--glass-outline),
      inset 0 1px 0 var(--glass-highlight-soft);
    font: inherit;
    font-size: 11px;
  }

  .runtime {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 9px;
    padding: 7px 8px 11px;
  }

  .runtime-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-danger);
  }

  .runtime-dot.online {
    background: var(--color-success);
  }

  .runtime strong,
  .runtime span {
    display: block;
  }

  .runtime strong {
    color: var(--color-ink);
    font-size: 12px;
    font-weight: 620;
  }

  .runtime span {
    margin-top: 2px;
    color: var(--color-muted);
    font-size: 11px;
  }

  .utility-controls {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 7px;
  }

  .refresh,
  .theme-toggle {
    display: flex;
    width: 100%;
    min-height: 38px;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    color: var(--color-ink);
    background: var(--glass-control-background);
    box-shadow:
      0 0 0 0.5px var(--glass-outline),
      inset 0 1px 0 var(--glass-highlight-soft),
      inset 0 -1px 0 var(--glass-shade);
    font-size: 12px;
    font-weight: 600;
  }

  .refresh:hover:not(:disabled),
  .theme-toggle:hover { background: var(--glass-control-background-hover); }

  :global(.spin) {
    animation: spin 900ms linear infinite;
  }

  main {
    width: 100%;
    max-width: 1540px;
    min-width: 0;
    justify-self: center;
    padding: 26px clamp(22px, 3vw, 44px) 42px;
  }

  .notice-layer {
    position: fixed;
    z-index: var(--z-banner);
    top: 16px;
    right: clamp(16px, 2.4vw, 32px);
    width: min(440px, calc(100vw - 32px));
    pointer-events: none;
  }

  .notice-layer :global(.banner) {
    pointer-events: auto;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 900px) {
    .shell { grid-template-columns: 200px minmax(0, 1fr); }
    main { padding-inline: 20px; }
  }

  @media (max-width: 760px) {
    .shell { display: block; }
    aside {
      position: static;
      height: auto;
      overflow: visible;
      border-right: 0;
      border-bottom: 1px solid var(--color-border);
      padding: 12px 14px;
      background: var(--color-sidebar-strong);
    }
    .brand { padding: 0 2px 10px; }
    nav {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(min(122px, 100%), 1fr));
      gap: 5px;
      min-width: 0;
      padding: 0;
    }
    nav p { display: none; }
    nav a {
      justify-content: center;
      margin: 0;
      padding-inline: 10px;
      text-align: center;
      white-space: normal;
    }
    .sidebar-footer {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: stretch;
      gap: 7px;
      margin-top: 7px;
      padding-top: 8px;
    }
    .language-control {
      grid-template-columns: auto minmax(0, 1fr);
      gap: 6px;
      margin: 0;
      padding: 5px 6px;
    }
    .language-control > span { display: none; }
    .language-control select {
      grid-column: auto;
      min-width: 0;
      padding-inline: 5px;
    }
    .runtime {
      grid-column: 1 / -1;
      grid-row: 2;
      min-width: 0;
      padding: 5px 7px;
    }
    .runtime div { min-width: 0; }
    .runtime strong,
    .runtime span {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .utility-controls {
      grid-column: 2;
      grid-row: 1;
      grid-template-columns: 44px 44px;
    }
    .refresh,
    .theme-toggle { width: 44px; padding: 0; }
    .refresh-label,
    .theme-toggle span { display: none; }
    main { padding: 24px 16px 42px; }
    .notice-layer { top: 12px; right: 12px; left: 12px; width: auto; }
  }

  @media (max-width: 440px) {
    aside { padding-inline: 10px; }
    .brand span { display: none; }
    .sidebar-footer { gap: 5px; }
    .runtime { gap: 6px; padding-inline: 5px; }
    .runtime strong { font-size: 10px; }
    .runtime span { font-size: var(--font-size-label); }
    main { padding-inline: 12px; }
  }
</style>
