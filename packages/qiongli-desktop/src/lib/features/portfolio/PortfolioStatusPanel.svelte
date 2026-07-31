<script lang="ts">
  import type {
    PortfolioMaintenancePreview,
    PortfolioStatus
  } from '@qiongli/app-api';
  import {
    Activity,
    Database,
    RefreshCw,
    RotateCcw,
    ShieldCheck,
    Trash2
  } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { ActionGroup, MetricCard, MetricGrid, SectionHeader, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  import { portfolioStatusCode } from '.';

  type PortfolioMaintenanceOperation = PortfolioMaintenancePreview['operation'];

  let {
    status,
    busy,
    onDoctor,
    onPreviewMaintenance
  }: {
    status: PortfolioStatus;
    busy: boolean;
    onDoctor: () => void;
    onPreviewMaintenance: (operation: PortfolioMaintenanceOperation) => void;
  } = $props();
</script>

<Card.Root class="portfolio-status-panel" aria-labelledby="portfolio-status-title">
  <SectionHeader
    eyebrow={i18n.t('portfolio.statusEyebrow')}
    title={i18n.t('portfolio.statusTitle')}
    titleId="portfolio-status-title"
    description={i18n.reason(status.reasonCode)}
  >
    {#snippet metadata()}
      <StatusBadge
        status={portfolioStatusCode(status)}
        label={i18n.t(`portfolio.state.${status.state}`)}
      />
    {/snippet}
  </SectionHeader>

  <div class="metrics-wrap">
    <MetricGrid label={i18n.t('portfolio.metricsAria')}>
      <MetricCard value={status.projectCount} label={i18n.t('portfolio.projects')}>
        {#snippet icon()}<Database size={17} />{/snippet}
      </MetricCard>
      <MetricCard value={status.nodeCount} label={i18n.t('portfolio.nodes')} tone="info">
        {#snippet icon()}<Activity size={17} />{/snippet}
      </MetricCard>
      <MetricCard value={status.edgeCount} label={i18n.t('portfolio.edges')}>
        {#snippet icon()}<RotateCcw size={17} />{/snippet}
      </MetricCard>
      <MetricCard value={status.contributionCount} label={i18n.t('portfolio.contributions')} tone="success">
        {#snippet icon()}<ShieldCheck size={17} />{/snippet}
      </MetricCard>
    </MetricGrid>
  </div>

  <dl class="identity">
    <div>
      <dt>{i18n.t('portfolio.libraryRevision')}</dt>
      <dd>r{status.libraryRevision}</dd>
    </div>
    <div>
      <dt>{i18n.t('portfolio.catalogGeneration')}</dt>
      <dd>{status.catalogGeneration ?? i18n.t('common.none')}</dd>
    </div>
    <div>
      <dt>{i18n.t('portfolio.catalogIdentity')}</dt>
      <dd><code>{status.catalogId ?? i18n.t('portfolio.noCatalog')}</code></dd>
    </div>
  </dl>

  <ActionGroup class="actions" label={i18n.t('portfolio.statusTitle')}>
    <Button variant="outline" disabled={busy} onclick={onDoctor}>
      <ShieldCheck size={16} aria-hidden="true" />
      {i18n.t('portfolio.runDoctor')}
    </Button>
    <Button
      variant="outline"
      disabled={busy || !status.capabilities.canReconcile}
      onclick={() => onPreviewMaintenance('reconcile')}
    >
      <RefreshCw size={16} aria-hidden="true" />
      {i18n.t('portfolio.reconcile')}
    </Button>
    <Button
      variant="outline"
      disabled={busy || !status.capabilities.canRebuild}
      onclick={() => onPreviewMaintenance('full-rebuild')}
    >
      <RotateCcw size={16} aria-hidden="true" />
      {i18n.t('portfolio.fullRebuild')}
    </Button>
    <Button
      variant="destructive"
      disabled={busy || !status.capabilities.canDeleteDerivedState}
      onclick={() => onPreviewMaintenance('delete-derived-state')}
    >
      <Trash2 size={16} aria-hidden="true" />
      {i18n.t('portfolio.deleteDerived')}
    </Button>
  </ActionGroup>
</Card.Root>

<style>
  :global(.portfolio-status-panel) { min-width: 0; padding: var(--ui-panel-padding); }
  .metrics-wrap { margin-top: 10px; }
  .identity {
    display: grid;
    grid-template-columns: minmax(120px, 0.45fr) minmax(130px, 0.55fr) minmax(0, 2fr);
    gap: 8px;
    margin: 12px 0 0;
  }
  .identity div {
    min-width: 0;
    border-top: 1px solid var(--color-border);
    padding-top: 9px;
  }
  dt {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
    text-transform: uppercase;
  }
  dd { min-width: 0; margin: 4px 0 0; color: var(--color-ink); font-size: 12px; }
  code { overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
  :global(.actions) { margin-top: 10px; }
  @media (max-width: 760px) {
    .identity { grid-template-columns: 1fr 1fr; }
    .identity div:last-child { grid-column: 1 / -1; }
  }
  @media (max-width: 480px) {
    .identity { grid-template-columns: 1fr; }
    .identity div:last-child { grid-column: auto; }
  }
</style>
