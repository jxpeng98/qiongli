<script lang="ts">
  import type { AppSnapshot } from '@qiongli/app-api';
  import {
    ArrowRight,
    CalendarClock,
    CircleDot,
    GitCommitHorizontal,
    Link2,
    LoaderCircle
  } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { InfoGrid } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';

  import {
    timelineIdentityHref,
    timelineRelatedIdentityKind,
    timelineWorkspaceIsEmpty,
    type TimelineSelection,
    type TimelineWorkspace
  } from '.';

  let {
    workspace,
    selection,
    projects,
    loadingMore,
    onLoadMore
  }: {
    workspace: TimelineWorkspace;
    selection: TimelineSelection;
    projects: AppSnapshot['researchLibrary']['projects'];
    loadingMore: boolean;
    onLoadMore: () => void;
  } = $props();

  let projectNames = $derived(
    new Map(projects.map((project) => [project.projectId, project.displayName]))
  );

  function shortIdentity(value: string): string {
    return value.length > 24 ? `${value.slice(0, 20)}…` : value;
  }
</script>

<Card.Root class="results" role="region" aria-labelledby="timeline-results-title">
  <header class="results-heading">
    <div>
      <p class="eyebrow">{i18n.t('timeline.resultsEyebrow')}</p>
      <h2 id="timeline-results-title">{i18n.t('timeline.resultsTitle')}</h2>
      <p>{i18n.t('timeline.resultsDetail')}</p>
    </div>
    <div class="identity">
      <span>{i18n.t(`timeline.mode.${selection.mode}`)}</span>
      <code title={workspace.queryId}>{shortIdentity(workspace.queryId)}</code>
    </div>
  </header>

  <InfoGrid compact columns={3} class="summary" aria-label={i18n.t('timeline.resultSummary')}>
    <div><strong>{workspace.events.length}</strong><span>{i18n.t('timeline.loaded')}</span></div>
    <div><strong>{workspace.matchedEventCount}</strong><span>{i18n.t('timeline.matched')}</span></div>
    <div>
      <strong>{workspace.projectId ? 1 : workspace.events
        .flatMap((event) => event.projectIds)
        .filter((value, index, values) => values.indexOf(value) === index).length}</strong>
      <span>{i18n.t('timeline.projectScopeCount')}</span>
    </div>
  </InfoGrid>

  <div class="evidence-note">
    <CalendarClock size={18} aria-hidden="true" />
    <p>{i18n.t('timeline.attributionBoundary')}</p>
  </div>

  {#if timelineWorkspaceIsEmpty(workspace)}
    <div class="empty">
      <CircleDot size={24} aria-hidden="true" />
      <h3>{i18n.t('timeline.emptyTitle')}</h3>
      <p>{i18n.t('timeline.emptyDetail')}</p>
    </div>
  {:else}
    <ol class="timeline-list">
      {#each workspace.events as event (event.eventId)}
        <li>
          <div class="rail" aria-hidden="true"><span></span></div>
          <article aria-label={`${i18n.label(event.kind)} · ${i18n.date(event.occurredAtUnix, true)}`}>
            <header class="event-heading">
              <div>
                <strong>{i18n.label(event.kind)}</strong>
                <time datetime={new Date(event.occurredAtUnix * 1_000).toISOString()}>
                  {i18n.date(event.occurredAtUnix, true)}
                </time>
              </div>
              <span class="timestamp-source">
                {i18n.t('timeline.timestampEvidence', {
                  source: i18n.label(event.timestampSource)
                })}
              </span>
            </header>

            <dl class="event-details">
              {#if event.fromProjectRevision !== null || event.toProjectRevision !== null}
                <div>
                  <dt>{i18n.t('timeline.revisionEvidence')}</dt>
                  <dd>
                    {event.fromProjectRevision === null
                      ? i18n.t('common.none') : `r${event.fromProjectRevision}`}
                    <ArrowRight size={13} aria-hidden="true" />
                    {event.toProjectRevision === null
                      ? i18n.t('common.none') : `r${event.toProjectRevision}`}
                  </dd>
                </div>
              {/if}
              {#if event.lifecycle}
                <div><dt>{i18n.t('timeline.lifecycle')}</dt><dd>{i18n.label(event.lifecycle)}</dd></div>
              {/if}
              {#if event.source}
                <div><dt>{i18n.t('timeline.captureSource')}</dt><dd>{i18n.label(event.source)}</dd></div>
              {/if}
              {#if event.delivery}
                <div><dt>{i18n.t('timeline.delivery')}</dt><dd>{i18n.label(event.delivery)}</dd></div>
              {/if}
              {#if event.deliveryState}
                <div><dt>{i18n.t('timeline.deliveryState')}</dt><dd>{i18n.label(event.deliveryState)}</dd></div>
              {/if}
              {#if event.deliveryGeneration !== null}
                <div><dt>{i18n.t('timeline.deliveryGeneration')}</dt><dd>{event.deliveryGeneration}</dd></div>
              {/if}
              {#if event.deliveryReason}
                <div><dt>{i18n.t('timeline.deliveryReason')}</dt><dd>{i18n.reason(event.deliveryReason)}</dd></div>
              {/if}
              {#if event.assignmentOutcome}
                <div><dt>{i18n.t('timeline.assignmentOutcome')}</dt><dd>{i18n.label(event.assignmentOutcome)}</dd></div>
              {/if}
              {#if event.resolutionItemId}
                <div>
                  <dt>{i18n.t('timeline.resolutionItem')}</dt>
                  <dd>
                    {i18n.label(event.resolutionItemKind ?? 'unknown')} ·
                    {i18n.label(event.resolutionDisposition ?? 'unknown')}
                  </dd>
                </div>
              {/if}
            </dl>

            <section class="projects" aria-label={i18n.t('timeline.relatedProjects')}>
              <h3><GitCommitHorizontal size={15} aria-hidden="true" />{i18n.t('timeline.relatedProjects')}</h3>
              <ul>
                {#each event.projectIds as projectId}
                  <li>
                    <a href={timelineIdentityHref(projectId) ?? '/academic-graph'}>
                      <span>{projectNames.get(projectId) ?? i18n.t('timeline.unknownProject')}</span>
                      <code>{projectId}</code>
                    </a>
                  </li>
                {/each}
              </ul>
            </section>

            <details class="identities">
              <summary>
                <Link2 size={14} aria-hidden="true" />
                {i18n.t('timeline.relatedIdentities', { count: event.relatedIds.length })}
              </summary>
              <ul>
                {#each event.relatedIds as relatedId}
                  {@const href = timelineIdentityHref(relatedId, event.projectIds[0] ?? null)}
                  <li>
                    <span>{i18n.t(`timeline.identity.${timelineRelatedIdentityKind(relatedId)}`)}</span>
                    {#if href}
                      <a {href}><code>{relatedId}</code></a>
                    {:else}
                      <code>{relatedId}</code>
                    {/if}
                  </li>
                {/each}
              </ul>
            </details>

            <footer>
              <span>{i18n.t('timeline.eventIdentity')}</span>
              <code>{event.eventId}</code>
            </footer>
          </article>
        </li>
      {/each}
    </ol>
  {/if}

  {#if workspace.nextCursor}
    <Button class="load-more" variant="outline" disabled={loadingMore} onclick={onLoadMore}>
      {#if loadingMore}<LoaderCircle class="spin" size={16} aria-hidden="true" />{/if}
      {loadingMore ? i18n.t('timeline.loadingMore') : i18n.t('timeline.loadMore')}
    </Button>
  {/if}
</Card.Root>

<style>
  :global(.results) { min-width: 0; padding: var(--ui-panel-padding); }
  .results-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .results-heading p:not(.eyebrow) {
    max-width: 680px;
    margin: 5px 0 0;
    color: var(--color-muted);
    font-size: 11px;
    line-height: 1.5;
  }
  .identity { display: grid; justify-items: end; gap: 5px; }
  .identity span { color: var(--color-muted); font-size: 10px; font-weight: 700; }
  code {
    max-width: 100%;
    overflow-wrap: anywhere;
    color: var(--color-ink);
    font-size: 10px;
  }
  :global(.summary) { margin-top: 10px; }
  :global(.summary strong), :global(.summary span) { display: block; }
  :global(.summary strong) { color: var(--color-ink-strong); font-size: 16px; }
  :global(.summary span) { margin-top: 2px; color: var(--color-muted); font-size: 10px; }
  .evidence-note {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    margin-top: 10px;
    border-radius: var(--radius-inset);
    padding: 10px 11px;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }
  .evidence-note p { margin: 0; font-size: 11px; line-height: 1.5; }
  .timeline-list {
    display: grid;
    gap: 0;
    margin: 10px 0 0;
    padding: 0;
    list-style: none;
  }
  .timeline-list > li {
    display: grid;
    min-width: 0;
    grid-template-columns: 18px minmax(0, 1fr);
    gap: 10px;
  }
  .rail { position: relative; }
  .rail::after {
    position: absolute;
    top: 16px;
    bottom: -1px;
    left: 8px;
    width: 1px;
    background: var(--color-border);
    content: "";
  }
  .timeline-list > li:last-child .rail::after { display: none; }
  .rail span {
    position: absolute;
    top: 14px;
    left: 4px;
    z-index: 1;
    width: 9px;
    height: 9px;
    border: 2px solid white;
    border-radius: var(--radius-pill);
    background: var(--color-accent);
    box-shadow: 0 0 0 1px var(--color-accent);
  }
  article {
    min-width: 0;
    margin-bottom: 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    padding: 9px;
    background: var(--color-surface-subtle);
  }
  .event-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 9px;
  }
  .event-heading strong, .event-heading time { display: block; }
  .event-heading strong { color: var(--color-ink-strong); font-size: 13px; }
  .event-heading time { margin-top: 3px; color: var(--color-muted); font-size: 10px; }
  .timestamp-source {
    min-width: 0;
    max-width: 290px;
    flex: none;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    padding: 4px 8px;
    color: var(--color-muted);
    background: var(--color-control);
    font-size: var(--font-size-label);
    font-weight: 700;
    text-align: right;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .event-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 7px;
    margin: 11px 0 0;
  }
  .event-details > div {
    border-left: 2px solid var(--color-border);
    padding-left: 8px;
  }
  dt { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 700; }
  dd {
    display: flex;
    align-items: center;
    gap: 4px;
    margin: 3px 0 0;
    color: var(--color-ink);
    font-size: 10px;
    line-height: 1.4;
  }
  .projects { margin-top: 11px; }
  .projects h3 {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--color-muted);
    font-size: 10px;
  }
  .projects ul, .identities ul { margin: 7px 0 0; padding: 0; list-style: none; }
  .projects ul { display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 6px; }
  .projects a {
    display: grid;
    min-width: 0;
    gap: 2px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-control);
    padding: 7px 8px;
    color: var(--color-accent-strong);
    background: var(--color-control);
    text-decoration: none;
  }
  .projects a:hover { border-color: var(--color-accent); }
  .projects a span { font-size: 10px; font-weight: 700; }
  .identities { margin-top: 8px; }
  .identities summary {
    display: inline-flex;
    min-height: 32px;
    align-items: center;
    gap: 6px;
    color: var(--color-accent-strong);
    cursor: pointer;
    font-size: 10px;
    font-weight: 700;
  }
  .identities li {
    display: grid;
    min-width: 0;
    grid-template-columns: minmax(120px, 0.3fr) minmax(0, 1fr);
    gap: 8px;
    border-top: 1px solid var(--color-border);
    padding: 6px 0;
  }
  .identities li > span { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 700; }
  .identities a { min-width: 0; color: var(--color-accent-strong); }
  article > footer {
    display: grid;
    min-width: 0;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    margin-top: 9px;
    border-top: 1px solid var(--color-border);
    padding-top: 8px;
  }
  article > footer span { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 700; }
  :global(.load-more) { width: 100%; margin-top: 4px; }
  .empty { padding: 14px 10px 12px; color: var(--color-muted); text-align: center; }
  .empty h3 { margin: 9px 0 0; color: var(--color-ink-strong); font-size: 15px; }
  .empty p { margin: 5px auto 0; max-width: 520px; font-size: 11px; line-height: 1.5; }
  @media (max-width: 640px) {
    .results-heading, .event-heading { flex-direction: column; }
    .identity { justify-items: start; }
    :global(.summary) { grid-template-columns: 1fr; }
    .timestamp-source { max-width: none; text-align: left; }
    .projects ul { grid-template-columns: 1fr; }
    .identities li { grid-template-columns: 1fr; gap: 3px; }
  }
</style>
