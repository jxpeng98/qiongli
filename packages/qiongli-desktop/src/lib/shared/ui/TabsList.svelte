<script lang="ts">
  import type { Snippet } from 'svelte';
  import { TabsPrimitive } from './primitives';
  import { uiClasses } from './styles';

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

<TabsPrimitive.List
  {id}
  aria-label={ariaLabel}
  class={uiClasses('ui-tabs-list', className)}
>
  {@render children()}
</TabsPrimitive.List>
