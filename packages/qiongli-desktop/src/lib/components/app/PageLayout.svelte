<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/utils';
  import PageHeader from './PageHeader.svelte';

  type Props = Omit<HTMLAttributes<HTMLDivElement>, 'children'> & {
    eyebrow: string;
    title: string;
    description: string;
    actions?: Snippet;
    children: Snippet;
    spacing?: 'compact' | 'default';
  };

  let {
    eyebrow,
    title,
    description,
    actions,
    children,
    spacing = 'default',
    class: className,
    ...restProps
  }: Props = $props();
</script>

<div
  data-slot="page-layout"
  data-spacing={spacing}
  class={cn('page-layout', className)}
  {...restProps}
>
  <PageHeader {eyebrow} {title} {description} {actions} />
  <div class="page-content" data-slot="page-content">
    {@render children()}
  </div>
</div>

<style>
  .page-layout,
  .page-content {
    min-width: 0;
  }

  .page-content {
    display: grid;
    gap: var(--ui-section-gap);
  }

  .page-layout[data-spacing='compact'] .page-content {
    gap: var(--ui-content-gap);
  }
</style>
