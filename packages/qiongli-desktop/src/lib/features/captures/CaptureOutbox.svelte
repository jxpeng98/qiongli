<script lang="ts">
  import type {
    AppIntent,
    CaptureDeliveryView
  } from '@qiongli/app-api';
  import {
    AlertTriangle,
    CheckCircle2,
    Clock3,
    RefreshCw,
    RotateCcw,
    Send,
    ShieldCheck,
    XCircle
  } from '@lucide/svelte';
  import { tick } from 'svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/shared/ui';
  import {
    deliveryNeedsAttention,
    deliveryStatus,
    prioritizeDeliveries
  } from '.';

  type CaptureDeliveryRetryCause = Extract<
    AppIntent,
    { action: 'retry-capture-delivery' }
  >['cause'];

  let {
    entries,
    currentProjectRevision,
    selectedEnvelopeId,
    loading,
    truncated,
    onInspect,
    onRetry,
    onCancel,
    onAcknowledge,
    onLoadMore
  }: {
    entries: CaptureDeliveryView[];
    currentProjectRevision: number;
    selectedEnvelopeId: string | null;
    loading: boolean;
    truncated: boolean;
    onInspect: (delivery: CaptureDeliveryView) => void;
    onRetry: (delivery: CaptureDeliveryView, cause: CaptureDeliveryRetryCause) => void;
    onCancel: (delivery: CaptureDeliveryView) => void;
    onAcknowledge: (delivery: CaptureDeliveryView, resultingProjectRevision: number) => void;
    onLoadMore: () => void;
  } = $props();

  let retryCauses = $state<Record<string, CaptureDeliveryRetryCause | ''>>({});
  let pendingCancellation = $state<string | null>(null);
  let cancellationTriggers: Record<string, HTMLButtonElement | undefined> = {};
  let keepButtons: Record<string, HTMLButtonElement | undefined> = {};

  let ordered = $derived(prioritizeDeliveries(entries));
  let attentionCount = $derived(entries.filter(deliveryNeedsAttention).length);

  function selectRetryCause(event: Event, envelopeId: string): void {
    retryCauses[envelopeId] =
      (event.currentTarget as HTMLSelectElement).value as CaptureDeliveryRetryCause | '';
  }

  function requestRetry(delivery: CaptureDeliveryView): void {
    const cause = retryCauses[delivery.envelopeId];
    if (cause) onRetry(delivery, cause);
  }

  async function requestCancellation(envelopeId: string): Promise<void> {
    pendingCancellation = envelopeId;
    await tick();
    keepButtons[envelopeId]?.focus();
  }

  async function keepDelivery(envelopeId: string): Promise<void> {
    pendingCancellation = null;
    await tick();
    cancellationTriggers[envelopeId]?.focus();
  }

  function handleCancellationKeydown(event: KeyboardEvent, envelopeId: string): void {
    if (event.key !== 'Escape') return;
    event.preventDefault();
    void keepDelivery(envelopeId);
  }
</script>

<div
  id="capture-panel-outbox"
  class="surface outbox"
  role="tabpanel"
  aria-labelledby="capture-tab-outbox"
>
  <div class="heading">
    <div>
      <p class="eyebrow">{i18n.t('captures.outboxEyebrow')}</p>
      <h2>{i18n.t('captures.outboxTitle')}</h2>
      <p>{i18n.t('captures.outboxSummary', {
        total: entries.length,
        attention: attentionCount
      })}</p>
    </div>
    <StatusBadge
      status={attentionCount > 0 ? 'attention' : 'ready'}
      label={attentionCount > 0
        ? i18n.t('captures.outboxAttention', { count: attentionCount })
        : i18n.t('captures.outboxClear')}
    />
  </div>

  {#if entries.length === 0}
    <div class="empty">
      <CheckCircle2 size={28} aria-hidden="true" />
      <h3>{i18n.t('captures.outboxEmptyTitle')}</h3>
      <p>{i18n.t('captures.outboxEmptyDetail')}</p>
    </div>
  {:else}
    <div class="delivery-list">
      {#each ordered as delivery (delivery.envelopeId)}
        <article class:selected={selectedEnvelopeId === delivery.envelopeId}>
          <button class="delivery-main" type="button" onclick={() => onInspect(delivery)}>
            <span class="state-icon" class:attention={deliveryNeedsAttention(delivery)}>
              {#if delivery.state === 'acknowledged'}
                <ShieldCheck size={18} aria-hidden="true" />
              {:else if delivery.state === 'cancelled'}
                <XCircle size={18} aria-hidden="true" />
              {:else if delivery.state === 'retry-required' || delivery.state === 'conflicted'}
                <AlertTriangle size={18} aria-hidden="true" />
              {:else}
                <Send size={18} aria-hidden="true" />
              {/if}
            </span>
            <span class="delivery-title">
              <strong>{i18n.label(delivery.state)}</strong>
              <small>{i18n.reason(delivery.lastReason)}</small>
            </span>
            <span class="facts">
              <span>{i18n.t('captures.generation', { generation: delivery.generation })}</span>
              <span>{i18n.t('captures.retries', { count: delivery.retryCount })}</span>
              <span>{i18n.label(delivery.source)}</span>
            </span>
            <time datetime={new Date(delivery.updatedAtUnix * 1000).toISOString()}>
              <Clock3 size={13} aria-hidden="true" />
              {i18n.date(delivery.updatedAtUnix, true)}
            </time>
            <StatusBadge status={deliveryStatus(delivery)} label={i18n.label(delivery.state)} />
          </button>

          <div class="actions">
            {#if delivery.capabilities.canRetry}
              <label>
                <span>{i18n.t('captures.retryCause')}</span>
                <select
                  value={retryCauses[delivery.envelopeId] ?? ''}
                  onchange={(event) => selectRetryCause(event, delivery.envelopeId)}
                  disabled={loading}
                >
                  <option value="">{i18n.t('captures.chooseRetryCause')}</option>
                  <option value="process-interrupted">{i18n.label('process-interrupted')}</option>
                  <option value="transport-unavailable">{i18n.label('transport-unavailable')}</option>
                  <option value="destination-unavailable">{i18n.label('destination-unavailable')}</option>
                  <option value="recovery-required">{i18n.label('recovery-required')}</option>
                  <option value="conflict-resolved">{i18n.label('conflict-resolved')}</option>
                </select>
              </label>
              <button
                class="button-secondary"
                type="button"
                disabled={loading || !retryCauses[delivery.envelopeId]}
                onclick={() => requestRetry(delivery)}
              >
                <RotateCcw size={14} aria-hidden="true" />
                {i18n.t('captures.retryDelivery')}
              </button>
            {/if}

            {#if delivery.capabilities.canAcknowledge && delivery.destination}
              <button
                class="button-primary"
                type="button"
                disabled={loading}
                onclick={() => onAcknowledge(delivery, currentProjectRevision)}
              >
                <ShieldCheck size={14} aria-hidden="true" />
                {i18n.t('captures.reviewAcknowledgement')}
              </button>
            {/if}

            {#if delivery.capabilities.canCancel}
              {#if pendingCancellation === delivery.envelopeId}
                <div
                  class="cancel-confirm"
                  role="group"
                  aria-label={i18n.t('captures.cancelConfirm')}
                >
                  <span>{i18n.t('captures.cancelConfirm')}</span>
                  <button
                    class="button-danger"
                    type="button"
                    disabled={loading}
                    onclick={() => onCancel(delivery)}
                    onkeydown={(event) => handleCancellationKeydown(event, delivery.envelopeId)}
                  >{i18n.t('captures.cancelDelivery')}</button>
                  <button
                    bind:this={keepButtons[delivery.envelopeId]}
                    class="button-quiet"
                    type="button"
                    disabled={loading}
                    onclick={() => keepDelivery(delivery.envelopeId)}
                    onkeydown={(event) => handleCancellationKeydown(event, delivery.envelopeId)}
                  >{i18n.t('captures.keepDelivery')}</button>
                </div>
              {:else}
                <button
                  bind:this={cancellationTriggers[delivery.envelopeId]}
                  class="button-secondary"
                  type="button"
                  disabled={loading}
                  onclick={() => requestCancellation(delivery.envelopeId)}
                >
                  <XCircle size={14} aria-hidden="true" />
                  {i18n.t('captures.cancelDelivery')}
                </button>
              {/if}
            {/if}
          </div>

          {#if selectedEnvelopeId === delivery.envelopeId}
            <div class="details" aria-live="polite" aria-atomic="true">
              <div>
                <span>{i18n.t('captures.destination')}</span>
                <strong>{delivery.destination
                  ? `${delivery.destination.projectId} · r${delivery.destination.expectedProjectRevision}`
                  : i18n.t('captures.unboundDestination')}</strong>
              </div>
              <div>
                <span>{i18n.t('captures.recordDigest')}</span>
                <code>{delivery.recordSha256.slice(0, 16)}…</code>
              </div>
              <div>
                <span>{i18n.t('captures.acknowledgement')}</span>
                <strong>{delivery.acknowledgement
                  ? i18n.t('captures.acknowledgedRevision', {
                    revision: delivery.acknowledgement.resultingProjectRevision
                  })
                  : i18n.t('captures.notAcknowledged')}</strong>
              </div>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}

  {#if truncated}
    <button class="button-secondary load-more" type="button" disabled={loading} onclick={onLoadMore}>
      <RefreshCw size={14} class={loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('captures.loadMore')}
    </button>
  {/if}
</div>

<style>
  .outbox { padding: 14px; }
  .heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; }
  .heading h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; }
  .heading > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }
  .empty { display: grid; min-height: 180px; place-items: center; align-content: center; color: var(--color-success); text-align: center; }
  .empty h3 { margin: 11px 0 0; color: var(--color-ink-strong); }
  .empty p { max-width: 620px; margin: 7px 0 0; color: var(--color-muted); }
  .delivery-list { margin-top: 16px; border-top: 1px solid var(--color-border); }
  article { border-bottom: 1px solid var(--color-border); padding: 8px 0; }
  article.selected { margin-inline: -7px; border: 1px solid #7dd3fc; border-radius: 12px; padding-inline: 7px; background: var(--color-accent-soft); }
  .delivery-main { display: grid; width: 100%; min-height: 60px; grid-template-columns: auto minmax(180px, 1fr) minmax(190px, .8fr) 150px auto; align-items: center; gap: 10px; border: 0; padding: 4px; color: inherit; background: transparent; text-align: left; cursor: pointer; }
  .delivery-main:focus-visible { outline: 3px solid rgb(3 105 161 / .3); outline-offset: 2px; }
  .state-icon { display: grid; width: 34px; height: 34px; place-items: center; border-radius: 10px; color: var(--color-success); background: var(--color-success-soft); }
  .state-icon.attention { color: var(--color-warning); background: var(--color-warning-soft); }
  .delivery-title strong, .delivery-title small { display: block; }
  .delivery-title strong { color: var(--color-ink-strong); font-size: 13px; }
  .delivery-title small { margin-top: 4px; color: var(--color-muted); font-size: 10px; line-height: 1.35; }
  .facts { display: flex; flex-wrap: wrap; gap: 5px; }
  .facts span { border: 1px solid var(--color-border); border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: white; font-size: 10px; font-weight: 700; }
  time { display: inline-flex; align-items: center; gap: 5px; color: var(--color-muted); font-size: 10px; }
  .actions { display: flex; flex-wrap: wrap; align-items: end; justify-content: flex-end; gap: 7px; margin-top: 6px; }
  .actions label { display: grid; gap: 3px; min-width: min(240px, 100%); }
  .actions label span { color: var(--color-muted); font-size: 9px; font-weight: 800; text-transform: uppercase; }
  select { min-height: 44px; border: 1px solid var(--color-border-strong); border-radius: 9px; padding: 5px 8px; color: var(--color-ink); background: white; font: inherit; font-size: 11px; }
  .actions button { display: inline-flex; min-height: 44px; align-items: center; gap: 6px; padding: 6px 9px; font-size: 11px; }
  .cancel-confirm { display: flex; align-items: center; gap: 7px; border: 1px solid #fecaca; border-radius: 10px; padding: 6px; color: #991b1b; background: var(--color-danger-soft); font-size: 11px; }
  .button-danger { border: 1px solid #ef4444; border-radius: 8px; color: white; background: #b91c1c; font-weight: 750; }
  .details { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; margin-top: 9px; border-top: 1px solid #bae6fd; padding: 10px 4px 2px; }
  .details div { min-width: 0; }
  .details span, .details strong, .details code { display: block; }
  .details span { color: var(--color-muted); font-size: 9px; font-weight: 800; text-transform: uppercase; }
  .details strong, .details code { margin-top: 4px; overflow-wrap: anywhere; color: var(--color-ink); font-size: 10px; }
  .details code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
  .load-more { display: inline-flex; align-items: center; gap: 7px; margin-top: 14px; }

  @media (max-width: 1040px) {
    .delivery-main { grid-template-columns: auto minmax(180px, 1fr) minmax(150px, .7fr) auto; }
    time { display: none; }
  }

  @media (max-width: 700px) {
    .outbox { padding: 12px; }
    .heading { flex-direction: column; gap: 10px; }
    .delivery-main { grid-template-columns: auto minmax(0, 1fr) auto; }
    .facts { grid-column: 2 / -1; }
    .actions { justify-content: flex-start; padding-left: 44px; }
    .cancel-confirm { align-items: stretch; flex-direction: column; }
    .details { grid-template-columns: 1fr; padding-left: 44px; }
  }
</style>
