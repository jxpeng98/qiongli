<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
</script>

<svelte:head>
  <title>{i18n.t('integrations.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<Card.Root class="redirect" role="status" aria-live="polite">
  {i18n.t('backend.movedToIntegrations')}
  <Button href="/client-integrations#legacy-credential-cleanup" variant="outline">
    {i18n.t('nav.integrations')}
  </Button>
</Card.Root>

<style>
  :global(.redirect) { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 16px; color: var(--color-muted); font-size: 11px; }
  @media (max-width: 520px) { :global(.redirect) { align-items: stretch; flex-direction: column; } }
</style>
