<script lang="ts">
  import { KeyRound, SearchCheck, ShieldCheck, Trash2 } from '@lucide/svelte';

  import type { LiteratureProvider } from '@qiongli/app-api';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';
  import { ActionGroup, DescriptionTip, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Input } from '$lib/components/ui/input';

  const app = useAppState();
  type CredentialProvider = 'openalex' | 'semantic-scholar';

  let initializedRevision = $state<number | null>(null);
  let enabled = $state<Record<LiteratureProvider, boolean>>({
    openalex: false,
    'semantic-scholar': false,
    crossref: false,
    pubmed: false,
    arxiv: false
  });
  let secrets = $state<Record<CredentialProvider, string>>({
    openalex: '',
    'semantic-scholar': ''
  });

  let providers = $derived(app.snapshot?.configuration.providers ?? []);
  let canMutate = $derived(
    app.snapshot?.capabilities.apply === true
      && app.snapshot.configuration.revision !== null
  );

  $effect(() => {
    const revision = app.snapshot?.configuration.revision ?? null;
    if (revision === null || revision === initializedRevision) return;
    for (const provider of app.snapshot?.configuration.providers ?? []) {
      enabled[provider.provider] = provider.enabled;
    }
    initializedRevision = revision;
  });

  function statusFor(readiness: string): 'ready' | 'attention' | 'disabled' | 'unavailable' {
    if (readiness === 'ready') return 'ready';
    if (readiness === 'disabled') return 'disabled';
    if (readiness === 'unavailable') return 'unavailable';
    return 'attention';
  }

  function isCredentialProvider(provider: LiteratureProvider): provider is CredentialProvider {
    return provider === 'openalex' || provider === 'semantic-scholar';
  }

  function secretValue(provider: LiteratureProvider): string {
    return isCredentialProvider(provider) ? secrets[provider] : '';
  }

  function updateSecret(provider: LiteratureProvider, value: string): void {
    if (isCredentialProvider(provider)) secrets[provider] = value;
  }

  async function previewSecretFor(provider: LiteratureProvider): Promise<void> {
    if (isCredentialProvider(provider)) await previewSecret(provider);
  }

  async function previewSecretRemovalFor(provider: LiteratureProvider): Promise<void> {
    if (isCredentialProvider(provider)) await previewSecretRemoval(provider);
  }

  async function previewEnablement(): Promise<void> {
    const revision = app.snapshot?.configuration.revision;
    if (revision === null || revision === undefined) return;
    await app.execute({
      action: 'preview-provider-settings',
      expectedRevision: revision,
      providersEnabled: {
        openalex: enabled.openalex,
        semanticScholar: enabled['semantic-scholar'],
        crossref: enabled.crossref,
        pubmed: enabled.pubmed,
        arxiv: enabled.arxiv
      }
    });
  }

  async function previewSecret(provider: CredentialProvider): Promise<void> {
    const value = secrets[provider];
    if (!value) return;
    try {
      await app.execute({
        action: 'preview-provider-secret-change',
        provider,
        change: 'replace',
        value
      });
    } finally {
      secrets[provider] = '';
    }
  }

  async function previewSecretRemoval(provider: CredentialProvider): Promise<void> {
    await app.execute({
      action: 'preview-provider-secret-change',
      provider,
      change: 'remove'
    });
  }

  async function testProvider(provider: LiteratureProvider): Promise<void> {
    await app.execute({ action: 'test-literature-provider', provider });
  }
</script>

{#if app.snapshot}
  <Card.Root class="providers" role="region" aria-labelledby="literature-providers-title">
    <header>
      <span class="provider-mark"><KeyRound size={18} aria-hidden="true" /></span>
      <div>
        <p class="eyebrow">MCP</p>
        <div class="title-row">
          <h2 id="literature-providers-title">{i18n.t('providers.title')}</h2>
          <DescriptionTip text={i18n.t('providers.description')} />
        </div>
      </div>
      <StatusBadge status={app.snapshot.configuration.secretStore} />
    </header>

    {#if !app.snapshot.capabilities.apply}
      <p class="authority-note">{i18n.t('providers.authorityRequired')}</p>
    {/if}

    <div class="provider-list">
      {#each providers as provider (provider.provider)}
        <article>
          <div class="provider-summary">
            <label>
              <Checkbox
                checked={enabled[provider.provider]}
                disabled={app.loading || !canMutate}
                onclick={() => enabled[provider.provider] = !enabled[provider.provider]}
              />
              <strong>{i18n.label(provider.provider)}</strong>
            </label>
            <StatusBadge
              status={statusFor(provider.readiness)}
              label={i18n.t(`providers.readiness.${provider.readiness}`)}
            />
          </div>

          {#if isCredentialProvider(provider.provider)}
            <div class="credential-row">
              <Input
                type="password"
                autocomplete="off"
                value={secretValue(provider.provider)}
                disabled={app.loading || !canMutate}
                placeholder={provider.secretReferencePresent
                  ? i18n.t('providers.keyReplacePlaceholder')
                  : i18n.t('providers.keyPlaceholder')}
                oninput={(event) => updateSecret(provider.provider, event.currentTarget.value)}
              />
              <Button
                variant="outline"
                disabled={app.loading || !canMutate || secretValue(provider.provider).length === 0}
                onclick={() => previewSecretFor(provider.provider)}
              >
                <ShieldCheck size={14} aria-hidden="true" />
                {i18n.t(provider.secretReferencePresent ? 'providers.previewReplace' : 'providers.previewSave')}
              </Button>
              <Button
                variant="ghost"
                disabled={app.loading || !canMutate || !provider.secretReferencePresent}
                onclick={() => previewSecretRemovalFor(provider.provider)}
              >
                <Trash2 size={14} aria-hidden="true" />
                {i18n.t('providers.previewRemove')}
              </Button>
            </div>
          {/if}

          <div class="provider-footer">
            <small>{provider.secretReferencePresent
              ? i18n.t('providers.keyStored')
              : i18n.t('providers.keyNotStored')}</small>
            <Button
              variant="ghost"
              size="sm"
              disabled={app.loading || !provider.enabled}
              onclick={() => testProvider(provider.provider)}
            >
              <SearchCheck size={14} aria-hidden="true" />
              {i18n.t('providers.test')}
            </Button>
          </div>
        </article>
      {/each}
    </div>

    <ActionGroup class="provider-actions">
      <Button disabled={app.loading || !canMutate} onclick={previewEnablement}>
        {i18n.t('providers.previewEnablement')}
      </Button>
    </ActionGroup>
  </Card.Root>
{/if}

<style>
  :global(.providers) { overflow: hidden; margin-top: 10px; }
  header { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; border-bottom: 1px solid var(--color-border); padding: var(--ui-panel-padding); }
  .provider-mark { display: grid; width: 34px; height: 34px; place-items: center; border-radius: var(--radius-control); color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .eyebrow { margin: 0; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 760; text-transform: uppercase; }
  .title-row { display: flex; align-items: center; gap: 7px; }
  h2 { margin: 2px 0 0; color: var(--color-ink-strong); font-size: 14px; }
  .authority-note { margin: 0; border-bottom: 1px solid var(--color-border); padding: 9px var(--ui-panel-padding); color: var(--color-warning-strong); background: var(--color-warning-soft); font-size: var(--font-size-label); }
  .provider-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); }
  article { min-width: 0; border-bottom: 1px solid var(--color-border); padding: 11px var(--ui-panel-padding); }
  article:nth-child(odd) { border-right: 1px solid var(--color-border); }
  .provider-summary, .provider-footer, .credential-row { display: flex; align-items: center; gap: 8px; }
  .provider-summary { justify-content: space-between; }
  .provider-summary label { display: flex; align-items: center; gap: 8px; font-size: var(--font-size-label); }
  .credential-row { margin-top: 10px; }
  .credential-row :global(input) { flex: 1; }
  .provider-footer { justify-content: space-between; margin-top: 7px; color: var(--color-muted); }
  .provider-footer small { font-size: var(--font-size-micro); }
  :global(.provider-actions) { justify-content: flex-end; padding: 10px var(--ui-panel-padding); }
  @media (max-width: 760px) {
    .provider-list { grid-template-columns: 1fr; }
    article:nth-child(odd) { border-right: 0; }
    .credential-row { align-items: stretch; flex-direction: column; }
  }
</style>
