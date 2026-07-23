<script lang="ts">
  import { Cable, KeyRound, ShieldCheck, Trash2 } from '@lucide/svelte';

  import { PageHeader } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let legacyCredential = $derived(app.snapshot?.configuration.legacyCredential ?? null);

  function previewCredentialRemoval(): Promise<unknown> {
    return app.execute({ action: 'preview-remove-agent-backend-credential' });
  }
</script>

<PageHeader
  eyebrow={i18n.t('backend.legacyEyebrow')}
  title={i18n.t('backend.legacyTitle')}
  description={i18n.t('backend.legacyDescription')}
/>

{#if !app.snapshot || !legacyCredential}
  <section class="surface loading" aria-busy="true">{i18n.t('common.loading')}</section>
{:else}
  <section class="surface host-boundary" aria-labelledby="host-boundary-title">
    <Cable size={23} aria-hidden="true" />
    <div>
      <p class="eyebrow">{i18n.t('backend.hostEyebrow')}</p>
      <h2 id="host-boundary-title">{i18n.t('backend.hostTitle')}</h2>
      <p>{i18n.t('backend.hostDescription')}</p>
      <div class="actions">
        <a class="button-primary" href="/client-integrations">{i18n.t('backend.openIntegrations')}</a>
        <a class="button-secondary" href="/orchestrator">{i18n.t('backend.openOrchestrator')}</a>
      </div>
    </div>
  </section>

  <section class="surface legacy-credential" aria-labelledby="legacy-credential-title">
    <KeyRound size={21} aria-hidden="true" />
    <div>
      <p class="eyebrow">{i18n.t('backend.legacyCredentialEyebrow')}</p>
      <h2 id="legacy-credential-title">{i18n.t('backend.legacyCredentialTitle')}</h2>
      <p>
        {legacyCredential.referencePresent
          ? i18n.t('backend.legacyCredentialPresent')
          : i18n.t('backend.legacyCredentialMissing')}
      </p>
      <p class="help">{i18n.t('backend.legacyCredentialHelp')}</p>
    </div>
    <button
      class="button-danger"
      type="button"
      disabled={app.loading || !legacyCredential.cleanupAvailable}
      onclick={previewCredentialRemoval}
    >
      <Trash2 size={16} aria-hidden="true" />
      {i18n.t('backend.legacyRemove')}
    </button>
  </section>

  <section class="surface guarantee" aria-labelledby="guarantee-title">
    <ShieldCheck size={21} aria-hidden="true" />
    <div>
      <h2 id="guarantee-title">{i18n.t('backend.guaranteeTitle')}</h2>
      <p>{i18n.t('backend.guaranteeDescription')}</p>
    </div>
  </section>
{/if}

<style>
  .loading { padding: 22px; color: var(--color-muted); }
  .host-boundary,
  .legacy-credential,
  .guarantee {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: start;
    gap: 13px;
    padding: 18px;
  }
  .legacy-credential,
  .guarantee { margin-top: 10px; }
  .host-boundary { border-left: 3px solid var(--color-accent); }
  .guarantee { grid-template-columns: auto minmax(0, 1fr); color: var(--color-success); }
  h2, p { margin-top: 0; }
  h2 { margin-bottom: 6px; color: var(--color-ink-strong); font-size: 16px; }
  p { margin-bottom: 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .eyebrow {
    margin-bottom: 5px;
    color: var(--color-accent-strong);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: .1em;
    text-transform: uppercase;
  }
  .help { margin-top: 7px; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
  .actions a { text-decoration: none; }
  .legacy-credential button { display: inline-flex; align-items: center; gap: 7px; }
  @media (max-width: 620px) {
    .host-boundary,
    .legacy-credential { grid-template-columns: auto minmax(0, 1fr); }
    .legacy-credential button { grid-column: 1 / -1; justify-self: start; }
  }
</style>
