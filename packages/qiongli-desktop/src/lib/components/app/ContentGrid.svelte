<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/utils';

  type Props = Omit<HTMLAttributes<HTMLDivElement>, 'children'> & {
    columns?: 1 | 2 | 3 | 4;
    collapse?: 'none' | 'sm' | 'md' | 'lg';
    gap?: 'compact' | 'default';
    lastSpan?: 1 | 2 | 3 | 4;
    children: Snippet;
  };

  let {
    columns = 2,
    collapse = 'md',
    gap = 'default',
    lastSpan = 1,
    class: className,
    children,
    ...restProps
  }: Props = $props();
</script>

<div
  data-slot="content-grid"
  data-columns={columns}
  data-collapse={collapse}
  data-gap={gap}
  data-last-span={lastSpan}
  class={cn('content-grid', className)}
  {...restProps}
>
  {@render children()}
</div>

<style>
  .content-grid {
    display: grid;
    min-width: 0;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-items: start;
    gap: var(--ui-section-gap);
  }

  .content-grid[data-columns='1'] { grid-template-columns: minmax(0, 1fr); }
  .content-grid[data-columns='3'] { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .content-grid[data-columns='4'] { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  .content-grid[data-gap='compact'] { gap: var(--ui-content-gap); }
  .content-grid[data-last-span='2'] > :global(:last-child) { grid-column: span 2; }
  .content-grid[data-last-span='3'] > :global(:last-child) { grid-column: span 3; }
  .content-grid[data-last-span='4'] > :global(:last-child) { grid-column: span 4; }

  @media (max-width: 960px) {
    .content-grid[data-collapse='lg'] { grid-template-columns: minmax(0, 1fr); }
    .content-grid[data-collapse='lg'] > :global(:last-child) { grid-column: auto; }
  }

  @media (max-width: 760px) {
    .content-grid[data-collapse='md'] { grid-template-columns: minmax(0, 1fr); }
    .content-grid[data-collapse='md'] > :global(:last-child) { grid-column: auto; }
  }

  @media (max-width: 520px) {
    .content-grid[data-collapse='sm'] { grid-template-columns: minmax(0, 1fr); }
    .content-grid[data-collapse='sm'] > :global(:last-child) { grid-column: auto; }
  }
</style>
