<script lang="ts">
  import { CircleAlert, CircleCheck, Info, X } from '@lucide/svelte';

  import type { AppNotice } from '../app-state.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { materialClass } from '$lib/shared/ui/styles';

  let { notice, onDismiss }: { notice: AppNotice; onDismiss: () => void } = $props();

  let timer: ReturnType<typeof setTimeout> | null = null;
  let remainingMilliseconds = 0;
  let dismissDeadline = 0;
  let paused = false;

  $effect(() => {
    const currentNotice = notice;
    clearDismissTimer();
    paused = false;
    remainingMilliseconds = noticeDuration(currentNotice.tone);
    scheduleDismiss();
    return clearDismissTimer;
  });

  function noticeDuration(tone: AppNotice['tone']): number {
    if (tone === 'danger') return 12_000;
    if (tone === 'warning') return 8_000;
    return 5_000;
  }

  function scheduleDismiss(): void {
    if (paused || remainingMilliseconds <= 0) return;
    dismissDeadline = Date.now() + remainingMilliseconds;
    timer = setTimeout(() => {
      timer = null;
      onDismiss();
    }, remainingMilliseconds);
  }

  function clearDismissTimer(): void {
    if (timer !== null) clearTimeout(timer);
    timer = null;
  }

  function pauseDismiss(): void {
    if (paused) return;
    paused = true;
    if (timer !== null) {
      remainingMilliseconds = Math.max(0, dismissDeadline - Date.now());
      clearDismissTimer();
    }
  }

  function resumeDismiss(event: FocusEvent): void {
    if (
      event.type === 'focusout'
      && event.currentTarget instanceof HTMLElement
      && event.relatedTarget instanceof Node
      && event.currentTarget.contains(event.relatedTarget)
    ) return;
    if (!paused) return;
    paused = false;
    scheduleDismiss();
  }
</script>

<section
  class={materialClass('glass-strong', 'banner', notice.tone)}
  role={notice.tone === 'danger' ? 'alert' : 'status'}
  aria-live={notice.tone === 'danger' ? 'assertive' : 'polite'}
  aria-atomic="true"
  onfocusin={pauseDismiss}
  onfocusout={resumeDismiss}
>
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
  <span class="lifetime" aria-hidden="true"></span>
</section>

<style>
  .banner {
    --notice-duration: 5s;
    --notice-surface: var(--glass-surface-strong);
    --glass-base: var(--notice-surface);

    position: relative;
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: 11px;
    overflow: hidden;
    border-radius: var(--radius-glass);
    padding: 13px 14px;
    color: var(--color-ink);
    box-shadow:
      var(--shadow-overlay),
      inset 0 1px 0 color-mix(in srgb, var(--color-ink-strong) 10%, transparent);
    animation: banner-enter 180ms ease-out both;
  }

  .success {
    border-color: var(--color-success-border);
    color: var(--color-success);
    --notice-surface: color-mix(in srgb, var(--color-success-soft) 72%, var(--glass-surface-strong));
  }

  .danger {
    --notice-duration: 12s;

    border-color: var(--color-danger-border);
    color: var(--color-danger);
    --notice-surface: color-mix(in srgb, var(--color-danger-soft) 72%, var(--glass-surface-strong));
  }

  .warning {
    --notice-duration: 8s;

    border-color: var(--color-warning-border);
    color: var(--color-warning-strong);
    --notice-surface: color-mix(in srgb, var(--color-warning-soft) 72%, var(--glass-surface-strong));
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
    width: 44px;
    min-height: 44px;
    align-items: center;
    justify-content: center;
    border: 0;
    border-radius: 7px;
    padding: 8px;
    color: inherit;
    background: transparent;
  }

  button:hover {
    background: var(--glass-control-hover);
  }

  .lifetime {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 2px;
    background: currentColor;
    opacity: 0.28;
    transform-origin: left;
    animation: lifetime-countdown var(--notice-duration) linear both;
  }

  .banner:focus-within .lifetime {
    animation-play-state: paused;
  }

  @keyframes banner-enter {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
  }

  @keyframes lifetime-countdown {
    to { transform: scaleX(0); }
  }

  @media (prefers-reduced-motion: reduce) {
    .banner,
    .lifetime {
      animation: none;
    }

    .lifetime { display: none; }
  }

  @media (max-width: 480px) {
    .banner { gap: 8px; padding-left: 11px; }
    strong { font-size: 13px; }
    p { font-size: 11px; }
  }
</style>
