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
  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/components/app';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { NativeSelect } from '$lib/components/ui/native-select';
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
  let cancelTriggers = $state<Record<string, HTMLButtonElement | null>>({});

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

  function restoreCancelFocus(event: Event, envelopeId: string): void {
    event.preventDefault();
    cancelTriggers[envelopeId]?.focus();
  }

</script>

<Card.Root class="outbox">
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
          <Button class="delivery-main" variant="ghost" onclick={() => onInspect(delivery)}>
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
          </Button>

          <div class="actions">
            {#if delivery.capabilities.canRetry}
              <label>
                <span>{i18n.t('captures.retryCause')}</span>
                <NativeSelect
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
                </NativeSelect>
              </label>
              <Button
                variant="outline"
                disabled={loading || !retryCauses[delivery.envelopeId]}
                onclick={() => requestRetry(delivery)}
              >
                <RotateCcw size={14} aria-hidden="true" />
                {i18n.t('captures.retryDelivery')}
              </Button>
            {/if}

            {#if delivery.capabilities.canAcknowledge && delivery.destination}
              <Button
                disabled={loading}
                onclick={() => onAcknowledge(delivery, currentProjectRevision)}
              >
                <ShieldCheck size={14} aria-hidden="true" />
                {i18n.t('captures.reviewAcknowledgement')}
              </Button>
            {/if}

            {#if delivery.capabilities.canCancel}
              <AlertDialog.Root>
                <AlertDialog.Trigger>
                  {#snippet child({ props })}
                    <Button
                      {...props}
                      variant="outline"
                      disabled={loading}
                      onclick={(event) => {
                        cancelTriggers[delivery.envelopeId] = event.currentTarget as HTMLButtonElement;
                        if (typeof props.onclick === 'function') props.onclick(event);
                      }}
                    >
                      <XCircle size={14} aria-hidden="true" />
                      {i18n.t('captures.cancelDelivery')}
                    </Button>
                  {/snippet}
                </AlertDialog.Trigger>
                <AlertDialog.Content
                  onCloseAutoFocus={(event) => restoreCancelFocus(event, delivery.envelopeId)}
                >
                  <AlertDialog.Header>
                    <AlertDialog.Title>{i18n.t('captures.cancelConfirm')}</AlertDialog.Title>
                    <AlertDialog.Description>{i18n.reason(delivery.lastReason)}</AlertDialog.Description>
                  </AlertDialog.Header>
                  <AlertDialog.Footer>
                    <AlertDialog.Cancel>{i18n.t('captures.keepDelivery')}</AlertDialog.Cancel>
                    <AlertDialog.Action variant="destructive" onclick={() => onCancel(delivery)}>{i18n.t('captures.cancelDelivery')}</AlertDialog.Action>
                  </AlertDialog.Footer>
                </AlertDialog.Content>
              </AlertDialog.Root>
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
    <Button class="load-more" variant="outline" disabled={loading} onclick={onLoadMore}>
      <RefreshCw size={14} class={loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('captures.loadMore')}
    </Button>
  {/if}
</Card.Root>

<style>
  :global(.outbox) { padding: 14px; }
  .heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; }
  .heading h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; }
  .heading > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }
  .empty { display: grid; min-height: 180px; place-items: center; align-content: center; color: var(--color-success); text-align: center; }
  .empty h3 { margin: 11px 0 0; color: var(--color-ink-strong); }
  .empty p { max-width: 620px; margin: 7px 0 0; color: var(--color-muted); }
  .delivery-list { margin-top: 16px; border-top: 1px solid var(--color-border); }
  article { border-bottom: 1px solid var(--color-border); padding: 8px 0; }
  article.selected { margin-inline: -7px; border: 1px solid var(--color-accent-border); border-radius: 12px; padding-inline: 7px; background: var(--color-accent-soft); }
  :global(.delivery-main) { display: grid; width: 100%; height: auto; min-height: 60px; grid-template-columns: auto minmax(180px, 1fr) minmax(190px, .8fr) 150px auto; align-items: center; gap: 10px; border: 0; padding: 4px; color: inherit; background: transparent; text-align: left; white-space: normal; cursor: pointer; }
  :global(.delivery-main:focus-visible) { outline: 3px solid color-mix(in srgb, var(--color-focus) 34%, transparent); outline-offset: 2px; }
  .state-icon { display: grid; width: 34px; height: 34px; place-items: center; border-radius: 10px; color: var(--color-success); background: var(--color-success-soft); }
  .state-icon.attention { color: var(--color-warning); background: var(--color-warning-soft); }
  .delivery-title strong, .delivery-title small { display: block; }
  .delivery-title strong { color: var(--color-ink-strong); font-size: 13px; }
  .delivery-title small { margin-top: 4px; color: var(--color-muted); font-size: 10px; line-height: 1.35; }
  .facts { display: flex; flex-wrap: wrap; gap: 5px; }
  .facts span { max-width: 100%; overflow: hidden; border: 1px solid var(--color-border); border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: var(--color-control); font-size: 10px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
  time { display: inline-flex; align-items: center; gap: 5px; color: var(--color-muted); font-size: 10px; }
  .actions { display: flex; flex-wrap: wrap; align-items: end; justify-content: flex-end; gap: 7px; margin-top: 6px; }
  .actions label { display: grid; gap: 3px; min-width: min(240px, 100%); }
  .actions label span { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 800; text-transform: uppercase; }
  .actions label :global([data-slot='native-select-wrapper']) { width: 100%; }
  .actions :global([data-slot='button']) { min-height: 44px; padding: 6px 9px; font-size: 11px; }
  .details { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; margin-top: 9px; border-top: 1px solid var(--color-accent-border); padding: 10px 4px 2px; }
  .details div { min-width: 0; }
  .details span, .details strong, .details code { display: block; }
  .details span { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 800; text-transform: uppercase; }
  .details strong, .details code { margin-top: 4px; overflow-wrap: anywhere; color: var(--color-ink); font-size: 10px; }
  .details code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
  :global(.load-more) { margin-top: 14px; }

  @media (max-width: 1040px) {
    :global(.outbox) { padding: 12px; }
    .heading { flex-direction: column; gap: 10px; }
    :global(.delivery-main) { grid-template-columns: auto minmax(0, 1fr) auto; }
    .facts { grid-column: 2 / -1; }
    time { display: none; }
    .actions { justify-content: flex-start; padding-left: 44px; }
    .details { grid-template-columns: 1fr; padding-left: 44px; }
  }
</style>
