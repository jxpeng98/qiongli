<script lang="ts">
  import type { StatusCode } from '@qiongli/app-api';

  let { status, label = humanize(status) }: { status: StatusCode; label?: string } = $props();

  function humanize(value: string): string {
    return value.replaceAll('-', ' ').replace(/^./, (letter) => letter.toUpperCase());
  }
</script>

<span class="status" class:ready={status === 'ready'} class:danger={['blocked', 'conflict', 'invalid', 'insecure', 'recovery-required'].includes(status)} class:warn={['attention', 'drifted', 'busy'].includes(status)}>
  <span class="dot" aria-hidden="true"></span>
  {label}
</span>

<style>
  .status {
    display: inline-flex;
    min-height: 26px;
    align-items: center;
    gap: 7px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    padding: 3px 9px;
    color: var(--color-muted);
    background: var(--color-surface-subtle);
    font-size: 12px;
    font-weight: 750;
    white-space: nowrap;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--color-border-strong);
  }

  .ready {
    border-color: #a7f3d0;
    color: #065f46;
    background: var(--color-success-soft);
  }

  .ready .dot {
    background: var(--color-success);
  }

  .warn {
    border-color: #fde68a;
    color: #854d0e;
    background: var(--color-warning-soft);
  }

  .warn .dot {
    background: var(--color-warning);
  }

  .danger {
    border-color: #fecaca;
    color: #991b1b;
    background: var(--color-danger-soft);
  }

  .danger .dot {
    background: var(--color-danger);
  }
</style>
