<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, children }: { label: string; children: Snippet } = $props();
</script>

<section class="metric-grid" aria-label={label}>
  {@render children()}
</section>

<style>
  .metric-grid {
    display: grid;
    min-width: 0;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--ui-metric-grid-gap);
  }

  @media (max-width: 760px) {
    .metric-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }

  @media (max-width: 520px) {
    .metric-grid { grid-template-columns: 1fr; }
  }
</style>
