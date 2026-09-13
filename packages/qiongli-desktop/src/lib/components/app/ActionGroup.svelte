<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  let {
    label,
    align = 'start',
    compact = false,
    class: className,
    children
  }: {
    label?: string;
    align?: 'start' | 'end';
    compact?: boolean;
    class?: string;
    children: Snippet;
  } = $props();
</script>

<div
  class={cn('action-group', align === 'end' && 'action-group--end', compact && 'action-group--compact', className)}
  role={label ? 'group' : undefined}
  aria-label={label}
>
  {@render children()}
</div>

<style>
  .action-group {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }

  .action-group--end { justify-content: flex-end; }
  .action-group--compact { gap: var(--space-1); }

  @media (max-width: 520px) {
    .action-group:not(.action-group--compact) {
      display: grid;
      width: 100%;
      grid-template-columns: repeat(auto-fit, minmax(min(136px, 100%), 1fr));
    }

    .action-group:not(.action-group--compact) :global(button),
    .action-group:not(.action-group--compact) :global(a) {
      min-width: 0;
      width: 100%;
      justify-content: center;
    }
  }
</style>
