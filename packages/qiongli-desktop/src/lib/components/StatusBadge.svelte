<script lang="ts">
  import type { StatusCode } from '@qiongli/app-api';
  import { i18n } from '$lib/i18n.svelte';

  let { status, label }: { status: StatusCode; label?: string } = $props();
</script>

<span
  class="status"
  class:ready={status === 'ready'}
  class:danger={['blocked', 'conflict', 'invalid', 'insecure', 'recovery-required'].includes(status)}
  class:warn={['attention', 'drifted', 'busy'].includes(status)}
  title={label ?? i18n.label(status)}
>
  <span class="dot" aria-hidden="true"></span>
  <span class="label">{label ?? i18n.label(status)}</span>
</span>

<style>
  .status {
    display: inline-flex;
    max-width: 100%;
    min-height: 24px;
    min-width: 0;
    align-items: center;
    flex: 0 1 auto;
    gap: 6px;
    border-radius: 999px;
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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dot {
    width: 6px;
    height: 6px;
    flex: none;
    border-radius: 50%;
    background: var(--color-border-strong);
  }

  .ready {
    color: var(--color-success);
    background: var(--color-success-soft);
  }

  .ready .dot {
    background: var(--color-success);
  }

  .warn {
    color: var(--color-warning-strong);
    background: var(--color-warning-soft);
  }

  .warn .dot {
    background: var(--color-warning);
  }

  .danger {
    color: var(--color-danger);
    background: var(--color-danger-soft);
  }

  .danger .dot {
    background: var(--color-danger);
  }
</style>
