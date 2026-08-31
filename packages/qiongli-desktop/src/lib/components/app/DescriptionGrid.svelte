<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/utils';

  type Props = Omit<HTMLAttributes<HTMLDListElement>, 'children'> & {
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

<dl
  data-slot="description-grid"
  data-columns={columns}
  data-compact={compact || undefined}
  class={cn('description-grid', className)}
  {...restProps}
>
  {@render children()}
</dl>

<style>
  .description-grid {
    display: grid;
    min-width: 0;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-2);
    margin: 0;
  }

  .description-grid[data-columns='1'] { grid-template-columns: minmax(0, 1fr); }
  .description-grid[data-columns='3'] { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .description-grid[data-columns='4'] { grid-template-columns: repeat(4, minmax(0, 1fr)); }

  :global(.description-grid > div) {
    min-width: 0;
    margin: 0;
    border: 0;
    border-radius: var(--radius-inset);
    padding: var(--ui-panel-padding);
    background: var(--color-surface-subtle);
  }

  :global(.description-grid[data-compact='true'] > div) { padding: 10px; }

  @media (max-width: 760px) {
    .description-grid[data-columns='3'],
    .description-grid[data-columns='4'] { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }

  @media (max-width: 520px) {
    .description-grid { grid-template-columns: minmax(0, 1fr); }
  }
</style>
