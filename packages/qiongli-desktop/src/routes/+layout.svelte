<script lang="ts">
  import { BookOpenText, Cable, Database, Info, Languages, LayoutDashboard, Network, RefreshCw } from '@lucide/svelte';
  import { page } from '$app/state';
  import { onMount } from 'svelte';

  import '../app.css';
  import { ConfirmationDialog, FeedbackBanner } from '$lib/shared/ui';
  import ProjectWorkspaceBar from '$lib/features/project-workspace/ProjectWorkspaceBar.svelte';
  import { isProjectWorkspaceRoute } from '$lib/features/project-workspace';
  import { provideAppState, provideProjectWorkspace } from '$lib/context';
  import { i18n, type Locale } from '$lib/i18n.svelte';

  let { children } = $props();
  const app = provideAppState();
  const projectWorkspace = provideProjectWorkspace();
  let previewFocusTarget = $state<HTMLElement | null>(null);
  let primaryNavigation = $state<HTMLElement | null>(null);

  const navigation = [
    { href: '/overview', label: 'nav.overview', icon: LayoutDashboard },
    { href: '/research-library', label: 'nav.library', icon: BookOpenText },
    { href: '/portfolio', label: 'nav.portfolio', icon: Database },
    { href: '/client-integrations', label: 'nav.integrations', icon: Cable },
    { href: '/about', label: 'nav.about', icon: Info }
  ];

  onMount(() => {
    i18n.initialize();
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
    page.url.pathname;
    if (
      !primaryNavigation
      || typeof window === 'undefined'
      || !window.matchMedia('(max-width: 700px)').matches
    ) return;
    const frame = window.requestAnimationFrame(() => {
      primaryNavigation?.querySelector<HTMLElement>('[aria-current="page"]')
        ?.scrollIntoView({ block: 'nearest', inline: 'center' });
    });
    return () => window.cancelAnimationFrame(frame);
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
  <aside class="glass-material">
    <div class="brand">
      <div class="mark" aria-hidden="true"><Network size={23} strokeWidth={1.9} /></div>
      <div>
        <strong>Qiongli</strong>
        <span>{i18n.t('app.subtitle')}</span>
      </div>
    </div>

    <nav bind:this={primaryNavigation} aria-label={i18n.t('nav.primary')}>
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
    color: white;
    background: var(--color-ink-strong);
    font-size: 12px;
    font-weight: 750;
    text-decoration: none;
  }

  .skip-link:focus { transform: translateY(0); }

  aside {
    position: sticky;
    top: 0;
    display: flex;
    height: 100vh;
    overflow-y: auto;
    flex-direction: column;
    border-right: 1px solid var(--color-border);
    padding: 18px 14px 14px;
    background: rgb(235 234 229 / 0.72);
    box-shadow:
      inset -1px 0 0 rgb(255 255 255 / 0.56),
      10px 0 34px rgb(44 48 43 / 0.035);
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
    background: rgb(220 229 225 / 0.74);
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
    border-radius: 5px;
    padding: 9px 10px;
    color: #4c4a44;
    font-size: 13px;
    font-weight: 560;
    text-decoration: none;
    transition: background-color 140ms ease, color 140ms ease;
  }

  nav a:hover {
    color: var(--color-ink-strong);
    background: rgb(255 255 255 / 0.5);
  }

  nav a[aria-current='page'] {
    color: var(--color-accent-strong);
    background: rgb(255 255 255 / 0.5);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.68);
    font-weight: 650;
  }

  nav a[aria-current='page']::before {
    position: absolute;
    top: 9px;
    bottom: 9px;
    left: -7px;
    width: 2px;
    border-radius: 1px;
    background: var(--color-accent);
    content: '';
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
    border-radius: 5px;
    padding: 3px 8px;
    color: var(--color-ink);
    background: var(--glass-control);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.66);
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

  .refresh {
    display: flex;
    width: 100%;
    min-height: 38px;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid var(--color-border);
    border-radius: 5px;
    color: var(--color-ink);
    background: var(--glass-control);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.66);
    font-size: 12px;
    font-weight: 600;
  }

  .refresh:hover:not(:disabled) { background: var(--glass-control-hover); }

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

  @media (max-width: 700px) {
    .shell { display: block; }
    aside {
      position: static;
      height: auto;
      overflow: hidden;
      border-right: 0;
      border-bottom: 1px solid var(--color-border);
      padding: 12px 14px;
      background: rgb(235 234 229 / 0.82);
    }
    .brand { padding: 0 2px 10px; }
    nav {
      display: flex;
      overflow-x: auto;
      overscroll-behavior-inline: contain;
      gap: 5px;
      padding: 0 0 5px;
      scroll-snap-type: inline proximity;
      scroll-behavior: smooth;
      scrollbar-width: thin;
    }
    nav p { display: none; }
    nav a {
      flex: 0 0 auto;
      justify-content: center;
      margin: 0;
      padding-inline: 10px;
      scroll-snap-align: center;
      text-align: center;
      white-space: nowrap;
    }
    nav a[aria-current='page']::before { display: none; }
    .sidebar-footer {
      display: grid;
      grid-template-columns: minmax(136px, 1fr) minmax(120px, .75fr) 44px;
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
    .refresh {
      width: 44px;
      padding: 0;
    }
    .refresh-label { display: none; }
    main { padding: 24px 16px 42px; }
    .notice-layer { top: 12px; right: 12px; left: 12px; width: auto; }
  }

  @media (max-width: 440px) {
    aside { padding-inline: 10px; }
    .brand span { display: none; }
    .sidebar-footer {
      grid-template-columns: minmax(126px, 1fr) minmax(108px, .8fr) 44px;
      gap: 5px;
    }
    .runtime { gap: 6px; padding-inline: 5px; }
    .runtime strong { font-size: 10px; }
    .runtime span { font-size: var(--font-size-label); }
    main { padding-inline: 12px; }
  }
</style>
