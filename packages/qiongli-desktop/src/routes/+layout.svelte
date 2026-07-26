<script lang="ts">
  import { BookOpenText, Boxes, Cable, CalendarClock, Database, GitBranch, Inbox, Info, Languages, LayoutDashboard, Network, RefreshCw } from '@lucide/svelte';
  import { page } from '$app/state';
  import { onMount } from 'svelte';

  import '../app.css';
  import { ConfirmationDialog, FeedbackBanner } from '$lib/shared/ui';
  import { provideAppState } from '$lib/context';
  import { i18n, type Locale } from '$lib/i18n.svelte';

  let { children } = $props();
  const app = provideAppState();

  const navigation = [
    { href: '/overview', label: 'nav.overview', icon: LayoutDashboard },
    { href: '/research-library', label: 'nav.library', icon: BookOpenText },
    { href: '/academic-graph', label: 'nav.graph', icon: Network },
    { href: '/captures', label: 'nav.captures', icon: Inbox },
    { href: '/portfolio', label: 'nav.portfolio', icon: Database },
    { href: '/timeline', label: 'nav.timeline', icon: CalendarClock },
    { href: '/workflow-content', label: 'nav.content', icon: Boxes },
    { href: '/orchestrator', label: 'nav.orchestrator', icon: GitBranch },
    { href: '/client-integrations', label: 'nav.integrations', icon: Cable },
    { href: '/about', label: 'nav.about', icon: Info }
  ];

  onMount(() => {
    i18n.initialize();
    void app.refresh();
  });

  function changeLanguage(event: Event): void {
    const locale = (event.currentTarget as HTMLSelectElement).value as Locale;
    if (locale === i18n.locale) return;
    app.dismissNotice();
    i18n.setLocale(locale);
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
  <aside>
    <div class="brand">
      <div class="mark" aria-hidden="true"><Network size={23} strokeWidth={1.9} /></div>
      <div>
        <strong>Qiongli</strong>
        <span>{i18n.t('app.subtitle')}</span>
      </div>
    </div>

    <nav aria-label={i18n.t('nav.primary')}>
      <p>{i18n.t('nav.workspace')}</p>
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
      <button class="refresh" type="button" disabled={app.loading} onclick={() => app.refresh()}>
        <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('sidebar.refresh')}
      </button>
    </div>
  </aside>

  <main id="main-content" tabindex="-1">
    {#if app.notice}
      <FeedbackBanner notice={app.notice} onDismiss={() => app.dismissNotice()} />
    {/if}
    {@render children()}
  </main>
</div>

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
    busy={app.loading}
    onConfirm={confirmOperation}
    onCancel={cancelOperation}
  />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: 208px minmax(0, 1fr);
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
    padding: 16px 12px 12px;
    background: rgb(255 255 255 / 0.9);
    backdrop-filter: blur(18px);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 0 6px 14px;
  }

  .mark {
    display: grid;
    width: 36px;
    height: 36px;
    place-items: center;
    border-radius: 10px;
    color: white;
    background: var(--color-ink-strong);
    box-shadow: 0 8px 20px rgb(2 6 23 / 0.16);
  }

  .brand strong,
  .brand span {
    display: block;
  }

  .brand strong {
    color: var(--color-ink-strong);
    font-size: 17px;
    letter-spacing: -0.02em;
  }

  .brand span {
    margin-top: 2px;
    color: var(--color-muted);
    font-size: 11px;
    font-weight: 650;
  }

  nav p {
    margin: 0 9px 8px;
    color: #64748b;
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  nav a {
    display: flex;
    min-height: 40px;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
    border: 1px solid transparent;
    border-radius: 10px;
    padding: 9px 10px;
    color: #334155;
    font-size: 13px;
    font-weight: 680;
    text-decoration: none;
  }

  nav a:hover {
    border-color: var(--color-border);
    background: var(--color-surface-subtle);
  }

  nav a[aria-current='page'] {
    border-color: #bae6fd;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .sidebar-footer {
    margin-top: auto;
    border-top: 1px solid var(--color-border);
    padding-top: 14px;
  }

  .language-control {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 4px 7px;
    margin-bottom: 8px;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    padding: 7px 8px;
    color: var(--color-muted);
    background: var(--color-surface-subtle);
  }

  .language-control span {
    font-size: 10px;
    font-weight: 750;
  }

  .language-control select {
    grid-column: 1 / -1;
    width: 100%;
    min-height: 30px;
    border: 1px solid var(--color-border-strong);
    border-radius: 7px;
    padding: 3px 7px;
    color: var(--color-ink);
    background: white;
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
    box-shadow: 0 0 0 4px rgb(4 120 87 / 0.1);
  }

  .runtime strong,
  .runtime span {
    display: block;
  }

  .runtime strong {
    color: var(--color-ink);
    font-size: 12px;
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
    border-radius: 9px;
    color: var(--color-ink);
    background: white;
    font-size: 12px;
    font-weight: 700;
  }

  :global(.spin) {
    animation: spin 900ms linear infinite;
  }

  main {
    min-width: 0;
    padding: 20px clamp(18px, 2.6vw, 34px) 22px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 900px) {
    .shell { grid-template-columns: 192px minmax(0, 1fr); }
    main { padding-inline: 20px; }
  }

  @media (max-width: 700px) {
    .shell { display: block; }
    aside {
      position: static;
      height: auto;
      border-right: 0;
      border-bottom: 1px solid var(--color-border);
      padding: 14px 16px;
    }
    .brand { padding: 0 4px 13px; }
    nav { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 5px; }
    nav p { display: none; }
    nav a { justify-content: center; margin: 0; padding-inline: 7px; text-align: center; }
    .sidebar-footer { display: grid; grid-template-columns: minmax(0, 1fr) minmax(140px, 0.7fr); align-items: center; gap: 8px; margin-top: 12px; padding-top: 10px; }
    .runtime { padding: 5px 8px; }
    main { padding: 26px 18px 46px; }
  }

  @media (max-width: 440px) {
    nav { grid-template-columns: 1fr; }
    nav a { justify-content: flex-start; }
    .sidebar-footer { grid-template-columns: 1fr; }
  }
</style>
