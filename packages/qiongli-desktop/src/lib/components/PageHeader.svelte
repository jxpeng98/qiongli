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
    gap: 32px;
    margin-bottom: 24px;
  }

  h1 {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: clamp(28px, 3vw, 38px);
    font-weight: 780;
    letter-spacing: -0.035em;
    line-height: 1.08;
  }

  .description {
    max-width: 760px;
    margin: 10px 0 0;
    color: var(--color-muted);
    font-size: 15px;
    line-height: 1.65;
  }

  .actions {
    display: flex;
    flex: none;
    gap: 10px;
    padding-top: 18px;
  }

  @media (max-width: 700px) {
    .header { flex-direction: column; gap: 14px; }
    .actions { padding-top: 0; }
  }
</style>
