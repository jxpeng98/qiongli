<script lang="ts">
  import { ArrowDownToLine, CheckCircle2, Download, Info, RefreshCw, RotateCcw, ShieldCheck } from '@lucide/svelte';
  import { onDestroy } from 'svelte';

  import type { AppIntent, UpdateView } from '@qiongli/app-api';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';
  import { createUpdatePollingController, type UpdatePollResult } from '$lib/update-polling';

  const app = useAppState();
  let update = $derived(app.snapshot?.update ?? null);
  let pollingPaused = $state(false);
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
        <div><span>{i18n.t('about.reason')}</span><code>{update.reasonCode}</code></div>
      </div>

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
        <p class="packaged-note"><ShieldCheck size={16} aria-hidden="true" />{i18n.t('about.packagedOnly')}</p>
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
  </div>
{/if}

<style>
  .loading { padding: 20px; color: var(--color-muted); }
  .about-grid { display: grid; grid-template-columns: minmax(250px, .72fr) minmax(430px, 1.4fr); gap: 12px; }
  .product-card, .update-card { padding: 16px; }
  .section-heading { display: flex; align-items: flex-start; gap: 10px; }
  .icon { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 9px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  dl { margin: 14px 0 0; }
  dl > div { display: grid; grid-template-columns: 90px minmax(0, 1fr); gap: 12px; border-top: 1px solid var(--color-border); padding: 10px 0; }
  dt { color: var(--color-muted); font-size: 10px; font-weight: 750; }
  dd { margin: 0; color: var(--color-ink); font-size: 11px; font-weight: 650; }
  dd code { display: block; margin-top: 3px; color: var(--color-muted); font-size: 8px; overflow-wrap: anywhere; }
  .update-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; }
  .update-heading .section-heading p:last-child { margin: 4px 0 0; color: var(--color-muted); font-size: 10px; line-height: 1.4; }
  .stream-row { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 12px; margin: 14px 0 0; border: 0; border-block: 1px solid var(--color-border); padding: 9px 0; }
  .stream-row legend { float: left; padding: 0; color: var(--color-muted); font-size: 10px; font-weight: 750; }
  .stream-tabs { display: flex; border: 1px solid var(--color-border); border-radius: 8px; padding: 2px; background: var(--color-surface-subtle); }
  .stream-tabs label { display: inline-flex; min-height: 44px; align-items: center; border-radius: 6px; padding: 4px 13px; color: var(--color-muted); font-size: 10px; font-weight: 750; cursor: pointer; }
  .stream-tabs label[data-selected='true'] { color: var(--color-accent-strong); background: white; box-shadow: 0 1px 3px rgb(15 23 42 / .12); }
  .stream-tabs label:focus-within { outline: 2px solid var(--color-accent); outline-offset: 2px; }
  .stream-tabs label:has(input:disabled) { cursor: not-allowed; opacity: .48; }
  .update-facts { display: grid; grid-template-columns: .8fr .8fr 1.4fr; gap: 8px; margin-top: 10px; }
  .update-facts > div { min-width: 0; border: 1px solid var(--color-border); border-radius: 9px; padding: 9px; background: var(--color-surface-subtle); }
  .update-facts span, .update-facts strong, .update-facts code { display: block; }
  .update-facts span { color: var(--color-muted); font-size: 8px; font-weight: 750; text-transform: uppercase; }
  .update-facts strong { margin-top: 4px; color: var(--color-ink-strong); font-size: 12px; }
  .update-facts code { margin-top: 4px; overflow-wrap: anywhere; color: var(--color-ink); font-size: 8px; }
  .packaged-note, .current-note { display: flex; align-items: flex-start; gap: 7px; margin: 10px 0 0; border-radius: 9px; padding: 9px 10px; color: #854d0e; background: var(--color-warning-soft); font-size: 9px; line-height: 1.4; }
  .current-note { color: var(--color-success); background: var(--color-success-soft); }
  .polling-warning { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 10px; border: 1px solid #f59e0b; border-radius: 9px; padding: 9px 10px; color: #854d0e; background: var(--color-warning-soft); }
  .polling-warning strong, .polling-warning span { display: block; }
  .polling-warning strong { font-size: 10px; }
  .polling-warning span { margin-top: 2px; font-size: 9px; line-height: 1.4; }
  .polling-warning button { flex: none; min-height: 44px; font-size: 9px; }
  .update-progress { margin-top: 10px; }
  .update-progress div { display: flex; justify-content: space-between; color: var(--color-muted); font-size: 9px; }
  progress { width: 100%; height: 7px; margin-top: 5px; accent-color: var(--color-accent); }
  .update-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }
  .update-actions button { min-height: 44px; font-size: 10px; }
  :global(.spin) { animation: spin 900ms linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (max-width: 900px) { .about-grid { grid-template-columns: 1fr; } }
  @media (max-width: 560px) { .update-heading, .stream-row { align-items: flex-start; flex-direction: column; } .update-facts { grid-template-columns: 1fr; } .update-actions { align-items: stretch; flex-direction: column; } }
</style>
