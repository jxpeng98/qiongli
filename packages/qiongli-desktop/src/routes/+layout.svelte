<script lang="ts">
  import { BookOpenText, Boxes, Cable, Inbox, LayoutDashboard, Network, RefreshCw } from '@lucide/svelte';
  import { page } from '$app/state';
  import { onMount } from 'svelte';

  import '../app.css';
  import { ConfirmationDialog, FeedbackBanner } from '$lib/shared/ui';
  import { provideAppState } from '$lib/context';

  let { children } = $props();
  const app = provideAppState();

  const navigation = [
    { href: '/overview', label: 'Overview', icon: LayoutDashboard },
    { href: '/research-library', label: 'Research Library', icon: BookOpenText },
    { href: '/captures', label: 'Capture Inbox', icon: Inbox },
    { href: '/workflow-content', label: 'Workflow Content', icon: Boxes },
    { href: '/client-integrations', label: 'Client Integrations', icon: Cable }
  ];

  onMount(() => {
    void app.refresh();
  });

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
  <aside>
    <div class="brand">
      <div class="mark" aria-hidden="true"><Network size={23} strokeWidth={1.9} /></div>
      <div>
        <strong>Qiongli</strong>
        <span>Research system</span>
      </div>
    </div>

    <nav aria-label="Primary navigation">
      <p>Workspace</p>
      {#each navigation as item}
        <a href={item.href} aria-current={page.url.pathname === item.href ? 'page' : undefined}>
          <item.icon size={18} strokeWidth={1.9} aria-hidden="true" />
          {item.label}
        </a>
      {/each}
    </nav>

    <div class="sidebar-footer">
      <div class="runtime">
        <span class:online={app.bridgeReady} class="runtime-dot" aria-hidden="true"></span>
        <div>
          <strong>{app.bridgeReady ? 'Native service' : 'Bridge unavailable'}</strong>
          <span>{app.snapshot?.product.version ?? 'Connecting…'}</span>
        </div>
      </div>
      <button class="refresh" type="button" disabled={app.loading} onclick={() => app.refresh()}>
        <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        Refresh status
      </button>
    </div>
  </aside>

  <main>
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
    busy={app.loading}
    onConfirm={confirmOperation}
    onCancel={cancelOperation}
  />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: 232px minmax(0, 1fr);
    min-height: 100vh;
  }

  aside {
    position: sticky;
    top: 0;
    display: flex;
    height: 100vh;
    flex-direction: column;
    border-right: 1px solid var(--color-border);
    padding: 22px 16px 16px;
    background: rgb(255 255 255 / 0.9);
    backdrop-filter: blur(18px);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 0 7px 24px;
  }

  .mark {
    display: grid;
    width: 40px;
    height: 40px;
    place-items: center;
    border-radius: 12px;
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
    min-height: 42px;
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
    padding: 38px clamp(28px, 4vw, 64px) 64px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 900px) {
    .shell { grid-template-columns: 200px minmax(0, 1fr); }
    main { padding-inline: 24px; }
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
