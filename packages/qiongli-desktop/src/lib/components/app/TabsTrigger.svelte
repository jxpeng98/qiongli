<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import { cn } from '$lib/utils';

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

<Tabs.Trigger
  {id}
  {value}
  {disabled}
  aria-controls={ariaControls}
  class={cn('ui-tabs-trigger', className)}
>
  {@render children()}
</Tabs.Trigger>
