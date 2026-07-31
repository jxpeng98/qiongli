<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import { uiClasses } from './styles';

  type Props = {
    id?: string;
    class?: string;
    value: string;
    onValueChange?: (value: string) => void;
    orientation?: 'horizontal' | 'vertical';
    activationMode?: 'automatic' | 'manual';
    loop?: boolean;
    disabled?: boolean;
    children: Snippet;
  };

  let {
    value,
    onValueChange,
    orientation = 'horizontal',
    activationMode = 'automatic',
    loop = true,
    disabled = false,
    id,
    class: className,
    children
  }: Props = $props();
</script>

<Tabs.Root
  {id}
  {value}
  {onValueChange}
  {orientation}
  {activationMode}
  {loop}
  {disabled}
  class={uiClasses('ui-tabs-root', className)}
>
  {@render children()}
</Tabs.Root>
