<script lang="ts">
  import type { Snippet } from 'svelte';

  let { eyebrow, title, description, actions }: {
    eyebrow: string;
    title: string;
    description: string;
    actions?: Snippet;
  } = $props();
</script>

<header class="header">
  <div>
    <p class="eyebrow">{eyebrow}</p>
    <h1>{title}</h1>
    <p class="description">{description}</p>
  </div>
  {#if actions}
    <div class="actions">{@render actions()}</div>
  {/if}
</header>

<style>
  .header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 22px;
    margin-bottom: 15px;
  }

  h1 {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: clamp(24px, 2.5vw, 31px);
    font-weight: 780;
    letter-spacing: -0.035em;
    line-height: 1.08;
  }

  .description {
    max-width: 820px;
    margin: 6px 0 0;
    color: var(--color-muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .actions {
    display: flex;
    flex: none;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 11px;
  }

  @media (max-width: 860px) {
    .header { flex-direction: column; gap: 14px; }
    .actions { width: 100%; justify-content: flex-start; padding-top: 0; }
  }
</style>
