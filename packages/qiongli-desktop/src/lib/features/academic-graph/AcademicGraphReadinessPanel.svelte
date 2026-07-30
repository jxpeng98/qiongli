<script lang="ts">
  import type {
    AcademicGraphQueryResult,
    AcademicGraphReadiness
  } from '@qiongli/app-api';
  import {
    AlertTriangle,
    CheckCircle2,
    CircleDashed,
    FileQuestion,
    Network
  } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/shared/ui';

  import {
    academicGraphReadinessStatus,
    effectiveAcademicGraphReadiness
  } from './readiness';

  let {
    readiness,
    result,
    compact = false
  }: {
    readiness: AcademicGraphReadiness;
    result: AcademicGraphQueryResult;
    compact?: boolean;
  } = $props();

  let state = $derived(effectiveAcademicGraphReadiness(readiness, result));
  let status = $derived(academicGraphReadinessStatus(state));
  let title = $derived(i18n.t(`graph.readiness.state.${state}.title`));
  let detail = $derived(i18n.t(`graph.readiness.state.${state}.detail`));
  let nextAction = $derived(i18n.t(`graph.readiness.action.${state}`));
</script>

<section
  class="surface readiness"
  class:needs-content={state === 'empty-project' || state === 'no-recognized-artifacts'}
  class:compact
  aria-labelledby="academic-graph-readiness-title"
>
  <div class="summary">
    <div class="identity">
      <span class="state-icon" aria-hidden="true">
        {#if state === 'visualizable'}
          <CheckCircle2 size={20} />
        {:else if state === 'bounded-truncated' || state === 'sparse' || state === 'nodes-without-edges'}
          <AlertTriangle size={20} />
        {:else if state === 'no-recognized-artifacts'}
          <FileQuestion size={20} />
        {:else}
          <CircleDashed size={20} />
        {/if}
      </span>
      <div>
        <p class="eyebrow">{i18n.t('graph.readiness.eyebrow')}</p>
        <h2 id="academic-graph-readiness-title">{title}</h2>
        <p class="detail">{detail}</p>
      </div>
    </div>

    <StatusBadge {status} label={i18n.t(`graph.readiness.badge.${state}`)} />

    <dl class="facts" aria-label={i18n.t('graph.readiness.factsAria')}>
      <div>
        <dt>{i18n.t('graph.readiness.sources')}</dt>
        <dd>{readiness.presentSourceCount}/{readiness.recognizedSourceCount}</dd>
      </div>
      <div>
        <dt>{i18n.t('graph.readiness.semanticNodes')}</dt>
        <dd>{readiness.semanticNodeCount}</dd>
      </div>
      <div>
        <dt>{i18n.t('graph.readiness.relations')}</dt>
        <dd>{readiness.relationCount}</dd>
      </div>
    </dl>
  </div>

  <p class="next-action"><Network size={15} aria-hidden="true" />{nextAction}</p>

  <details
    id="graph-sources"
    open={state === 'empty-project' || state === 'no-recognized-artifacts'}
  >
    <summary>
      {i18n.t('graph.readiness.inspectSources')}
      <span>
        {readiness.missingSourceCount + readiness.invalidSourceCount
          + readiness.unsupportedSourceCount}
        {i18n.t('graph.readiness.attentionSuffix')}
      </span>
    </summary>
    <ul>
      {#each readiness.sources as source (source.artifactPath)}
        <li>
          <code title={source.artifactPath}>{source.artifactPath}</code>
          <span class:source-attention={source.state === 'invalid' || source.state === 'unsupported'}>
            {i18n.t(`graph.readiness.sourceState.${source.state}`)}
          </span>
          <small>
            {i18n.t('graph.readiness.sourceCounts', {
              nodes: source.nodeCount,
              edges: source.edgeCount,
              diagnostics: source.diagnosticCount
            })}
          </small>
        </li>
      {/each}
    </ul>
  </details>
</section>

<style>
  .readiness {
    min-width: 0;
    margin-bottom: 12px;
    overflow: hidden;
  }

  .summary {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
    gap: 12px 18px;
    padding: 16px;
  }

  .identity {
    display: flex;
    min-width: 0;
    align-items: flex-start;
    gap: 10px;
  }

  .state-icon {
    display: grid;
    width: 36px;
    height: 36px;
    flex: none;
    place-items: center;
    border-radius: 10px;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .needs-content .state-icon {
    color: var(--color-muted);
    background: var(--color-surface-subtle);
  }

  h2 {
    margin: 1px 0 2px;
    font-size: 16px;
    line-height: 1.25;
  }

  .detail {
    max-width: 760px;
    margin: 0;
    color: var(--color-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  .facts {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    margin: 0;
  }

  .facts div {
    display: flex;
    min-width: 0;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    padding: 8px 10px;
    background: var(--color-surface-subtle);
  }

  dt {
    overflow: hidden;
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 720;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  dd {
    flex: none;
    margin: 0;
    color: var(--color-ink-strong);
    font-size: 15px;
    font-weight: 800;
  }

  .next-action {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 0;
    border-top: 1px solid var(--color-border);
    padding: 9px 16px;
    color: var(--color-muted);
    background: var(--color-surface-subtle);
    font-size: 11px;
    line-height: 1.45;
  }

  details {
    margin: 0;
    border-top: 1px solid var(--color-border);
  }

  summary {
    display: flex;
    min-height: 44px;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 16px;
    color: var(--color-ink);
    font-size: 11px;
    font-weight: 750;
    cursor: pointer;
  }

  summary span {
    color: var(--color-muted);
    font-weight: 680;
    white-space: nowrap;
  }

  ul {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 8px 12px;
    border-top: 1px solid var(--color-border);
    padding: 9px 16px;
  }

  code {
    min-width: 0;
    overflow: hidden;
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  li > span {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
    white-space: nowrap;
  }

  li > span.source-attention {
    color: #854d0e;
  }

  small {
    color: var(--color-muted);
    font-size: 10px;
    white-space: nowrap;
  }

  @media (max-width: 680px) {
    .summary {
      grid-template-columns: minmax(0, 1fr);
    }

    .summary > :global(.status) {
      grid-row: 1;
      justify-self: end;
    }

    .facts {
      grid-column: 1;
      grid-template-columns: 1fr;
    }

    li {
      grid-template-columns: minmax(0, 1fr) auto;
    }

    small {
      grid-column: 1 / -1;
    }
  }

  @media (max-width: 420px) {
    .summary > :global(.status) {
      grid-row: auto;
      justify-self: start;
    }
  }

  .compact .summary {
    align-items: center;
    padding-block: 12px;
  }

  .compact .detail,
  .compact .facts,
  .compact .next-action {
    display: none;
  }

  .compact .identity {
    align-items: center;
  }

  .compact .state-icon {
    width: 32px;
    height: 32px;
  }
</style>
