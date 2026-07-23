<script lang="ts">
  import { CheckCircle2, ChevronDown, CircleDot, Cloud, Laptop, PackageOpen, PackagePlus, RefreshCw, SearchCheck, ShieldAlert, Trash2, Wrench } from '@lucide/svelte';

  import type { IntegrationSelection, IntegrationTarget } from '@qiongli/app-api';
  import { connectionStatus, integrationEligible } from '$lib/features/client-integrations';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let selected = $state<IntegrationSelection>({ codex: true, claudeCode: true });
  let activeTarget = $state<IntegrationTarget>('codex');
  let expanded = $state(false);
  let initializedSelection = false;

  let activeIntegration = $derived(
    app.snapshot?.integrations.find((integration) => integration.target === activeTarget) ?? null
  );
  let codexIntegration = $derived(
    app.snapshot?.integrations.find((integration) => integration.target === 'codex') ?? null
  );
  let claudeIntegration = $derived(
    app.snapshot?.integrations.find((integration) => integration.target === 'claude-code') ?? null
  );

  $effect(() => {
    if (app.snapshot && !initializedSelection) {
      selected = {
        codex: integrationEligible(app.snapshot.integrations[0]),
        claudeCode: integrationEligible(app.snapshot.integrations[1])
      };
      if (!integrationEligible(app.snapshot.integrations[0]) && integrationEligible(app.snapshot.integrations[1])) {
        activeTarget = 'claude-code';
      }
      initializedSelection = true;
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
    if (!['ArrowLeft', 'ArrowRight'].includes(event.key)) return;
    event.preventDefault();
    activate(activeTarget === 'codex' ? 'claude-code' : 'codex');
    window.setTimeout(() => document.getElementById(`tab-${activeTarget}`)?.focus());
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

  async function updateSelected(): Promise<void> {
    await app.execute({ action: 'preview-update-integrations', selection: selected });
  }

  async function removeSelected(): Promise<void> {
    await app.execute({ action: 'preview-remove-integrations', selection: selected });
  }

  async function repairAll(): Promise<void> {
    await app.execute({ action: 'preview-repair-all' });
  }
</script>

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
  <section class="surface empty" aria-busy="true">{i18n.t('integrations.loading')}</section>
{:else}
  <section class="authority surface" class:installable={app.snapshot.capabilities.apply}>
    {#if app.snapshot.capabilities.apply}<CheckCircle2 size={18} aria-hidden="true" />{:else}<ShieldAlert size={18} aria-hidden="true" />{/if}
    <div><strong>{i18n.dynamic(app.snapshot.product.trust.label)}</strong><p>{i18n.t(app.snapshot.capabilities.apply ? 'integrations.applyNotice' : 'integrations.inspectNotice')}</p></div>
    <code>{app.snapshot.product.trust.reasonCode}</code>
  </section>

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
      <div><strong>{i18n.t('integrations.evidence')}</strong><code>{activeIntegration.evidenceCode}</code></div>
    </div>

    <div class="panel-footer">
      <label class="include"><input type="checkbox" checked={isSelected(activeIntegration.target)} disabled={!integrationEligible(activeIntegration)} onchange={(event) => setSelected(activeIntegration.target, event.currentTarget.checked)} />{i18n.t('integrations.include')}</label>
      <button class="paths-toggle" type="button" aria-expanded={expanded} onclick={() => expanded = !expanded}><ChevronDown size={15} class={expanded ? 'rotated' : undefined} aria-hidden="true" />{i18n.t('integrations.paths')} ({activeIntegration.paths.length})</button>
    </div>

    {#if expanded}
      <div class="paths">
        {#if activeIntegration.paths.length === 0}<p>{i18n.t('integrations.noPaths')}</p>{:else}
          {#each activeIntegration.paths as path}<div><code>{path.symbolicPath}</code><span>{path.surface} · {path.scope} · {path.management}</span><StatusBadge status={path.state} /></div>{/each}
        {/if}
      </div>
    {/if}
  </div>

  <section class="action-bar surface">
    <div class="selection"><strong>{[selected.codex && 'Codex', selected.claudeCode && 'Claude Code'].filter(Boolean).join(' + ') || i18n.label('none')}</strong><span>{i18n.t('integrations.install')}</span></div>
    <div class="actions">
      <button class="button-secondary" type="button" disabled={app.loading || (!selected.codex && !selected.claudeCode)} onclick={verifySelected}><SearchCheck size={15} aria-hidden="true" />{i18n.t('integrations.verify')}</button>
      <button class="button-secondary" type="button" disabled={app.loading || (!selected.codex && !selected.claudeCode)} onclick={updateSelected}><RefreshCw size={15} aria-hidden="true" />{i18n.t('integrations.update')}</button>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={repairAll}><Wrench size={15} aria-hidden="true" />{i18n.t('integrations.repair')}</button>
      <button class="button-danger" type="button" disabled={app.loading || (!selected.codex && !selected.claudeCode)} onclick={removeSelected}><Trash2 size={15} aria-hidden="true" />{i18n.t('integrations.remove')}</button>
      <button class="button-primary" type="button" disabled={app.loading || (!selected.codex && !selected.claudeCode)} onclick={previewSelected}><PackagePlus size={15} aria-hidden="true" />{i18n.t('integrations.install')}</button>
    </div>
  </section>

  <section class="execution-surfaces" aria-labelledby="execution-surfaces-title">
    <div class="surface-heading">
      <div>
        <p class="eyebrow">{i18n.t('integrations.surfacesEyebrow')}</p>
        <h2 id="execution-surfaces-title">{i18n.t('integrations.surfacesTitle')}</h2>
      </div>
      <p>{i18n.t('integrations.surfacesDescription')}</p>
    </div>

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
  </section>
{/if}

<style>
  .empty { padding: 20px; color: var(--color-muted); }
  .authority { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; margin-bottom: 10px; border-color: #fde68a; padding: 10px 12px; color: #854d0e; background: var(--color-warning-soft); }
  .authority.installable { border-color: #a7f3d0; color: #065f46; background: var(--color-success-soft); }
  .authority strong { font-size: 11px; }
  .authority p { margin: 2px 0 0; color: inherit; font-size: 10px; line-height: 1.35; }
  .authority code { color: inherit; font-size: 9px; }
  .tabs { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px; margin-bottom: 8px; }
  .tabs > button { display: grid; min-height: 48px; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 9px; border: 1px solid var(--color-border); border-radius: 10px; padding: 7px 10px; color: var(--color-muted); background: var(--color-surface-subtle); text-align: left; }
  .tabs > button[aria-selected='true'] { border-color: var(--color-accent); color: var(--color-accent-strong); background: white; box-shadow: 0 0 0 2px rgb(3 105 161 / .1); }
  .tabs strong, .tabs small { display: block; }
  .tabs strong { color: var(--color-ink-strong); font-size: 12px; }
  .tabs small { margin-top: 2px; font-size: 9px; }
  .client-panel { overflow: hidden; }
  .client-header { display: flex; align-items: center; justify-content: space-between; gap: 18px; padding: 12px 14px; }
  .client-title { display: flex; min-width: 220px; align-items: center; gap: 9px; }
  .client-mark { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 9px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 16px; }
  .client-title p { margin: 3px 0 0; color: var(--color-muted); font-size: 9px; }
  .headline-facts { display: flex; align-items: center; }
  .headline-facts > div { min-width: 126px; border-left: 1px solid var(--color-border); padding: 2px 12px; }
  .headline-facts span, .headline-facts strong, .headline-facts small { display: block; }
  .headline-facts > div > span { margin-bottom: 3px; color: var(--color-muted); font-size: 9px; font-weight: 750; }
  .headline-facts strong { color: var(--color-ink-strong); font-size: 11px; }
  .headline-facts small { margin-top: 2px; color: var(--color-muted); font-size: 8px; }
  .content-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-block: 1px solid var(--color-border); background: var(--color-surface-subtle); }
  .content-grid > div { display: flex; min-height: 46px; align-items: center; justify-content: space-between; gap: 8px; border-right: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); padding: 7px 10px; }
  .content-grid > div:nth-child(3n) { border-right: 0; }
  .content-grid > div:nth-last-child(-n + 3) { border-bottom: 0; }
  .content-grid > div > span:first-child { color: var(--color-muted); font-size: 10px; font-weight: 700; }
  .observed { display: flex; align-items: flex-end; flex-direction: column; gap: 2px; }
  .observed small { color: var(--color-muted); font-size: 8px; }
  .meta-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; padding: 10px 14px; }
  .meta-grid strong, .meta-grid span, .meta-grid code { display: block; }
  .meta-grid strong { margin-bottom: 3px; color: var(--color-muted); font-size: 8px; letter-spacing: .04em; text-transform: uppercase; }
  .meta-grid span, .meta-grid code { overflow-wrap: anywhere; color: var(--color-ink); font-size: 9px; }
  .panel-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-top: 1px solid var(--color-border); padding: 8px 14px; }
  .include { display: flex; align-items: center; gap: 7px; color: var(--color-ink); font-size: 10px; font-weight: 700; }
  .include input { width: 16px; height: 16px; accent-color: var(--color-accent); }
  .paths-toggle { display: flex; align-items: center; gap: 6px; border: 0; padding: 4px; color: var(--color-accent-strong); background: transparent; font-size: 10px; font-weight: 700; }
  :global(.rotated) { transform: rotate(180deg); }
  .paths { border-top: 1px solid var(--color-border); padding: 0 14px 8px; }
  .paths p { color: var(--color-muted); font-size: 10px; }
  .paths > div { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 9px; border-bottom: 1px solid var(--color-border); padding: 7px 0; }
  .paths code, .paths span { overflow-wrap: anywhere; color: var(--color-muted); font-size: 9px; }
  .action-bar { display: flex; align-items: center; justify-content: space-between; gap: 14px; margin-top: 9px; padding: 10px 12px; border-color: var(--color-border-strong); }
  .selection strong, .selection span { display: block; }
  .selection strong { color: var(--color-ink-strong); font-size: 11px; }
  .selection span { margin-top: 2px; color: var(--color-muted); font-size: 8px; }
  .actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 6px; }
  .actions button { min-height: 34px; font-size: 10px; }
  .execution-surfaces { margin-top: 22px; }
  .surface-heading { display: flex; align-items: end; justify-content: space-between; gap: 20px; margin-bottom: 10px; }
  .surface-heading h2 { margin-top: 0; }
  .surface-heading > p { max-width: 620px; margin: 0; color: var(--color-muted); font-size: 11px; line-height: 1.5; }
  .surface-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
  .surface-card { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: start; gap: 10px; padding: 13px; }
  .surface-card h3 { margin: 0 0 4px; color: var(--color-ink-strong); font-size: 12px; }
  .surface-card p { margin: 0; color: var(--color-muted); font-size: 10px; line-height: 1.45; }
  .surface-note { margin: 8px 0 0; color: var(--color-muted); font-size: 9px; line-height: 1.45; }
  @media (max-width: 840px) { .client-header { align-items: flex-start; flex-direction: column; } .headline-facts { width: 100%; } .headline-facts > div { flex: 1; } .meta-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .action-bar { align-items: flex-start; flex-direction: column; } .actions { justify-content: flex-start; } }
  @media (max-width: 700px) { .authority { grid-template-columns: auto 1fr; } .authority code { grid-column: 2; } .tabs, .surface-grid { grid-template-columns: 1fr; } .surface-heading { align-items: flex-start; flex-direction: column; gap: 6px; } .content-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .content-grid > div, .content-grid > div:nth-child(3n) { border-right: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); } .content-grid > div:nth-child(2n) { border-right: 0; } .content-grid > div:nth-last-child(-n + 2) { border-bottom: 0; } .panel-footer { align-items: flex-start; flex-direction: column; } }
  @media (max-width: 460px) { .headline-facts, .actions { align-items: stretch; flex-direction: column; } .headline-facts > div { border-left: 0; border-top: 1px solid var(--color-border); } .content-grid, .meta-grid { grid-template-columns: 1fr; } .content-grid > div { border-right: 0 !important; border-bottom: 1px solid var(--color-border) !important; } .content-grid > div:last-child { border-bottom: 0 !important; } .actions button { width: 100%; } }
</style>
