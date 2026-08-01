<script lang="ts">
  import { Info } from '@lucide/svelte';

  import { Button } from '$lib/components/ui/button';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import { i18n } from '$lib/i18n.svelte';

  const propsId = $props.id();
  const assistiveDescriptionId = `description-${propsId}`;
  const tooltipId = `description-tooltip-${propsId}`;

  let {
    text,
    side = 'bottom',
    align = 'start'
  }: {
    text: string;
    side?: 'top' | 'right' | 'bottom' | 'left';
    align?: 'start' | 'center' | 'end';
  } = $props();
</script>

<span class="description-tip">
  <Tooltip.Provider delayDuration={120}>
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            class="description-tip-trigger"
            variant="ghost"
            size="icon-sm"
            aria-label={i18n.t('common.moreInformation')}
            aria-describedby={assistiveDescriptionId}
          >
            <Info size={13} aria-hidden="true" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content
        {side}
        {align}
        sideOffset={6}
        class="description-tip-content"
        id={tooltipId}
        role="tooltip"
      >
        {text}
      </Tooltip.Content>
    </Tooltip.Root>
  </Tooltip.Provider>
  <span class="description-sr sr-only" id={assistiveDescriptionId}>{text}</span>
</span>

<style>
  .description-tip {
    display: inline-flex;
    min-width: 0;
    flex: none;
    align-items: center;
  }

  :global(.description-tip-trigger) {
    color: var(--color-muted);
  }

  :global(.description-tip-trigger:hover),
  :global(.description-tip-trigger:focus-visible) {
    color: var(--color-ink-strong);
  }

  :global(.description-tip-content) {
    max-width: min(360px, calc(100vw - 24px));
    border-radius: var(--radius-control);
    padding: 8px 10px;
    font-size: 12px;
    font-weight: 500;
    line-height: 1.45;
    overflow-wrap: anywhere;
    text-wrap: pretty;
  }
</style>
