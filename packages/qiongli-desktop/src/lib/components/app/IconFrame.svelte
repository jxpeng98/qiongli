<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    size = 'default',
    tone = 'accent',
    children
  }: {
    size?: 'sm' | 'default';
    tone?: 'accent' | 'neutral';
    children: Snippet;
  } = $props();
</script>

<span data-slot="icon-frame" class="icon-frame" data-size={size} data-tone={tone} aria-hidden="true">
  {@render children()}
</span>

<style>
  .icon-frame {
    display: grid;
    width: var(--ui-icon-container-size);
    height: var(--ui-icon-container-size);
    flex: none;
    place-items: center;
    border-radius: var(--ui-icon-container-radius);
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .icon-frame[data-size='sm'] { width: 24px; height: 24px; }
  .icon-frame[data-tone='neutral'] { color: var(--color-ink); background: var(--color-surface-subtle); }
</style>
