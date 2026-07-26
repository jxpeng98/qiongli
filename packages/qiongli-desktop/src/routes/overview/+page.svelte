<script lang="ts">
  import { ArrowRight, BookOpenText, Boxes, Cable, Database, RefreshCw, ShieldCheck, TerminalSquare } from '@lucide/svelte';

  import { connectionStatus } from '$lib/features/client-integrations';
  import { readyAreaCount } from '$lib/features/overview';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();

  let readyCount = $derived(app.snapshot ? readyAreaCount(app.snapshot) : 0);
</script>

<svelte:head>
  <title>{i18n.t('overview.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('overview.eyebrow')}
  title={i18n.t('overview.title')}
  description={i18n.t('overview.description')}
>
  {#snippet actions()}
    <button class="button-secondary" type="button" disabled={app.loading} onclick={() => app.refresh()}>
      <RefreshCw size={16} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section
    class="surface loading"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >
    <div class="skeleton wide"></div>
    <div class="skeleton"></div>
    <p>{app.bridgeReady ? i18n.t('overview.loading') : i18n.t('overview.startDesktop')}</p>
  </section>
{:else}
  <section class="summary surface">
    <div>
      <p class="eyebrow">{i18n.t('overview.currentApp')}</p>
      <h2>Qiongli {app.snapshot.product.version}</h2>
      <p>{app.snapshot.product.operatingSystem} · {app.snapshot.product.architecture} · {app.snapshot.product.build}</p>
    </div>
    <div class="health">
      <strong>{readyCount}/5</strong>
      <span>{i18n.t('overview.readyAreas')}</span>
    </div>
    <div class="authority">
      <ShieldCheck size={19} aria-hidden="true" />
      <div>
        <strong>{i18n.dynamic(app.snapshot.product.trust.label)}</strong>
        <code>{app.snapshot.product.trust.reasonCode}</code>
      </div>
    </div>
  </section>

  <div class="status-grid">
    <article class="surface status-card project-card">
      <div class="card-icon"><BookOpenText size={20} aria-hidden="true" /></div>
      <div class="card-title">
        <h3>{i18n.t('overview.library')}</h3>
        <StatusBadge
          status={app.snapshot.researchLibrary.health === 'ready' || app.snapshot.researchLibrary.health === 'empty' ? 'ready' : 'attention'}
          label={app.snapshot.researchLibrary.health === 'empty' ? 'Empty' : undefined}
        />
      </div>
      <p>{i18n.t('overview.projectCount', { count: app.snapshot.researchLibrary.projects.length })}</p>
      <a href="/research-library">{i18n.t('overview.openLibrary')} <ArrowRight size={15} aria-hidden="true" /></a>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><Boxes size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>{i18n.t('overview.embedded')}</h3><StatusBadge status={app.snapshot.content.status} /></div>
      <p>{i18n.t('overview.entryCount', { count: app.snapshot.content.entryCount, pack: app.snapshot.content.packId })}</p>
      <a href="/workflow-content">{i18n.t('overview.reviewProfiles')} <ArrowRight size={15} aria-hidden="true" /></a>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><Database size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>{i18n.t('overview.config')}</h3><StatusBadge status={app.snapshot.configuration.status} /></div>
      <p>{app.snapshot.configuration.revision === null ? i18n.t('overview.noRevision') : i18n.t('overview.revisionLoaded', { revision: app.snapshot.configuration.revision })}</p>
      <span class="meta">{i18n.t('overview.rustOwned')}</span>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><TerminalSquare size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>{i18n.t('overview.mcp')}</h3><StatusBadge status={app.snapshot.mcp.status} /></div>
      <p>{i18n.t('overview.toolCount', { count: app.snapshot.mcp.publicToolCount })}</p>
      <span class="meta">{i18n.t('overview.noPython')}</span>
    </article>

    <article class="surface status-card">
      <div class="card-icon"><ShieldCheck size={20} aria-hidden="true" /></div>
      <div class="card-title"><h3>{i18n.t('overview.changes')}</h3><StatusBadge status={app.snapshot.capabilities.apply ? 'ready' : 'write-unsupported'} label={app.snapshot.capabilities.apply ? i18n.t('overview.available') : i18n.t('overview.inspectOnly')} /></div>
      <p>{app.snapshot.capabilities.apply ? i18n.t('overview.canApply') : i18n.t('overview.cannotApply')}</p>
      <span class="meta">{i18n.t('overview.projectAuthority')}</span>
    </article>
  </div>

  <section class="clients surface">
    <div class="section-heading">
      <div>
        <p class="eyebrow">{i18n.t('overview.clientBoundary')}</p>
        <h2>{i18n.t('overview.detectedClients')}</h2>
      </div>
      <a class="button-quiet" href="/client-integrations"><Cable size={16} aria-hidden="true" />{i18n.t('overview.manage')}</a>
    </div>
    <div class="client-list">
      {#each app.snapshot.integrations as integration}
        <article>
          <div>
            <h3>{integration.label}</h3>
            <p>{integration.client.detected ? i18n.t('overview.clientDetected', { version: integration.client.version ?? i18n.label('unknown') }) : i18n.t('overview.clientMissing')}</p>
          </div>
          <div class="split-status">
            <span>{i18n.t('overview.client')} <StatusBadge status={integration.client.status} /></span>
            <span>{i18n.t('overview.qiongli')} <StatusBadge status={connectionStatus(integration.connection.state)} label={i18n.label(integration.connection.state)} /></span>
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
    gap: 18px;
    margin-bottom: 10px;
    padding: 13px 15px;
    border-left: 3px solid var(--color-accent);
  }

  .summary h2 { margin: 0; color: var(--color-ink-strong); font-size: 23px; letter-spacing: -0.025em; }
  .summary p:not(.eyebrow) { margin: 6px 0 0; color: var(--color-muted); font-size: 13px; }
  .health { border-left: 1px solid var(--color-border); padding-left: 26px; text-align: center; }
  .health strong, .health span { display: block; }
  .health strong { color: var(--color-ink-strong); font-size: 25px; }
  .health span { margin-top: 2px; color: var(--color-muted); font-size: 11px; font-weight: 650; }
  .authority { display: grid; grid-template-columns: auto 1fr; gap: 8px; border-radius: 9px; padding: 8px 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .authority { grid-column: 1 / -1; }
  .authority strong, .authority code { display: block; }
  .authority strong { font-size: 12px; }
  .authority code { margin-top: 4px; color: var(--color-muted); font-size: 10px; overflow-wrap: anywhere; }

  .status-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
  .project-card { grid-column: auto; }
  .status-card { position: relative; min-height: 128px; padding: 12px; }
  .card-icon { display: grid; width: 30px; height: 30px; place-items: center; margin-bottom: 8px; border-radius: 8px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .card-title { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
  h3 { margin: 0; color: var(--color-ink-strong); font-size: 15px; }
  .status-card p { margin: 7px 0 10px; color: var(--color-muted); font-size: 10px; line-height: 1.4; }
  .status-card a, .meta { display: inline-flex; align-items: center; gap: 6px; color: var(--color-accent-strong); font-size: 12px; font-weight: 720; text-decoration: none; }
  .meta { color: var(--color-muted); font-weight: 600; }

  .clients { margin-top: 9px; padding: 12px 14px; }
  .section-heading { display: flex; align-items: center; justify-content: space-between; gap: 20px; }
  .section-heading h2 { margin: 0; color: var(--color-ink-strong); font-size: 19px; }
  .section-heading a { text-decoration: none; }
  .client-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; margin-top: 9px; border-top: 1px solid var(--color-border); }
  .client-list article { display: flex; align-items: center; justify-content: space-between; gap: 14px; padding: 10px 2px 2px; }
  .client-list article:last-child { border-bottom: 0; padding-bottom: 0; }
  .client-list p { margin: 5px 0 0; color: var(--color-muted); font-size: 12px; }
  .split-status { display: flex; align-items: center; gap: 18px; }
  .split-status > span { display: flex; align-items: center; gap: 8px; color: var(--color-muted); font-size: 11px; font-weight: 650; }

  @media (max-width: 700px) {
    .summary { grid-template-columns: minmax(0, 1fr) auto; gap: 16px; padding: 18px; }
    .status-grid, .client-list { grid-template-columns: 1fr; }
    .section-heading, .client-list article { align-items: flex-start; flex-direction: column; }
    .split-status { flex-wrap: wrap; }
  }

  @media (max-width: 440px) {
    .summary { grid-template-columns: 1fr; }
    .health { border-left: 0; border-top: 1px solid var(--color-border); padding: 12px 0 0; text-align: left; }
  }
</style>
