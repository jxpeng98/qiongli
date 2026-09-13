<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
</script>

<svelte:head>
  <title>{i18n.t('content.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<Card.Root class="redirect" role="status" aria-live="polite">
  {i18n.t('content.movedToIntegrations')}
  <Button href="/client-integrations#workflow-content" variant="outline">
    {i18n.t('nav.integrations')}
  </Button>
</Card.Root>

<style>
  :global(.redirect) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 9px;
    padding: var(--ui-panel-padding);
    color: var(--color-muted);
    font-size: var(--font-size-label);
  }
  @media (max-width: 520px) { :global(.redirect) { align-items: stretch; flex-direction: column; } }
</style>
