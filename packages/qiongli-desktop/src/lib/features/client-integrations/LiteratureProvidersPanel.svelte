<script lang="ts">
  import { KeyRound, SearchCheck, ShieldCheck, Trash2 } from '@lucide/svelte';

  import type { AppSnapshot, LiteratureProvider, LiteratureProviderConfigurationField } from '@qiongli/app-api';
  import type { AppState } from '$lib/app-state.svelte';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';
  import { ActionGroup, DescriptionTip, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Input } from '$lib/components/ui/input';

  let { appState }: { appState?: AppState } = $props();
  const contextApp = useAppState();
  let app = $derived(appState ?? contextApp);
  type ProviderView = AppSnapshot['configuration']['providers'][number];

  let initializedRevision = $state<number | null>(null);
  let enabled = $state<Record<LiteratureProvider, boolean>>({
    openalex: false,
    'semantic-scholar': false,
    crossref: false,
    pubmed: false,
    arxiv: false
  });
  let fieldValues = $state<Record<string, string>>({});

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

  function fieldKey(
    provider: LiteratureProvider,
    field: LiteratureProviderConfigurationField
  ): string {
    return `${provider}:${field}`;
  }

  function fieldValue(
    provider: LiteratureProvider,
    field: LiteratureProviderConfigurationField
  ): string {
    return fieldValues[fieldKey(provider, field)] ?? '';
  }

  function updateField(
    provider: LiteratureProvider,
    field: LiteratureProviderConfigurationField,
    value: string
  ): void {
    fieldValues[fieldKey(provider, field)] = value;
  }

  function fieldLabel(field: LiteratureProviderConfigurationField): string {
    return i18n.t(field === 'api-key' ? 'providers.apiKeyLabel' : 'providers.contactLabel');
  }

  function configurationSummary(provider: ProviderView): string {
    if (provider.configurationFields.length === 0) {
      return i18n.t('providers.noConfigurationRequired');
    }
    return provider.configurationFields.map((field) => i18n.t(
      field.field === 'api-key'
        ? field.configured ? 'providers.keyStored' : 'providers.keyNotStored'
        : field.configured ? 'providers.contactStored' : 'providers.contactNotStored'
    )).join(' · ');
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
      },
      publicSettingChanges: []
    });
  }

  async function previewField(
    provider: LiteratureProvider,
    field: LiteratureProviderConfigurationField,
    change: 'replace' | 'remove'
  ): Promise<void> {
    const key = fieldKey(provider, field);
    const value = fieldValues[key] ?? '';
    if (change === 'replace' && !value) return;
    try {
      if (field === 'api-key') {
        await app.execute(change === 'replace'
          ? { action: 'preview-provider-secret-change', provider, change, value }
          : { action: 'preview-provider-secret-change', provider, change });
        return;
      }
      const revision = app.snapshot?.configuration.revision;
      if (revision === null || revision === undefined) return;
      await app.execute({
        action: 'preview-provider-settings',
        expectedRevision: revision,
        providersEnabled: {
          openalex: providers.find((item) => item.provider === 'openalex')?.enabled ?? false,
          semanticScholar: providers.find((item) => item.provider === 'semantic-scholar')?.enabled ?? false,
          crossref: providers.find((item) => item.provider === 'crossref')?.enabled ?? false,
          pubmed: providers.find((item) => item.provider === 'pubmed')?.enabled ?? false,
          arxiv: providers.find((item) => item.provider === 'arxiv')?.enabled ?? false
        },
        publicSettingChanges: [change === 'replace'
          ? { provider, change, value }
          : { provider, change }]
      });
    } finally {
      fieldValues[key] = '';
    }
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

          {#each provider.configurationFields as field (field.field)}
            <div class="credential-row">
              <span class="field-label">{fieldLabel(field.field)}</span>
              <Input
                type={field.field === 'api-key' ? 'password' : 'email'}
                autocomplete={field.field === 'api-key' ? 'off' : 'email'}
                maxlength={field.field === 'api-key' ? 4096 : 320}
                aria-label={`${i18n.label(provider.provider)} ${fieldLabel(field.field)}`}
                value={fieldValue(provider.provider, field.field)}
                disabled={app.loading || !canMutate}
                placeholder={field.field === 'api-key'
                  ? i18n.t(field.configured ? 'providers.keyReplacePlaceholder' : 'providers.keyPlaceholder')
                  : i18n.t(field.configured ? 'providers.contactReplacePlaceholder' : 'providers.contactPlaceholder')}
                oninput={(event) => updateField(provider.provider, field.field, event.currentTarget.value)}
              />
              <Button
                variant="outline"
                disabled={app.loading || !canMutate || fieldValue(provider.provider, field.field).length === 0}
                onclick={() => previewField(provider.provider, field.field, 'replace')}
              >
                <ShieldCheck size={14} aria-hidden="true" />
                {i18n.t(field.configured ? 'providers.previewReplace' : 'providers.previewSave')}
              </Button>
              <Button
                variant="ghost"
                disabled={app.loading || !canMutate || !field.configured}
                onclick={() => previewField(provider.provider, field.field, 'remove')}
              >
                <Trash2 size={14} aria-hidden="true" />
                {i18n.t('providers.previewRemove')}
              </Button>
            </div>
          {/each}

          <div class="provider-footer">
            <small>{configurationSummary(provider)}</small>
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
  .field-label { width: 76px; flex: 0 0 auto; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 720; }
  .credential-row :global(input) { flex: 1; }
  .provider-footer { justify-content: space-between; margin-top: 7px; color: var(--color-muted); }
  .provider-footer small { font-size: var(--font-size-micro); }
  :global(.provider-actions) { justify-content: flex-end; padding: 10px var(--ui-panel-padding); }
  @media (max-width: 760px) {
    .provider-list { grid-template-columns: 1fr; }
    article:nth-child(odd) { border-right: 0; }
    .credential-row { align-items: stretch; flex-direction: column; }
    .field-label { width: auto; }
  }
</style>
