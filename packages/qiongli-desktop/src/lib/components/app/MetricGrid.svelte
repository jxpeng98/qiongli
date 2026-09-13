<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';

  type Props = Omit<HTMLAttributes<HTMLElement>, 'children'> & {
    label: string;
    children: Snippet;
  };

  let { label, class: className, children, ...restProps }: Props = $props();
</script>

<section class={cn('metric-grid', className)} aria-label={label} {...restProps}>
  {@render children()}
</section>

<style>
  .metric-grid {
    display: grid;
    min-width: 0;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--ui-metric-grid-gap);
  }

  @media (max-width: 760px) {
    .metric-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }

  @media (max-width: 520px) {
    .metric-grid { grid-template-columns: 1fr; }
  }
</style>
