<script lang="ts">
  import { BookOpen, CheckCircle2, ChevronDown, CircleDot, Cloud, FolderOpen, KeyRound, Laptop, PackageCheck, PackageOpen, PackagePlus, PlugZap, RefreshCw, SearchCheck, ShieldAlert, Trash2, Wrench } from '@lucide/svelte';

  import type { AppIntent, AppSnapshot, IntegrationSelection, IntegrationTarget } from '@qiongli/app-api';
  import {
    connectionStatus,
    integrationBatchActions,
    integrationEligible,
    integrationForTarget,
    integrationSelectionDisabled,
    integrationTabTarget
  } from '$lib/features/client-integrations';
  import WorkflowContentPanel from '$lib/features/client-integrations/WorkflowContentPanel.svelte';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let selected = $state<IntegrationSelection>({ codex: true, claudeCode: true });
  let activeTarget = $state<IntegrationTarget>('codex');
  let expanded = $state(false);
  let initializedSelection = false;
  type ProviderConflict = AppSnapshot['legacyMigration']['providerConflicts'][number];
  type ProviderStrategy = Extract<AppIntent, { action: 'prepare-legacy-migration' }>['providerResolutions'][number]['strategy'];
  let providerStrategies = $state<Partial<Record<ProviderConflict['provider'], ProviderStrategy>>>({});

  let activeIntegration = $derived(
    integrationForTarget(app.snapshot, activeTarget)
  );
  let codexIntegration = $derived(
    integrationForTarget(app.snapshot, 'codex')
  );
  let claudeIntegration = $derived(
    integrationForTarget(app.snapshot, 'claude-code')
  );
  let zotero = $derived(app.snapshot?.zotero ?? null);
  let legacyCredential = $derived(app.snapshot?.configuration.legacyCredential ?? null);
  let batchActions = $derived(integrationBatchActions(app.snapshot, selected));

  $effect(() => {
    if (app.snapshot && codexIntegration && claudeIntegration && !initializedSelection) {
      selected = {
        codex: integrationEligible(codexIntegration),
        claudeCode: integrationEligible(claudeIntegration)
      };
      if (!integrationEligible(codexIntegration) && integrationEligible(claudeIntegration)) {
        activeTarget = 'claude-code';
      }
      initializedSelection = true;
    }
  });

  $effect(() => {
    if (!initializedSelection || !app.snapshot || !codexIntegration || !claudeIntegration) return;
    const nextSelection = {
      codex: selected.codex && integrationEligible(codexIntegration),
      claudeCode: selected.claudeCode && integrationEligible(claudeIntegration)
    };
    if (nextSelection.codex !== selected.codex || nextSelection.claudeCode !== selected.claudeCode) {
      selected = nextSelection;
    }
  });

  $effect(() => {
    for (const conflict of app.snapshot?.legacyMigration.providerConflicts ?? []) {
      if (!providerStrategies[conflict.provider]) {
        providerStrategies[conflict.provider] = conflict.defaultStrategy;
      }
    }
  });

  function isSelected(target: IntegrationTarget): boolean {
    return target === 'codex' ? selected.codex : selected.claudeCode;
  }

  function setSelected(target: IntegrationTarget, value: boolean): void {
    if (target === 'codex') selected.codex = value;
    else selected.claudeCode = value;
  }

  function activate(target: IntegrationTarget): void {
    activeTarget = target;
    expanded = false;
  }

  function handleTabKey(event: KeyboardEvent): void {
    const nextTarget = integrationTabTarget(activeTarget, event.key);
    if (!nextTarget) return;
    event.preventDefault();
    activate(nextTarget);
    window.setTimeout(() => document.getElementById(`tab-${nextTarget}`)?.focus());
  }

  async function rediscover(): Promise<void> {
    await app.execute({ action: 'refresh-integration-discovery' });
  }

  async function previewSelected(): Promise<void> {
    await app.execute({ action: 'preview-install-selected', selection: selected });
  }

  async function verifySelected(): Promise<void> {
    await app.execute({ action: 'verify-integrations', selection: selected });
  }

  async function reconcileSelected(): Promise<void> {
    await app.execute({ action: 'preview-reconcile-integrations', selection: selected });
  }

  async function removeSelected(): Promise<void> {
    await app.execute({ action: 'preview-remove-integrations', selection: selected });
  }
  async function previewCredentialRemoval(): Promise<void> {
    await app.execute({ action: 'preview-remove-agent-backend-credential' });
  }

  async function refreshZotero(): Promise<void> {
    await app.execute({ action: 'refresh-zotero-integration' });
  }

  async function prepareZotero(): Promise<void> {
    await app.execute({ action: 'preview-zotero-companion-stage' });
  }

  async function revealZoteroCompanion(): Promise<void> {
    await app.execute({ action: 'reveal-zotero-companion' });
  }

  async function openZotero(): Promise<void> {
    await app.execute({ action: 'open-zotero' });
  }

  async function verifyZotero(): Promise<void> {
    await app.execute({ action: 'verify-zotero-integration' });
  }

  function zoteroStateDetail(state: AppSnapshot['zotero']['state']): string {
    return i18n.t(`integrations.zoteroState.${state}`);
  }

  function formatArtifactBytes(size: number | null): string {
    if (size === null) return i18n.label('unknown');
    return `${(size / 1024).toFixed(1)} KiB`;
  }

  function migrationTitle(state: string): string {
    if (state === 'review-required') return i18n.t('integrations.migrationReviewTitle');
    if (state === 'recovery-required') return i18n.t('integrations.migrationRecoveryTitle');
    if (state === 'complete') return i18n.t('integrations.migrationCompleteTitle');
    if (state === 'available') return i18n.t('integrations.migrationAvailableTitle');
    if (state === 'unavailable') return i18n.t('integrations.migrationUnavailableTitle');
    return i18n.t('integrations.migrationInProgressTitle');
  }

  function migrationDetail(state: string, eligible: number, review: number): string {
    if (state === 'review-required') {
      return i18n.t('integrations.migrationReviewDetail', { review });
    }
    if (state === 'recovery-required') {
      return i18n.t('integrations.migrationRecoveryDetail');
    }
    if (state === 'complete') {
      return i18n.t('integrations.migrationCompleteDetail');
    }
    if (state === 'available') {
      return i18n.t('integrations.migrationAvailableDetail', { eligible });
    }
    if (state === 'unavailable') {
      return i18n.t('integrations.migrationUnavailableDetail');
    }
    return i18n.t('integrations.migrationInProgressDetail', {
      state: i18n.label(state)
    });
  }

  async function advanceLegacyMigration(): Promise<void> {
    const action = app.snapshot?.legacyMigration.nextAction;
    if (!action || action === 'none' || action === 'review') return;
    await app.execute(
      action === 'start'
        ? {
            action: 'prepare-legacy-migration',
            providerResolutions: (app.snapshot?.legacyMigration.providerConflicts ?? []).map(
              (conflict) => ({
                provider: conflict.provider,
                strategy: providerStrategies[conflict.provider] ?? conflict.defaultStrategy
              })
            )
          }
        : { action: 'preview-legacy-migration-next' }
    );
  }
</script>

<svelte:head>
  <title>{i18n.t('integrations.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('integrations.eyebrow')}
  title={i18n.t('integrations.title')}
  description={i18n.t('integrations.description')}
>
  {#snippet actions()}
    <button class="button-secondary" type="button" disabled={app.loading} onclick={rediscover}>
      <RefreshCw size={15} aria-hidden="true" />{i18n.t('integrations.refresh')}
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot || !activeIntegration}
  <section
    class="surface empty"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >{i18n.t('integrations.loading')}</section>
{:else}
  <section class="authority surface" class:installable={app.snapshot.capabilities.apply}>
    {#if app.snapshot.capabilities.apply}<CheckCircle2 size={18} aria-hidden="true" />{:else}<ShieldAlert size={18} aria-hidden="true" />{/if}
    <div>
      <strong>{i18n.dynamic(app.snapshot.product.trust.label)}</strong>
      <details>
        <summary>{i18n.t('common.details')}</summary>
        <p>{i18n.t(app.snapshot.capabilities.apply ? 'integrations.applyNotice' : 'integrations.inspectNotice')}</p>
        <code>{app.snapshot.product.trust.reasonCode}</code>
      </details>
    </div>
  </section>

  {#if app.snapshot.legacyMigration.state !== 'not-detected'}
    <section
      class="migration surface"
      class:review={app.snapshot.legacyMigration.state === 'review-required'}
      class:recovery={app.snapshot.legacyMigration.state === 'recovery-required'}
      class:complete={app.snapshot.legacyMigration.state === 'complete'}
      role="status"
    >
      <ShieldAlert size={18} aria-hidden="true" />
      <div>
        <strong>{migrationTitle(app.snapshot.legacyMigration.state)}</strong>
        <p>{migrationDetail(
          app.snapshot.legacyMigration.state,
          app.snapshot.legacyMigration.eligibleItems,
          app.snapshot.legacyMigration.reviewItems
        )}</p>
        <details class="migration-details">
          <summary>{i18n.t('common.details')}</summary>
          <p class="project-note">{i18n.t('integrations.migrationProjectNote')}</p>
          <small>{app.snapshot.legacyMigration.reasonCode}</small>
        </details>
        {#if app.snapshot.legacyMigration.providerConflicts.length > 0 && app.snapshot.legacyMigration.nextAction === 'start'}
          <div class="provider-conflicts">
            <strong>{i18n.t('integrations.providerConflictTitle')}</strong>
            <p>{i18n.t('integrations.providerConflictDetail')}</p>
            {#each app.snapshot.legacyMigration.providerConflicts as conflict}
              <label>
                <span>
                  <b>{i18n.label(conflict.provider)}</b>
                  <small>{conflict.differingFields.map((field) => i18n.label(field)).join(' · ')}</small>
                  {#if conflict.legacySecretPresent}
                    <small>{i18n.t('integrations.providerSecretReferenceOnly')}</small>
                  {/if}
                </span>
                <select
                  value={providerStrategies[conflict.provider] ?? conflict.defaultStrategy}
                  disabled={app.loading}
                  onchange={(event) => providerStrategies[conflict.provider] = event.currentTarget.value as ProviderStrategy}
                >
                  <option value="keep-v2">{i18n.t('integrations.providerStrategy.keep-v2')}</option>
                  <option value="merge-compatible">{i18n.t('integrations.providerStrategy.merge-compatible')}</option>
                  <option value="use-legacy">{i18n.t('integrations.providerStrategy.use-legacy')}</option>
                </select>
              </label>
            {/each}
          </div>
        {/if}
      </div>
      {#if app.snapshot.legacyMigration.nextAction !== 'none' && app.snapshot.legacyMigration.nextAction !== 'review'}
        <button
          class="button-secondary"
          type="button"
          disabled={app.loading || !app.snapshot.capabilities.apply}
          onclick={advanceLegacyMigration}
        >
          {i18n.t(`integrations.migrationAction.${app.snapshot.legacyMigration.nextAction}`)}
        </button>
      {:else}
        <code>{app.snapshot.legacyMigration.state}</code>
      {/if}
    </section>
  {/if}

  {#if legacyCredential?.referencePresent}
    <section id="legacy-credential-cleanup" class="legacy-credential-cleanup surface" aria-labelledby="legacy-credential-title">
      <KeyRound size={18} aria-hidden="true" />
      <div>
        <strong id="legacy-credential-title">{i18n.t('backend.legacyCredentialTitle')}</strong>
        <p>{i18n.t('backend.legacyCredentialHelp')}</p>
      </div>
      <button
        class="button-danger"
        type="button"
        disabled={app.loading || !legacyCredential.cleanupAvailable}
        onclick={previewCredentialRemoval}
      >
        <Trash2 size={15} aria-hidden="true" />
        {i18n.t('backend.legacyRemove')}
      </button>
    </section>
  {/if}

  {#if zotero}
    <section class="zotero surface" aria-labelledby="zotero-integration-title">
      <header class="zotero-header">
        <span class="zotero-mark"><BookOpen size={20} aria-hidden="true" /></span>
        <div>
          <p class="eyebrow">{i18n.t('integrations.zoteroEyebrow')}</p>
          <h2 id="zotero-integration-title">{i18n.t('integrations.zoteroTitle')}</h2>
          <p>{zoteroStateDetail(zotero.state)}</p>
        </div>
        <StatusBadge status={zotero.status} label={i18n.label(zotero.state)} />
      </header>

      <div class="zotero-facts">
        <div><span>{i18n.t('integrations.zoteroAvailableVersion')}</span><strong>{zotero.availableCompanionVersion ?? i18n.label('unavailable')}</strong></div>
        <div><span>{i18n.t('integrations.zoteroDetectedVersion')}</span><strong>{zotero.zoteroVersion ?? i18n.label('not-observed')}</strong></div>
        <div><span>{i18n.t('integrations.zoteroSupportedRange')}</span><strong>{zotero.supportedZoteroMinVersion} – {zotero.supportedZoteroMaxVersion}</strong></div>
        <div><span>{i18n.t('integrations.zoteroEndpoint')}</span><strong>{zotero.endpointVersion ?? '—'} / {zotero.supportedEndpointVersion}</strong></div>
        <div><span>{i18n.t('integrations.zoteroArtifactSize')}</span><strong>{formatArtifactBytes(zotero.availableCompanionSizeBytes)}</strong></div>
      </div>

      <details class="zotero-technical">
        <summary>{i18n.t('common.details')}</summary>
        <dl>
          <div><dt>{i18n.t('integrations.zoteroDigest')}</dt><dd><code>{zotero.availableCompanionSha256 ?? '—'}</code></dd></div>
          <div><dt>{i18n.t('integrations.evidence')}</dt><dd><code>{zotero.reasonCode}</code></dd></div>
        </dl>
      </details>

      <div class="zotero-boundary">
        <PlugZap size={17} aria-hidden="true" />
        <div>
          <strong>{i18n.t('integrations.zoteroAccessTitle')}</strong>
          <p>{i18n.t('integrations.zoteroAccessDetail')}</p>
          <small>{i18n.t('integrations.zoteroRestartDetail')}</small>
        </div>
      </div>

      <div class="zotero-footer">
        <p>{i18n.t('integrations.zoteroFallback', { formats: zotero.fallbackFormats.join(', ') })}</p>
        <div class="zotero-actions">
          <button class="button-secondary" type="button" disabled={app.loading} onclick={refreshZotero}><RefreshCw size={15} aria-hidden="true" />{i18n.t('integrations.zoteroRefresh')}</button>
          <button class="button-secondary" type="button" disabled={app.loading || !zotero.canReveal} onclick={revealZoteroCompanion}><FolderOpen size={15} aria-hidden="true" />{i18n.t('integrations.zoteroReveal')}</button>
          <button class="button-secondary" type="button" disabled={app.loading || !zotero.canOpenZotero} onclick={openZotero}><BookOpen size={15} aria-hidden="true" />{i18n.t('integrations.zoteroOpen')}</button>
          <button class="button-secondary" type="button" disabled={app.loading || !zotero.canVerify} onclick={verifyZotero}><SearchCheck size={15} aria-hidden="true" />{i18n.t('integrations.zoteroVerify')}</button>
          <button class="button-primary" type="button" disabled={app.loading || !zotero.canPrepareInstall || zotero.installationPrepared} onclick={prepareZotero}><PackageCheck size={15} aria-hidden="true" />{i18n.t(zotero.installationPrepared ? 'integrations.zoteroPrepared' : 'integrations.zoteroPrepare')}</button>
        </div>
      </div>
    </section>
  {/if}

  <div class="tabs" role="tablist" aria-label={i18n.t('integrations.eyebrow')}>
    {#each app.snapshot.integrations as integration}
      <button
        id={`tab-${integration.target}`}
        type="button"
        role="tab"
        aria-selected={activeTarget === integration.target}
        aria-controls={`panel-${integration.target}`}
        tabindex={activeTarget === integration.target ? 0 : -1}
        onclick={() => activate(integration.target)}
        onkeydown={handleTabKey}
      >
        <CircleDot size={16} aria-hidden="true" />
        <span><strong>{integration.label}</strong><small>{integration.client.detected ? integration.client.version ?? i18n.label('unknown') : i18n.label('missing')}</small></span>
        <StatusBadge status={connectionStatus(integration.connection.state)} label={i18n.label(integration.connection.state)} />
      </button>
    {/each}
  </div>

  <div id={`panel-${activeIntegration.target}`} role="tabpanel" aria-labelledby={`tab-${activeIntegration.target}`} class="surface client-panel">
    <header class="client-header">
      <div class="client-title">
        <span class="client-mark"><CircleDot size={20} aria-hidden="true" /></span>
        <div><h2>{activeIntegration.label}</h2><p>{i18n.dynamic(activeIntegration.discovery)}</p></div>
      </div>
      <div class="headline-facts">
        <div><span>{i18n.t('integrations.clientVersion')}</span><strong>{activeIntegration.client.version ?? i18n.label('missing')}</strong></div>
        <div><span>{i18n.t('integrations.pluginVersion')}</span><strong>{activeIntegration.plugin.installedVersion ?? i18n.t('integrations.notInstalled')}</strong><small>{i18n.t('integrations.availableVersion', { version: activeIntegration.plugin.availableVersion })}</small></div>
        <div><span>{i18n.t('integrations.connection')}</span><StatusBadge status={connectionStatus(activeIntegration.connection.state)} label={i18n.label(activeIntegration.connection.state)} /></div>
      </div>
    </header>

    <div class="content-grid">
      <div><span>{i18n.t('integrations.source')}</span><StatusBadge status={activeIntegration.managedContent.source} /></div>
      <div><span>{i18n.t('integrations.skills')}</span><StatusBadge status={activeIntegration.managedContent.skills} /></div>
      <div><span>{i18n.t('integrations.marketplace')}</span><StatusBadge status={activeIntegration.managedContent.marketplace} /></div>
      <div><span>{i18n.t('integrations.registration')}</span><StatusBadge status={activeIntegration.managedContent.registration} /></div>
      <div><span>{i18n.t('integrations.activation')}</span><span class="observed"><StatusBadge status={activeIntegration.managedContent.activation} /><small>{i18n.label(activeIntegration.managedContent.activationObservation)}</small></span></div>
      <div><span>{i18n.t('integrations.mcp')}</span><span class="observed"><StatusBadge status={activeIntegration.managedContent.mcpAttachment} /><small>{i18n.label(activeIntegration.managedContent.mcpAttachmentObservation)}</small></span></div>
    </div>

    <div class="meta-grid">
      <div><strong>{i18n.t('integrations.location')}</strong><span>{i18n.dynamic(activeIntegration.symbolicLocation)}</span></div>
      <div><strong>{i18n.t('integrations.policy')}</strong><span>{i18n.dynamic(activeIntegration.activationPolicy)}</span></div>
      <div><strong>{i18n.t('integrations.ownership')}</strong><span>{i18n.dynamic(activeIntegration.ownership)}</span></div>
      <div class:attention={['upgrade-client', 'resolve-conflict', 'unavailable'].includes(activeIntegration.nextAction)}>
        <strong>{i18n.t('integrations.nextStep')}</strong>
        <span>{i18n.t(`integrations.nextAction.${activeIntegration.nextAction}`)}</span>
      </div>
    </div>

    <div class="panel-footer">
      <label class="include"><input type="checkbox" checked={isSelected(activeIntegration.target)} disabled={integrationSelectionDisabled(activeIntegration, app.loading)} onchange={(event) => setSelected(activeIntegration.target, event.currentTarget.checked)} />{i18n.t('integrations.include')}</label>
      <button class="paths-toggle" type="button" aria-expanded={expanded} onclick={() => expanded = !expanded}><ChevronDown size={15} class={expanded ? 'rotated' : undefined} aria-hidden="true" />{i18n.t('integrations.paths')} ({activeIntegration.paths.length})</button>
    </div>

    {#if expanded}
      <div class="paths">
        {#if activeIntegration.paths.length === 0}<p>{i18n.t('integrations.noPaths')}</p>{:else}
          {#each activeIntegration.paths as path}<div><code>{path.symbolicPath}</code><span>{path.surface} · {path.scope} · {path.management}</span><StatusBadge status={path.state} /></div>{/each}
        {/if}
        <small class="evidence">{i18n.t('integrations.evidence')}: <code>{activeIntegration.evidenceCode}</code></small>
      </div>
    {/if}
  </div>

  <section class="action-bar surface" aria-busy={app.loading}>
    <div class="selection"><strong>{[selected.codex && 'Codex', selected.claudeCode && 'Claude Code'].filter(Boolean).join(' + ') || i18n.label('none')}</strong><span>{i18n.t('integrations.batchScope')}</span></div>
    <div class="actions">
      <button class="button-secondary" type="button" disabled={app.loading || !batchActions.verify} onclick={verifySelected}><SearchCheck size={15} aria-hidden="true" />{i18n.t('integrations.verify')}</button>
      <button class="button-secondary" type="button" disabled={app.loading || !batchActions.reconcile} onclick={reconcileSelected}><Wrench size={15} aria-hidden="true" />{i18n.t('integrations.reconcile')}</button>
      <button class="button-danger" type="button" disabled={app.loading || !batchActions.remove} onclick={removeSelected}><Trash2 size={15} aria-hidden="true" />{i18n.t('integrations.remove')}</button>
      <button class="button-primary" type="button" disabled={app.loading || !batchActions.install} onclick={previewSelected}><PackagePlus size={15} aria-hidden="true" />{i18n.t('integrations.install')}</button>
    </div>
  </section>

  <WorkflowContentPanel />

  <details class="execution-surfaces" aria-labelledby="execution-surfaces-title">
    <summary class="surface-heading">
      <div>
        <p class="eyebrow">{i18n.t('integrations.surfacesEyebrow')}</p>
        <h2 id="execution-surfaces-title">{i18n.t('integrations.surfacesTitle')}</h2>
      </div>
      <span>{i18n.t('common.details')}</span>
    </summary>

    <p class="surface-description">{i18n.t('integrations.surfacesDescription')}</p>

    <div class="surface-grid">
      <article class="surface surface-card">
        <Laptop size={19} aria-hidden="true" />
        <div>
          <h3>{i18n.t('integrations.codexLocalTitle')}</h3>
          <p>{i18n.t('integrations.codexLocalDescription')}</p>
        </div>
        <StatusBadge
          status={codexIntegration?.managedContent.mcpAttachment === 'ready' ? 'ready' : 'attention'}
          label={i18n.t('integrations.fullLocal')}
        />
      </article>

      <article class="surface surface-card">
        <Laptop size={19} aria-hidden="true" />
        <div>
          <h3>{i18n.t('integrations.claudeCodeLocalTitle')}</h3>
          <p>{i18n.t('integrations.claudeCodeLocalDescription')}</p>
        </div>
        <StatusBadge
          status={claudeIntegration?.managedContent.mcpAttachment === 'ready' ? 'ready' : 'attention'}
          label={i18n.t('integrations.fullLocal')}
        />
      </article>

      <article class="surface surface-card">
        <PackageOpen size={19} aria-hidden="true" />
        <div>
          <h3>{i18n.t('integrations.claudeDesktopTitle')}</h3>
          <p>{i18n.t('integrations.claudeDesktopDescription')}</p>
        </div>
        <StatusBadge status="attention" label={i18n.t('integrations.manualMcpb')} />
      </article>

      <article class="surface surface-card">
        <Cloud size={19} aria-hidden="true" />
        <div>
          <h3>{i18n.t('integrations.remoteTitle')}</h3>
          <p>{i18n.t('integrations.remoteDescription')}</p>
        </div>
        <StatusBadge status="disabled" label={i18n.t('integrations.remoteOnly')} />
      </article>
    </div>

    <p class="surface-note">{i18n.t('integrations.surfaceEvidenceNote')}</p>
  </details>
{/if}

<style>
  .empty { padding: 20px; color: var(--color-muted); }
  .authority { display: grid; grid-template-columns: auto minmax(0, 1fr); align-items: center; gap: 10px; margin-bottom: 10px; border-color: #fde68a; padding: 10px 12px; color: #854d0e; background: var(--color-warning-soft); }
  .authority.installable { border-color: #a7f3d0; color: #065f46; background: var(--color-success-soft); }
  .authority strong { font-size: 11px; }
  .authority details, .migration-details { margin-top: 3px; }
  .authority summary, .migration-details summary { width: fit-content; cursor: pointer; color: inherit; font-size: var(--font-size-micro); font-weight: 750; }
  .authority p { margin: 5px 0 0; color: inherit; font-size: var(--font-size-label); line-height: 1.35; }
  .authority code { color: inherit; font-size: var(--font-size-label); }
  .migration { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; margin-bottom: 10px; border-color: #7dd3fc; padding: 10px 12px; color: #075985; background: #f0f9ff; }
  .migration.review { border-color: #fbbf24; color: #92400e; background: var(--color-warning-soft); }
  .migration.recovery { border-color: #fca5a5; color: #991b1b; background: #fef2f2; }
  .migration.complete { border-color: #a7f3d0; color: #065f46; background: var(--color-success-soft); }
  .migration strong { font-size: 11px; }
  .migration p { margin: 2px 0 0; color: inherit; font-size: 10px; line-height: 1.4; }
  .migration .project-note { margin-top: 5px; opacity: .82; }
  .migration small { display: block; margin-top: 3px; color: inherit; font-family: var(--font-mono); font-size: var(--font-size-micro); opacity: .75; }
  .migration code { color: inherit; font-size: var(--font-size-label); }
  .legacy-credential-cleanup { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; margin-bottom: 10px; border-color: #fca5a5; padding: 10px 12px; color: #991b1b; background: #fef2f2; }
  .legacy-credential-cleanup strong { display: block; font-size: 11px; }
  .legacy-credential-cleanup p { margin: 2px 0 0; color: inherit; font-size: var(--font-size-label); line-height: 1.4; }
  .legacy-credential-cleanup button { display: inline-flex; min-height: 40px; align-items: center; gap: 6px; white-space: nowrap; }
  .provider-conflicts { display: grid; gap: 6px; margin-top: 9px; border-top: 1px solid rgb(7 89 133 / .2); padding-top: 8px; }
  .provider-conflicts > p { margin: 0; }
  .provider-conflicts label { display: grid; grid-template-columns: minmax(0, 1fr) minmax(150px, auto); align-items: center; gap: 10px; }
  .provider-conflicts b { display: block; font-size: 10px; }
  .provider-conflicts select { min-height: 36px; border: 1px solid var(--color-border-strong); border-radius: 8px; padding: 5px 8px; color: var(--color-ink); background: white; font-size: 10px; }
  .zotero { overflow: hidden; margin-bottom: 10px; }
  .zotero-header { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 11px; padding: 13px 14px; }
  .zotero-mark { display: grid; width: 38px; height: 38px; place-items: center; border-radius: 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .zotero-header .eyebrow { margin: 0 0 2px; font-size: var(--font-size-micro); }
  .zotero-header h2 { font-size: 14px; }
  .zotero-header p:last-child { margin: 3px 0 0; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.4; }
  .zotero-facts { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 7px; border-block: 1px solid var(--color-border); padding: 9px 11px; background: var(--color-surface-subtle); }
  .zotero-facts > div { min-width: 0; border: 1px solid var(--color-border); border-radius: 8px; padding: 8px 9px; background: white; }
  .zotero-facts span, .zotero-facts strong { display: block; }
  .zotero-facts span { margin-bottom: 3px; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 750; text-transform: uppercase; }
  .zotero-facts strong { overflow-wrap: anywhere; color: var(--color-ink-strong); font-size: var(--font-size-label); }
  .zotero-technical { border-bottom: 1px solid var(--color-border); padding: 0 14px; color: var(--color-muted); }
  .zotero-technical summary { width: fit-content; min-height: 36px; padding-block: 8px; cursor: pointer; font-size: var(--font-size-micro); font-weight: 750; }
  .zotero-technical dl { display: grid; gap: 6px; margin: 0; padding-bottom: 10px; }
  .zotero-technical dl > div { min-width: 0; }
  .zotero-technical dt { font-size: var(--font-size-micro); font-weight: 750; text-transform: uppercase; }
  .zotero-technical dd { margin: 3px 0 0; }
  .zotero-technical code { display: block; overflow-wrap: anywhere; color: var(--color-ink); font-size: var(--font-size-micro); }
  .zotero-boundary { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 9px; padding: 11px 14px; color: #075985; background: #f0f9ff; }
  .zotero-boundary strong { display: block; font-size: 10px; }
  .zotero-boundary p, .zotero-boundary small { display: block; margin: 3px 0 0; color: inherit; font-size: var(--font-size-label); line-height: 1.45; }
  .zotero-boundary small { opacity: .78; }
  .zotero-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-top: 1px solid var(--color-border); padding: 9px 12px; }
  .zotero-footer > p { max-width: 480px; margin: 0; color: var(--color-muted); font-size: var(--font-size-micro); line-height: 1.4; }
  .zotero-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 6px; }
  .zotero-actions button { min-height: 40px; font-size: var(--font-size-label); }
  .tabs { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px; margin-bottom: 8px; }
  .tabs > button { display: grid; min-height: 48px; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 9px; border: 1px solid var(--color-border); border-radius: 10px; padding: 7px 10px; color: var(--color-muted); background: var(--color-surface-subtle); text-align: left; }
  .tabs > button[aria-selected='true'] { border-color: var(--color-accent); color: var(--color-accent-strong); background: white; box-shadow: 0 0 0 2px rgb(3 105 161 / .1); }
  .tabs strong, .tabs small { display: block; }
  .tabs strong { color: var(--color-ink-strong); font-size: 12px; }
  .tabs small { margin-top: 2px; font-size: var(--font-size-label); }
  .client-panel { overflow: hidden; }
  .client-header { display: flex; align-items: center; justify-content: space-between; gap: 18px; padding: 12px 14px; }
  .client-title { display: flex; min-width: 220px; align-items: center; gap: 9px; }
  .client-mark { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 9px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 16px; }
  .client-title p { margin: 3px 0 0; color: var(--color-muted); font-size: var(--font-size-label); }
  .headline-facts { display: flex; align-items: center; }
  .headline-facts > div { min-width: 126px; border-left: 1px solid var(--color-border); padding: 2px 12px; }
  .headline-facts span, .headline-facts strong, .headline-facts small { display: block; }
  .headline-facts > div > span { margin-bottom: 3px; color: var(--color-muted); font-size: var(--font-size-label); font-weight: 750; }
  .headline-facts strong { color: var(--color-ink-strong); font-size: 11px; }
  .headline-facts small { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-micro); }
  .content-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-block: 1px solid var(--color-border); background: var(--color-surface-subtle); }
  .content-grid > div { display: flex; min-height: 46px; align-items: center; justify-content: space-between; gap: 8px; border-right: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); padding: 7px 10px; }
  .content-grid > div:nth-child(3n) { border-right: 0; }
  .content-grid > div:nth-last-child(-n + 3) { border-bottom: 0; }
  .content-grid > div > span:first-child { color: var(--color-muted); font-size: 10px; font-weight: 700; }
  .observed { display: flex; align-items: flex-end; flex-direction: column; gap: 2px; }
  .observed small { color: var(--color-muted); font-size: var(--font-size-micro); }
  .meta-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; padding: 10px 14px; }
  .meta-grid strong, .meta-grid span { display: block; }
  .meta-grid strong { margin-bottom: 3px; color: var(--color-muted); font-size: var(--font-size-micro); letter-spacing: .04em; text-transform: uppercase; }
  .meta-grid span { overflow-wrap: anywhere; color: var(--color-ink); font-size: var(--font-size-label); }
  .meta-grid .attention span { color: var(--color-warning); font-weight: 750; }
  .panel-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-top: 1px solid var(--color-border); padding: 8px 14px; }
  .include { display: flex; min-height: 44px; align-items: center; gap: 7px; color: var(--color-ink); font-size: 10px; font-weight: 700; }
  .include input { width: 16px; height: 16px; accent-color: var(--color-accent); }
  .paths-toggle { display: flex; min-height: 44px; align-items: center; gap: 6px; border: 0; padding: 8px 4px; color: var(--color-accent-strong); background: transparent; font-size: 10px; font-weight: 700; }
  :global(.rotated) { transform: rotate(180deg); }
  .paths { border-top: 1px solid var(--color-border); padding: 0 14px 8px; }
  .paths p { color: var(--color-muted); font-size: 10px; }
  .paths > div { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 9px; border-bottom: 1px solid var(--color-border); padding: 7px 0; }
  .paths code, .paths span { overflow-wrap: anywhere; color: var(--color-muted); font-size: var(--font-size-label); }
  .paths .evidence { display: block; padding-top: 7px; color: var(--color-muted); font-size: var(--font-size-micro); }
  .action-bar { display: flex; align-items: center; justify-content: space-between; gap: 14px; margin-top: 9px; padding: 10px 12px; border-color: var(--color-border-strong); }
  .selection strong, .selection span { display: block; }
  .selection strong { color: var(--color-ink-strong); font-size: 11px; }
  .selection span { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-micro); }
  .actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 6px; }
  .actions button { min-height: 44px; font-size: 10px; }
  .execution-surfaces { margin-top: 14px; }
  .surface-heading { display: flex; min-height: 48px; align-items: center; justify-content: space-between; gap: 20px; border-block: 1px solid var(--color-border); padding: 8px 2px; cursor: pointer; }
  .surface-heading h2 { margin-top: 0; }
  .surface-heading > span { color: var(--color-accent-strong); font-size: var(--font-size-label); font-weight: 750; }
  .surface-description { max-width: 720px; margin: 9px 0; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.45; }
  .surface-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
  .surface-card { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: start; gap: 10px; padding: 13px; }
  .surface-card h3 { margin: 0 0 4px; color: var(--color-ink-strong); font-size: 12px; }
  .surface-card p { margin: 0; color: var(--color-muted); font-size: 10px; line-height: 1.45; }
  .surface-note { margin: 8px 0 0; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.45; }
  @media (max-width: 1000px) { .zotero-facts { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
  @media (max-width: 840px) { .client-header { align-items: flex-start; flex-direction: column; } .headline-facts { width: 100%; } .headline-facts > div { flex: 1; } .meta-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .zotero-facts { grid-template-columns: repeat(2, minmax(0, 1fr)); } .action-bar, .zotero-footer { align-items: flex-start; flex-direction: column; } .actions, .zotero-actions { justify-content: flex-start; } }
  @media (max-width: 700px) { .tabs, .surface-grid { grid-template-columns: 1fr; } .content-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .content-grid > div, .content-grid > div:nth-child(3n) { border-right: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); } .content-grid > div:nth-child(2n) { border-right: 0; } .content-grid > div:nth-last-child(-n + 2) { border-bottom: 0; } .panel-footer { align-items: flex-start; flex-direction: column; } .legacy-credential-cleanup { grid-template-columns: auto minmax(0, 1fr); } .legacy-credential-cleanup button { grid-column: 1 / -1; justify-self: start; } }
  @media (max-width: 460px) { .headline-facts, .actions, .zotero-actions { align-items: stretch; flex-direction: column; } .headline-facts > div { border-left: 0; border-top: 1px solid var(--color-border); } .content-grid, .meta-grid, .zotero-facts { grid-template-columns: 1fr; } .content-grid > div { border-right: 0 !important; border-bottom: 1px solid var(--color-border) !important; } .content-grid > div:last-child { border-bottom: 0 !important; } .actions button, .zotero-actions button { width: 100%; } }
</style>
