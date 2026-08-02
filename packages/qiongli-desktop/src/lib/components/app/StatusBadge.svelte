<script lang="ts">
  import type { StatusCode } from '@qiongli/app-api';
  import { Badge } from '$lib/components/ui/badge';
  import { i18n } from '$lib/i18n.svelte';
  import { cn } from '$lib/utils';

  let { status, label }: { status: StatusCode; label?: string } = $props();

  const isDanger = $derived(
    ['blocked', 'conflict', 'invalid', 'insecure', 'recovery-required'].includes(status)
  );
  const isWarning = $derived(['attention', 'drifted', 'busy'].includes(status));
</script>

<Badge
  variant={isDanger ? 'destructive' : 'secondary'}
  class={cn(
    'status',
    status === 'ready' && 'ready',
    isDanger && 'danger',
    isWarning && 'warn'
  )}
  title={label ?? i18n.label(status)}
>
  <span class="dot" aria-hidden="true"></span>
  <span class="label">{label ?? i18n.label(status)}</span>
</Badge>

<style>
  :global(.status) {
    display: inline-flex;
    max-width: 100%;
    min-height: 24px;
    min-width: 0;
    align-items: center;
    flex: none;
    gap: 6px;
    border-radius: var(--radius-pill);
    padding: 2px 7px;
    color: var(--color-muted);
    background: transparent;
    font-size: 11px;
    font-weight: 620;
    line-height: 1.25;
    white-space: nowrap;
    text-align: left;
  }

  .label {
    min-width: 0;
    white-space: nowrap;
  }

  .dot {
    width: 6px;
    height: 6px;
    flex: none;
    border-radius: 50%;
    background: var(--color-border-strong);
  }

  :global(.status.ready) {
    color: var(--color-success);
    background: var(--color-success-soft);
  }

  :global(.status.ready) .dot {
    background: var(--color-success);
  }

  :global(.status.warn) {
    color: var(--color-warning-strong);
    background: var(--color-warning-soft);
  }

  :global(.status.warn) .dot {
    background: var(--color-warning);
  }

  :global(.status.danger) {
    color: var(--color-danger);
    background: var(--color-danger-soft);
  }

  :global(.status.danger) .dot {
    background: var(--color-danger);
  }
</style>
