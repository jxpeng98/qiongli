<script lang="ts">
  import type { Snippet } from 'svelte';
  import DescriptionTip from './DescriptionTip.svelte';

  let {
    eyebrow,
    title,
    titleId,
    level = 2,
    variant = 'default',
    description,
    icon,
    actions,
    metadata
  }: {
    eyebrow?: string;
    title: string;
    titleId?: string;
    level?: 2 | 3;
    variant?: 'default' | 'panel';
    description?: string;
    icon?: Snippet;
    actions?: Snippet;
    metadata?: Snippet;
  } = $props();
</script>

<header data-slot="section-header" class="section-header" data-variant={variant}>
  <div class="identity">
    {#if icon}
      <span class="icon" aria-hidden="true">{@render icon()}</span>
    {/if}
    <div class="copy">
      {#if eyebrow}<p class="eyebrow">{eyebrow}</p>{/if}
      <div class="title-row">
        {#if level === 3}<h3 id={titleId}>{title}</h3>{:else}<h2 id={titleId}>{title}</h2>{/if}
        {#if description}<DescriptionTip text={description} />{/if}
      </div>
    </div>
  </div>
  {#if metadata}<div class="metadata">{@render metadata()}</div>{/if}
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
  .section-header {
    display: flex;
    min-width: 0;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--ui-section-header-gap);
  }

  .section-header[data-variant='panel'] {
    border-bottom: 1px solid var(--color-border);
    padding: var(--ui-panel-padding);
  }

  .identity {
    display: flex;
    min-width: 0;
    flex: 1 1 260px;
    align-items: flex-start;
    gap: var(--ui-section-header-gap);
  }

  .copy { min-width: 0; }

  .title-row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-1);
  }

  .icon {
    display: grid;
    width: var(--ui-icon-container-size);
    height: var(--ui-icon-container-size);
    flex: none;
    place-items: center;
    border-radius: var(--ui-icon-container-radius);
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .eyebrow { margin-bottom: var(--space-1); }

  h2,
  h3 {
    min-width: 0;
    margin: 0;
    color: var(--color-ink-strong);
    font-size: var(--ui-section-title-size);
    font-weight: var(--ui-heading-weight);
    letter-spacing: -0.015em;
    line-height: 1.25;
    overflow-wrap: anywhere;
    text-wrap: balance;
  }

  .metadata,
  .actions {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex: 0 1 auto;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
  }

  @media (max-width: 520px) {
    .section-header { flex-direction: column; }
    .identity { width: 100%; flex: 0 1 auto; }
    .metadata,
    .actions { width: 100%; justify-content: flex-start; }
  }
</style>
