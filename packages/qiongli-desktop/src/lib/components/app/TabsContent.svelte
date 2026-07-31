<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import { cn } from '$lib/utils';

  type Props = {
    id?: string;
    class?: string;
    value: string;
    children: Snippet;
  };

  let {
    id,
    value,
    class: className,
    children
  }: Props = $props();
</script>

<Tabs.Content
  {id}
  {value}
  class={cn('ui-tabs-content', className)}
>
  {@render children()}
</Tabs.Content>
