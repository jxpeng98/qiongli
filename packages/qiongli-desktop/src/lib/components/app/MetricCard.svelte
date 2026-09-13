<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import type { UiTone } from './types';

  let {
    value,
    label,
    tone = 'neutral',
    icon
  }: {
    value: string | number;
    label: string;
    tone?: UiTone;
    icon?: Snippet;
  } = $props();
</script>

<Card.Root size="sm" class="metric-card {tone}">
  {#if icon}<span class="metric-icon" aria-hidden="true">{@render icon()}</span>{/if}
  <div>
    <strong>{value}</strong>
    <span>{label}</span>
  </div>
</Card.Root>

<style>
  :global(.metric-card) {
    display: flex;
    min-width: 0;
    min-height: var(--ui-metric-min-height);
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    gap: var(--ui-metric-gap);
    padding: var(--ui-metric-padding);
    border-color: transparent;
    background: var(--color-surface-subtle);
    box-shadow: none;
    text-align: left;
  }

  :global(.metric-card) > div { min-width: 0; }

  .metric-icon {
    display: grid;
    width: var(--ui-metric-icon-size);
    height: var(--ui-metric-icon-size);
    flex: none;
    place-items: center;
    border-radius: var(--ui-icon-container-radius);
    color: var(--ui-metric-accent, var(--color-accent-strong));
    background: var(--ui-metric-icon-bg, var(--color-accent-soft));
  }

  strong,
  div > span { display: block; }

  strong {
    color: var(--color-ink-strong);
    font-size: var(--ui-metric-value-size);
    font-weight: var(--ui-heading-weight);
    line-height: 1;
  }

  div > span {
    margin-top: 3px;
    color: var(--color-muted);
    font-size: var(--ui-metric-label-size);
    font-weight: 600;
  }

  :global(.metric-card.info) {
    --ui-metric-accent: var(--color-info);
    --ui-metric-icon-bg: var(--color-info-soft);
  }

  :global(.metric-card.success) {
    --ui-metric-accent: var(--color-success);
    --ui-metric-icon-bg: var(--color-success-soft);
  }

  :global(.metric-card.warning) {
    --ui-metric-accent: var(--color-warning-strong);
    --ui-metric-icon-bg: var(--color-warning-soft);
  }

  :global(.metric-card.danger) {
    --ui-metric-accent: var(--color-danger);
    --ui-metric-icon-bg: var(--color-danger-soft);
  }
</style>
