<script lang="ts">
  import type {
    AcademicGraphEntityReference,
    AcademicGraphRevisionComparison
  } from '@qiongli/app-api';
  import { ArrowRight, GitCompareArrows, History } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { MetricCard, MetricGrid, SectionHeader, StatePanel } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

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

{#if !comparison}
  <StatePanel title={i18n.t('graph.comparisonTitle')} description={i18n.t('graph.comparisonNoBaseline')}>
    {#snippet icon()}<History size={18} />{/snippet}
  </StatePanel>
{:else}
  <Card.Root class="comparison" role="region" aria-labelledby="graph-comparison-title">
    <header>
      <SectionHeader eyebrow={i18n.t('graph.comparisonEyebrow')} title={i18n.t('graph.comparisonTitle')} titleId="graph-comparison-title" description={i18n.t('graph.comparisonDescription')}>
        {#snippet metadata()}
          <strong class="comparison-status" class:clear={!comparison.hasChanges}>
        {comparison.hasChanges
          ? i18n.t('graph.comparisonChanged')
          : i18n.t('graph.comparisonUnchanged')}
          </strong>
        {/snippet}
      </SectionHeader>
    </header>

    <p class="revision" aria-label={i18n.t('graph.comparisonRevisionAria')}>
      <span>{i18n.t('graph.revision', { revision: comparison.beforeProjectRevision })}</span>
      <ArrowRight size={15} aria-hidden="true" />
      <span>{i18n.t('graph.revision', { revision: comparison.afterProjectRevision })}</span>
    </p>

    <div class="metrics-wrap">
      <MetricGrid label={i18n.t('graph.comparisonTitle')}>
        <MetricCard value={comparison.sourceChangeCount} label={i18n.t('graph.comparisonSources')} />
        <MetricCard value={comparison.nodeChangeCount} label={i18n.t('graph.comparisonNodes')} />
        <MetricCard value={comparison.edgeChangeCount} label={i18n.t('graph.comparisonEdges')} />
        <MetricCard value={delta(comparison.riskDelta.totalSignalCount)} label={i18n.t('graph.comparisonRiskDelta')} tone={comparison.riskDelta.totalSignalCount > 0 ? 'warning' : 'neutral'} />
      </MetricGrid>
    </div>

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
                <Button variant="ghost" disabled={disabled} onclick={() => onInspect({ kind: 'node', id: change.nodeId })}>
                  {i18n.t('graph.comparisonInspect')}
                </Button>
              {/if}
            </article>
          {/each}
          {#each comparison.edgeChanges as change (change.edgeId)}
            <article>
              <span class="kind" data-kind={change.changeKind}>{i18n.label(change.changeKind)}</span>
              <strong>{i18n.label(change.after?.relation ?? change.before?.relation ?? '')}</strong>
              <code>{change.edgeId}</code>
              {#if change.after}
                <Button variant="ghost" disabled={disabled} onclick={() => onInspect({ kind: 'edge', id: change.edgeId })}>
                  {i18n.t('graph.comparisonInspect')}
                </Button>
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
  </Card.Root>
{/if}

<style>
  :global(.comparison) { min-width: 0; margin-bottom: 12px; overflow: hidden; }
  header { border-bottom: 1px solid var(--color-border); padding: 14px 16px; }
  .comparison-status { max-width: 100%; overflow: hidden; border-radius: 999px; padding: 5px 9px; color: var(--color-warning-strong); background: var(--color-warning-soft); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .comparison-status.clear { color: var(--color-success); background: var(--color-success-soft); }
  .unchanged { display: flex; align-items: center; gap: 8px; margin: 0; padding: 18px 16px; color: var(--color-muted); font-size: 12px; }
  .revision { display: flex; align-items: center; gap: 8px; margin: 0; padding: 12px 16px 0; color: var(--color-muted); font-size: 11px; font-weight: 750; }
  .metrics-wrap { --ui-metric-min-height: 58px; padding: 12px 16px; }
  .actions { margin: 0 16px 12px; border-left: 3px solid #f59e0b; padding: 4px 0 4px 12px; }
  .actions h3 { margin: 0 0 5px; font-size: 12px; }
  .actions ul { margin: 0; padding-left: 17px; color: var(--color-muted); font-size: 11px; line-height: 1.6; }
  details { border-top: 1px solid var(--color-border); }
  summary { display: flex; align-items: center; gap: 7px; padding: 11px 16px; color: var(--color-accent-strong); font-size: 11px; font-weight: 750; cursor: pointer; }
  .change-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; padding: 0 16px 16px; }
  article { display: grid; min-width: 0; grid-template-columns: auto 1fr; gap: 5px 8px; border: 1px solid var(--color-border); border-radius: 9px; padding: 10px; }
  article strong { min-width: 0; overflow-wrap: anywhere; font-size: 11px; }
  article code { grid-column: 1 / -1; overflow: hidden; color: var(--color-muted); font-size: var(--font-size-label); text-overflow: ellipsis; }
  .kind { max-width: 100%; align-self: start; overflow: hidden; border-radius: 999px; padding: 2px 6px; color: var(--color-info); background: var(--color-info-soft); font-size: var(--font-size-label); font-weight: 800; text-overflow: ellipsis; white-space: nowrap; }
  .kind[data-kind='removed'] { color: var(--color-danger); background: var(--color-danger-soft); }
  .kind[data-kind='modified'] { color: var(--color-warning-strong); background: var(--color-warning-soft); }
  article :global([data-slot='button']) { min-height: 44px; grid-column: 1 / -1; justify-self: start; padding-inline: 0; color: var(--color-accent-strong); font-size: 10px; font-weight: 750; }
  @media (max-width: 760px) { .change-grid { grid-template-columns: 1fr 1fr; } }
  @media (max-width: 520px) { .change-grid { grid-template-columns: 1fr; } }
</style>
