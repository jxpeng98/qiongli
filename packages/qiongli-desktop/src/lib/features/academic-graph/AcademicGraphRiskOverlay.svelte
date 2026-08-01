<script lang="ts">
  import type { AcademicGraphEntityReference } from '@qiongli/app-api';
  import { AlertTriangle, ShieldCheck } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

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

<Card.Root class="risk-overlay" role="region" aria-labelledby="graph-risk-title">
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
          <Button variant="ghost" disabled={disabled} onclick={() => onInspect(risk.entity)}>{i18n.t('graph.riskInspect')}</Button>
        </li>
      {/each}
    </ol>
  {/if}
</Card.Root>

<style>
  :global(.risk-overlay) { min-width: 0; margin-bottom: 9px; overflow: hidden; }
  header { display: flex; align-items: flex-start; justify-content: space-between; gap: 9px; border-bottom: 1px solid var(--color-border); padding: 8px 10px; }
  header h2 { margin: 0; font-size: 16px; }
  header p:last-child { max-width: 760px; margin: 5px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.5; }
  header > strong { max-width: 100%; flex: 0 0 auto; overflow: hidden; border-radius: var(--radius-pill); padding: 5px 9px; color: var(--color-danger); background: var(--color-danger-soft); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  header > strong.clear { color: var(--color-success); background: var(--color-success-soft); }
  ol { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; margin: 0; padding: var(--ui-panel-padding); list-style: none; }
  li { min-width: 0; border: 1px solid var(--color-warning-border); border-left: 4px solid var(--color-warning); border-radius: var(--radius-inset); padding: 9px; background: var(--color-warning-soft); }
  li[data-severity='high'] { border-color: var(--color-danger-border); border-left-color: var(--color-danger); background: var(--color-danger-soft); }
  .risk-heading { display: flex; align-items: center; gap: 6px; color: var(--color-warning-strong); }
  li[data-severity='high'] .risk-heading { color: var(--color-danger); }
  .risk-heading span { max-width: 100%; margin-left: auto; overflow: hidden; border-radius: var(--radius-pill); padding: 2px 6px; background: var(--color-surface-muted); font-size: var(--font-size-label); font-weight: 800; text-overflow: ellipsis; white-space: nowrap; }
  li p { margin: 7px 0 0; color: var(--color-muted); font-size: 11px; line-height: 1.5; overflow-wrap: anywhere; }
  .statement { color: var(--color-ink); }
  .statement span { margin: 0 4px; color: var(--color-warning-strong); font-weight: 750; }
  .limit { border-top: 1px solid rgb(251 146 60 / 0.3); padding-top: 7px; }
  .source { display: flex; flex-wrap: wrap; gap: 4px; }
  .source code { color: var(--color-muted); font-size: 10px; }
  li :global([data-slot='button']) { min-height: 44px; margin-top: 4px; padding-inline: 0; color: var(--color-accent-strong); font-size: 11px; font-weight: 750; }
  .empty { display: flex; align-items: center; gap: 7px; margin: 0; padding: 9px; color: var(--color-success); font-size: 12px; font-weight: 700; }
  @media (max-width: 760px) { ol { grid-template-columns: 1fr; } }
  @media (max-width: 520px) { header { flex-direction: column; } }
</style>
