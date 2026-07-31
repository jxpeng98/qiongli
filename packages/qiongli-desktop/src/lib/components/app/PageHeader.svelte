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
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--ui-panel-padding-lg);
    margin-bottom: 22px;
    padding: 2px 0;
  }

  .header > :first-child {
    min-width: 0;
    flex: 1 1 300px;
  }

  h1 {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: clamp(27px, 2.6vw, 36px);
    font-weight: var(--ui-heading-weight);
    letter-spacing: -0.045em;
    line-height: 1.08;
    overflow-wrap: anywhere;
    text-wrap: balance;
  }

  .eyebrow {
    display: inline-flex;
    width: fit-content;
    max-width: 100%;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 8px;
    border-radius: var(--radius-pill);
    padding: 3px 8px;
    color: var(--color-ink);
    background: var(--color-surface-subtle);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0;
    overflow-wrap: anywhere;
  }

  .description {
    max-width: 780px;
    margin: 8px 0 0;
    color: var(--color-muted);
    font-size: 14px;
    line-height: 1.5;
    text-wrap: pretty;
  }

  .actions {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex: 0 1 auto;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--space-2);
    padding-top: 22px;
  }

  @media (max-width: 860px) {
    .header { flex-direction: column; gap: 10px; }
    .header > :first-child { flex-basis: auto; }
    .actions { width: 100%; justify-content: flex-start; padding-top: 0; }
  }

  @media (max-width: 620px) {
    .header { gap: 10px; margin-bottom: 18px; padding: 8px 0 0; }
    h1 { font-size: clamp(26px, 8vw, 32px); }
    .description {
      font-size: 14px;
    }
    .actions {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(min(140px, 100%), 1fr));
      gap: 6px;
    }
    .actions :global(button),
    .actions :global(a),
    .actions :global(label),
    .actions :global(details) {
      min-width: 0;
      width: 100%;
    }
    .actions :global(button),
    .actions :global(a) {
      justify-content: center;
    }
    .actions :global(select) {
      width: 100%;
      min-width: 0;
    }
  }
</style>
