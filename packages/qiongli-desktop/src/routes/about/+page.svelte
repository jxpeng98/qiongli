<script lang="ts">
  import { ArrowDownToLine, CheckCircle2, Download, Info, RefreshCw, RotateCcw, ShieldCheck, TerminalSquare } from '@lucide/svelte';
  import { onDestroy } from 'svelte';

  import type { AppIntent, AppSnapshot, UpdateView } from '@qiongli/app-api';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';
  import { createUpdatePollingController, type UpdatePollResult } from '$lib/update-polling';

  const app = useAppState();
  let update = $derived(app.snapshot?.update ?? null);
  let pollingPaused = $state(false);
  let cliTestResult = $state<AppSnapshot['cli'] | null>(null);
  let closeAttempted = false;

  function updateLabel(value: UpdateView['phase']): string {
    return i18n.t(`update.${value}`);
  }

  function byteSize(value: number | null): string {
    if (value === null) return '—';
    return new Intl.NumberFormat(i18n.locale === 'zh-CN' ? 'zh-CN' : 'en-GB', {
      style: 'unit', unit: 'megabyte', unitDisplay: 'short', maximumFractionDigits: 1
    }).format(value / 1_000_000);
  }

  function isBusy(value: UpdateView): boolean {
    return ['checking', 'downloading', 'verifying', 'staging', 'installing', 'awaiting-restart', 'cancelling'].includes(value.phase);
  }

  function cliStateLabel(state: AppSnapshot['cli']['state']): string {
    return i18n.t(`about.cliState.${state}`);
  }

  function cliPathLabel(state: AppSnapshot['cli']['pathState']): string {
    return i18n.t(`about.cliPath.${state}`);
  }

  async function previewCliInstall(): Promise<void> {
    cliTestResult = null;
    await app.execute({ action: 'preview-cli-install' });
  }

  async function refreshCliStatus(): Promise<void> {
    cliTestResult = null;
    await app.refresh();
  }

  async function testCliCommand(): Promise<void> {
    cliTestResult = null;
    const event = await app.execute({ action: 'test-cli-command' });
    if (event?.type === 'snapshot') cliTestResult = event.snapshot.cli;
  }

  function selectUpdateStream(event: Event): void {
    const input = event.currentTarget as HTMLInputElement;
    if (!input.checked) return;
    void executeUpdate({
      action: 'select-update-stream',
      stream: input.value as 'stable' | 'beta'
    });
  }

  const updatePolling = createUpdatePollingController({
    async poll(): Promise<UpdatePollResult> {
      // AppState currently exposes one loading flag, so never overlap polling
      // with a refresh or another intent that already owns the native bridge.
      if (app.loading) return 'busy';
      const event = await app.execute({ action: 'poll-update' });
      if (event?.type !== 'update-changed') return 'failed';
      return isBusy(event.update) ? 'busy' : 'settled';
    },
    onPauseChange(paused) {
      pollingPaused = paused;
    }
  });

  $effect(() => {
    updatePolling.sync(update !== null && isBusy(update));
  });

  let updateAnnouncement = $derived.by(() => {
    if (!update) return '';
    if (!update.progress || update.progress.indeterminate) return updateLabel(update.phase);
    const percent = Math.min(
      100,
      Math.floor(update.progress.completedSteps / update.progress.totalSteps * 4) * 25
    );
    return i18n.t('about.progressAnnouncement', {
      phase: updateLabel(update.phase),
      percent
    });
  });

  $effect(() => {
    if (!app.closeRequested) {
      closeAttempted = false;
      return;
    }
    if (closeAttempted || typeof window === 'undefined') return;
    closeAttempted = true;
    void import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow().close())
      .catch(() => {
        app.notice = {
          tone: 'danger',
          title: i18n.t('notice.updateCloseFailed'),
          detail: i18n.t('notice.updateCloseFailedDetail')
        };
      });
  });

  onDestroy(() => updatePolling.destroy());

  async function executeUpdate(intent: AppIntent): Promise<void> {
    const event = await app.execute(intent);
    if (event?.type === 'update-changed' && isBusy(event.update)) {
      updatePolling.sync(true);
      updatePolling.retry();
    }
  }
</script>

<svelte:head>
  <title>{i18n.t('about.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('about.eyebrow')}
  title={i18n.t('about.title')}
  description={i18n.t('about.description')}
/>

{#if !app.snapshot || !update}
  <section
    class="surface loading"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >{i18n.t('common.loading')}</section>
{:else}
  <div class="about-grid">
    <section class="surface product-card">
      <div class="section-heading"><span class="icon"><Info size={20} aria-hidden="true" /></span><div><p class="eyebrow">Qiongli</p><h2>{i18n.t('about.product')}</h2></div></div>
      <dl>
        <div><dt>{i18n.t('common.version')}</dt><dd>{app.snapshot.product.version}</dd></div>
        <div><dt>{i18n.t('about.build')}</dt><dd>{app.snapshot.product.build}</dd></div>
        <div><dt>{i18n.t('about.system')}</dt><dd>{app.snapshot.product.operatingSystem} · {app.snapshot.product.architecture}</dd></div>
        <div><dt>{i18n.t('about.authority')}</dt><dd>{i18n.dynamic(app.snapshot.product.trust.label)}<code>{app.snapshot.product.trust.reasonCode}</code></dd></div>
      </dl>
    </section>

    <section class="surface update-card">
      <header class="update-heading">
        <div class="section-heading"><span class="icon"><ArrowDownToLine size={20} aria-hidden="true" /></span><div><p class="eyebrow">{i18n.t('about.updateEyebrow')}</p><h2>{i18n.t('about.updates')}</h2><p>{i18n.t('about.updateDescription')}</p></div></div>
        <StatusBadge status={update.status} label={updateLabel(update.phase)} />
      </header>

      <fieldset class="stream-row">
        <legend>{i18n.t('about.stream')}</legend>
        <div class="stream-tabs">
          <label data-selected={update.selectedStream === 'stable'}>
            <input
              class="sr-only"
              type="radio"
              name="update-stream"
              value="stable"
              checked={update.selectedStream === 'stable'}
              disabled={!update.canSelectStream || app.loading}
              onchange={selectUpdateStream}
            />
            <span>{i18n.t('about.stable')}</span>
          </label>
          <label data-selected={update.selectedStream === 'beta'}>
            <input
              class="sr-only"
              type="radio"
              name="update-stream"
              value="beta"
              checked={update.selectedStream === 'beta'}
              disabled={!update.canSelectStream || app.loading}
              onchange={selectUpdateStream}
            />
            <span>{i18n.t('about.beta')}</span>
          </label>
        </div>
      </fieldset>

      <div class="update-facts">
        <div><span>{i18n.t('about.available')}</span><strong>{update.availableVersion ?? '—'}</strong></div>
        <div><span>{i18n.t('about.size')}</span><strong>{byteSize(update.archiveSizeBytes)}</strong></div>
      </div>

      <details class="update-technical">
        <summary>{i18n.t('common.details')}</summary>
        <span>{i18n.t('about.reason')}</span>
        <code>{update.reasonCode}</code>
      </details>

      {#if update.progress}
        <div class="update-progress">
          <div><span>{i18n.dynamic(update.progress.label)}</span><strong>{update.progress.indeterminate ? '…' : `${update.progress.completedSteps}/${update.progress.totalSteps}`}</strong></div>
          <progress
            max={update.progress.totalSteps}
            value={update.progress.indeterminate ? undefined : update.progress.completedSteps}
            aria-label={updateLabel(update.phase)}
          ></progress>
        </div>
      {:else if update.phase === 'unavailable'}
        <p class="packaged-note">
          <ShieldCheck size={16} aria-hidden="true" />
          {i18n.t(update.reasonCode === 'native-update-local-build-unavailable'
            ? 'about.localBuildUpdateUnavailable'
            : 'about.packagedOnly')}
        </p>
      {:else if update.phase === 'current'}
        <p class="current-note"><CheckCircle2 size={16} aria-hidden="true" />{updateLabel(update.phase)}</p>
      {/if}

      <p class="sr-only" role="status" aria-live="polite" aria-atomic="true">
        {updateAnnouncement}
      </p>

      {#if pollingPaused}
        <div class="polling-warning" role="alert">
          <div>
            <strong>{i18n.t('about.pollingPaused')}</strong>
            <span>{i18n.t('about.pollingPausedDetail')}</span>
          </div>
          <button class="button-secondary" type="button" onclick={() => updatePolling.retry()}>
            {i18n.t('about.retryStatus')}
          </button>
        </div>
      {/if}

      <div class="update-actions">
        <button class="button-secondary" type="button" disabled={!update.canCheck || app.loading} onclick={() => executeUpdate({ action: 'check-for-updates' })}><RefreshCw size={15} class={update.phase === 'checking' ? 'spin' : undefined} aria-hidden="true" />{i18n.t('about.check')}</button>
        <button class="button-secondary" type="button" disabled={!update.canPrepare || app.loading} onclick={() => executeUpdate({ action: 'prepare-update' })}><Download size={15} aria-hidden="true" />{i18n.t('about.prepare')}</button>
        <button class="button-primary" type="button" disabled={!update.canInstall || app.loading} onclick={() => executeUpdate({ action: 'preview-update-install' })}><ArrowDownToLine size={15} aria-hidden="true" />{i18n.t('about.install')}</button>
        <button class="button-quiet" type="button" disabled={!update.canCancel || app.loading} onclick={() => executeUpdate({ action: 'cancel-update' })}><RotateCcw size={15} aria-hidden="true" />{i18n.t('about.cancel')}</button>
      </div>
    </section>

    <section class="surface cli-card">
      <header class="cli-heading">
        <div class="section-heading">
          <span class="icon"><TerminalSquare size={20} aria-hidden="true" /></span>
          <div>
            <p class="eyebrow">{i18n.t('about.cliEyebrow')}</p>
            <h2>{i18n.t('about.cliTitle')}</h2>
            <p>{i18n.t('about.cliDescription')}</p>
          </div>
        </div>
        <StatusBadge status={app.snapshot.cli.status} label={cliStateLabel(app.snapshot.cli.state)} />
      </header>

      <div class="cli-facts">
        <div>
          <span>{i18n.t('about.cliInstalled')}</span>
          <strong>{app.snapshot.cli.installedVersion ?? i18n.t('integrations.notInstalled')}</strong>
        </div>
        <div>
          <span>{i18n.t('about.cliAvailable')}</span>
          <strong>{app.snapshot.cli.availableVersion}</strong>
        </div>
        <div>
          <span>{i18n.t('about.cliTarget')}</span>
          <code>{app.snapshot.cli.symbolicTarget}</code>
        </div>
        <div>
          <span>{i18n.t('about.cliPathStatus')}</span>
          <StatusBadge status={app.snapshot.cli.pathStatus} label={cliPathLabel(app.snapshot.cli.pathState)} />
        </div>
      </div>

      <div
        class="cli-guidance"
        class:ready={app.snapshot.cli.pathState === 'active' || app.snapshot.cli.pathState === 'configured'}
      >
        {#if app.snapshot.cli.pathState === 'not-configured'}
          <p>{i18n.t('about.cliPathNotConfigured')}</p>
          <code>export PATH="$HOME/.local/bin:$PATH"</code>
        {:else if app.snapshot.cli.pathState === 'configured'}
          <p>{i18n.t('about.cliPathConfigured')}</p>
          <code>command -v qiongli &amp;&amp; qiongli --version</code>
        {:else if app.snapshot.cli.pathState === 'shadowed'}
          <p>{i18n.t('about.cliPathShadowed')}</p>
          <code>type -a qiongli; "$HOME/.local/bin/qiongli" --version</code>
        {:else if app.snapshot.cli.pathState === 'version-mismatch'}
          <p>{i18n.t('about.cliPathVersionMismatch')}</p>
          <code>type -a qiongli; "$HOME/.local/bin/qiongli" --version</code>
        {:else if app.snapshot.cli.pathState === 'not-observable'}
          <p>{i18n.t('about.cliPathNotObservable')}</p>
          <code>command -v qiongli &amp;&amp; qiongli --version</code>
        {:else}
          <p>{i18n.t('about.cliPathActive')}</p>
        {/if}
        <details>
          <summary>{i18n.t('about.cliTechnicalDetail')}</summary>
          <small>{app.snapshot.cli.reasonCode}</small>
        </details>
      </div>

      <div class="cli-actions">
        <button
          class="button-primary"
          type="button"
          disabled={!app.snapshot.cli.canInstall || app.loading}
          onclick={previewCliInstall}
        >
          <ArrowDownToLine size={15} aria-hidden="true" />
          {app.snapshot.cli.state === 'missing' ? i18n.t('about.cliInstall') : i18n.t('about.cliUpdate')}
        </button>
        <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshCliStatus}>
          <RefreshCw size={15} aria-hidden="true" />{i18n.t('about.cliRefresh')}
        </button>
        <button
          class="button-secondary"
          type="button"
          disabled={app.loading || !app.snapshot.cli.canTest}
          onclick={testCliCommand}
        >
          <TerminalSquare size={15} aria-hidden="true" />{i18n.t('about.cliTest')}
        </button>
      </div>

      {#if cliTestResult}
        <div class="cli-test-result" role="status" aria-live="polite">
          <StatusBadge
            status={cliTestResult.pathStatus}
            label={cliPathLabel(cliTestResult.pathState)}
          />
          <span>{i18n.reason(cliTestResult.reasonCode)}</span>
        </div>
      {/if}
    </section>
  </div>
{/if}

<style>
  .loading { padding: 20px; color: var(--color-muted); }
  .about-grid { display: grid; grid-template-columns: minmax(250px, .72fr) minmax(430px, 1.4fr); gap: 12px; }
  .product-card, .update-card, .cli-card { padding: 16px; }
  .section-heading { display: flex; align-items: flex-start; gap: 10px; }
  .icon { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 9px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  dl { margin: 14px 0 0; }
  dl > div { display: grid; grid-template-columns: 90px minmax(0, 1fr); gap: 12px; border-top: 1px solid var(--color-border); padding: 10px 0; }
  dt { color: var(--color-muted); font-size: 10px; font-weight: 750; }
  dd { margin: 0; color: var(--color-ink); font-size: 11px; font-weight: 650; }
  dd code { display: block; margin-top: 3px; color: var(--color-muted); font-size: var(--font-size-micro); overflow-wrap: anywhere; }
  .update-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; }
  .update-heading .section-heading p:last-child { margin: 4px 0 0; color: var(--color-muted); font-size: 10px; line-height: 1.4; }
  .cli-card { grid-column: 1 / -1; }
  .cli-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; }
  .cli-heading .section-heading p:last-child { margin: 4px 0 0; color: var(--color-muted); font-size: 10px; line-height: 1.4; }
  .cli-facts { display: grid; grid-template-columns: .75fr .75fr 1.5fr 1fr; gap: 8px; margin-top: 14px; }
  .cli-facts > div { min-width: 0; border: 1px solid var(--color-border); border-radius: 9px; padding: 9px; background: var(--color-surface-subtle); }
  .cli-facts span, .cli-facts strong, .cli-facts code { display: block; }
  .cli-facts > div > span { color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 750; text-transform: uppercase; }
  .cli-facts strong, .cli-facts code { margin-top: 4px; overflow-wrap: anywhere; color: var(--color-ink-strong); font-size: 10px; }
  .cli-guidance { margin-top: 10px; border-radius: 9px; padding: 9px 10px; color: #854d0e; background: var(--color-warning-soft); }
  .cli-guidance.ready { color: var(--color-success); background: var(--color-success-soft); }
  .cli-guidance p { margin: 0; font-size: var(--font-size-label); line-height: 1.4; }
  .cli-guidance code { display: inline-block; margin-top: 6px; border-radius: 5px; padding: 3px 5px; color: inherit; background: rgb(255 255 255 / .64); font-size: var(--font-size-micro); }
  .cli-guidance details { margin-top: 6px; }
  .cli-guidance summary { width: fit-content; cursor: pointer; font-size: var(--font-size-micro); font-weight: 750; }
  .cli-guidance small { display: block; margin-top: 5px; color: inherit; font-family: var(--font-mono); font-size: var(--font-size-micro); opacity: .72; }
  .cli-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }
  .cli-actions button { min-height: 44px; font-size: 10px; }
  .cli-test-result { display: flex; align-items: center; gap: 9px; margin-top: 9px; border-top: 1px solid var(--color-border); padding-top: 9px; color: var(--color-muted); font-size: var(--font-size-label); }
  .stream-row { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 12px; margin: 14px 0 0; border: 0; border-block: 1px solid var(--color-border); padding: 9px 0; }
  .stream-row legend { float: left; padding: 0; color: var(--color-muted); font-size: 10px; font-weight: 750; }
  .stream-tabs { display: flex; border: 1px solid var(--color-border); border-radius: 8px; padding: 2px; background: var(--color-surface-subtle); }
  .stream-tabs label { display: inline-flex; min-height: 44px; align-items: center; border-radius: 6px; padding: 4px 13px; color: var(--color-muted); font-size: 10px; font-weight: 750; cursor: pointer; }
  .stream-tabs label[data-selected='true'] { color: var(--color-accent-strong); background: white; box-shadow: 0 1px 3px rgb(15 23 42 / .12); }
  .stream-tabs label:focus-within { outline: 2px solid var(--color-accent); outline-offset: 2px; }
  .stream-tabs label:has(input:disabled) { cursor: not-allowed; opacity: .48; }
  .update-facts { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; margin-top: 10px; }
  .update-facts > div { min-width: 0; border: 1px solid var(--color-border); border-radius: 9px; padding: 9px; background: var(--color-surface-subtle); }
  .update-facts span, .update-facts strong { display: block; }
  .update-facts span { color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 750; text-transform: uppercase; }
  .update-facts strong { margin-top: 4px; color: var(--color-ink-strong); font-size: 12px; }
  .update-technical { margin-top: 8px; color: var(--color-muted); }
  .update-technical summary { width: fit-content; min-height: 36px; padding-block: 8px; cursor: pointer; font-size: var(--font-size-micro); font-weight: 750; }
  .update-technical span, .update-technical code { display: block; }
  .update-technical span { font-size: var(--font-size-micro); font-weight: 750; text-transform: uppercase; }
  .update-technical code { margin-top: 4px; overflow-wrap: anywhere; color: var(--color-ink); font-size: var(--font-size-micro); }
  .packaged-note, .current-note { display: flex; align-items: flex-start; gap: 7px; margin: 10px 0 0; border-radius: 9px; padding: 9px 10px; color: #854d0e; background: var(--color-warning-soft); font-size: var(--font-size-label); line-height: 1.4; }
  .current-note { color: var(--color-success); background: var(--color-success-soft); }
  .polling-warning { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 10px; border: 1px solid #f59e0b; border-radius: 9px; padding: 9px 10px; color: #854d0e; background: var(--color-warning-soft); }
  .polling-warning strong, .polling-warning span { display: block; }
  .polling-warning strong { font-size: 10px; }
  .polling-warning span { margin-top: 2px; font-size: var(--font-size-label); line-height: 1.4; }
  .polling-warning button { flex: none; min-height: 44px; font-size: var(--font-size-label); }
  .update-progress { margin-top: 10px; }
  .update-progress div { display: flex; justify-content: space-between; color: var(--color-muted); font-size: var(--font-size-label); }
  progress { width: 100%; height: 7px; margin-top: 5px; accent-color: var(--color-accent); }
  .update-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }
  .update-actions button { min-height: 44px; font-size: 10px; }
  :global(.spin) { animation: spin 900ms linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (max-width: 900px) { .about-grid { grid-template-columns: 1fr; } }
  @media (max-width: 760px) { .cli-facts { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
  @media (max-width: 560px) { .update-heading, .cli-heading, .stream-row { align-items: flex-start; flex-direction: column; } .update-facts, .cli-facts { grid-template-columns: 1fr; } .update-actions, .cli-actions { align-items: stretch; flex-direction: column; } }
</style>
