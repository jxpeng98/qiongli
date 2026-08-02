<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import * as Card from '$lib/components/ui/card';
  import type { UiTone } from './types';
  import { cn } from '$lib/utils';
  import DescriptionTip from './DescriptionTip.svelte';

  type Props = Omit<HTMLAttributes<HTMLDivElement>, 'children' | 'title' | 'role'> & {
    tone?: UiTone;
    centered?: boolean;
    title?: string;
    description?: string;
    descriptionMode?: 'visible' | 'tooltip';
    icon?: Snippet;
    metadata?: Snippet;
    actions?: Snippet;
    children?: Snippet;
    role?: 'status' | 'alert';
    busy?: boolean;
    live?: 'polite' | 'assertive';
    atomic?: boolean;
  };

  let {
    tone = 'neutral',
    centered = false,
    title,
    description,
    descriptionMode = 'visible',
    icon,
    metadata,
    actions,
    children,
    role,
    busy,
    live,
    atomic,
    class: className,
    ...restProps
  }: Props = $props();
</script>

<Card.Root
  size="sm"
  class={cn('state-panel', tone, centered && 'centered', className)}
  {role}
  aria-busy={busy}
  aria-live={live}
  aria-atomic={atomic}
  {...restProps}
>
  {#if icon}<span class="state-icon" aria-hidden="true">{@render icon()}</span>{/if}
  <div class="content">
    {#if title}
      <div class="title-row">
        <h2>{title}</h2>
        {#if description && descriptionMode === 'tooltip'}
          <DescriptionTip text={description} />
        {/if}
      </div>
    {/if}
    {#if description && descriptionMode === 'visible'}
      <p>{description}</p>
    {:else if description && !title}
      <DescriptionTip text={description} />
    {/if}
    {#if children}<div class="body">{@render children()}</div>{/if}
    {#if actions}<div class="actions">{@render actions()}</div>{/if}
  </div>
  {#if metadata}<div class="metadata">{@render metadata()}</div>{/if}
</Card.Root>

<style>
  :global(.state-panel) {
    display: flex;
    min-width: 0;
    min-height: var(--ui-state-min-height);
    flex-direction: row;
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

  .title-row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-1);
  }

  p {
    max-width: 680px;
    margin: var(--space-1) 0 0;
    color: var(--color-muted);
    font-size: var(--ui-supporting-text-size);
    line-height: 1.5;
  }

  .body {
    margin-top: 6px;
    color: var(--color-muted);
  }

  .content > .body:first-child { margin-top: 0; }

  .actions {
    display: flex;
    max-width: 100%;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }

  .metadata {
    display: flex;
    min-width: 0;
    flex: 0 0 auto;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
  }

  :global(.state-panel.centered) {
    min-height: var(--ui-state-centered-min-height);
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  :global(.state-panel.centered) .content { display: grid; justify-items: center; }
  :global(.state-panel.centered) .actions { justify-content: center; }

  :global(.state-panel.info) {
    --ui-state-accent: var(--color-info);
    --ui-state-icon-bg: var(--color-info-soft);
  }

  :global(.state-panel.success) {
    --ui-state-accent: var(--color-success);
    --ui-state-icon-bg: var(--color-success-soft);
    border-color: var(--color-success-border);
    background: var(--color-success-soft);
  }

  :global(.state-panel.warning) {
    --ui-state-accent: var(--color-warning-strong);
    --ui-state-icon-bg: var(--color-warning-soft);
    border-color: var(--color-warning-border);
    background: var(--color-warning-soft);
  }

  :global(.state-panel.danger) {
    --ui-state-accent: var(--color-danger);
    --ui-state-icon-bg: var(--color-danger-soft);
    border-color: var(--color-danger-border);
    background: var(--color-danger-soft);
  }

  @media (max-width: 520px) {
    :global(.state-panel) { padding: var(--ui-state-padding); flex-wrap: wrap; }
    .metadata { width: 100%; justify-content: flex-start; }
    .actions { align-items: stretch; flex-direction: column; }
    .actions :global(button),
    .actions :global(a) { width: 100%; justify-content: center; }
  }
</style>
