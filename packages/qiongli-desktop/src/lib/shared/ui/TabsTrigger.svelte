<script lang="ts">
  import type { Snippet } from 'svelte';
  import { TabsPrimitive } from './primitives';
  import { uiClasses } from './styles';

  type Props = {
    id?: string;
    class?: string;
    value: string;
    disabled?: boolean;
    'aria-controls'?: string;
    children: Snippet;
  };

  let {
    id,
    value,
    disabled = false,
    'aria-controls': ariaControls,
    class: className,
    children
  }: Props = $props();
</script>

<TabsPrimitive.Trigger
  {id}
  {value}
  {disabled}
  aria-controls={ariaControls}
  class={uiClasses('ui-tabs-trigger', className)}
>
  {@render children()}
</TabsPrimitive.Trigger>
