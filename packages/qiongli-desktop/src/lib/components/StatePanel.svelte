<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { UiTone } from '$lib/shared/ui/styles';

  let {
    tone = 'neutral',
    centered = false,
    title,
    description,
    icon,
    metadata,
    actions,
    children,
    role,
    busy,
    live,
    atomic
  }: {
    tone?: UiTone;
    centered?: boolean;
    title?: string;
    description?: string;
    icon?: Snippet;
    metadata?: Snippet;
    actions?: Snippet;
    children?: Snippet;
    role?: 'status' | 'alert';
    busy?: boolean;
    live?: 'polite' | 'assertive';
    atomic?: boolean;
  } = $props();
</script>

<section
  class="surface state-panel {tone}"
  class:centered
  {role}
  aria-busy={busy}
  aria-live={live}
  aria-atomic={atomic}
>
  {#if icon}<span class="state-icon" aria-hidden="true">{@render icon()}</span>{/if}
  <div class="content">
    {#if title}<h2>{title}</h2>{/if}
    {#if description}<p>{description}</p>{/if}
    {#if children}<div class="body">{@render children()}</div>{/if}
    {#if actions}<div class="actions">{@render actions()}</div>{/if}
  </div>
  {#if metadata}<div class="metadata">{@render metadata()}</div>{/if}
</section>

<style>
  .state-panel {
    display: flex;
    min-width: 0;
    min-height: var(--ui-state-min-height);
    align-items: flex-start;
    gap: var(--ui-state-gap);
    padding: var(--ui-state-padding);
    color: var(--ui-state-accent, var(--color-muted));
  }

  .content {
    min-width: 0;
    flex: 1 1 auto;
  }

  .state-icon {
    display: grid;
    width: var(--ui-state-icon-size);
    height: var(--ui-state-icon-size);
    flex: none;
    place-items: center;
    border-radius: var(--ui-icon-container-radius);
    color: var(--ui-state-accent, var(--color-accent-strong));
    background: var(--ui-state-icon-bg, var(--color-accent-soft));
  }

  h2 {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: var(--ui-state-title-size);
    font-weight: var(--ui-heading-weight);
  }

  p {
    max-width: 680px;
    margin: var(--space-1) 0 0;
    color: var(--color-muted);
    font-size: var(--ui-supporting-text-size);
    line-height: 1.6;
  }

  .body {
    margin-top: var(--space-2);
    color: var(--color-muted);
  }

  .content > .body:first-child { margin-top: 0; }

  .actions {
    display: flex;
    max-width: 100%;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-3);
  }

  .metadata {
    display: flex;
    min-width: 0;
    flex: 0 0 auto;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .centered {
    min-height: var(--ui-state-centered-min-height);
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .centered .content { display: grid; justify-items: center; }
  .centered .actions { justify-content: center; }

  .info {
    --ui-state-accent: var(--color-info);
    --ui-state-icon-bg: var(--color-info-soft);
  }

  .success {
    --ui-state-accent: var(--color-success);
    --ui-state-icon-bg: var(--color-success-soft);
    border-color: var(--color-success-border);
    background: var(--color-success-soft);
  }

  .warning {
    --ui-state-accent: var(--color-warning-strong);
    --ui-state-icon-bg: var(--color-warning-soft);
    border-color: var(--color-warning-border);
    background: var(--color-warning-soft);
  }

  .danger {
    --ui-state-accent: var(--color-danger);
    --ui-state-icon-bg: var(--color-danger-soft);
    border-color: var(--color-danger-border);
    background: var(--color-danger-soft);
  }

  @media (max-width: 520px) {
    .state-panel { padding: var(--space-4); flex-wrap: wrap; }
    .metadata { width: 100%; justify-content: flex-start; }
    .actions { align-items: stretch; flex-direction: column; }
    .actions :global(button),
    .actions :global(a) { width: 100%; justify-content: center; }
  }
</style>
