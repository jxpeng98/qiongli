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
  import { StatusBadge } from '$lib/shared/ui';

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

<section class="surface status-panel" aria-labelledby="portfolio-status-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('portfolio.statusEyebrow')}</p>
      <h2 id="portfolio-status-title">{i18n.t('portfolio.statusTitle')}</h2>
      <p>{i18n.reason(status.reasonCode)}</p>
    </div>
    <StatusBadge
      status={portfolioStatusCode(status)}
      label={i18n.t(`portfolio.state.${status.state}`)}
    />
  </header>

  <div class="metrics" aria-label={i18n.t('portfolio.metricsAria')}>
    <article>
      <Database size={17} aria-hidden="true" />
      <strong>{status.projectCount}</strong>
      <span>{i18n.t('portfolio.projects')}</span>
    </article>
    <article>
      <Activity size={17} aria-hidden="true" />
      <strong>{status.nodeCount}</strong>
      <span>{i18n.t('portfolio.nodes')}</span>
    </article>
    <article>
      <RotateCcw size={17} aria-hidden="true" />
      <strong>{status.edgeCount}</strong>
      <span>{i18n.t('portfolio.edges')}</span>
    </article>
    <article>
      <ShieldCheck size={17} aria-hidden="true" />
      <strong>{status.contributionCount}</strong>
      <span>{i18n.t('portfolio.contributions')}</span>
    </article>
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

  <div class="actions">
    <button class="button-secondary" type="button" disabled={busy} onclick={onDoctor}>
      <ShieldCheck size={16} aria-hidden="true" />
      {i18n.t('portfolio.runDoctor')}
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={busy || !status.capabilities.canReconcile}
      onclick={() => onPreviewMaintenance('reconcile')}
    >
      <RefreshCw size={16} aria-hidden="true" />
      {i18n.t('portfolio.reconcile')}
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={busy || !status.capabilities.canRebuild}
      onclick={() => onPreviewMaintenance('full-rebuild')}
    >
      <RotateCcw size={16} aria-hidden="true" />
      {i18n.t('portfolio.fullRebuild')}
    </button>
    <button
      class="button-danger"
      type="button"
      disabled={busy || !status.capabilities.canDeleteDerivedState}
      onclick={() => onPreviewMaintenance('delete-derived-state')}
    >
      <Trash2 size={16} aria-hidden="true" />
      {i18n.t('portfolio.deleteDerived')}
    </button>
  </div>
</section>

<style>
  .status-panel { min-width: 0; padding: 16px; }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
  }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 18px; }
  header p:not(.eyebrow) {
    max-width: 680px;
    margin: 5px 0 0;
    color: var(--color-muted);
    font-size: 12px;
    line-height: 1.5;
  }
  .metrics {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
    margin-top: 14px;
  }
  .metrics article {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 2px 8px;
    min-width: 0;
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 10px;
    color: var(--color-accent-strong);
    background: var(--color-surface-subtle);
  }
  .metrics strong { color: var(--color-ink-strong); font-size: 18px; }
  .metrics span {
    grid-column: 1 / -1;
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
    text-transform: uppercase;
  }
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
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
  @media (max-width: 760px) {
    .metrics { grid-template-columns: 1fr 1fr; }
    .identity { grid-template-columns: 1fr 1fr; }
    .identity div:last-child { grid-column: 1 / -1; }
  }
  @media (max-width: 480px) {
    header { flex-direction: column; }
    .metrics, .identity { grid-template-columns: 1fr; }
    .identity div:last-child { grid-column: auto; }
    .actions > button { width: 100%; }
  }
</style>
