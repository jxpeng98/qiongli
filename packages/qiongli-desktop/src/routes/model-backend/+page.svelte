<script lang="ts">
  import { Bot, CheckCircle2, KeyRound, LockKeyhole, Network, ShieldCheck } from '@lucide/svelte';

  import type { StatusCode } from '@qiongli/app-api';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let apiKey = $state('');

  let backend = $derived(app.snapshot?.configuration.openaiBackend ?? null);
  let readinessStatus = $derived.by<StatusCode>(() => {
    if (!backend) return 'unavailable';
    if (backend.readiness === 'ready') return 'ready';
    if (backend.readiness === 'disabled') return 'disabled';
    if (backend.readiness === 'secret-store-unavailable') return 'unavailable';
    return 'attention';
  });

  function previewEnablement(): Promise<unknown> | undefined {
    const revision = app.snapshot?.configuration.revision;
    if (revision === null || revision === undefined || !backend) return;
    return app.execute({
      action: 'preview-agent-backend-settings',
      expectedRevision: revision,
      enabled: !backend.enabled
    });
  }

  async function previewCredential(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!apiKey) return;
    const pending = app.execute({ action: 'preview-agent-backend-credential', apiKey });
    apiKey = '';
    await pending;
  }

  function previewCredentialRemoval(): Promise<unknown> {
    return app.execute({ action: 'preview-remove-agent-backend-credential' });
  }

  function testConnection(): Promise<unknown> {
    return app.execute({ action: 'test-open-ai-backend' });
  }
</script>

<PageHeader
  eyebrow={i18n.t('backend.eyebrow')}
  title={i18n.t('backend.title')}
  description={i18n.t('backend.description')}
/>

{#if !app.snapshot || !backend}
  <section class="surface loading" aria-busy="true">{i18n.t('backend.loading')}</section>
{:else}
  <section class="backend-summary surface" aria-labelledby="backend-name">
    <div class="backend-icon"><Bot size={22} aria-hidden="true" /></div>
    <div>
      <p class="eyebrow">{i18n.t('backend.directApi')}</p>
      <div class="title-line">
        <h2 id="backend-name">OpenAI Responses</h2>
        <StatusBadge status={readinessStatus} label={i18n.label(backend.readiness)} />
      </div>
      <p>{backend.backendId} · {backend.model}</p>
    </div>
    <button
      class={backend.enabled ? 'button-secondary' : 'button-primary'}
      type="button"
      disabled={app.loading || !app.snapshot.capabilities.agentBackendConfig || app.snapshot.configuration.revision === null}
      onclick={previewEnablement}
    >
      {backend.enabled ? i18n.t('backend.disable') : i18n.t('backend.enable')}
    </button>
  </section>

  <div class="control-grid">
    <section class="surface credential-card" aria-labelledby="credential-title">
      <div class="section-heading">
        <span class="section-icon"><KeyRound size={19} aria-hidden="true" /></span>
        <div>
          <p class="eyebrow">{i18n.t('backend.credentialEyebrow')}</p>
          <h2 id="credential-title">{i18n.t('backend.credentialTitle')}</h2>
        </div>
      </div>

      <div class="credential-state">
        <LockKeyhole size={17} aria-hidden="true" />
        <span>{backend.secretReferencePresent ? i18n.t('backend.credentialStored') : i18n.t('backend.credentialMissing')}</span>
      </div>

      <form onsubmit={previewCredential}>
        <label for="openai-api-key">{i18n.t('backend.apiKey')}</label>
        <p id="openai-key-help">{i18n.t('backend.apiKeyHelp')}</p>
        <input
          id="openai-api-key"
          type="password"
          name="openai-api-key"
          bind:value={apiKey}
          autocomplete="off"
          autocapitalize="none"
          spellcheck="false"
          aria-describedby="openai-key-help"
          disabled={app.loading || !app.snapshot.capabilities.agentBackendConfig}
        />
        <div class="actions">
          <button class="button-primary" type="submit" disabled={app.loading || apiKey.length === 0}>
            {backend.secretReferencePresent ? i18n.t('backend.replaceKey') : i18n.t('backend.saveKey')}
          </button>
          <button
            class="button-danger"
            type="button"
            disabled={app.loading || !backend.secretReferencePresent}
            onclick={previewCredentialRemoval}
          >
            {i18n.t('backend.removeKey')}
          </button>
        </div>
      </form>
    </section>

    <section class="surface test-card" aria-labelledby="connection-title">
      <div class="section-heading">
        <span class="section-icon"><Network size={19} aria-hidden="true" /></span>
        <div>
          <p class="eyebrow">{i18n.t('backend.connectionEyebrow')}</p>
          <h2 id="connection-title">{i18n.t('backend.connectionTitle')}</h2>
        </div>
      </div>
      <p>{i18n.t('backend.connectionHelp')}</p>
      <button
        class="button-secondary"
        type="button"
        disabled={app.loading || !backend.testAvailable || !app.snapshot.capabilities.agentBackendTest}
        onclick={testConnection}
      >
        <CheckCircle2 size={16} aria-hidden="true" />
        {i18n.t('backend.test')}
      </button>
      {#if !backend.testAvailable}
        <p class="requirement" role="status">{i18n.t('backend.testUnavailable')}</p>
      {/if}
    </section>
  </div>

  <section class="surface boundary" aria-labelledby="boundary-title">
    <ShieldCheck size={21} aria-hidden="true" />
    <div>
      <p class="eyebrow">{i18n.t('backend.boundaryEyebrow')}</p>
      <h2 id="boundary-title">{i18n.t('backend.boundaryTitle')}</h2>
      <p>{i18n.t('backend.boundaryDescription')}</p>
    </div>
    <dl>
      <div><dt>{i18n.t('backend.storage')}</dt><dd>{i18n.t('backend.disabled')}</dd></div>
      <div><dt>{i18n.t('backend.hostedTools')}</dt><dd>{i18n.t('backend.disabled')}</dd></div>
      <div><dt>{i18n.t('backend.model')}</dt><dd>{backend.model}</dd></div>
    </dl>
  </section>
{/if}

<style>
  .loading { padding: 22px; color: var(--color-muted); }
  .backend-summary { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 13px; padding: 14px 16px; border-left: 3px solid var(--color-accent); }
  .backend-icon, .section-icon { display: grid; place-items: center; border-radius: 9px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .backend-icon { width: 40px; height: 40px; }
  .section-icon { width: 34px; height: 34px; }
  .title-line { display: flex; align-items: center; gap: 9px; }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 16px; }
  .backend-summary > div > p:last-child { margin: 5px 0 0; color: var(--color-muted); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 10px; }
  .control-grid { display: grid; grid-template-columns: minmax(0, 1.3fr) minmax(250px, .7fr); gap: 10px; margin-top: 10px; }
  .credential-card, .test-card { padding: 16px; }
  .section-heading { display: grid; grid-template-columns: auto 1fr; align-items: center; gap: 10px; }
  .credential-state { display: flex; align-items: center; gap: 7px; margin: 13px 0; border-radius: 8px; padding: 8px 10px; color: var(--color-muted); background: var(--color-surface-subtle); font-size: 11px; font-weight: 650; }
  form label { display: block; color: var(--color-ink); font-size: 12px; font-weight: 750; }
  form p, .test-card > p, .boundary p { margin: 5px 0 9px; color: var(--color-muted); font-size: 11px; line-height: 1.5; }
  input { width: 100%; min-height: 38px; border: 1px solid var(--color-border-strong); border-radius: 8px; padding: 7px 10px; color: var(--color-ink); background: white; font: inherit; }
  .actions { display: flex; gap: 8px; margin-top: 10px; }
  .test-card > button { margin-top: 9px; }
  .requirement { border-left: 2px solid var(--color-warning); padding-left: 9px; color: var(--color-warning) !important; }
  .boundary { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: start; gap: 11px; margin-top: 10px; padding: 14px 16px; color: var(--color-success); }
  .boundary h2 { font-size: 14px; }
  .boundary dl { display: flex; gap: 8px; margin: 0; }
  .boundary dl div { min-width: 100px; border-left: 1px solid var(--color-border); padding-left: 10px; }
  .boundary dt { color: var(--color-muted); font-size: 9px; font-weight: 750; text-transform: uppercase; }
  .boundary dd { margin: 3px 0 0; color: var(--color-ink); font-size: 11px; font-weight: 700; }
  @media (max-width: 800px) { .control-grid { grid-template-columns: 1fr; } .boundary { grid-template-columns: auto 1fr; } .boundary dl { grid-column: 1 / -1; } }
  @media (max-width: 520px) { .backend-summary { grid-template-columns: auto 1fr; } .backend-summary > button { grid-column: 1 / -1; } .actions, .boundary dl { align-items: stretch; flex-direction: column; } }
</style>
