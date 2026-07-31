<script lang="ts">
  import type {
    CaptureAssignmentView,
    CaptureDeliveryView,
    CaptureResolutionPreview,
    CaptureResolutionSelection,
    CaptureResolutionView
  } from '@qiongli/app-api';
  import {
    AlertTriangle,
    ArrowRightLeft,
    CheckCircle2,
    ClipboardCheck,
    RefreshCw,
    Scale
  } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { NativeSelect } from '$lib/components/ui/native-select';
  import {
    assignmentStatus,
    completeResolutionSelections,
    prioritizeAssignments,
    resolutionForAssignment
  } from '.';

  type ProjectOption = {
    projectId: string;
    displayName: string;
  };

  let {
    deliveries,
    assignments,
    resolutions,
    projects,
    plan,
    loading,
    assignmentsTruncated,
    resolutionsTruncated,
    onPreviewAssignment,
    onInspectAssignment,
    onLoadResolutionPlan,
    onPreviewResolution,
    onInspectResolution,
    onLoadMoreAssignments,
    onLoadMoreResolutions
  }: {
    deliveries: CaptureDeliveryView[];
    assignments: CaptureAssignmentView[];
    resolutions: CaptureResolutionView[];
    projects: ProjectOption[];
    plan: CaptureResolutionPreview | null;
    loading: boolean;
    assignmentsTruncated: boolean;
    resolutionsTruncated: boolean;
    onPreviewAssignment: (
      delivery: CaptureDeliveryView,
      projectId: string,
      decision: 'assign' | 'reject'
    ) => void;
    onInspectAssignment: (assignment: CaptureAssignmentView) => void;
    onLoadResolutionPlan: (assignment: CaptureAssignmentView) => void;
    onPreviewResolution: (
      assignment: CaptureAssignmentView,
      selections: CaptureResolutionSelection[]
    ) => void;
    onInspectResolution: (resolution: CaptureResolutionView) => void;
    onLoadMoreAssignments: () => void;
    onLoadMoreResolutions: () => void;
  } = $props();

  let targetProjects = $state<Record<string, string>>({});
  let selectedAssignmentId = $state<string | null>(null);
  let selectionValues =
    $state<Record<string, CaptureResolutionSelection['disposition'] | ''>>({});
  let activePlanDigest = $state<string | null>(null);

  let candidates = $derived(deliveries.filter((delivery) =>
    delivery.destination === null
    && delivery.state !== 'acknowledged'
    && delivery.state !== 'cancelled'
  ));
  let orderedAssignments = $derived(prioritizeAssignments(assignments));
  let attentionCount = $derived(
    candidates.length + assignments.filter((assignment) =>
      assignment.state === 'pending' || assignment.canResolve
    ).length
  );
  let selectedAssignment = $derived(
    assignments.find((assignment) => assignment.intentId === selectedAssignmentId) ?? null
  );
  let completeSelections = $derived.by(() => {
    if (!plan || plan.assignmentReceiptId !== selectedAssignment?.receiptId) return null;
    return completeResolutionSelections(plan, new Map(
      Object.entries(selectionValues).filter((entry): entry is [
        string,
        CaptureResolutionSelection['disposition']
      ] => entry[1] !== '')
    ));
  });

  $effect(() => {
    if (!plan) {
      if (activePlanDigest !== null || Object.keys(selectionValues).length > 0) {
        activePlanDigest = null;
        selectionValues = {};
      }
      return;
    }
    if (plan.planDigest === activePlanDigest) return;
    activePlanDigest = plan.planDigest;
    selectionValues = {};
  });

  function chooseTarget(event: Event, envelopeId: string): void {
    targetProjects[envelopeId] = (event.currentTarget as HTMLSelectElement).value;
  }

  function chooseDisposition(event: Event, itemId: string): void {
    selectionValues[itemId] = (event.currentTarget as HTMLSelectElement)
      .value as CaptureResolutionSelection['disposition'] | '';
  }

  function loadPlan(assignment: CaptureAssignmentView): void {
    selectedAssignmentId = assignment.intentId;
    onLoadResolutionPlan(assignment);
  }
</script>

<Card.Root class="conflicts">
  <div class="heading">
    <div>
      <p class="eyebrow">{i18n.t('captures.conflictsEyebrow')}</p>
      <h2>{i18n.t('captures.conflictsTitle')}</h2>
      <p>{i18n.t('captures.conflictsSummary', {
        candidates: candidates.length,
        assignments: assignments.length
      })}</p>
    </div>
    <StatusBadge
      status={attentionCount > 0 ? 'attention' : 'ready'}
      label={attentionCount > 0
        ? i18n.t('captures.conflictsAttention', { count: attentionCount })
        : i18n.t('captures.conflictsClear')}
    />
  </div>

  <section class="assignment-section" aria-labelledby="assignment-candidates-title">
    <h3 id="assignment-candidates-title">{i18n.t('captures.assignmentCandidates')}</h3>
    <p>{i18n.t('captures.assignmentCandidatesDetail')}</p>
    {#if candidates.length === 0}
      <div class="empty-row">
        <CheckCircle2 size={18} aria-hidden="true" />
        <span>{i18n.t('captures.noAssignmentCandidates')}</span>
      </div>
    {:else}
      <div class="candidate-list">
        {#each candidates as delivery (delivery.envelopeId)}
          <article>
            <div class="candidate-copy">
              <strong>{i18n.label(delivery.source)}</strong>
              <small>{i18n.reason(delivery.lastReason)}</small>
              <code>{delivery.envelopeId.slice(0, 20)}…</code>
            </div>
            <label>
              <span>{i18n.t('captures.targetProject')}</span>
              <NativeSelect
                value={targetProjects[delivery.envelopeId] ?? ''}
                disabled={loading}
                onchange={(event) => chooseTarget(event, delivery.envelopeId)}
              >
                <option value="">{i18n.t('captures.chooseTargetProject')}</option>
                {#each projects as project (project.projectId)}
                  <option value={project.projectId}>{project.displayName}</option>
                {/each}
              </NativeSelect>
            </label>
            <div class="candidate-actions">
              <Button
                disabled={loading || !targetProjects[delivery.envelopeId]}
                onclick={() => onPreviewAssignment(
                  delivery,
                  targetProjects[delivery.envelopeId] ?? '',
                  'assign'
                )}
              >
                <ArrowRightLeft size={14} aria-hidden="true" />
                {i18n.t('captures.reviewAssignment')}
              </Button>
              <Button
                variant="outline"
                disabled={loading || !targetProjects[delivery.envelopeId]}
                onclick={() => onPreviewAssignment(
                  delivery,
                  targetProjects[delivery.envelopeId] ?? '',
                  'reject'
                )}
              >
                {i18n.t('captures.reviewRejection')}
              </Button>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </section>

  <section class="assignment-section" aria-labelledby="assignment-history-title">
    <h3 id="assignment-history-title">{i18n.t('captures.assignmentHistory')}</h3>
    <p>{i18n.t('captures.assignmentHistoryDetail')}</p>
    {#if orderedAssignments.length === 0}
      <div class="empty-row"><span>{i18n.t('captures.noAssignments')}</span></div>
    {:else}
      <div class="assignment-list">
        {#each orderedAssignments as assignment (assignment.intentId)}
          {@const resolution = resolutionForAssignment(assignment, resolutions)}
          <article class:selected={selectedAssignmentId === assignment.intentId}>
            <Button
              class="assignment-main"
              variant="ghost"
              onclick={() => {
                selectedAssignmentId = assignment.intentId;
                onInspectAssignment(assignment);
              }}
            >
              <span>
                <strong>{i18n.label(assignment.outcome ?? assignment.state)}</strong>
                <small>{i18n.t('captures.targetRevision', {
                  revision: assignment.targetProjectRevision
                })}</small>
              </span>
              <span class="lineage">
                <code>{assignment.sourceEnvelopeId.slice(0, 16)}…</code>
                {#if assignment.childEnvelopeId}
                  <span aria-hidden="true">→</span>
                  <code>{assignment.childEnvelopeId.slice(0, 16)}…</code>
                {/if}
              </span>
              <StatusBadge
                status={assignmentStatus(assignment)}
                label={assignment.canResolve
                  ? i18n.t('captures.academicReviewRequired')
                  : i18n.label(assignment.state)}
              />
            </Button>
            <div class="assignment-actions">
              {#if assignment.canResolve && assignment.receiptId}
                <Button
                  disabled={loading}
                  onclick={() => loadPlan(assignment)}
                >
                  <Scale size={14} aria-hidden="true" />
                  {i18n.t('captures.resolveAcademicMeaning')}
                </Button>
              {/if}
              {#if resolution}
                <Button
                  variant="outline"
                  disabled={loading}
                  onclick={() => onInspectResolution(resolution)}
                >
                  <ClipboardCheck size={14} aria-hidden="true" />
                  {i18n.t('captures.inspectResolution')}
                </Button>
              {/if}
            </div>
          </article>
        {/each}
      </div>
    {/if}
    {#if assignmentsTruncated}
      <Button
        class="load-more"
        variant="outline"
        disabled={loading}
        onclick={onLoadMoreAssignments}
      >
        <RefreshCw size={14} class={loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('captures.loadMoreAssignments')}
      </Button>
    {/if}
    {#if resolutionsTruncated}
      <Button
        class="load-more"
        variant="outline"
        disabled={loading}
        onclick={onLoadMoreResolutions}
      >
        <RefreshCw size={14} class={loading ? 'spin' : undefined} aria-hidden="true" />
        {i18n.t('captures.loadMoreResolutions')}
      </Button>
    {/if}
  </section>

  {#if plan && selectedAssignment?.receiptId === plan.assignmentReceiptId}
    <section class="resolution-plan" aria-labelledby="resolution-plan-title">
      <div>
        <p class="eyebrow">{i18n.t('captures.resolutionPlanEyebrow')}</p>
        <h3 id="resolution-plan-title">{i18n.t('captures.resolutionPlanTitle')}</h3>
        <p>{i18n.t('captures.resolutionPlanDetail', { count: plan.items.length })}</p>
      </div>
      <ol>
        {#each plan.items as item (item.itemId)}
          <li>
            <div class="item-copy">
              <strong>{i18n.label(item.kind)}</strong>
              <span>{item.sourceSummary}</span>
              {#if item.currentSummary}
                <small>{i18n.t('captures.currentMeaning')}: {item.currentSummary}</small>
              {/if}
              <small>{item.explanation}</small>
            </div>
            <label>
              <span>{i18n.t('captures.disposition')}</span>
              <NativeSelect
                value={selectionValues[item.itemId] ?? ''}
                disabled={loading}
                onchange={(event) => chooseDisposition(event, item.itemId)}
              >
                <option value="">{i18n.t('captures.chooseDisposition')}</option>
                {#each item.allowedDispositions as disposition}
                  <option value={disposition}>{i18n.label(disposition)}</option>
                {/each}
              </NativeSelect>
            </label>
          </li>
        {/each}
      </ol>
      <div class="resolution-actions">
        <p class:complete={completeSelections !== null}>
          {completeSelections
            ? i18n.t('captures.allItemsSelected')
            : i18n.t('captures.selectEveryItem')}
        </p>
        <Button
          disabled={loading || !completeSelections || !selectedAssignment}
          onclick={() => {
            if (selectedAssignment && completeSelections) {
              onPreviewResolution(selectedAssignment, completeSelections);
            }
          }}
        >
          <Scale size={14} aria-hidden="true" />
          {i18n.t('captures.reviewResolution')}
        </Button>
      </div>
    </section>
  {/if}
</Card.Root>

<style>
  :global(.conflicts) { padding: var(--ui-panel-padding); }
  .heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
  .heading h2, .assignment-section h3, .resolution-plan h3 { margin: 0; color: var(--color-ink-strong); }
  .heading h2 { font-size: 20px; }
  .heading > div > p:last-child, .assignment-section > p, .resolution-plan > div > p:last-child {
    margin: 7px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.55;
  }
  .assignment-section, .resolution-plan { margin-top: 10px; border-top: 1px solid var(--color-border); padding-top: 10px; }
  .assignment-section h3, .resolution-plan h3 { font-size: 15px; }
  .empty-row { display: flex; min-height: 64px; align-items: center; gap: 8px; color: var(--color-success); }
  .candidate-list, .assignment-list { display: grid; gap: 8px; margin-top: 12px; }
  .candidate-list article {
    display: grid; grid-template-columns: minmax(180px, 1fr) minmax(220px, .8fr) auto;
    align-items: end; gap: 10px; border: 1px solid var(--color-border); border-radius: var(--radius-inset);
    padding: 10px; background: var(--color-surface-subtle);
  }
  .candidate-copy strong, .candidate-copy small, .candidate-copy code { display: block; }
  .candidate-copy strong { color: var(--color-ink-strong); font-size: 12px; }
  .candidate-copy small { margin-top: 4px; color: var(--color-muted); font-size: 10px; }
  .candidate-copy code { margin-top: 5px; color: var(--color-accent-strong); font-size: var(--font-size-label); }
  label { display: grid; gap: 4px; }
  label > span { color: var(--color-muted); font-size: var(--font-size-label); font-weight: 800; text-transform: uppercase; }
  label :global([data-slot='native-select-wrapper']) { width: 100%; }
  .candidate-actions, .assignment-actions, .resolution-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 7px; }
  .candidate-actions :global([data-slot='button']), .assignment-actions :global([data-slot='button']), .resolution-actions :global([data-slot='button']), :global(.load-more) {
    display: inline-flex; min-height: 44px; align-items: center; gap: 6px; padding: 6px 9px; font-size: 11px;
  }
  .assignment-list article { border: 1px solid var(--color-border); border-radius: var(--radius-inset); padding: 8px; }
  .assignment-list article.selected { border-color: var(--color-accent-border); background: var(--color-accent-soft); }
  :global(.assignment-main) {
    display: grid; width: 100%; grid-template-columns: minmax(150px, .8fr) minmax(220px, 1.2fr) auto;
    align-items: center; gap: 10px; border: 0; padding: 4px; color: inherit; background: transparent;
    text-align: left; cursor: pointer;
  }
  :global(.assignment-main:focus-visible) { outline: 3px solid color-mix(in srgb, var(--color-focus) 34%, transparent); outline-offset: 2px; }
  :global(.assignment-main) strong, :global(.assignment-main) small { display: block; }
  :global(.assignment-main) strong { color: var(--color-ink-strong); font-size: 12px; }
  :global(.assignment-main) small { margin-top: 4px; color: var(--color-muted); font-size: 10px; }
  .lineage { display: flex; min-width: 0; align-items: center; gap: 6px; color: var(--color-muted); }
  .lineage code { overflow: hidden; color: var(--color-accent-strong); font-size: var(--font-size-label); text-overflow: ellipsis; }
  .assignment-actions { justify-content: flex-end; margin-top: 6px; }
  :global(.load-more) { margin: 10px 7px 0 0; }
  .resolution-plan { border: 1px solid var(--color-warning-border); border-radius: var(--radius-inset); padding: var(--ui-panel-padding); background: var(--color-warning-soft); }
  .resolution-plan ol { display: grid; gap: 8px; margin: 10px 0 0; padding: 0; list-style: none; }
  .resolution-plan li {
    display: grid; grid-template-columns: minmax(0, 1fr) minmax(210px, .45fr);
    gap: 12px; border: 1px solid color-mix(in srgb, var(--color-warning) 55%, var(--color-border)); border-radius: var(--radius-inset); padding: 11px; background: var(--color-control);
  }
  .item-copy strong, .item-copy span, .item-copy small { display: block; }
  .item-copy strong { color: var(--color-ink-strong); font-size: 12px; }
  .item-copy span { margin-top: 5px; color: var(--color-ink); font-size: 11px; line-height: 1.5; }
  .item-copy small { margin-top: 5px; color: var(--color-muted); font-size: 10px; line-height: 1.45; }
  .resolution-actions { justify-content: space-between; margin-top: 12px; }
  .resolution-actions p { margin: 0; color: var(--color-warning); font-size: 11px; font-weight: 700; }
  .resolution-actions p.complete { color: var(--color-success); }
  code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

  @media (max-width: 860px) {
    .candidate-list article { grid-template-columns: 1fr 1fr; }
    .candidate-actions { grid-column: 1 / -1; }
    :global(.assignment-main) { grid-template-columns: minmax(140px, 1fr) auto; }
    .lineage { grid-column: 1 / -1; }
  }

  @media (max-width: 620px) {
    :global(.conflicts) { padding: 12px; }
    .heading { flex-direction: column; gap: 10px; }
    .candidate-list article, .resolution-plan li { grid-template-columns: 1fr; }
    .candidate-actions { grid-column: auto; }
    :global(.assignment-main) { grid-template-columns: 1fr; }
    .lineage { grid-column: auto; flex-wrap: wrap; }
    .assignment-actions, .resolution-actions { justify-content: flex-start; }
  }
</style>
