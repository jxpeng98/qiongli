<script lang="ts">
  import { ArrowRight, BookOpenText, Boxes, Cable, Database, RefreshCw, ShieldCheck, TerminalSquare } from '@lucide/svelte';

  import { connectionStatus } from '$lib/features/client-integrations';
  import { readyAreaCount } from '$lib/features/overview';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';

  const app = useAppState();

  let readyCount = $derived(app.snapshot ? readyAreaCount(app.snapshot) : 0);
</script>

<PageHeader
  eyebrow="System overview"
  title="A clear view of your research system"
  description="Runtime health, embedded workflow content, and client integrations are reported independently so a detected app is never confused with installed Qiongli content."
>
  {#snippet actions()}
    <button class="button-secondary" type="button" disabled={app.loading} onclick={() => app.refresh()}>
      <RefreshCw size={16} aria-hidden="true" />
      Refresh
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section class="surface loading" aria-busy="true">
    <div class="skeleton wide"></div>
    <div class="skeleton"></div>
    <p>{app.bridgeReady ? 'Loading the native application snapshot…' : 'Start this interface through the Qiongli desktop application.'}</p>
  </section>
{:else}
  <section class="summary surface">
    <div>
      <p class="eyebrow">Current application</p>
      <h2>Qiongli {app.snapshot.product.version}</h2>
      <p>{app.snapshot.product.operatingSystem} · {app.snapshot.product.architecture} · {app.snapshot.product.build}</p>
    </div>
    <div class="health">
      <strong>{readyCount}/5</strong>
      <span>core areas ready</span>
    </div>
    <div class="authority">
      <ShieldCheck size={19} aria-hidden="true" />
      <div>
        <strong>{app.snapshot.product.trust.label}</strong>
        <code>{app.snapshot.product.trust.reasonCode}</code>
      </div>
    </div>
  </section>

  <div class="status-grid">
    <article class="surface status-card project-card">
      <div class="card-icon"><BookOpenText size={20} aria-hidden="true" /></div>
      <div class="card-title">
        <h3>Research library</h3>
        <StatusBadge
          status={app.snapshot.researchLibrary.health === 'ready' || app.snapshot.researchLibrary.health === 'empty' ? 'ready' : 'attention'}
          label={app.snapshot.researchLibrary.health === 'empty' ? 'Empty' : undefined}
        />
      </div>
      <p>{app.snapshot.researchLibrary.projects.length} article projects share one local, private index.</p>
      <a href="/research-library">Open project library <ArrowRight size={15} aria-hidden="true" /></a>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><Boxes size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>Embedded content</h3><StatusBadge status={app.snapshot.content.status} /></div>
      <p>{app.snapshot.content.entryCount} verified entries in {app.snapshot.content.packId}.</p>
      <a href="/workflow-content">Review workflow profiles <ArrowRight size={15} aria-hidden="true" /></a>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><Database size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>Global configuration</h3><StatusBadge status={app.snapshot.configuration.status} /></div>
      <p>{app.snapshot.configuration.revision === null ? 'No readable revision.' : `Revision ${app.snapshot.configuration.revision} is loaded.`}</p>
      <span class="meta">Rust-owned state</span>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><TerminalSquare size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>Lite MCP</h3><StatusBadge status={app.snapshot.mcp.status} /></div>
      <p>{app.snapshot.mcp.publicToolCount} public tools are embedded in the application.</p>
      <span class="meta">No Python runtime required</span>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><ShieldCheck size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>Client integration changes</h3><StatusBadge status={app.snapshot.capabilities.apply ? 'ready' : 'write-unsupported'} label={app.snapshot.capabilities.apply ? 'Available' : 'Inspect only'} /></div>
      <p>{app.snapshot.capabilities.apply ? 'This build can preview and confirm managed client/plugin changes.' : 'Client/plugin changes require packaged-product authority; local Research Library transactions remain independent.'}</p>
      <span class="meta">Project authority is reported separately</span>
    </article>
  </div>

  <section class="clients surface">
    <div class="section-heading">
      <div>
        <p class="eyebrow">Client boundary</p>
        <h2>Detected clients and Qiongli content</h2>
      </div>
      <a class="button-quiet" href="/client-integrations"><Cable size={16} aria-hidden="true" />Manage integrations</a>
    </div>
    <div class="client-list">
      {#each app.snapshot.integrations as integration}
        <article>
          <div>
            <h3>{integration.label}</h3>
            <p>{integration.client.detected ? `Client ${integration.client.version ?? 'version unknown'} detected` : 'Client executable not detected'}</p>
          </div>
          <div class="split-status">
            <span>Client <StatusBadge status={integration.client.status} /></span>
            <span>Qiongli <StatusBadge status={connectionStatus(integration.connection.state)} label={integration.connection.label} /></span>
          </div>
        </article>
      {/each}
    </div>
  </section>
{/if}

<style>
  .loading {
    min-height: 220px;
    padding: 30px;
  }

  .loading p { color: var(--color-muted); }
  .skeleton { width: 42%; height: 18px; margin-bottom: 14px; border-radius: 6px; background: #e2e8f0; }
  .skeleton.wide { width: 68%; height: 30px; }

  .summary {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 26px;
    margin-bottom: 18px;
    padding: 22px 24px;
    border-top: 3px solid var(--color-accent);
  }

  .summary h2 { margin: 0; color: var(--color-ink-strong); font-size: 23px; letter-spacing: -0.025em; }
  .summary p:not(.eyebrow) { margin: 6px 0 0; color: var(--color-muted); font-size: 13px; }
  .health { border-left: 1px solid var(--color-border); padding-left: 26px; text-align: center; }
  .health strong, .health span { display: block; }
  .health strong { color: var(--color-ink-strong); font-size: 25px; }
  .health span { margin-top: 2px; color: var(--color-muted); font-size: 11px; font-weight: 650; }
  .authority { display: grid; grid-template-columns: auto 1fr; gap: 10px; border-radius: 11px; padding: 13px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .authority { grid-column: 1 / -1; }
  .authority strong, .authority code { display: block; }
  .authority strong { font-size: 12px; }
  .authority code { margin-top: 4px; color: var(--color-muted); font-size: 10px; overflow-wrap: anywhere; }

  .status-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
  .project-card { grid-column: 1 / -1; }
  .status-card { position: relative; min-height: 180px; padding: 20px 20px 18px; }
  .card-icon { display: grid; width: 36px; height: 36px; place-items: center; margin-bottom: 14px; border-radius: 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .card-title { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
  h3 { margin: 0; color: var(--color-ink-strong); font-size: 15px; }
  .status-card p { margin: 10px 0 16px; color: var(--color-muted); font-size: 13px; line-height: 1.55; }
  .status-card a, .meta { display: inline-flex; align-items: center; gap: 6px; color: var(--color-accent-strong); font-size: 12px; font-weight: 720; text-decoration: none; }
  .meta { color: var(--color-muted); font-weight: 600; }

  .clients { margin-top: 18px; padding: 22px 24px; }
  .section-heading { display: flex; align-items: center; justify-content: space-between; gap: 20px; }
  .section-heading h2 { margin: 0; color: var(--color-ink-strong); font-size: 19px; }
  .section-heading a { text-decoration: none; }
  .client-list { margin-top: 16px; border-top: 1px solid var(--color-border); }
  .client-list article { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 16px 2px; border-bottom: 1px solid var(--color-border); }
  .client-list article:last-child { border-bottom: 0; padding-bottom: 0; }
  .client-list p { margin: 5px 0 0; color: var(--color-muted); font-size: 12px; }
  .split-status { display: flex; align-items: center; gap: 18px; }
  .split-status > span { display: flex; align-items: center; gap: 8px; color: var(--color-muted); font-size: 11px; font-weight: 650; }

  @media (max-width: 700px) {
    .summary { grid-template-columns: minmax(0, 1fr) auto; gap: 16px; padding: 18px; }
    .status-grid { grid-template-columns: 1fr; }
    .section-heading, .client-list article { align-items: flex-start; flex-direction: column; }
    .split-status { flex-wrap: wrap; }
  }

  @media (max-width: 440px) {
    .summary { grid-template-columns: 1fr; }
    .health { border-left: 0; border-top: 1px solid var(--color-border); padding: 12px 0 0; text-align: left; }
  }
</style>
