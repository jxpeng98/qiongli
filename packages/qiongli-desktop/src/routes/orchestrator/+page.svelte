<script lang="ts">
  import { Cable, GitBranch, MonitorCog, ShieldCheck } from '@lucide/svelte';

  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let readyProjects = $derived(
    app.snapshot?.researchLibrary.projects.filter(
      (project) => project.lifecycle === 'active' && project.health === 'ready'
    ) ?? []
  );
</script>

<PageHeader
  eyebrow={i18n.t('orchestrator.hostEyebrow')}
  title={i18n.t('orchestrator.hostTitle')}
  description={i18n.t('orchestrator.hostDescription')}
/>

{#if !app.snapshot}
  <section class="surface loading" aria-busy="true">{i18n.t('common.loading')}</section>
{:else}
  <section class="surface boundary" aria-labelledby="orchestrator-boundary-title">
    <GitBranch size={23} aria-hidden="true" />
    <div>
      <p class="eyebrow">{i18n.t('orchestrator.controlPlaneEyebrow')}</p>
      <h2 id="orchestrator-boundary-title">{i18n.t('orchestrator.controlPlaneTitle')}</h2>
      <p>{i18n.t('orchestrator.controlPlaneDescription')}</p>
    </div>
    <StatusBadge status="attention" label={i18n.t('orchestrator.handoffPending')} />
  </section>

  <div class="summary-grid">
    <section class="surface summary-card" aria-labelledby="project-summary-title">
      <ShieldCheck size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.projectSummaryEyebrow')}</p>
        <h2 id="project-summary-title">{i18n.t('orchestrator.projectSummaryTitle')}</h2>
        <p>{i18n.t('orchestrator.projectSummaryDescription', { count: readyProjects.length })}</p>
        <a href="/research-library">{i18n.t('backend.openLibrary')}</a>
      </div>
    </section>

    <section class="surface summary-card" aria-labelledby="execution-summary-title">
      <MonitorCog size={20} aria-hidden="true" />
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.executionOwnerEyebrow')}</p>
        <h2 id="execution-summary-title">{i18n.t('orchestrator.executionOwnerTitle')}</h2>
        <p>{i18n.t('orchestrator.executionOwnerDescription')}</p>
      </div>
    </section>
  </div>

  <section aria-labelledby="host-list-title">
    <div class="section-title">
      <div>
        <p class="eyebrow">{i18n.t('orchestrator.hostsEyebrow')}</p>
        <h2 id="host-list-title">{i18n.t('orchestrator.hostsTitle')}</h2>
      </div>
      <a class="button-secondary" href="/client-integrations">
        <Cable size={16} aria-hidden="true" />
        {i18n.t('backend.openIntegrations')}
      </a>
    </div>

    <div class="host-grid">
      {#each app.snapshot.integrations as integration (integration.target)}
        <article class="surface host-card">
          <header>
            <div>
              <h3>{integration.label}</h3>
              <p>{integration.client.version ?? i18n.t('common.unavailable')}</p>
            </div>
            <StatusBadge status={integration.overall} label={integration.connection.label} />
          </header>
          <dl>
            <div>
              <dt>{i18n.t('orchestrator.pluginState')}</dt>
              <dd>{i18n.label(integration.managedContent.source)}</dd>
            </div>
            <div>
              <dt>{i18n.t('orchestrator.activationState')}</dt>
              <dd>{i18n.label(integration.managedContent.activation)}</dd>
            </div>
            <div>
              <dt>{i18n.t('orchestrator.fullMcpState')}</dt>
              <dd>{i18n.label(integration.managedContent.mcpAttachment)}</dd>
            </div>
          </dl>
          <p class="next-action">{i18n.label(integration.nextAction)}</p>
        </article>
      {/each}
    </div>
  </section>

  <section class="surface nonclaim" aria-labelledby="orchestrator-nonclaim-title">
    <ShieldCheck size={20} aria-hidden="true" />
    <div>
      <h2 id="orchestrator-nonclaim-title">{i18n.t('orchestrator.nonclaimTitle')}</h2>
      <p>{i18n.t('orchestrator.nonclaimDescription')}</p>
    </div>
  </section>
{/if}

<style>
  .loading { padding: 22px; color: var(--color-muted); }
  .boundary,
  .summary-card,
  .nonclaim {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: start;
    gap: 13px;
    padding: 18px;
  }
  .boundary { border-left: 3px solid var(--color-accent); }
  .summary-grid,
  .host-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    margin-top: 10px;
  }
  .summary-card { grid-template-columns: auto minmax(0, 1fr); }
  .summary-card a { color: var(--color-accent-strong); font-weight: 750; }
  .section-title {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 12px;
    margin: 22px 0 10px;
  }
  .section-title a {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    text-decoration: none;
  }
  .host-card { padding: 17px; }
  .host-card header { display: flex; align-items: start; justify-content: space-between; gap: 12px; }
  .host-card dl { display: grid; gap: 7px; margin: 14px 0; }
  .host-card dl div { display: flex; justify-content: space-between; gap: 12px; }
  .host-card dt { color: var(--color-muted); font-size: 11px; }
  .host-card dd { margin: 0; color: var(--color-ink); font-size: 11px; font-weight: 750; }
  .next-action { border-left: 2px solid var(--color-accent); padding-left: 9px; }
  .nonclaim {
    grid-template-columns: auto minmax(0, 1fr);
    margin-top: 14px;
    color: var(--color-success);
  }
  h2, h3, p { margin-top: 0; }
  h2 { margin-bottom: 6px; color: var(--color-ink-strong); font-size: 16px; }
  h3 { margin-bottom: 4px; color: var(--color-ink-strong); font-size: 15px; }
  p { margin-bottom: 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .eyebrow {
    margin-bottom: 5px;
    color: var(--color-accent-strong);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: .1em;
    text-transform: uppercase;
  }
  @media (max-width: 760px) {
    .summary-grid,
    .host-grid { grid-template-columns: 1fr; }
    .boundary { grid-template-columns: auto minmax(0, 1fr); }
    .boundary :global(.status-badge) { grid-column: 1 / -1; justify-self: start; }
  }
  @media (max-width: 520px) {
    .section-title,
    .host-card header { align-items: stretch; flex-direction: column; }
    .section-title a { justify-content: center; }
  }
</style>
