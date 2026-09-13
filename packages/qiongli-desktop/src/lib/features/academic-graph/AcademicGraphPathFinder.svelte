<script lang="ts">
  import type {
    AcademicGraphPathQuery,
    AcademicGraphPathResult,
    AcademicGraphSnapshot
  } from '@qiongli/app-api';
  import { Route } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { NativeSelect } from '$lib/components/ui/native-select';

  let {
    graph,
    result,
    disabled = false,
    onQuery
  }: {
    graph: AcademicGraphSnapshot;
    result: AcademicGraphPathResult | null;
    disabled?: boolean;
    onQuery: (query: Omit<AcademicGraphPathQuery, 'expectedProjectionId'>) => Promise<boolean>;
  } = $props();

  let sourceNodeId = $state('');
  let targetNodeId = $state('');
  let maxHops = $state(6);
  let queryState = $state<'idle' | 'querying' | 'failed'>('idle');
  let activeProjectionId = $state('');

  let matchingResult = $derived(
    result
      && result.sourceNodeId === sourceNodeId
      && result.targetNodeId === targetNodeId
      && result.maxHops === maxHops
      ? result : null
  );
  let sameEndpoint = $derived(sourceNodeId !== '' && sourceNodeId === targetNodeId);
  let missingEndpoint = $derived(sourceNodeId === '' || targetNodeId === '');

  $effect(() => {
    if (graph.projectionId !== activeProjectionId) {
      activeProjectionId = graph.projectionId;
      sourceNodeId = graph.nodes.find((node) => node.nodeType === 'research-question')?.nodeId
        ?? graph.nodes.find((node) => node.nodeType === 'project')?.nodeId
        ?? graph.nodes[0]?.nodeId
        ?? '';
      targetNodeId = graph.nodes.find((node) =>
        node.nodeType === 'manuscript-section' && node.nodeId !== sourceNodeId)?.nodeId
        ?? graph.nodes.find((node) => node.nodeType === 'claim' && node.nodeId !== sourceNodeId)?.nodeId
        ?? graph.nodes.filter((node) => node.nodeId !== sourceNodeId).at(-1)?.nodeId
        ?? '';
      maxHops = 6;
      queryState = 'idle';
    }
  });

  async function queryPath(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (missingEndpoint || sameEndpoint || disabled || queryState === 'querying') {
      return;
    }
    queryState = 'querying';
    try {
      queryState = await onQuery({ sourceNodeId, targetNodeId, maxHops }) ? 'idle' : 'failed';
    } catch {
      queryState = 'failed';
    }
  }
</script>

<Card.Root class="path-finder" role="region" aria-labelledby="graph-path-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('graph.pathEyebrow')}</p>
      <h2 id="graph-path-title">{i18n.t('graph.pathTitle')}</h2>
      <p>{i18n.t('graph.pathDescription')}</p>
    </div>
    <span class="header-icon"><Route size={21} aria-hidden="true" /></span>
  </header>

  <form onsubmit={queryPath} aria-label={i18n.t('graph.pathForm')}>
    <label>
      <span>{i18n.t('graph.pathSource')}</span>
      <NativeSelect bind:value={sourceNodeId} disabled={disabled}>
        {#each graph.nodes as node (node.nodeId)}
          <option value={node.nodeId}>{node.label} · {i18n.label(node.nodeType)}</option>
        {/each}
      </NativeSelect>
    </label>
    <label>
      <span>{i18n.t('graph.pathTarget')}</span>
      <NativeSelect bind:value={targetNodeId} disabled={disabled}>
        {#each graph.nodes as node (node.nodeId)}
          <option value={node.nodeId}>{node.label} · {i18n.label(node.nodeType)}</option>
        {/each}
      </NativeSelect>
    </label>
    <label>
      <span>{i18n.t('graph.pathMaxHops')}</span>
      <NativeSelect bind:value={maxHops} disabled={disabled}>
        {#each [1, 2, 4, 6, 8, 12] as hops}
          <option value={hops}>{hops}</option>
        {/each}
      </NativeSelect>
    </label>
    <Button type="submit" disabled={disabled || missingEndpoint || sameEndpoint || queryState === 'querying'}>
      <Route size={15} aria-hidden="true" />
      {queryState === 'querying' ? i18n.t('graph.pathQuerying') : i18n.t('graph.pathFind')}
    </Button>
  </form>

  {#if missingEndpoint || sameEndpoint}
    <p class="path-notice" role="status">{i18n.t('graph.pathDistinct')}</p>
  {:else if queryState === 'failed'}
    <p class="path-notice failed" role="alert">{i18n.t('graph.pathFailed')}</p>
  {:else if matchingResult?.status === 'not-found'}
    <p class="path-notice" role="status">{i18n.t(
      maxHops === 1 ? 'graph.pathNotFoundOne' : 'graph.pathNotFound',
      { hops: maxHops }
    )}</p>
  {:else if matchingResult?.status === 'found'}
    <div class="path-result" aria-live="polite">
      <p class="path-summary">{i18n.t(
        matchingResult.hopCount === 1 ? 'graph.pathSummaryOne' : 'graph.pathSummary',
        { hops: matchingResult.hopCount }
      )}</p>
      <ol>
        {#each matchingResult.steps as step, index (step.edgeId)}
          {@const edge = matchingResult.edges[index]}
          {@const from = matchingResult.nodes[index]}
          {@const to = matchingResult.nodes[index + 1]}
          {#if edge && from && to}
            <li>
              <span class="step-number">{i18n.t('graph.pathStep', { step: step.sequence })}</span>
              <div class="statement">
                <strong>{from.label}</strong>
                <span>{i18n.label(edge.relation)} · {i18n.t(step.traversal === 'forward' ? 'graph.pathForward' : 'graph.pathReverse')}</span>
                <strong>{to.label}</strong>
              </div>
              <p>{edge.rationale}</p>
              <dl>
                <div><dt>{i18n.t('graph.confidence')}</dt><dd>{i18n.label(edge.confidence)}</dd></div>
                <div><dt>{i18n.t('graph.status')}</dt><dd>{i18n.label(edge.status)}</dd></div>
                <div><dt>{i18n.t('graph.strength')}</dt><dd>{i18n.label(edge.inferenceStrength)}</dd></div>
              </dl>
              <details><summary>{i18n.t('graph.evidenceLimit')}</summary><p>{edge.evidenceLimit}</p></details>
            </li>
          {/if}
        {/each}
      </ol>
    </div>
  {/if}
</Card.Root>

<style>
  :global(.path-finder) { min-width: 0; margin-bottom: 9px; overflow: hidden; }
  header { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; border-bottom: 1px solid var(--color-border); padding: 8px 10px; }
  header h2, header p { margin: 0; }
  header h2 { font-size: 16px; }
  header div > p:last-child { max-width: 760px; margin-top: 5px; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.5; }
  .header-icon { display: inline-flex; flex: 0 0 auto; color: var(--color-accent-strong); }
  form { display: grid; grid-template-columns: minmax(180px, 1fr) minmax(180px, 1fr) minmax(90px, 0.35fr) auto; align-items: end; gap: 7px; padding: 8px 10px; }
  label { display: grid; min-width: 0; gap: 5px; color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 750; }
  label :global([data-slot='native-select-wrapper']) { width: 100%; }
  .path-notice, .path-summary { margin: 0; border-top: 1px solid var(--color-border); padding: 7px 10px; color: var(--color-muted); font-size: var(--font-size-label); }
  .path-notice.failed { color: var(--color-danger); }
  .path-summary { color: var(--color-accent-strong); font-weight: 750; }
  ol { display: grid; gap: 0; margin: 0; padding: 0; list-style: none; }
  li { display: grid; gap: 7px; border-top: 1px solid var(--color-border); padding: 8px 10px; }
  .step-number { width: fit-content; max-width: 100%; overflow: hidden; border-radius: var(--radius-pill); padding: 2px 7px; color: var(--color-accent-strong); background: var(--color-accent-soft); font-size: var(--font-size-label); font-weight: 800; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
  .statement { display: grid; grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr); align-items: center; gap: 10px; font-size: var(--font-size-supporting); }
  .statement strong:last-child { text-align: right; }
  .statement span { color: var(--color-accent-strong); font-size: var(--font-size-micro); font-weight: 750; text-align: center; }
  li > p, details { margin: 0; color: var(--color-muted); font-size: var(--font-size-label); line-height: 1.55; }
  dl { display: flex; flex-wrap: wrap; gap: 10px; margin: 0; }
  dl div { display: flex; gap: 4px; }
  dt { color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 700; }
  dd { margin: 0; font-size: var(--font-size-micro); font-weight: 750; }
  summary { cursor: pointer; font-weight: 700; }
  @media (max-width: 900px) { form { grid-template-columns: 1fr 1fr; } }
  @media (max-width: 560px) { form { grid-template-columns: 1fr; } .statement { grid-template-columns: 1fr; } .statement strong:last-child, .statement span { text-align: left; } }
</style>
