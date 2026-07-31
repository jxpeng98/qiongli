<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    eyebrow,
    title,
    titleId,
    level = 2,
    description,
    icon,
    actions,
    metadata
  }: {
    eyebrow?: string;
    title: string;
    titleId?: string;
    level?: 2 | 3;
    description?: string;
    icon?: Snippet;
    actions?: Snippet;
    metadata?: Snippet;
  } = $props();
</script>

<header class="section-header">
  <div class="identity">
    {#if icon}
      <span class="icon" aria-hidden="true">{@render icon()}</span>
    {/if}
    <div class="copy">
      {#if eyebrow}<p class="eyebrow">{eyebrow}</p>{/if}
      {#if level === 3}<h3 id={titleId}>{title}</h3>{:else}<h2 id={titleId}>{title}</h2>{/if}
      {#if description}<p class="description">{description}</p>{/if}
    </div>
  </div>
  {#if metadata}<div class="metadata">{@render metadata()}</div>{/if}
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
  .section-header {
    display: flex;
    min-width: 0;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--ui-section-header-gap);
  }

  .identity {
    display: flex;
    min-width: 0;
    align-items: flex-start;
    gap: var(--ui-section-header-gap);
  }

  .copy { min-width: 0; }

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
    margin: 0;
    color: var(--color-ink-strong);
    font-size: var(--ui-section-title-size);
    font-weight: var(--ui-heading-weight);
    letter-spacing: -0.015em;
    line-height: 1.25;
  }

  .description {
    max-width: 760px;
    margin: var(--space-1) 0 0;
    color: var(--color-muted);
    font-size: var(--ui-supporting-text-size);
    line-height: 1.5;
  }

  .metadata,
  .actions {
    display: flex;
    min-width: 0;
    flex: 0 0 auto;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  @media (max-width: 520px) {
    .section-header { flex-direction: column; }
    .metadata,
    .actions { width: 100%; justify-content: flex-start; }
  }
</style>
