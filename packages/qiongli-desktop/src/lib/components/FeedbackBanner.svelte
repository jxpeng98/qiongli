<script lang="ts">
  import { CircleAlert, CircleCheck, Info, X } from '@lucide/svelte';

  import type { AppNotice } from '../app-state.svelte';
  import { i18n } from '$lib/i18n.svelte';

  let { notice, onDismiss }: { notice: AppNotice; onDismiss: () => void } = $props();
</script>

<section class="banner {notice.tone}" aria-live="polite">
  {#if notice.tone === 'success'}
    <CircleCheck size={19} aria-hidden="true" />
  {:else if notice.tone === 'warning' || notice.tone === 'danger'}
    <CircleAlert size={19} aria-hidden="true" />
  {:else}
    <Info size={19} aria-hidden="true" />
  {/if}
  <div>
    <strong>{notice.title}</strong>
    <p>{notice.detail}</p>
  </div>
  <button type="button" aria-label={i18n.t('notice.dismiss')} onclick={onDismiss}>
    <X size={18} aria-hidden="true" />
  </button>
</section>

<style>
  .banner {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: 11px;
    margin-bottom: 20px;
    border: 1px solid var(--color-border);
    border-radius: 12px;
    padding: 13px 14px;
    color: var(--color-ink);
    background: var(--color-accent-soft);
  }

  .success {
    border-color: #a7f3d0;
    color: #065f46;
    background: var(--color-success-soft);
  }

  .danger {
    border-color: #fecaca;
    color: #991b1b;
    background: var(--color-danger-soft);
  }

  .warning {
    border-color: #fde68a;
    color: #92400e;
    background: var(--color-warning-soft);
  }

  strong {
    font-size: 14px;
  }

  p {
    margin: 3px 0 0;
    color: inherit;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
    line-height: 1.45;
  }

  button {
    display: inline-flex;
    border: 0;
    border-radius: 7px;
    padding: 3px;
    color: inherit;
    background: transparent;
  }
</style>
