<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import { cn } from '$lib/utils';

  let {
    label,
    class: className,
    fields,
    actions
  }: {
    label: string;
    class?: string;
    fields: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<Card.Root class={cn('filter-bar', className)} aria-label={label}>
  <div class="filter-fields">{@render fields()}</div>
  {#if actions}<div class="filter-actions">{@render actions()}</div>{/if}
</Card.Root>

<style>
  :global(.filter-bar) {
    display: flex;
    min-width: 0;
    align-items: end;
    gap: var(--space-3);
    padding: var(--space-4);
  }

  .filter-fields {
    display: grid;
    min-width: 0;
    flex: 1 1 auto;
    grid-template-columns: repeat(auto-fit, minmax(min(180px, 100%), 1fr));
    gap: var(--space-3);
  }

  .filter-actions {
    display: flex;
    min-width: 0;
    flex: 0 1 auto;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  @media (max-width: 720px) {
    :global(.filter-bar) { align-items: stretch; flex-direction: column; }
    .filter-actions { width: 100%; }
    .filter-actions :global(button) { flex: 1 1 136px; }
  }
</style>
