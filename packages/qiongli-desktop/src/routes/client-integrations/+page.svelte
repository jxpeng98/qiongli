<script lang="ts">
  import { CheckCircle2, ChevronDown, CircleDot, PackagePlus, RefreshCw, SearchCheck, ShieldAlert } from '@lucide/svelte';

  import type { IntegrationSelection } from '@qiongli/app-api';
  import { connectionStatus, integrationEligible } from '$lib/features/client-integrations';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';

  const app = useAppState();
  let selected = $state<IntegrationSelection>({ codex: true, claudeCode: true });
  let expanded = $state<Record<string, boolean>>({});
  let initializedSelection = false;

  $effect(() => {
    if (app.snapshot && !initializedSelection) {
      selected = {
        codex: integrationEligible(app.snapshot.integrations[0]),
        claudeCode: integrationEligible(app.snapshot.integrations[1])
      };
      initializedSelection = true;
    }
  });

  async function rediscover(): Promise<void> {
    await app.execute({ action: 'refresh-integration-discovery' });
  }

  async function previewSelected(): Promise<void> {
    await app.execute({ action: 'preview-install-selected', selection: selected });
  }

  async function verifySelected(): Promise<void> {
    await app.execute({ action: 'verify-integrations', selection: selected });
  }

  function isSelected(target: 'codex' | 'claude-code'): boolean {
    return target === 'codex' ? selected.codex : selected.claudeCode;
  }

  function setSelected(target: 'codex' | 'claude-code', value: boolean): void {
    if (target === 'codex') selected.codex = value;
    else selected.claudeCode = value;
  }

</script>

<PageHeader
  eyebrow="Client integrations"
  title="Connect Qiongli without hiding what changes"
  description="Client detection answers whether Codex or Claude Code is installed. Managed-content status separately answers whether the Qiongli plugin, Skills, Lite MCP, registration, and activation are healthy."
>
  {#snippet actions()}
    <button class="button-secondary" type="button" disabled={app.loading} onclick={rediscover}>
      <RefreshCw size={16} aria-hidden="true" />
      Detect again
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section class="surface empty" aria-busy="true">Detecting supported clients and managed Qiongli content…</section>
{:else}
  <section class="authority surface" class:installable={app.snapshot.capabilities.apply}>
    {#if app.snapshot.capabilities.apply}
      <CheckCircle2 size={22} aria-hidden="true" />
    {:else}
      <ShieldAlert size={22} aria-hidden="true" />
    {/if}
    <div>
      <strong>{app.snapshot.product.trust.label}</strong>
      <p>{app.snapshot.capabilities.apply ? 'Previews can be confirmed after you review destinations and approvals.' : 'Detection and verification work, but installation confirmation is unavailable in this build.'}</p>
    </div>
    <code>{app.snapshot.product.trust.reasonCode}</code>
  </section>

  <div class="integration-list">
    {#each app.snapshot.integrations as integration}
      <article class="surface integration-card">
        <div class="client-row">
          <label class="select-client">
            <input
              type="checkbox"
              checked={isSelected(integration.target)}
              disabled={!integrationEligible(integration)}
              onchange={(event) => setSelected(integration.target, event.currentTarget.checked)}
              aria-label={`Select ${integration.label}`}
            />
          </label>
          <div class="client-mark"><CircleDot size={22} aria-hidden="true" /></div>
          <div class="client-title">
            <div><h2>{integration.label}</h2><StatusBadge status={connectionStatus(integration.connection.state)} label={integration.connection.label} /></div>
            <p>{integration.client.detected ? `Client ${integration.client.version ?? 'version unknown'} · compatibility ${integration.client.compatibility}` : integration.discovery}</p>
          </div>
          <div class="overall">
            <span>Qiongli plugin</span>
            <strong>{integration.plugin.installedVersion ?? 'Not installed'}</strong>
            <small>Available {integration.plugin.availableVersion}</small>
          </div>
        </div>

        <div class="content-grid">
          <div><span>Plugin source</span><StatusBadge status={integration.managedContent.source} /></div>
          <div><span>Skills</span><StatusBadge status={integration.managedContent.skills} /></div>
          <div><span>Marketplace</span><StatusBadge status={integration.managedContent.marketplace} /></div>
          <div><span>Registration</span><StatusBadge status={integration.managedContent.registration} /></div>
          <div><span>Activation</span><span class="observed"><StatusBadge status={integration.managedContent.activation} /><small>{integration.managedContent.activationObservation}</small></span></div>
          <div><span>Lite MCP attachment</span><span class="observed"><StatusBadge status={integration.managedContent.mcpAttachment} /><small>{integration.managedContent.mcpAttachmentObservation}</small></span></div>
        </div>

        {#if integration.legacyDetected}
          <p class="legacy-note">Existing Qiongli 1.x content was found and is preserved as unmanaged legacy evidence. Qiongli 2 uses the separate <code>qiongli-next</code> identity.</p>
        {/if}

        <div class="evidence">
          <div><strong>Ownership</strong><span>{integration.ownership}</span></div>
          <div><strong>Next action</strong><span>{integration.nextAction}</span></div>
          <div><strong>Evidence</strong><code>{integration.evidenceCode}</code></div>
        </div>

        <button class="paths-toggle" type="button" aria-expanded={expanded[integration.target] ?? false} onclick={() => expanded[integration.target] = !(expanded[integration.target] ?? false)}>
          <ChevronDown size={16} class={expanded[integration.target] ? 'rotated' : undefined} aria-hidden="true" />
          {integration.paths.length} detected integration paths
        </button>
        {#if expanded[integration.target]}
          <div class="paths">
            {#if integration.paths.length === 0}
              <p>No supported path evidence was discovered.</p>
            {:else}
              {#each integration.paths as path}
                <div>
                  <code>{path.symbolicPath}</code>
                  <span>{path.surface} · {path.scope} · {path.management}</span>
                  <StatusBadge status={path.state} />
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </article>
    {/each}
  </div>

  <section class="action-bar surface">
    <div>
      <p class="eyebrow">Selected clients</p>
      <strong>{[selected.codex && 'Codex', selected.claudeCode && 'Claude Code'].filter(Boolean).join(' + ') || 'None selected'}</strong>
      <span>“Install” means the Qiongli plugin bundle, never the Codex or Claude Code application.</span>
    </div>
    <div class="actions">
      <button class="button-secondary" type="button" disabled={app.loading || (!selected.codex && !selected.claudeCode)} onclick={verifySelected}>
        <SearchCheck size={16} aria-hidden="true" />Verify Qiongli content
      </button>
      <button class="button-primary" type="button" disabled={app.loading || (!selected.codex && !selected.claudeCode)} onclick={previewSelected}>
        <PackagePlus size={16} aria-hidden="true" />Preview Qiongli plugin install
      </button>
    </div>
  </section>
{/if}

<style>
  .empty { padding: 28px; color: var(--color-muted); }
  .authority { display: grid; grid-template-columns: auto 1fr auto; align-items: center; gap: 12px; margin-bottom: 16px; border-color: #fde68a; padding: 15px 17px; color: #854d0e; background: var(--color-warning-soft); }
  .authority.installable { border-color: #a7f3d0; color: #065f46; background: var(--color-success-soft); }
  .authority strong { font-size: 13px; }
  .authority p { margin: 3px 0 0; color: inherit; font-size: 12px; line-height: 1.45; }
  .authority code { color: inherit; font-size: 10px; }
  .integration-list { display: grid; gap: 14px; }
  .integration-card { overflow: hidden; }
  .client-row { display: grid; grid-template-columns: auto auto minmax(0, 1fr) auto; align-items: center; gap: 13px; padding: 20px 22px 17px; }
  .select-client input { width: 18px; height: 18px; accent-color: var(--color-accent); }
  .client-mark { display: grid; width: 42px; height: 42px; place-items: center; border-radius: 12px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .client-title > div { display: flex; align-items: center; gap: 10px; }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 18px; }
  .client-title p { margin: 5px 0 0; color: var(--color-muted); font-size: 11px; }
  .overall { display: flex; align-items: center; gap: 9px; }
  .overall { flex-direction: column; align-items: flex-end; gap: 2px; }
  .overall > span, .overall > small { color: var(--color-muted); font-size: 10px; font-weight: 680; }
  .overall > strong { color: var(--color-ink-strong); font-size: 12px; }
  .content-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); border-top: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); background: var(--color-surface-subtle); }
  .content-grid > div { display: flex; min-height: 54px; align-items: center; justify-content: space-between; gap: 10px; border-right: 1px solid var(--color-border); border-bottom: 1px solid var(--color-border); padding: 10px 14px; }
  .content-grid > div:nth-child(2n) { border-right: 0; }
  .content-grid > div:nth-last-child(-n + 2) { border-bottom: 0; }
  .content-grid span { color: var(--color-muted); font-size: 11px; font-weight: 650; }
  .content-grid .observed { display: flex; align-items: flex-end; flex-direction: column; gap: 3px; }
  .content-grid .observed small { color: var(--color-muted); font-size: 9px; font-weight: 650; }
  .legacy-note { margin: 0; border-bottom: 1px solid var(--color-border); padding: 11px 22px; color: #713f12; background: var(--color-warning-soft); font-size: 11px; line-height: 1.5; }
  .evidence { display: grid; grid-template-columns: 1fr 1fr 1.4fr; gap: 18px; padding: 15px 22px; }
  .evidence strong, .evidence span, .evidence code { display: block; }
  .evidence strong { margin-bottom: 4px; color: var(--color-muted); font-size: 10px; letter-spacing: 0.04em; text-transform: uppercase; }
  .evidence span, .evidence code { color: var(--color-ink); font-size: 11px; }
  .paths-toggle { display: flex; width: 100%; align-items: center; gap: 7px; border: 0; border-top: 1px solid var(--color-border); padding: 11px 22px; color: var(--color-accent-strong); background: white; font-size: 11px; font-weight: 720; text-align: left; }
  :global(.rotated) { transform: rotate(180deg); }
  .paths { padding: 0 22px 14px; }
  .paths p { color: var(--color-muted); font-size: 11px; }
  .paths > div { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 12px; border-top: 1px solid var(--color-border); padding: 10px 0; }
  .paths code { overflow-wrap: anywhere; color: var(--color-ink); font-size: 10px; }
  .paths span { color: var(--color-muted); font-size: 10px; }
  .action-bar { display: flex; align-items: center; justify-content: space-between; gap: 22px; margin-top: 18px; padding: 16px 18px; border-color: var(--color-border-strong); box-shadow: 0 16px 42px rgb(15 23 42 / 0.14); }
  .action-bar strong, .action-bar span { display: block; }
  .action-bar strong { color: var(--color-ink-strong); font-size: 14px; }
  .action-bar span { margin-top: 3px; color: var(--color-muted); font-size: 10px; }
  .actions { display: flex; gap: 10px; }

  @media (max-width: 700px) {
    .authority { grid-template-columns: auto 1fr; }
    .authority code { grid-column: 2; }
    .client-row { grid-template-columns: auto auto minmax(0, 1fr); padding-inline: 16px; }
    .overall { grid-column: 2 / -1; align-items: flex-start; }
    .evidence { grid-template-columns: 1fr; gap: 12px; padding-inline: 16px; }
    .action-bar { align-items: flex-start; flex-direction: column; }
    .actions { width: 100%; flex-wrap: wrap; }
    .paths-toggle, .legacy-note { padding-inline: 16px; }
  }

  @media (max-width: 440px) {
    .content-grid { grid-template-columns: 1fr; }
    .content-grid > div { border-right: 0; }
    .content-grid > div:nth-last-child(2) { border-bottom: 1px solid var(--color-border); }
    .actions > button { width: 100%; }
  }
</style>
