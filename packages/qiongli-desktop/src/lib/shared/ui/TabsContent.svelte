<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import { uiClasses } from './styles';

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
  class={uiClasses('ui-tabs-content', className)}
>
  {@render children()}
</Tabs.Content>
