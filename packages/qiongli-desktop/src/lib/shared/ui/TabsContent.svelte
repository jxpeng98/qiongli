<script lang="ts">
  import type { Snippet } from 'svelte';
  import { TabsPrimitive } from './primitives';
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

<TabsPrimitive.Content
  {id}
  {value}
  class={uiClasses('ui-tabs-content', className)}
>
  {@render children()}
</TabsPrimitive.Content>
