<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';

  import '../app.css';
  import { AppSidebar, FeedbackBanner, ProjectWorkspaceBar } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import { isProjectWorkspaceRoute } from '$lib/features/project-workspace';
  import { provideAppState, provideProjectWorkspace } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  let { children } = $props();
  const app = provideAppState();
  const projectWorkspace = provideProjectWorkspace();
  let previewFocusTarget = $state<HTMLElement | null>(null);
  let theme = $state<'light' | 'dark'>('light');

  const THEME_STORAGE_KEY = 'qiongli.theme';

  onMount(() => {
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

  function applyTheme(nextTheme: 'light' | 'dark', persist = true): void {
    theme = nextTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = nextTheme;
      document.querySelector<HTMLMetaElement>('meta[name="theme-color"]')
        ?.setAttribute('content', nextTheme === 'dark' ? '#0a0a0a' : '#ffffff');
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
    if (target && !target.closest('[role="dialog"], [role="alertdialog"]')) previewFocusTarget = target;
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

<Sidebar.Provider>
  <a class="skip-link" href="#main-content">{i18n.t('nav.skip')}</a>
  <AppSidebar currentPath={page.url.pathname} {theme} onToggleTheme={toggleTheme} />
  <Sidebar.Inset id="main-content" tabindex={-1} class="shell-main">
    <header class="mobile-shell-bar">
      <Sidebar.Trigger aria-label={i18n.t('nav.primary')} />
      <strong>Qiongli</strong>
    </header>
    <ProjectWorkspaceBar />
    {@render children()}
  </Sidebar.Inset>
</Sidebar.Provider>

{#if app.notice}
  <div class="notice-layer">
    {#key app.notice}
      <FeedbackBanner notice={app.notice} onDismiss={() => app.dismissNotice()} />
    {/key}
  </div>
{/if}

{#if app.preview}
  {#await import('$lib/components/app/ConfirmationDialog.svelte')}
    <div class="dialog-loading-scrim" role="status" aria-live="polite">
      <span class="dialog-loading-panel">{i18n.t('common.loading')}</span>
    </div>
  {:then { default: ConfirmationDialog }}
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
  {:catch}
    <div class="dialog-loading-scrim" role="alert">
      <span class="dialog-load-failed dialog-loading-panel">
        <strong>{i18n.t('notice.actionFailed')}</strong>
        <Button variant="outline" onclick={cancelOperation}>{i18n.t('common.close')}</Button>
      </span>
    </div>
  {/await}
{/if}

<style>
  :global([data-slot='sidebar-wrapper']) { min-width: 0; }

  :global(.shell-main) {
    width: 100%;
    max-width: 1600px;
    min-width: 0;
    margin-inline: auto;
    padding: var(--ui-page-padding-top) var(--ui-page-padding-inline) var(--ui-page-padding-bottom);
  }

  .mobile-shell-bar { display: none; }

  .skip-link {
    position: fixed;
    top: 8px;
    left: 8px;
    z-index: 100;
    transform: translateY(-160%);
    border-radius: var(--radius-control);
    padding: 6px 10px;
    color: var(--color-on-accent);
    background: var(--color-ink-strong);
    font-size: var(--font-size-supporting);
    font-weight: 750;
    text-decoration: none;
  }

  .skip-link:focus { transform: translateY(0); }

  :global(.spin) {
    animation: spin 900ms linear infinite;
  }

  .notice-layer {
    position: fixed;
    z-index: var(--z-banner);
    top: 12px;
    right: clamp(12px, 2vw, 24px);
    width: min(400px, calc(100vw - 24px));
    pointer-events: none;
  }

  .notice-layer :global(.banner) {
    pointer-events: auto;
  }

  .dialog-loading-scrim {
    position: fixed;
    z-index: var(--z-dialog-scrim);
    inset: 0;
    display: grid;
    place-items: center;
    padding: 20px;
    background: var(--color-scrim);
  }

  .dialog-loading-panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-dialog);
    padding: 14px 16px;
    color: var(--color-ink);
    background: var(--color-surface);
    box-shadow: var(--shadow-overlay);
    font-size: var(--font-size-supporting);
    font-weight: 720;
  }

  .dialog-load-failed {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 900px) {
    :global(.shell-main) { padding: 14px 12px 22px; }
  }

  @media (max-width: 767px) {
    :global(.shell-main) { padding: 0 8px 18px; }
    .mobile-shell-bar {
      position: sticky;
      top: 0;
      z-index: var(--z-sticky);
      display: flex;
      min-width: 0;
      align-items: center;
      gap: var(--space-2);
      margin-inline: -8px;
      border-bottom: 1px solid var(--color-border);
      padding: 5px 8px;
      color: var(--color-ink-strong);
      background: var(--color-sidebar-strong);
    }
    .notice-layer { top: 12px; right: 12px; left: 12px; width: auto; }
  }

  @media (max-width: 440px) {
    :global(.shell-main) { padding-inline: 6px; }
    .mobile-shell-bar { margin-inline: -6px; padding-inline: 6px; }
  }
</style>
