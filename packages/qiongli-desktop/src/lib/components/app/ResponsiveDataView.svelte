<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    desktop,
    mobile
  }: {
    label: string;
    desktop: Snippet;
    mobile: Snippet;
  } = $props();
</script>

<section class="responsive-data-view" aria-label={label}>
  <div class="desktop-view">{@render desktop()}</div>
  <div class="mobile-view">{@render mobile()}</div>
</section>

<style>
  .responsive-data-view,
  .desktop-view,
  .mobile-view { min-width: 0; }

  .mobile-view { display: none; }

  @media (max-width: 720px) {
    .desktop-view { display: none; }
    .mobile-view { display: grid; gap: var(--space-3); }
  }
</style>
