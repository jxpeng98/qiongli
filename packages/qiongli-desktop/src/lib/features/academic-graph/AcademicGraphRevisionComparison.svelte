<script lang="ts">
  import type {
    AcademicGraphEntityReference,
    AcademicGraphRevisionComparison
  } from '@qiongli/app-api';
  import { ArrowRight, GitCompareArrows, History } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';

  let {
    comparison,
    disabled = false,
    onInspect
  }: {
    comparison: AcademicGraphRevisionComparison | null;
    disabled?: boolean;
    onInspect: (entity: AcademicGraphEntityReference) => void;
  } = $props();

  function delta(value: number): string {
    return value > 0 ? `+${value}` : String(value);
  }
</script>

<section class="surface comparison" aria-labelledby="graph-comparison-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('graph.comparisonEyebrow')}</p>
      <h2 id="graph-comparison-title">{i18n.t('graph.comparisonTitle')}</h2>
      <p>{i18n.t('graph.comparisonDescription')}</p>
    </div>
    {#if comparison}
      <strong class:clear={!comparison.hasChanges}>
        {comparison.hasChanges
          ? i18n.t('graph.comparisonChanged')
          : i18n.t('graph.comparisonUnchanged')}
      </strong>
    {/if}
  </header>

  {#if !comparison}
    <p class="empty"><History size={18} aria-hidden="true" />{i18n.t('graph.comparisonNoBaseline')}</p>
  {:else}
    <p class="revision" aria-label={i18n.t('graph.comparisonRevisionAria')}>
      <span>{i18n.t('graph.revision', { revision: comparison.beforeProjectRevision })}</span>
      <ArrowRight size={15} aria-hidden="true" />
      <span>{i18n.t('graph.revision', { revision: comparison.afterProjectRevision })}</span>
    </p>

    <dl class="metrics">
      <div><dt>{i18n.t('graph.comparisonSources')}</dt><dd>{comparison.sourceChangeCount}</dd></div>
      <div><dt>{i18n.t('graph.comparisonNodes')}</dt><dd>{comparison.nodeChangeCount}</dd></div>
      <div><dt>{i18n.t('graph.comparisonEdges')}</dt><dd>{comparison.edgeChangeCount}</dd></div>
      <div><dt>{i18n.t('graph.comparisonRiskDelta')}</dt><dd>{delta(comparison.riskDelta.totalSignalCount)}</dd></div>
    </dl>

    {#if comparison.nextActions.length > 0}
      <div class="actions">
        <h3>{i18n.t('graph.comparisonNextActions')}</h3>
        <ul>
          {#each comparison.nextActions as action}<li>{i18n.t(`graph.comparisonAction.${action}`)}</li>{/each}
        </ul>
      </div>
    {/if}

    {#if comparison.nodeChanges.length > 0 || comparison.edgeChanges.length > 0 || comparison.sourceChanges.length > 0}
      <details>
        <summary><GitCompareArrows size={15} aria-hidden="true" />{i18n.t('graph.comparisonDetails')}</summary>
        <div class="change-grid">
          {#each comparison.nodeChanges as change (change.nodeId)}
            <article>
              <span class="kind" data-kind={change.changeKind}>{i18n.label(change.changeKind)}</span>
              <strong>{change.after?.label ?? change.before?.label}</strong>
              <code>{change.nodeId}</code>
              {#if change.after}
                <button type="button" disabled={disabled} onclick={() => onInspect({ kind: 'node', id: change.nodeId })}>
                  {i18n.t('graph.comparisonInspect')}
                </button>
              {/if}
            </article>
          {/each}
          {#each comparison.edgeChanges as change (change.edgeId)}
            <article>
              <span class="kind" data-kind={change.changeKind}>{i18n.label(change.changeKind)}</span>
              <strong>{i18n.label(change.after?.relation ?? change.before?.relation ?? '')}</strong>
              <code>{change.edgeId}</code>
              {#if change.after}
                <button type="button" disabled={disabled} onclick={() => onInspect({ kind: 'edge', id: change.edgeId })}>
                  {i18n.t('graph.comparisonInspect')}
                </button>
              {/if}
            </article>
          {/each}
          {#each comparison.sourceChanges as change (change.artifactPath)}
            <article>
              <span class="kind" data-kind={change.changeKind}>{i18n.label(change.changeKind)}</span>
              <strong>{i18n.t('graph.comparisonSource')}</strong>
              <code>{change.artifactPath}</code>
            </article>
          {/each}
        </div>
      </details>
    {:else}
      <p class="unchanged">{i18n.t('graph.comparisonNoChanges')}</p>
    {/if}
  {/if}
</section>

<style>
  .comparison { min-width: 0; margin-bottom: 12px; overflow: hidden; }
  header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  header h2 { margin: 0; font-size: 16px; }
  header p:last-child { max-width: 760px; margin: 5px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.5; }
  header > strong { flex: 0 0 auto; border-radius: 999px; padding: 5px 9px; color: #92400e; background: #fef3c7; font-size: 10px; }
  header > strong.clear { color: #166534; background: #dcfce7; }
  .empty, .unchanged { display: flex; align-items: center; gap: 8px; margin: 0; padding: 18px 16px; color: var(--color-muted); font-size: 12px; }
  .revision { display: flex; align-items: center; gap: 8px; margin: 0; padding: 12px 16px 0; color: var(--color-muted); font-size: 11px; font-weight: 750; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; margin: 0; padding: 12px 16px; }
  .metrics div { border: 1px solid var(--color-border); border-radius: 9px; padding: 9px; background: var(--color-surface-muted); }
  .metrics dt { color: var(--color-muted); font-size: 10px; font-weight: 750; }
  .metrics dd { margin: 3px 0 0; color: var(--color-ink); font-size: 17px; font-weight: 800; }
  .actions { margin: 0 16px 12px; border-left: 3px solid #f59e0b; padding: 4px 0 4px 12px; }
  .actions h3 { margin: 0 0 5px; font-size: 12px; }
  .actions ul { margin: 0; padding-left: 17px; color: var(--color-muted); font-size: 11px; line-height: 1.6; }
  details { border-top: 1px solid var(--color-border); }
  summary { display: flex; align-items: center; gap: 7px; padding: 11px 16px; color: var(--color-accent-strong); font-size: 11px; font-weight: 750; cursor: pointer; }
  .change-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; padding: 0 16px 16px; }
  article { display: grid; min-width: 0; grid-template-columns: auto 1fr; gap: 5px 8px; border: 1px solid var(--color-border); border-radius: 9px; padding: 10px; }
  article strong { min-width: 0; overflow-wrap: anywhere; font-size: 11px; }
  article code { grid-column: 1 / -1; overflow: hidden; color: var(--color-muted); font-size: 9px; text-overflow: ellipsis; }
  .kind { align-self: start; border-radius: 999px; padding: 2px 6px; color: #1d4ed8; background: #dbeafe; font-size: 9px; font-weight: 800; }
  .kind[data-kind='removed'] { color: #991b1b; background: #fee2e2; }
  .kind[data-kind='modified'] { color: #92400e; background: #fef3c7; }
  article button { grid-column: 1 / -1; justify-self: start; border: 0; padding: 0; color: var(--color-accent-strong); background: transparent; font: inherit; font-size: 10px; font-weight: 750; cursor: pointer; }
  article button:disabled { cursor: not-allowed; opacity: 0.55; }
  @media (max-width: 760px) { .metrics, .change-grid { grid-template-columns: 1fr 1fr; } }
  @media (max-width: 520px) { header { flex-direction: column; } .metrics, .change-grid { grid-template-columns: 1fr; } }
</style>
