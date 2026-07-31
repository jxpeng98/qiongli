<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import { cn } from '$lib/utils';

  type Props = {
    id?: string;
    class?: string;
    'aria-label'?: string;
    children: Snippet;
  };

  let {
    id,
    class: className,
    'aria-label': ariaLabel,
    children
  }: Props = $props();
</script>

<Tabs.List
  {id}
  aria-label={ariaLabel}
  class={cn('ui-tabs-list', className)}
>
  {@render children()}
</Tabs.List>
