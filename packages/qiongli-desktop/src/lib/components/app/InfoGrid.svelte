<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/utils';

  type Props = Omit<HTMLAttributes<HTMLDivElement>, 'children'> & {
    columns?: 1 | 2 | 3 | 4;
    compact?: boolean;
    children: Snippet;
  };

  let {
    columns = 2,
    compact = false,
    class: className,
    children,
    ...restProps
  }: Props = $props();
</script>

<div
  data-slot="info-grid"
  data-columns={columns}
  data-compact={compact || undefined}
  class={cn('info-grid', className)}
  {...restProps}
>
  {@render children()}
</div>

<style>
  .info-grid {
    display: grid;
    min-width: 0;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-2);
  }

  .info-grid[data-columns='1'] { grid-template-columns: minmax(0, 1fr); }
  .info-grid[data-columns='3'] { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .info-grid[data-columns='4'] { grid-template-columns: repeat(4, minmax(0, 1fr)); }

  :global(.info-grid > *) {
    min-width: 0;
    margin: 0;
    border: 0;
    border-radius: var(--radius-inset);
    padding: var(--ui-panel-padding);
    background: var(--color-surface-subtle);
  }

  :global(.info-grid[data-compact='true'] > *) { padding: 10px; }
</style>
