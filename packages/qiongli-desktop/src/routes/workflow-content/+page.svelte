<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
</script>

<svelte:head>
  <title>{i18n.t('content.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<section class="surface redirect" role="status" aria-live="polite">
  {i18n.t('content.movedToIntegrations')}
  <a href="/client-integrations#workflow-content">{i18n.t('nav.integrations')}</a>
</section>

<style>
  .redirect {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 16px;
    color: var(--color-muted);
    font-size: 11px;
  }
  a { color: var(--color-accent-strong); font-weight: 750; }
</style>
