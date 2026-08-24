<script lang="ts">
  import { BookOpen, CheckCircle2, ChevronDown, CircleDot, Cloud, FolderOpen, KeyRound, Laptop, PackageCheck, PackageOpen, PackagePlus, PlugZap, RefreshCw, SearchCheck, ShieldAlert, TerminalSquare, Trash2, Wrench } from '@lucide/svelte';

  import type { AppIntent, AppSnapshot, IntegrationSelection, IntegrationTarget } from '@qiongli/app-api';
  import {
    connectionStatus,
    integrationBatchActions,
    integrationEligible,
    integrationActivationCommands,
    integrationForTarget,
    hostIntegrationSkillsDetached,
    hostIntegrationSkillsStatus,
    integrationSelectionDisabled,
    integrationSetupStage
  } from '$lib/features/client-integrations';
  import LiteratureProvidersPanel from '$lib/features/client-integrations/LiteratureProvidersPanel.svelte';
  import WorkflowContentPanel from '$lib/features/client-integrations/WorkflowContentPanel.svelte';
  import {
    ActionGroup,
    ContentGrid,
    DescriptionGrid,
    DescriptionTip,
    PageLayout,
    SectionHeader,
    StatePanel,
    StatusBadge,
    TabsContent,
    TabsList,
    TabsRoot,
    TabsTrigger
  } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { NativeSelect } from '$lib/components/ui/native-select';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  type IntegrationSection = 'agents' | 'mcp' | 'migration' | 'zotero' | 'skills';
  let selected = $state<IntegrationSelection>({ codex: true, claudeCode: true });
  let activeSection = $state<IntegrationSection>('agents');
  let activeTarget = $state<IntegrationTarget>('codex');
  let expanded = $state(false);
  let initializedSelection = false;
  type ProviderConflict = AppSnapshot['legacyMigration']['providerConflicts'][number];
  type AppIntegration = AppSnapshot['integrations'][number];
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
  let migrationAvailable = $derived(
    app.snapshot !== null
      && (app.snapshot.legacyMigration.state !== 'not-detected'
        || app.snapshot.configuration.legacyCredential.referencePresent)
  );
  let fullMcpIntegrationReady = $derived(
    app.snapshot?.integrations.some((integration) => integration.connection.state === 'connected') ?? false
  );

  $effect(() => {
    if (app.mcpSelfTest?.state !== 'running') return;
    const timer = window.setTimeout(() => {
      void app.execute(
        { action: 'poll-full-mcp-self-test' },
        (event) => event.type === 'mcp-self-test-updated'
      );
    }, 50);
    return () => window.clearTimeout(timer);
  });

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

  $effect(() => {
    if ((activeSection === 'migration' && !migrationAvailable)
      || (activeSection === 'zotero' && !zotero)) {
      activeSection = 'agents';
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

  function changeActiveTarget(value: string): void {
    if (value === 'codex' || value === 'claude-code') activate(value);
  }

  function changeActiveSection(value: string): void {
    if (['agents', 'mcp', 'migration', 'zotero', 'skills'].includes(value)) {
      activeSection = value as IntegrationSection;
    }
  }

  function contentProvenance(integration: AppIntegration): string | null {
    if (integration.overall !== 'ready' || integration.nextAction !== 'current') return null;
    return i18n.t(integration.evidenceCode === 'client-managed-customized-current'
      ? 'content.variantCustomized'
      : 'content.variantCanonical');
  }

  async function rediscover(): Promise<void> {
    await app.execute({ action: 'refresh-integration-discovery' });
  }

  async function runFullMcpSelfTest(): Promise<void> {
    await app.execute(
      { action: 'run-full-mcp-self-test' },
      (event) => event.type === 'mcp-self-test-updated'
    );
  }

  async function cancelFullMcpSelfTest(): Promise<void> {
    await app.execute(
      { action: 'cancel-full-mcp-self-test' },
      (event) => event.type === 'mcp-self-test-updated'
    );
  }

  function fullMcpStatus(state: NonNullable<typeof app.mcpSelfTest>['state']) {
    if (state === 'passed') return 'ready' as const;
    if (state === 'running') return 'busy' as const;
    if (state === 'cancelled') return 'missing' as const;
    if (state === 'timed-out') return 'blocked' as const;
    return 'invalid' as const;
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

  function migrationTone(state: string): 'info' | 'success' | 'warning' | 'danger' {
    if (state === 'review-required') return 'warning';
    if (state === 'recovery-required') return 'danger';
    if (state === 'complete') return 'success';
    return 'info';
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

<PageLayout
  eyebrow={i18n.t('integrations.eyebrow')}
  title={i18n.t('integrations.title')}
  description={i18n.t('integrations.description')}
>
  {#snippet actions()}
    <Button variant="outline" disabled={app.loading} onclick={rediscover}>
      <RefreshCw size={15} aria-hidden="true" />{i18n.t('integrations.refresh')}
    </Button>
  {/snippet}

{#if !app.snapshot || !activeIntegration}
  <StatePanel
    centered
    role="status"
    busy
    live="polite"
    atomic
    description={i18n.t('integrations.loading')}
  />
{:else}
  <StatePanel tone={app.snapshot.capabilities.apply ? 'success' : 'warning'} title={i18n.dynamic(app.snapshot.product.trust.label)}>
      {#snippet icon()}
        {#if app.snapshot!.capabilities.apply}<CheckCircle2 size={18} />{:else}<ShieldAlert size={18} />{/if}
      {/snippet}
      <details class="state-details">
        <summary>{i18n.t('common.details')}</summary>
        <p>{i18n.t(app.snapshot.capabilities.apply ? 'integrations.applyNotice' : 'integrations.inspectNotice')}</p>
        <code>{app.snapshot.product.trust.reasonCode}</code>
      </details>
  </StatePanel>

  <TabsRoot value={activeSection} onValueChange={changeActiveSection} class="integration-workspace">
    <TabsList class="workspace-tabs" aria-label={i18n.t('integrations.sectionsLabel')}>
      <TabsTrigger value="agents">
        <CircleDot size={16} aria-hidden="true" />
        <span>{i18n.t('integrations.section.agents')}</span>
      </TabsTrigger>
      <TabsTrigger value="mcp">
        <SearchCheck size={16} aria-hidden="true" />
        <span>{i18n.t('integrations.section.mcp')}</span>
      </TabsTrigger>
      {#if migrationAvailable}
        <TabsTrigger value="migration">
          <ShieldAlert size={16} aria-hidden="true" />
          <span>{i18n.t('integrations.section.migration')}</span>
        </TabsTrigger>
      {/if}
      {#if zotero}
        <TabsTrigger value="zotero">
          <BookOpen size={16} aria-hidden="true" />
          <span>{i18n.t('integrations.section.zotero')}</span>
        </TabsTrigger>
      {/if}
      <TabsTrigger value="skills">
        <PackageOpen size={16} aria-hidden="true" />
        <span>{i18n.t('integrations.section.skills')}</span>
      </TabsTrigger>
    </TabsList>

    <TabsContent value="mcp" class="workspace-panel">
      <LiteratureProvidersPanel />
      <Card.Root class="full-mcp-self-test" role="status" aria-live="polite" aria-atomic="true">
        <header>
          <div>
            <p class="eyebrow">{i18n.t('integrations.fullMcpEyebrow')}</p>
            <h2>{i18n.t('integrations.fullMcpTitle')}</h2>
          </div>
          <StatusBadge
            status={app.mcpSelfTest ? fullMcpStatus(app.mcpSelfTest.state) : 'missing'}
            label={app.mcpSelfTest ? i18n.label(app.mcpSelfTest.state) : i18n.t('integrations.fullMcpNotRun')}
          />
        </header>
        <p>{i18n.t('integrations.fullMcpDescription')}</p>

        {#if app.mcpSelfTest}
          <DescriptionGrid class="full-mcp-facts">
            <div><dt>{i18n.t('integrations.fullMcpProfile')}</dt><dd>{app.mcpSelfTest.profile}</dd></div>
            <div><dt>{i18n.t('integrations.fullMcpVersion')}</dt><dd>{app.mcpSelfTest.productVersion}</dd></div>
            <div><dt>{i18n.t('integrations.fullMcpTools')}</dt><dd>{app.mcpSelfTest.publicToolCount}</dd></div>
          </DescriptionGrid>
          <div class="full-mcp-checks">
            {#each app.mcpSelfTest.checks as check (check.check)}
              <div>
                <span>{i18n.t(`integrations.fullMcpCheck.${check.check}`)}</span>
                <StatusBadge status={check.status} />
                <code>{check.code}</code>
              </div>
            {/each}
          </div>
        {:else if !fullMcpIntegrationReady}
          <p class="full-mcp-boundary">{i18n.t('integrations.fullMcpRequiresReady')}</p>
        {/if}

        <ActionGroup class="full-mcp-actions">
          <Button
            disabled={app.loading || !fullMcpIntegrationReady || app.mcpSelfTest?.state === 'running'}
            onclick={runFullMcpSelfTest}
          >
            <SearchCheck size={15} aria-hidden="true" />{i18n.t('integrations.fullMcpRun')}
          </Button>
          <Button
            variant="outline"
            disabled={app.mcpSelfTest?.state !== 'running'}
            onclick={cancelFullMcpSelfTest}
          >
            {i18n.t('integrations.fullMcpCancel')}
          </Button>
        </ActionGroup>
      </Card.Root>
      <details class="execution-surfaces" aria-labelledby="execution-surfaces-title">
        <summary class="surface-heading">
          <div>
            <p class="eyebrow">{i18n.t('integrations.surfacesEyebrow')}</p>
            <h2 id="execution-surfaces-title">{i18n.t('integrations.surfacesTitle')}</h2>
          </div>
          <span>{i18n.t('common.details')}</span>
        </summary>

        <p class="surface-description">{i18n.t('integrations.surfacesDescription')}</p>

        <ContentGrid columns={2} collapse="md">
          <Card.Root class="surface-card">
            <Laptop size={19} aria-hidden="true" />
            <div>
              <h3>{i18n.t('integrations.codexLocalTitle')}</h3>
              <p>{i18n.t('integrations.codexLocalDescription')}</p>
            </div>
            <StatusBadge
              status={codexIntegration?.managedContent.mcpAttachment === 'ready' ? 'ready' : 'attention'}
              label={i18n.t('integrations.fullLocal')}
            />
          </Card.Root>

          <Card.Root class="surface-card">
            <Laptop size={19} aria-hidden="true" />
            <div>
              <h3>{i18n.t('integrations.claudeCodeLocalTitle')}</h3>
              <p>{i18n.t('integrations.claudeCodeLocalDescription')}</p>
            </div>
            <StatusBadge
              status={claudeIntegration?.managedContent.mcpAttachment === 'ready' ? 'ready' : 'attention'}
              label={i18n.t('integrations.fullLocal')}
            />
          </Card.Root>

          <Card.Root class="surface-card">
            <PackageOpen size={19} aria-hidden="true" />
            <div>
              <h3>{i18n.t('integrations.claudeDesktopTitle')}</h3>
              <p>{i18n.t('integrations.claudeDesktopDescription')}</p>
            </div>
            <StatusBadge status="attention" label={i18n.t('integrations.manualMcpb')} />
          </Card.Root>

          <Card.Root class="surface-card">
            <Cloud size={19} aria-hidden="true" />
            <div>
              <h3>{i18n.t('integrations.remoteTitle')}</h3>
              <p>{i18n.t('integrations.remoteDescription')}</p>
            </div>
            <StatusBadge status="disabled" label={i18n.t('integrations.remoteOnly')} />
          </Card.Root>
        </ContentGrid>

        <p class="surface-note">{i18n.t('integrations.surfaceEvidenceNote')}</p>
      </details>
    </TabsContent>

    {#if migrationAvailable}
      <TabsContent value="migration" class="workspace-panel">
  {#if app.snapshot.legacyMigration.state !== 'not-detected'}
    <StatePanel
        tone={migrationTone(app.snapshot.legacyMigration.state)}
        role="status"
        title={migrationTitle(app.snapshot.legacyMigration.state)}
        description={migrationDetail(
          app.snapshot.legacyMigration.state,
          app.snapshot.legacyMigration.eligibleItems,
          app.snapshot.legacyMigration.reviewItems
        )}
        descriptionMode={['available', 'complete'].includes(app.snapshot.legacyMigration.state)
          ? 'tooltip'
          : 'visible'}
      >
        {#snippet icon()}<ShieldAlert size={18} />{/snippet}
        {#if !app.snapshot.capabilities.apply}
          <p class="migration-authority-note">{i18n.t('integrations.migrationAuthorityRequired')}</p>
        {/if}
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
                <NativeSelect
                  value={providerStrategies[conflict.provider] ?? conflict.defaultStrategy}
                  disabled={app.loading}
                  onchange={(event) => providerStrategies[conflict.provider] = event.currentTarget.value as ProviderStrategy}
                >
                  <option value="keep-v2">{i18n.t('integrations.providerStrategy.keep-v2')}</option>
                  <option value="merge-compatible">{i18n.t('integrations.providerStrategy.merge-compatible')}</option>
                  <option value="use-legacy">{i18n.t('integrations.providerStrategy.use-legacy')}</option>
                </NativeSelect>
              </label>
            {/each}
          </div>
        {/if}
        {#snippet metadata()}
          {#if app.snapshot!.legacyMigration.nextAction !== 'none' && app.snapshot!.legacyMigration.nextAction !== 'review'}
            <Button
              variant="outline"
              disabled={app.loading || !app.snapshot!.capabilities.apply}
              onclick={advanceLegacyMigration}
            >
              {i18n.t(`integrations.migrationAction.${app.snapshot!.legacyMigration.nextAction}`)}
            </Button>
          {:else}
            <code>{app.snapshot!.legacyMigration.state}</code>
          {/if}
        {/snippet}
    </StatePanel>
  {/if}

  {#if legacyCredential?.referencePresent}
    <StatePanel id="legacy-credential-cleanup" tone="danger" title={i18n.t('backend.legacyCredentialTitle')} description={i18n.t('backend.legacyCredentialHelp')}>
        {#snippet icon()}<KeyRound size={18} />{/snippet}
        {#snippet metadata()}
          <Button
            variant="destructive"
            disabled={app.loading || !legacyCredential.cleanupAvailable}
            onclick={previewCredentialRemoval}
          >
            <Trash2 size={15} aria-hidden="true" />
            {i18n.t('backend.legacyRemove')}
          </Button>
        {/snippet}
    </StatePanel>
  {/if}
      </TabsContent>
    {/if}

  {#if zotero}
    <TabsContent value="zotero" class="workspace-panel">
    <Card.Root class="zotero" role="region" aria-labelledby="zotero-integration-title">
      <SectionHeader variant="panel" eyebrow={i18n.t('integrations.zoteroEyebrow')} title={i18n.t('integrations.zoteroTitle')} titleId="zotero-integration-title" description={zoteroStateDetail(zotero.state)}>
        {#snippet icon()}<BookOpen size={20} />{/snippet}
        {#snippet metadata()}<StatusBadge status={zotero.status} label={i18n.label(zotero.state)} />{/snippet}
      </SectionHeader>

      <div class="zotero-facts">
        <div><span>{i18n.t('integrations.zoteroAvailableVersion')}</span><strong>{zotero.availableCompanionVersion ?? i18n.label('unavailable')}</strong></div>
        <div><span>{i18n.t('integrations.zoteroDetectedVersion')}</span><strong>{zotero.zoteroVersion ?? i18n.label('not-observed')}</strong></div>
        <div><span>{i18n.t('integrations.zoteroSupportedRange')}</span><strong>{zotero.supportedZoteroMinVersion} – {zotero.supportedZoteroMaxVersion}</strong></div>
        <div><span>{i18n.t('integrations.zoteroEndpoint')}</span><strong>{zotero.endpointVersion ?? '—'} / {zotero.supportedEndpointVersion}</strong></div>
        <div><span>{i18n.t('integrations.zoteroArtifactSize')}</span><strong>{formatArtifactBytes(zotero.availableCompanionSizeBytes)}</strong></div>
      </div>

      <details class="zotero-technical">
        <summary>{i18n.t('common.details')}</summary>
        <DescriptionGrid columns={1} compact class="zotero-technical-facts">
          <div><dt>{i18n.t('integrations.zoteroDigest')}</dt><dd><code>{zotero.availableCompanionSha256 ?? '—'}</code></dd></div>
          <div><dt>{i18n.t('integrations.evidence')}</dt><dd><code>{zotero.reasonCode}</code></dd></div>
        </DescriptionGrid>
      </details>

      <div class="zotero-boundary">
        <PlugZap size={17} aria-hidden="true" />
        <div class="zotero-boundary-title">
          <strong>{i18n.t('integrations.zoteroAccessTitle')}</strong>
          <DescriptionTip
            text={`${i18n.t('integrations.zoteroAccessDetail')} ${i18n.t('integrations.zoteroRestartDetail')}`}
            side="top"
          />
        </div>
      </div>

      <div class="zotero-footer">
        <p>{i18n.t('integrations.zoteroFallback', { formats: zotero.fallbackFormats.join(', ') })}</p>
        <ActionGroup class="zotero-actions">
          <Button variant="outline" disabled={app.loading} onclick={refreshZotero}><RefreshCw size={15} aria-hidden="true" />{i18n.t('integrations.zoteroRefresh')}</Button>
          <Button variant="outline" disabled={app.loading || !zotero.canReveal} onclick={revealZoteroCompanion}><FolderOpen size={15} aria-hidden="true" />{i18n.t('integrations.zoteroReveal')}</Button>
          <Button variant="outline" disabled={app.loading || !zotero.canOpenZotero} onclick={openZotero}><BookOpen size={15} aria-hidden="true" />{i18n.t('integrations.zoteroOpen')}</Button>
          <Button variant="outline" disabled={app.loading || !zotero.canVerify} onclick={verifyZotero}><SearchCheck size={15} aria-hidden="true" />{i18n.t('integrations.zoteroVerify')}</Button>
          <Button disabled={app.loading || !zotero.canPrepareInstall || zotero.installationPrepared} onclick={prepareZotero}><PackageCheck size={15} aria-hidden="true" />{i18n.t(zotero.installationPrepared ? 'integrations.zoteroPrepared' : 'integrations.zoteroPrepare')}</Button>
        </ActionGroup>
      </div>
    </Card.Root>
    </TabsContent>
  {/if}

  <TabsContent value="agents" class="workspace-panel agent-workspace-panel">
  <TabsRoot value={activeTarget} onValueChange={changeActiveTarget}>
    <TabsList class="integration-tabs" aria-label={i18n.t('integrations.eyebrow')}>
      {#each app.snapshot.integrations as integration}
        <TabsTrigger id={`tab-${integration.target}`} value={integration.target}>
          <CircleDot size={16} aria-hidden="true" />
          <span class="integration-tab-copy"><strong>{integration.label}</strong><small>{integration.client.detected ? integration.client.version ?? i18n.label('unknown') : i18n.label('missing')}</small></span>
          <StatusBadge status={connectionStatus(integration.connection.state)} label={i18n.label(integration.connection.state)} />
        </TabsTrigger>
      {/each}
    </TabsList>

    {#each app.snapshot.integrations as panelIntegration}
    <TabsContent id={`panel-${panelIntegration.target}`} value={panelIntegration.target} class="integration-client-panel">
      {#if activeIntegration.target === panelIntegration.target}
    <header class="client-header">
      <div class="client-title">
        <span class="client-mark"><CircleDot size={20} aria-hidden="true" /></span>
        <div class="client-title-row">
          <h2>{activeIntegration.label}</h2>
          <DescriptionTip text={i18n.dynamic(activeIntegration.discovery)} />
        </div>
      </div>
      <div class="headline-facts">
        <div><span>{i18n.t('integrations.clientVersion')}</span><strong>{activeIntegration.client.version ?? i18n.label('missing')}</strong></div>
        <div><span>{i18n.t('integrations.pluginVersion')}</span><strong>{activeIntegration.plugin.installedVersion ?? i18n.t('integrations.notInstalled')}</strong><small>{i18n.t('integrations.availableVersion', { version: activeIntegration.plugin.availableVersion })}</small>{#if contentProvenance(activeIntegration)}<small class="content-provenance">{contentProvenance(activeIntegration)}</small>{/if}</div>
        <div><span>{i18n.t('integrations.connection')}</span><StatusBadge status={connectionStatus(activeIntegration.connection.state)} label={i18n.label(activeIntegration.connection.state)} /></div>
      </div>
    </header>

    <section class="host-package" aria-labelledby="host-package-title">
      <header>
        <span class="package-mark"><PackageCheck size={18} aria-hidden="true" /></span>
        <div class="package-title-row">
          <h3 id="host-package-title">{i18n.t('integrations.hostPackage')}</h3>
          <DescriptionTip
            text={`${i18n.t('integrations.hostPackageDescription', {
              client: activeIntegration.label
            })} ${i18n.t('integrations.hostPackageBoundary')}`}
          />
        </div>
        <StatusBadge status={activeIntegration.overall} />
      </header>

      <div class="package-components">
        <article>
          <div>
            <span class="component-title">
              <strong>{i18n.t('integrations.component.plugin')}</strong>
              <DescriptionTip text={i18n.t('integrations.component.pluginDetail', {
                version: activeIntegration.plugin.installedVersion
                  ?? activeIntegration.plugin.availableVersion
              })} side="top" />
            </span>
          </div>
          <StatusBadge status={activeIntegration.managedContent.source} />
        </article>
        <article>
          <div>
            <span class="component-title">
              <strong>{i18n.t('integrations.component.skills')}</strong>
              <DescriptionTip text={i18n.t(hostIntegrationSkillsDetached(activeIntegration)
                ? 'integrations.component.skillsDetachedDetail'
                : 'integrations.component.skillsDetail')} side="top" />
            </span>
          </div>
          <StatusBadge status={hostIntegrationSkillsStatus(activeIntegration)} />
        </article>
        <article>
          <div>
            <span class="component-title">
              <strong>{i18n.t('integrations.component.registration')}</strong>
              <DescriptionTip text={i18n.t('integrations.component.registrationDetail')} side="top" />
            </span>
          </div>
          <span class="component-statuses">
            <StatusBadge status={activeIntegration.managedContent.registration} />
            <small>{i18n.t('integrations.marketplace')}: {i18n.label(activeIntegration.managedContent.marketplace)}</small>
          </span>
        </article>
        <article>
          <div>
            <span class="component-title">
              <strong>{i18n.t('integrations.component.mcp')}</strong>
              <DescriptionTip text={i18n.t('integrations.component.mcpDetail')} side="top" />
            </span>
          </div>
          <span class="component-statuses">
            <StatusBadge status={activeIntegration.managedContent.mcpAttachment} />
            <small>{i18n.label(activeIntegration.managedContent.mcpAttachmentObservation)}</small>
          </span>
        </article>
        <article>
          <div>
            <span class="component-title">
              <strong>{i18n.t('integrations.component.activation')}</strong>
              <DescriptionTip text={i18n.t('integrations.component.activationDetail')} side="top" />
            </span>
          </div>
          <span class="component-statuses">
            <StatusBadge status={activeIntegration.managedContent.activation} />
            <small>{i18n.label(activeIntegration.managedContent.activationObservation)}</small>
          </span>
        </article>
      </div>
    </section>

    <div class="meta-grid">
      <div><strong>{i18n.t('integrations.location')}</strong><span>{i18n.dynamic(activeIntegration.symbolicLocation)}</span></div>
      <div><strong>{i18n.t('integrations.policy')}</strong><span>{i18n.dynamic(activeIntegration.activationPolicy)}</span></div>
      <div><strong>{i18n.t('integrations.ownership')}</strong><span>{i18n.dynamic(activeIntegration.ownership)}</span></div>
      <div class:attention={['upgrade-client', 'resolve-conflict', 'unavailable'].includes(activeIntegration.nextAction)}>
        <strong>{i18n.t('integrations.nextStep')}</strong>
        <span>{i18n.t(`integrations.nextAction.${activeIntegration.nextAction}`)}</span>
      </div>
    </div>

    <div class="activation-guide" data-stage={integrationSetupStage(activeIntegration)}>
      <div class="activation-guide-title">
        <TerminalSquare size={15} aria-hidden="true" />
        <strong>{i18n.t(`integrations.setup.${integrationSetupStage(activeIntegration)}`)}</strong>
        <DescriptionTip
          text={i18n.t(`integrations.setupDetail.${integrationSetupStage(activeIntegration)}`, {
            client: activeIntegration.label
          })}
          side="top"
        />
      </div>
      {#if ['install', 'activate'].includes(integrationSetupStage(activeIntegration))}
        {#each integrationActivationCommands(activeIntegration) as command}
          <code>{command}</code>
        {/each}
      {/if}
    </div>

    <div class="panel-footer">
      <label class="include">
        <Checkbox
          checked={isSelected(activeIntegration.target)}
          disabled={integrationSelectionDisabled(activeIntegration, app.loading)}
          onclick={() => setSelected(activeIntegration.target, !isSelected(activeIntegration.target))}
        />
        {i18n.t('integrations.include')}
      </label>
      <Button class="paths-toggle" variant="ghost" aria-expanded={expanded} onclick={() => expanded = !expanded}><ChevronDown size={15} class={expanded ? 'rotated' : undefined} aria-hidden="true" />{i18n.t('integrations.paths')} ({activeIntegration.paths.length})</Button>
    </div>

    {#if expanded}
      <div class="paths">
        {#if activeIntegration.paths.length === 0}<p>{i18n.t('integrations.noPaths')}</p>{:else}
          {#each activeIntegration.paths as path}<div><code>{path.symbolicPath}</code><span>{path.surface} · {path.scope} · {path.management}</span><StatusBadge status={path.state} /></div>{/each}
        {/if}
        <small class="evidence">{i18n.t('integrations.evidence')}: <code>{activeIntegration.evidenceCode}</code></small>
      </div>
    {/if}
      {/if}
    </TabsContent>
    {/each}
  </TabsRoot>

  <Card.Root class="action-bar" aria-busy={app.loading}>
    <div class="selection"><strong>{[selected.codex && 'Codex', selected.claudeCode && 'Claude Code'].filter(Boolean).join(' + ') || i18n.label('none')}</strong><span>{i18n.t('integrations.batchScope')}</span></div>
    <ActionGroup class="actions">
      <Button variant="outline" disabled={app.loading || !batchActions.verify} onclick={verifySelected}><SearchCheck size={15} aria-hidden="true" />{i18n.t('integrations.verify')}</Button>
      <Button variant="outline" disabled={app.loading || !batchActions.reconcile} onclick={reconcileSelected}><Wrench size={15} aria-hidden="true" />{i18n.t('integrations.reconcile')}</Button>
      <Button variant="destructive" disabled={app.loading || !batchActions.remove} onclick={removeSelected}><Trash2 size={15} aria-hidden="true" />{i18n.t('integrations.remove')}</Button>
      <Button disabled={app.loading || !batchActions.install} onclick={previewSelected}><PackagePlus size={15} aria-hidden="true" />{i18n.t('integrations.install')}</Button>
    </ActionGroup>
  </Card.Root>
  </TabsContent>

  <TabsContent value="skills" class="workspace-panel">
    <WorkflowContentPanel />
  </TabsContent>

  </TabsRoot>
{/if}
</PageLayout>

<style>
  :global(.integration-workspace) { margin-top: 10px; }
  :global(.workspace-tabs) { display: flex; width: 100%; flex-wrap: wrap; gap: 4px; border: 1px solid var(--color-border); padding: 4px; background: var(--color-surface-subtle); }
  :global(.workspace-tabs [data-slot='tabs-trigger']) { min-width: 118px; min-height: 42px; flex: 1 1 118px; gap: 7px; border-color: transparent !important; color: var(--color-muted); background: transparent; box-shadow: none; font-size: var(--font-size-label); font-weight: 720; }
  :global(.workspace-tabs [data-slot='tabs-trigger']:hover) { color: var(--color-ink-strong); background: var(--color-surface); }
  :global(.workspace-tabs [data-slot='tabs-trigger'][data-state='active']) { border-color: var(--color-border-strong) !important; color: var(--color-accent-strong) !important; background: var(--color-surface) !important; box-shadow: 0 1px 2px color-mix(in srgb, var(--color-ink-strong) 8%, transparent); }
  :global(.workspace-panel) { margin-top: 8px; }
  :global(.full-mcp-self-test) { display: grid; gap: 9px; margin-bottom: 8px; padding: 11px; }
  :global(.full-mcp-self-test > header) { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  :global(.full-mcp-self-test h2), :global(.full-mcp-self-test p) { margin: 0; }
  :global(.full-mcp-self-test > p) { color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.45; }
  :global(.full-mcp-facts) { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .full-mcp-checks { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); border: 1px solid var(--color-border); }
  .full-mcp-checks > div { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 5px 8px; padding: 7px 8px; border: 0 solid var(--color-border); border-width: 0 1px 1px 0; }
  .full-mcp-checks span { font-size: var(--font-size-label); font-weight: 650; }
  .full-mcp-checks code { grid-column: 1 / -1; overflow-wrap: anywhere; color: var(--color-muted); font-size: var(--font-size-micro); }
  .full-mcp-boundary { color: var(--color-warning-strong) !important; font-weight: 650; }
  :global(.full-mcp-actions) { justify-content: flex-end; }
  :global(.agent-workspace-panel) { overflow: visible; }
  .state-details, .migration-details { margin-top: 3px; }
  .state-details summary, .migration-details summary { width: fit-content; cursor: pointer; color: inherit; font-size: var(--font-size-micro); font-weight: 620; }
  .state-details p { margin: 5px 0 0; color: inherit; font-size: var(--font-size-label); line-height: 1.35; }
  .state-details code { color: inherit; font-size: var(--font-size-label); }
  .migration-details .project-note { margin-top: 5px; opacity: .82; }
  .migration-authority-note { margin: 6px 0 0; color: var(--color-warning-strong); font-size: var(--font-size-label); font-weight: 650; }
  .migration-details small { display: block; margin-top: 3px; color: inherit; font-family: var(--font-mono); font-size: var(--font-size-micro); opacity: .75; }
  .provider-conflicts { display: grid; gap: 6px; margin-top: 9px; border-top: 1px solid var(--color-border); padding-top: 8px; }
  .provider-conflicts > p { margin: 0; }
  .provider-conflicts label { display: grid; grid-template-columns: minmax(0, 1fr) minmax(150px, auto); align-items: center; gap: 10px; }
  .provider-conflicts b { display: block; font-size: 10px; }
  .provider-conflicts :global([data-slot='native-select-wrapper']) { width: 100%; }
  :global(.zotero) { overflow: hidden; }
  .zotero-facts { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); border-block: 1px solid var(--color-border); padding: 0 11px; background: var(--color-surface-subtle); }
  .zotero-facts > div { min-width: 0; border-right: 1px solid var(--color-border); padding: 10px 9px; }
  .zotero-facts > div:last-child { border-right: 0; }
  .zotero-facts span, .zotero-facts strong { display: block; }
  .zotero-facts span { margin-bottom: 3px; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 620; }
  .zotero-facts strong { overflow-wrap: anywhere; color: var(--color-ink-strong); font-size: var(--font-size-label); }
  .zotero-technical { border-bottom: 1px solid var(--color-border); padding: 0 10px; color: var(--color-muted); }
  .zotero-technical summary { width: fit-content; min-height: 36px; padding-block: 8px; cursor: pointer; font-size: var(--font-size-micro); font-weight: 620; }
  :global(.zotero-technical-facts) { margin-bottom: 10px; }
  .zotero-technical dt { font-size: var(--font-size-micro); font-weight: 620; }
  .zotero-technical dd { margin: 3px 0 0; }
  .zotero-technical code { display: block; overflow-wrap: anywhere; color: var(--color-ink); font-size: var(--font-size-micro); }
  .zotero-boundary { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 7px; padding: 7px 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .zotero-boundary-title { display: flex; min-width: 0; align-items: center; gap: var(--space-1); }
  .zotero-boundary strong { font-size: 10px; }
  .zotero-footer { display: flex; align-items: center; justify-content: space-between; gap: 9px; border-top: 1px solid var(--color-border); padding: 7px 10px; }
  .zotero-footer > p { max-width: 480px; margin: 0; color: var(--color-muted); font-size: var(--font-size-micro); line-height: 1.4; }
  :global(.zotero-actions) { justify-content: flex-end; }
  :global(.zotero-actions) :global([data-slot='button']) { min-height: 40px; font-size: var(--font-size-label); }
  :global(.integration-tabs) { display: grid; width: 100%; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px; margin-bottom: 8px; }
  :global(.integration-tabs [data-slot='tabs-trigger']) { border: 1px solid var(--color-border) !important; color: var(--color-muted); background: var(--color-surface-subtle); box-shadow: none; }
  :global(.integration-tabs [data-slot='tabs-trigger']:hover) { color: var(--color-ink-strong); background: var(--color-surface); }
  :global(.integration-tabs [data-slot='tabs-trigger'][data-state='active']) { border-color: var(--color-border-strong) !important; color: var(--color-ink-strong) !important; background: var(--color-surface) !important; box-shadow: 0 1px 2px color-mix(in srgb, var(--color-ink-strong) 8%, transparent); }
  .integration-tab-copy { min-width: 0; }
  .integration-tab-copy strong, .integration-tab-copy small { display: block; }
  .integration-tab-copy strong { color: inherit; font-size: 12px; }
  .integration-tab-copy small { margin-top: 2px; font-size: var(--font-size-label); }
  :global(.integration-client-panel) { overflow: hidden; }
  .client-header { display: flex; align-items: center; justify-content: space-between; gap: 9px; padding: 8px 10px; }
  .client-title { display: flex; min-width: 220px; align-items: center; gap: 9px; }
  .client-title-row, .package-title-row, .component-title { display: flex; min-width: 0; align-items: center; gap: var(--space-1); }
  .client-mark { display: grid; width: 34px; height: 34px; flex: none; place-items: center; border-radius: var(--radius-control-inner); color: var(--color-accent-strong); background: var(--color-accent-soft); }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 16px; }
  .headline-facts { display: flex; align-items: center; }
  .headline-facts > div { min-width: 126px; border-left: 1px solid var(--color-border); padding: 2px 10px; }
  .headline-facts span, .headline-facts strong, .headline-facts small { display: block; }
  .headline-facts > div > span { margin-bottom: 3px; color: var(--color-muted); font-size: var(--font-size-label); font-weight: 620; }
  .headline-facts strong { color: var(--color-ink-strong); font-size: 11px; }
  .headline-facts small { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-micro); }
  .headline-facts .content-provenance { color: var(--color-success); font-weight: 750; }
  .host-package { border-block: 1px solid var(--color-border); background: var(--color-surface-subtle); }
  .host-package > header { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 7px; padding: 7px 10px; background: var(--color-surface); }
  .package-mark { display: grid; width: 32px; height: 32px; place-items: center; border-radius: var(--radius-control-inner); color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .host-package h3 { margin: 0; color: var(--color-ink-strong); font-size: 13px; }
  .package-components { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); border-top: 1px solid var(--color-border); }
  .package-components article { display: flex; min-width: 0; min-height: 72px; align-items: flex-start; justify-content: space-between; gap: 8px; padding: 9px; border-right: 1px solid var(--color-border); }
  .package-components article:last-child { border-right: 0; }
  .package-components article > div { min-width: 0; }
  .package-components strong, .package-components small { display: block; }
  .package-components strong { color: var(--color-ink-strong); font-size: 11px; line-height: 1.35; }
  .component-statuses { display: grid; flex: 0 1 auto; justify-items: end; gap: 4px; }
  .component-statuses > small { color: var(--color-muted); font-size: 10px; text-align: right; }
  .meta-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 7px; padding: 7px 10px; }
  .meta-grid strong, .meta-grid span { display: block; }
  .meta-grid strong { margin-bottom: 3px; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 620; letter-spacing: .02em; }
  .meta-grid span { overflow-wrap: anywhere; color: var(--color-ink); font-size: var(--font-size-label); }
  .meta-grid .attention span { color: var(--color-warning); font-weight: 750; }
  .activation-guide { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 10px; border-top: 1px solid var(--color-border); padding: 7px 10px; color: var(--color-muted); background: var(--color-surface-subtle); }
  .activation-guide[data-stage='activate'] { color: var(--color-warning-strong); background: var(--color-warning-soft); }
  .activation-guide[data-stage='repair'], .activation-guide[data-stage='blocked'] { color: var(--color-warning-strong); background: var(--color-warning-soft); }
  .activation-guide[data-stage='ready'] { color: var(--color-success); background: var(--color-success-soft); }
  .activation-guide-title { display: flex; min-width: 0; align-items: center; gap: var(--space-1); }
  .activation-guide-title strong { color: inherit; font-size: var(--font-size-label); }
  .activation-guide code { min-width: 0; overflow-wrap: anywhere; border-radius: var(--radius-control-inner); padding: 3px 5px; color: inherit; background: var(--color-surface); font-size: var(--font-size-micro); }
  .panel-footer { display: flex; align-items: center; justify-content: space-between; gap: 8px; border-top: 1px solid var(--color-border); padding: 6px 10px; }
  .include { display: flex; min-height: 44px; align-items: center; gap: 7px; color: var(--color-ink); font-size: 10px; font-weight: 700; }
  :global(.paths-toggle) { min-height: 44px; padding-inline: 4px; color: var(--color-accent-strong); font-size: 10px; font-weight: 700; }
  :global(.rotated) { transform: rotate(180deg); }
  .paths { border-top: 1px solid var(--color-border); padding: 0 10px 7px; }
  .paths p { color: var(--color-muted); font-size: 10px; }
  .paths > div { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 9px; border-bottom: 1px solid var(--color-border); padding: 7px 0; }
  .paths code, .paths span { overflow-wrap: anywhere; color: var(--color-muted); font-size: var(--font-size-label); }
  .paths .evidence { display: block; padding-top: 7px; color: var(--color-muted); font-size: var(--font-size-micro); }
  :global(.action-bar) { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-top: 8px; padding: 9px 10px; border-color: var(--color-border-strong); }
  .selection strong, .selection span { display: block; }
  .selection strong { color: var(--color-ink-strong); font-size: 11px; }
  .selection span { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-micro); }
  :global(.actions) { justify-content: flex-end; }
  :global(.actions) :global([data-slot='button']) { min-height: 44px; font-size: 10px; }
  .execution-surfaces { margin-top: 10px; }
  .surface-heading { display: flex; min-height: 44px; align-items: center; justify-content: space-between; gap: 12px; border-block: 1px solid var(--color-border); padding: 6px 2px; cursor: pointer; }
  .surface-heading h2 { margin-top: 0; }
  .surface-heading > span { color: var(--color-accent-strong); font-size: var(--font-size-label); font-weight: 750; }
  .surface-description { max-width: 720px; margin: 9px 0; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.45; }
  :global(.surface-card) { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: start; gap: 9px; padding: var(--ui-panel-padding); }
  :global(.surface-card) h3 { margin: 0 0 4px; color: var(--color-ink-strong); font-size: 12px; }
  :global(.surface-card) p { margin: 0; color: var(--color-muted); font-size: 10px; line-height: 1.45; }
  .surface-note { margin: 8px 0 0; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.45; }
  @media (max-width: 1100px) { .package-components { grid-template-columns: repeat(3, minmax(0, 1fr)); } .package-components article { border-bottom: 1px solid var(--color-border); } .package-components article:nth-child(3n) { border-right: 0; } .package-components article:nth-last-child(-n + 2) { border-bottom: 0; } }
  @media (max-width: 1000px) { .zotero-facts { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
  @media (max-width: 840px) { .client-header { align-items: flex-start; flex-direction: column; } .headline-facts { width: 100%; } .headline-facts > div { flex: 1; } .meta-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .zotero-facts { grid-template-columns: repeat(2, minmax(0, 1fr)); } :global(.action-bar), .zotero-footer { align-items: flex-start; flex-direction: column; } :global(.actions), :global(.zotero-actions) { justify-content: flex-start; } }
  @media (max-width: 900px) { :global(.integration-tabs) { grid-template-columns: 1fr; } .package-components { grid-template-columns: repeat(2, minmax(0, 1fr)); } .package-components article, .package-components article:nth-child(3n) { border-right: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); } .package-components article:nth-child(2n) { border-right: 0; } .package-components article:last-child { border-right: 0; border-bottom: 0; } .activation-guide, .panel-footer { align-items: flex-start; flex-direction: column; } }
  @media (max-width: 460px) { .headline-facts, :global(.actions), :global(.zotero-actions) { align-items: stretch; flex-direction: column; } .headline-facts > div { border-left: 0; border-top: 1px solid var(--color-border); } .package-components, .meta-grid, .zotero-facts { grid-template-columns: 1fr; } .package-components article { min-height: 0; border-right: 0 !important; border-bottom: 1px solid var(--color-border) !important; } .package-components article:last-child { border-bottom: 0 !important; } :global(.actions) :global([data-slot='button']), :global(.zotero-actions) :global([data-slot='button']) { width: 100%; } }
</style>
