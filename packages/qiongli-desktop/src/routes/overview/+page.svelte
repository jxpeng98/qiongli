<script lang="ts">
  import { ArrowRight, BookOpenText, Boxes, Cable, Database, RefreshCw, ShieldCheck, TerminalSquare } from '@lucide/svelte';

  import { connectionStatus } from '$lib/features/client-integrations';
  import { readyAreaCount } from '$lib/features/overview';
  import { ContentGrid, DescriptionTip, IconFrame, PageLayout, SectionHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();

  let readyCount = $derived(app.snapshot ? readyAreaCount(app.snapshot) : 0);
</script>

<svelte:head>
  <title>{i18n.t('overview.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageLayout
  eyebrow={i18n.t('overview.eyebrow')}
  title={i18n.t('overview.title')}
  description={i18n.t('overview.description')}
>
  {#snippet actions()}
    <Button variant="outline" disabled={app.loading} onclick={() => app.refresh()}>
      <RefreshCw size={16} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </Button>
  {/snippet}

{#if !app.snapshot}
  <StatePanel
    role="status"
    busy
    live="polite"
    atomic
    description={app.bridgeReady ? i18n.t('overview.loading') : i18n.t('overview.startDesktop')}
  >
    <div class="loading-skeletons">
      <Skeleton class="skeleton wide" />
      <Skeleton class="skeleton" />
    </div>
  </StatePanel>
{:else}
  <Card.Root class="summary">
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
  </Card.Root>

  <ContentGrid columns={3} collapse="sm" lastSpan={2} class="status-grid">
    <Card.Root class="status-card project-card">
      <IconFrame><BookOpenText size={18} /></IconFrame>
      <div class="card-title">
        <h3>{i18n.t('overview.library')}</h3>
        <DescriptionTip text={i18n.t('overview.projectCount', { count: app.snapshot.researchLibrary.projects.length })} />
        <StatusBadge
          status={app.snapshot.researchLibrary.health === 'ready' || app.snapshot.researchLibrary.health === 'empty' ? 'ready' : 'attention'}
          label={app.snapshot.researchLibrary.health === 'empty' ? 'Empty' : undefined}
        />
      </div>
      <a href="/research-library">{i18n.t('overview.openLibrary')} <ArrowRight size={15} aria-hidden="true" /></a>
    </Card.Root>

    <Card.Root class="status-card">
      <IconFrame><Boxes size={18} /></IconFrame>
      <div class="card-title">
        <h3>{i18n.t('overview.embedded')}</h3>
        <DescriptionTip text={i18n.t('overview.entryCount', { count: app.snapshot.content.entryCount, pack: app.snapshot.content.packId })} />
        <StatusBadge status={app.snapshot.content.status} />
      </div>
      <a href="/client-integrations#workflow-content">{i18n.t('overview.reviewProfiles')} <ArrowRight size={15} aria-hidden="true" /></a>
    </Card.Root>

    <Card.Root class="status-card status-card--metric">
      <IconFrame><Database size={18} /></IconFrame>
      <div class="card-title">
        <h3>{i18n.t('overview.config')}</h3>
        <DescriptionTip text={app.snapshot.configuration.revision === null ? i18n.t('overview.noRevision') : i18n.t('overview.revisionLoaded', { revision: app.snapshot.configuration.revision })} />
        <StatusBadge status={app.snapshot.configuration.status} />
      </div>
    </Card.Root>

    <Card.Root class="status-card status-card--metric">
      <IconFrame><TerminalSquare size={18} /></IconFrame>
      <div class="card-title">
        <h3>{i18n.t('overview.mcp')}</h3>
        <DescriptionTip text={i18n.t('overview.toolCount', { count: app.snapshot.mcp.publicToolCount })} />
        <StatusBadge status={app.snapshot.mcp.status} />
      </div>
    </Card.Root>

    <Card.Root class="status-card status-card--summary">
      <IconFrame><ShieldCheck size={18} /></IconFrame>
      <div class="card-title">
        <h3>{i18n.t('overview.changes')}</h3>
        <DescriptionTip text={app.snapshot.capabilities.apply ? i18n.t('overview.canApply') : i18n.t('overview.cannotApply')} side="top" align="end" />
        <StatusBadge status={app.snapshot.capabilities.apply ? 'ready' : 'write-unsupported'} label={app.snapshot.capabilities.apply ? i18n.t('overview.available') : i18n.t('overview.inspectOnly')} />
      </div>
    </Card.Root>
  </ContentGrid>

  <Card.Root class="clients">
    <SectionHeader eyebrow={i18n.t('overview.clientBoundary')} title={i18n.t('overview.detectedClients')}>
      {#snippet actions()}<Button variant="ghost" href="/client-integrations"><Cable size={16} aria-hidden="true" />{i18n.t('overview.manage')}</Button>{/snippet}
    </SectionHeader>
    <div class="client-list">
      {#each app.snapshot.integrations as integration}
        <article>
          <div class="client-identity">
            <h3>{integration.label}</h3>
            <DescriptionTip text={integration.client.detected ? i18n.t('overview.clientDetected', { version: integration.client.version ?? i18n.label('unknown') }) : i18n.t('overview.clientMissing')} side="left" align="end" />
          </div>
          <div class="split-status">
            <span>{i18n.t('overview.client')} <StatusBadge status={integration.client.status} /></span>
            <span>{i18n.t('overview.qiongli')} <StatusBadge status={connectionStatus(integration.connection.state)} label={i18n.label(integration.connection.state)} /></span>
          </div>
        </article>
      {/each}
    </div>
  </Card.Root>
{/if}
</PageLayout>

<style>
  .loading-skeletons { width: 100%; }
  :global(.skeleton) { width: 42%; height: 14px; margin-bottom: 8px; border-radius: var(--radius-control-inner); }
  :global(.skeleton.wide) { width: 68%; height: 24px; }

  :global(.summary) {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
    border-radius: var(--radius-card);
    padding: 12px 14px;
  }

  :global(.summary) h2 { margin: 0; color: var(--color-ink-strong); font-size: 21px; font-weight: 600; letter-spacing: -0.03em; }
  :global(.summary) p:not(.eyebrow) { margin: 4px 0 0; color: var(--color-muted); font-size: var(--font-size-supporting); }
  .health { border-left: 1px solid var(--color-border); padding-left: 14px; text-align: center; }
  .health strong, .health span { display: block; }
  .health strong { color: var(--color-ink-strong); font-size: 21px; font-weight: 600; }
  .health span { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-label); font-weight: 550; }
  .authority { display: grid; grid-template-columns: auto 1fr; gap: 7px; border-top: 1px solid var(--color-border); padding: 7px 0 0; color: var(--color-accent-strong); background: transparent; }
  .authority { grid-column: 1 / -1; }
  .authority strong, .authority code { display: block; }
  .authority strong { font-size: var(--font-size-supporting); font-weight: 620; }
  .authority code { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-micro); overflow-wrap: anywhere; }

  :global(.status-card) {
    position: relative;
    display: grid;
    min-height: 0;
    grid-template-columns: 28px minmax(0, 1fr);
    grid-template-areas:
      'icon title'
      'footer footer';
    align-content: start;
    gap: 6px 8px;
    padding: 10px;
  }
  :global(.status-card [data-slot='icon-frame']) { grid-area: icon; }
  :global(.status-card--metric) {
    grid-template-areas: 'icon title';
  }
  :global(.status-card--summary) {
    grid-template-areas: 'icon title';
    align-items: center;
  }
  .card-title { display: flex; min-width: 0; grid-area: title; align-items: center; align-self: center; justify-content: flex-start; gap: 6px 8px; flex-wrap: wrap; }
  h3 { margin: 0; color: var(--color-ink-strong); font-size: 14px; font-weight: 600; }
  :global(.status-card) a { display: inline-flex; width: fit-content; max-width: 100%; grid-area: footer; align-items: center; gap: 5px; color: var(--color-accent-strong); font-size: var(--font-size-label); font-weight: 550; text-decoration: none; }

  :global(.clients) { margin-top: 8px; padding: 10px; }
  .client-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; margin-top: 7px; border-top: 1px solid var(--color-border); }
  .client-list article { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 6px 1px 0; }
  .client-list article > div { min-width: 0; }
  .client-list article:last-child { border-bottom: 0; padding-bottom: 0; }
  .client-identity { display: flex; min-width: 0; align-items: center; gap: var(--space-1); }
  .split-status { display: flex; min-width: 0; flex-wrap: wrap; align-items: center; justify-content: flex-end; gap: 6px 10px; }
  .split-status > span { display: grid; min-width: 0; grid-template-columns: auto minmax(0, 1fr); align-items: center; gap: 6px; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 650; }
  :global(.split-status .status) { justify-self: start; }

  @media (max-width: 1200px) {
    .client-list { grid-template-columns: 1fr; }
  }

  @media (max-width: 700px) {
    :global(.summary) { grid-template-columns: minmax(0, 1fr) auto; gap: 10px; padding: 10px; }
    .client-list { grid-template-columns: 1fr; }
    .client-list article { align-items: flex-start; flex-direction: column; }
    .split-status { width: 100%; justify-content: flex-start; }
  }

  @media (max-width: 440px) {
    :global(.summary) { grid-template-columns: 1fr; }
    .health { border-left: 0; border-top: 1px solid var(--color-border); padding: 8px 0 0; text-align: left; }
  }
</style>
