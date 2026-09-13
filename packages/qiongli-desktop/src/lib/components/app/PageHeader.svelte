<script lang="ts">
  import type { Snippet } from 'svelte';
  import DescriptionTip from './DescriptionTip.svelte';

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
    <div class="title-row">
      <h1>{title}</h1>
      <DescriptionTip text={description} />
    </div>
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
    gap: var(--space-3);
    margin-bottom: var(--space-5);
    padding: 0;
  }

  .header > :first-child {
    min-width: 0;
    flex: 1 1 300px;
  }

  h1 {
    min-width: 0;
    margin: 0;
    color: var(--color-ink-strong);
    font-size: clamp(24px, 2.25vw, 32px);
    font-weight: var(--ui-heading-weight);
    letter-spacing: -0.045em;
    line-height: 1.08;
    overflow-wrap: anywhere;
    text-wrap: balance;
  }

  .title-row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 6px;
  }

  .eyebrow {
    display: inline-flex;
    width: fit-content;
    max-width: 100%;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: var(--space-2);
    padding: 0;
    color: var(--color-muted);
    background: transparent;
    font-size: var(--font-size-label);
    font-weight: 600;
    letter-spacing: 0.01em;
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex: 0 1 auto;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--space-2);
    padding-top: 14px;
  }

  @media (max-width: 860px) {
    .header { flex-direction: column; gap: var(--space-3); }
    .header > :first-child { flex-basis: auto; }
    .actions { width: 100%; justify-content: flex-start; padding-top: 0; }
  }

  @media (max-width: 620px) {
    .header { gap: var(--space-2); margin-bottom: var(--space-4); padding: 8px 0 0; }
    h1 { font-size: clamp(23px, 7.5vw, 28px); }
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
