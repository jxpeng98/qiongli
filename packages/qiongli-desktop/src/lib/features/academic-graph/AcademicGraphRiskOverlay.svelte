<script lang="ts">
  import type { AcademicGraphEntityReference } from '@qiongli/app-api';
  import { AlertTriangle, ShieldCheck } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';

  import type { AcademicGraphRiskOverlay } from './risk';

  let {
    overlay,
    disabled = false,
    onInspect
  }: {
    overlay: AcademicGraphRiskOverlay;
    disabled?: boolean;
    onInspect: (entity: AcademicGraphEntityReference) => void;
  } = $props();
</script>

<section class="surface risk-overlay" aria-labelledby="graph-risk-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('graph.riskEyebrow')}</p>
      <h2 id="graph-risk-title">{i18n.t('graph.riskTitle')}</h2>
      <p>{i18n.t('graph.riskDescription')}</p>
    </div>
    <strong class:clear={overlay.riskCount === 0}>
      {overlay.riskCount === 0 ? i18n.t('graph.riskClear') : i18n.t('graph.riskCount', { count: overlay.riskCount })}
    </strong>
  </header>

  {#if overlay.entries.length === 0}
    <p class="empty"><ShieldCheck size={18} aria-hidden="true" />{i18n.t('graph.riskEmpty')}</p>
  {:else}
    <ol>
      {#each overlay.entries as risk (risk.riskId)}
        <li data-severity={risk.severity}>
          <div class="risk-heading">
            <AlertTriangle size={16} aria-hidden="true" />
            <strong>{i18n.t(`graph.riskKind.${risk.kind}`)}</strong>
            <span>{i18n.t(`graph.riskSeverity.${risk.severity}`)}</span>
          </div>
          {#if risk.relation && risk.targetLabel}
            <p class="statement"><b>{risk.sourceLabel}</b> <span>{i18n.label(risk.relation)}</span> <b>{risk.targetLabel}</b></p>
          {:else}
            <p class="statement"><b>{risk.sourceLabel}</b></p>
          {/if}
          <p>{risk.rationale}</p>
          {#if risk.evidenceLimit}<p class="limit"><b>{i18n.t('graph.evidenceLimit')}:</b> {risk.evidenceLimit}</p>{/if}
          <p class="source"><code>{risk.artifactPath}</code><span>·</span><code>{risk.sourceAnchor}</code></p>
          <button type="button" disabled={disabled} onclick={() => onInspect(risk.entity)}>{i18n.t('graph.riskInspect')}</button>
        </li>
      {/each}
    </ol>
  {/if}
</section>

<style>
  .risk-overlay { min-width: 0; margin-bottom: 12px; overflow: hidden; }
  header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  header h2 { margin: 0; font-size: 16px; }
  header p:last-child { max-width: 760px; margin: 5px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.5; }
  header > strong { flex: 0 0 auto; border-radius: 999px; padding: 5px 9px; color: #991b1b; background: #fee2e2; font-size: 10px; }
  header > strong.clear { color: #166534; background: #dcfce7; }
  ol { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; margin: 0; padding: 14px; list-style: none; }
  li { min-width: 0; border: 1px solid #fed7aa; border-left: 4px solid #f97316; border-radius: 10px; padding: 12px; background: #fff7ed; }
  li[data-severity='high'] { border-color: #fecaca; border-left-color: #dc2626; background: #fff7f7; }
  .risk-heading { display: flex; align-items: center; gap: 6px; color: #9a3412; }
  li[data-severity='high'] .risk-heading { color: #991b1b; }
  .risk-heading span { margin-left: auto; border-radius: 999px; padding: 2px 6px; background: rgb(255 255 255 / 0.8); font-size: 9px; font-weight: 800; }
  li p { margin: 7px 0 0; color: var(--color-muted); font-size: 11px; line-height: 1.5; overflow-wrap: anywhere; }
  .statement { color: var(--color-ink); }
  .statement span { margin: 0 4px; color: #9a3412; font-weight: 750; }
  .limit { border-top: 1px solid rgb(251 146 60 / 0.3); padding-top: 7px; }
  .source { display: flex; flex-wrap: wrap; gap: 4px; }
  .source code { color: var(--color-muted); font-size: 10px; }
  li button { display: inline-flex; min-height: 44px; align-items: center; margin-top: 4px; border: 0; padding: 8px 0; color: var(--color-accent-strong); background: transparent; font: inherit; font-size: 11px; font-weight: 750; cursor: pointer; }
  li button:disabled { cursor: not-allowed; opacity: 0.55; }
  .empty { display: flex; align-items: center; gap: 8px; margin: 0; padding: 18px 16px; color: #166534; font-size: 12px; font-weight: 700; }
  @media (max-width: 760px) { ol { grid-template-columns: 1fr; } }
  @media (max-width: 520px) { header { flex-direction: column; } }
</style>
