<script lang="ts">
  import { Check, LoaderCircle, MapPin, ShieldCheck, X } from '@lucide/svelte';
  import { onDestroy, tick, untrack } from 'svelte';

  import type {
    CaptureAssignmentPreview,
    CaptureConsolidationPreview,
    CaptureDeliveryAcknowledgementPreview,
    CaptureIntakePreview,
    CaptureResolutionPreview,
    CaptureResolutionSelection,
    OperationPreview,
    PortfolioMaintenancePreview
  } from '@qiongli/app-api';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Button } from '$lib/components/ui/button';
  import { i18n } from '$lib/i18n.svelte';
  import { cn } from '$lib/utils';

  let {
    preview,
    intake = null,
    consolidation = null,
    acknowledgement = null,
    assignment = null,
    resolution = null,
    resolutionSelections = [],
    portfolioMaintenance = null,
    returnFocusTarget = null,
    busy,
    onConfirm,
    onCancel
  }: {
    preview: OperationPreview;
    intake?: CaptureIntakePreview | null;
    consolidation?: CaptureConsolidationPreview | null;
    acknowledgement?: CaptureDeliveryAcknowledgementPreview | null;
    assignment?: CaptureAssignmentPreview | null;
    resolution?: CaptureResolutionPreview | null;
    resolutionSelections?: CaptureResolutionSelection[];
    portfolioMaintenance?: PortfolioMaintenancePreview | null;
    returnFocusTarget?: HTMLElement | null;
    busy: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  let cancelButton = $state<HTMLButtonElement | null>(null);
  const capturedFocusTarget = untrack(() =>
    returnFocusTarget ?? (typeof document !== 'undefined'
      && document.activeElement instanceof HTMLElement
      && document.activeElement !== document.body
        ? document.activeElement
        : null)
  );
  let focusRestored = false;

  function focusCancel(event: Event): void {
    event.preventDefault();
    void tick().then(() => cancelButton?.focus());
  }

  function restoreFocus(event?: Event): void {
    event?.preventDefault();
    if (focusRestored) return;
    focusRestored = true;
    const applyFocus = () => {
      if (capturedFocusTarget?.isConnected) capturedFocusTarget.focus();
    };
    if (typeof window !== 'undefined' && 'requestAnimationFrame' in window) {
      window.requestAnimationFrame(applyFocus);
    } else {
      queueMicrotask(applyFocus);
    }
  }

  onDestroy(restoreFocus);

  function previewTitle(): string {
    if (preview.migrationRollback) return i18n.t('dialog.projectMigrationRollbackTitle');
    if (portfolioMaintenance) {
      return i18n.t(`dialog.portfolio.${portfolioMaintenance.operation}.title`);
    }
    if (!preview.migration) return preview.title;
    return i18n.t(
      preview.migration.mode === 'copy'
        ? 'dialog.projectMigrationTitle'
        : 'dialog.projectMigrationRecoveryTitle'
    );
  }

  function previewSummary(): string {
    if (preview.migrationRollback) {
      const number = new Intl.NumberFormat(i18n.locale);
      return i18n.t('dialog.projectMigrationRollbackSummary', {
        matched: number.format(
          preview.migrationRollback.reconciliation.matchedArtifactCount
        ),
        drifted: number.format(
          preview.migrationRollback.reconciliation.driftedArtifactCount
        ),
        gaps: number.format(
          preview.migrationRollback.reconciliation.continuityGapCount
        )
      });
    }
    if (portfolioMaintenance) {
      return i18n.t(`dialog.portfolio.${portfolioMaintenance.operation}.summary`);
    }
    if (!preview.migration) return preview.summary;
    const number = new Intl.NumberFormat(i18n.locale);
    return i18n.t(
      preview.migration.mode === 'copy'
        ? 'dialog.projectMigrationSummary'
        : 'dialog.projectMigrationRecoverySummary',
      {
        files: number.format(preview.migration.copiedFileCount),
        bytes: number.format(preview.migration.copiedBytes),
        excluded: number.format(preview.migration.excludedEntryCount),
        passes: number.format(preview.migration.graphRebuildPasses)
      }
    );
  }

  function selectedDisposition(itemId: string): string {
    return resolutionSelections.find((selection) => selection.itemId === itemId)?.disposition
      ?? i18n.t('dialog.selectionMissing');
  }

  function handleOverlayPointerDown(event: PointerEvent): void {
    if (!busy && event.target === event.currentTarget) onCancel();
  }

  function handleEscapeKeydown(event: KeyboardEvent): void {
    event.preventDefault();
    if (!busy) onCancel();
  }

</script>

<AlertDialog.Root open>
    <AlertDialog.Content
      class={cn(
        'content',
        'qiongli-confirmation-content'
      )}
      overlayProps={{
        class: 'overlay qiongli-confirmation-overlay',
        onpointerdown: handleOverlayPointerDown
      }}
      aria-busy={busy}
      onOpenAutoFocus={focusCancel}
      onCloseAutoFocus={restoreFocus}
      onEscapeKeydown={handleEscapeKeydown}
    >
      <div class="dialog-heading">
        <div class="icon"><ShieldCheck size={22} aria-hidden="true" /></div>
        <div>
          <AlertDialog.Title class="title qiongli-confirmation-title">{previewTitle()}</AlertDialog.Title>
          <AlertDialog.Description class="description qiongli-confirmation-description">{previewSummary()}</AlertDialog.Description>
        </div>
        <Button
          class="close"
          variant="ghost"
          size="icon"
          type="button"
          aria-label={i18n.t('dialog.cancelAria')}
          disabled={busy}
          onclick={onCancel}
        >
          <X size={18} aria-hidden="true" />
        </Button>
      </div>

      {#if preview.displayTarget}
        <div class="detail-row"><span>{i18n.t('common.destination')}</span><code>{preview.displayTarget}</code></div>
      {/if}
      {#if preview.planDigestSha256}
        <div class="detail-row"><span>{i18n.t('common.planDigest')}</span><code>{preview.planDigestSha256.slice(0, 16)}…</code></div>
      {/if}

      {#if intake}
        <section class="domain-review" aria-label={i18n.t('dialog.intakeReview')}>
          <div><span>{i18n.t('dialog.disposition')}</span><strong>{i18n.label(intake.disposition)}</strong></div>
          <div><span>{i18n.t('dialog.changes')}</span><strong>{intake.changeCount}</strong></div>
          <div><span>{i18n.t('dialog.decisions')}</span><strong>{intake.decisionCount}</strong></div>
          <div><span>{i18n.t('dialog.evidence')}</span><strong>{intake.evidenceCount}</strong></div>
        </section>
      {/if}

      {#if consolidation}
        <section
          class="consolidation-review"
          aria-label={i18n.t('dialog.consolidationReview')}
        >
          <div class="outcome"><span>{i18n.t('dialog.reviewOutcome')}</span><strong>{i18n.label(consolidation.outcome)}</strong></div>
          {#if consolidation.artifactDeltas.length > 0}
            <h3>{i18n.t('dialog.artifactDeltas')}</h3>
            <ul>
              {#each consolidation.artifactDeltas as delta}
                <li>
                  <code>{delta.relativePath}</code> · {i18n.label(delta.effect)} ·
                  {i18n.t('common.bytes', { count: delta.previousBytes })} →
                  {i18n.t('common.bytes', { count: delta.nextBytes })}
                </li>
              {/each}
            </ul>
          {/if}
          {#if consolidation.conflicts.length > 0}
            <h3>{i18n.t('dialog.conflicts')}</h3>
            <ul class="conflicts">
              {#each consolidation.conflicts as conflict}
                <li><strong>{conflict.kind}</strong><span>{conflict.resolution}</span></li>
              {/each}
            </ul>
          {/if}
        </section>
      {/if}

      {#if acknowledgement}
        <section
          class="continuity-review"
          aria-label={i18n.t('dialog.acknowledgementReview')}
        >
          <div class="continuity-facts">
            <div>
              <span>{i18n.t('dialog.deliveryGeneration')}</span>
              <strong>{acknowledgement.expectedGeneration}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.expectedRevision')}</span>
              <strong>r{acknowledgement.expectedProjectRevision}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.resultingRevision')}</span>
              <strong>r{acknowledgement.resultingProjectRevision}</strong>
            </div>
          </div>
          <p>{i18n.t('dialog.acknowledgementDetail')}</p>
        </section>
      {/if}

      {#if assignment}
        <section
          class="continuity-review"
          aria-label={i18n.t('dialog.assignmentReview')}
        >
          <div class="continuity-facts">
            <div>
              <span>{i18n.t('dialog.assignmentOutcome')}</span>
              <strong>{i18n.label(assignment.outcome)}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.bindingEffect')}</span>
              <strong>{i18n.label(assignment.bindingEffect)}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.expectedRevision')}</span>
              <strong>r{assignment.expectedProjectRevision}</strong>
            </div>
          </div>
          <p>{assignment.explanation}</p>
          {#if assignment.resolutionRequired}
            <p class="attention-note">{i18n.t('dialog.resolutionRequired')}</p>
          {/if}
        </section>
      {/if}

      {#if resolution}
        <section
          class="resolution-review"
          aria-label={i18n.t('dialog.resolutionReview')}
        >
          <div class="continuity-facts">
            <div>
              <span>{i18n.t('dialog.itemsReviewed')}</span>
              <strong>{resolution.items.length}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.expectedRevision')}</span>
              <strong>r{resolution.expectedProjectRevision}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.resultingRevision')}</span>
              <strong>r{resolution.nextProjectRevision}</strong>
            </div>
          </div>
          <ol class="resolution-items">
            {#each resolution.items as item}
              <li>
                <div>
                  <strong>{i18n.label(item.kind)}</strong>
                  <span>{i18n.label(item.counterpartState)}</span>
                </div>
                <p>{item.sourceSummary}</p>
                {#if item.currentSummary}
                  <p class="current-summary">{item.currentSummary}</p>
                {/if}
                <small>{item.explanation}</small>
                <span class="selected-disposition">
                  {i18n.t('dialog.selectedDisposition')}:
                  <strong>{i18n.label(selectedDisposition(item.itemId))}</strong>
                </span>
              </li>
            {/each}
          </ol>
        </section>
      {/if}

      {#if portfolioMaintenance}
        <section
          class="continuity-review"
          aria-label={i18n.t('dialog.portfolioReview')}
        >
          <div class="continuity-facts">
            <div>
              <span>{i18n.t('portfolio.libraryRevision')}</span>
              <strong>r{portfolioMaintenance.expectedLibraryRevision}</strong>
            </div>
            <div>
              <span>{i18n.t('portfolio.catalogGeneration')}</span>
              <strong>{portfolioMaintenance.expectedCatalogGeneration ?? i18n.t('common.none')}</strong>
            </div>
            <div>
              <span>{i18n.t('portfolio.contributions')}</span>
              <strong>{portfolioMaintenance.currentContributionCount}</strong>
            </div>
          </div>
          <p>{i18n.dynamic(portfolioMaintenance.explanation)}</p>
          <p class="attention-note">{i18n.t('dialog.portfolioCanonicalRetained')}</p>
        </section>
      {/if}

      {#if preview.migrationRollback}
        <section class="migration-rollback-review" aria-label={i18n.t('dialog.rollbackReconciliation')}>
          <div class="rollback-facts">
            <div>
              <span>{i18n.t('dialog.reconciliation')}</span>
              <strong>{i18n.label(preview.migrationRollback.reconciliation.status)}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.registration')}</span>
              <strong>{i18n.label(preview.migrationRollback.registrationState)}</strong>
            </div>
            <div>
              <span>{i18n.t('dialog.migrationMarker')}</span>
              <strong>{i18n.label(preview.migrationRollback.markerState)}</strong>
            </div>
          </div>
          <p>{i18n.t('dialog.rollbackSourceRetained')}</p>
          <ul>
            {#each preview.migrationRollback.reconciliation.artifacts.slice(0, 12) as artifact}
              <li>
                <span>{i18n.label(artifact.category)}</span>
                <code>{artifact.relativePath}</code>
                <strong>{i18n.label(artifact.state)}</strong>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if preview.approvalsRequired.length}
        <section class="approvals">
          <h3>{i18n.t('dialog.approvals')}</h3>
          <ul>
            {#each preview.approvalsRequired as approval}
              <li>{i18n.label(approval)}</li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if preview.blockedReason}
        <p class="blocked" role="alert">{i18n.t('dialog.blocked')} {i18n.reason(preview.blockedReason)}</p>
      {/if}

      {#if busy}
        <section class="execution" role="status" aria-live="polite" aria-atomic="true">
          <div class="execution-heading">
            <LoaderCircle class="execution-spin" size={18} aria-hidden="true" />
            <div>
              <strong>{i18n.t('dialog.executionTitle')}</strong>
              <span>{i18n.t('dialog.executionApplying')}</span>
            </div>
          </div>
          <ol>
            <li class="complete"><Check size={14} aria-hidden="true" />{i18n.t('dialog.executionReviewed')}</li>
            <li class="active"><LoaderCircle class="execution-spin" size={14} aria-hidden="true" />{i18n.t('dialog.executionWriting')}</li>
            <li><span class="step-dot" aria-hidden="true"></span>{i18n.t('dialog.executionRefresh')}</li>
          </ol>
          <div class="execution-target">
            <MapPin size={14} aria-hidden="true" />
            <span>{i18n.t('dialog.executionTarget')}</span>
            <code>{preview.displayTarget ?? i18n.t('dialog.executionManagedTargets')}</code>
          </div>
        </section>
      {/if}

      <div class="footer">
        <Button
          bind:ref={cancelButton}
          variant="outline"
          disabled={busy}
          onclick={onCancel}
        >{i18n.t('common.cancel')}</Button>
        <Button disabled={busy || !preview.canConfirm} onclick={onConfirm}>
          {busy ? i18n.t('dialog.applying') : i18n.t('dialog.confirm')}
        </Button>
      </div>
    </AlertDialog.Content>
</AlertDialog.Root>

<style>
  :global([data-slot='alert-dialog-overlay']) {
    position: fixed;
    inset: 0;
    z-index: var(--z-dialog-scrim);
    background: var(--color-scrim);
  }

  :global(.qiongli-confirmation-content) {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: var(--z-dialog);
    width: min(580px, calc(100vw - 48px));
    max-height: calc(100vh - 48px);
    overflow-x: hidden;
    overflow-y: auto;
    transform: translate(-50%, -50%);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
    padding: 22px;
    color: var(--color-ink);
    background: var(--color-surface);
    box-shadow: var(--shadow-overlay);
  }

  .dialog-heading {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: 13px;
  }

  .icon {
    display: grid;
    width: 42px;
    height: 42px;
    place-items: center;
    border-radius: 50%;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  :global(.qiongli-confirmation-title) {
    margin: 0;
    color: var(--color-ink-strong);
    font-size: 19px;
    font-weight: 680;
    line-height: 1.3;
  }

  :global(.qiongli-confirmation-description) {
    margin-bottom: 0;
    margin-top: 6px;
    color: var(--color-muted);
    font-size: 14px;
    line-height: 1.55;
  }

  :global(.qiongli-confirmation-content .close) {
    display: inline-flex;
    width: 44px;
    min-height: 44px;
    align-items: center;
    justify-content: center;
    border: 0;
    border-radius: var(--radius-control);
    padding: 8px;
    color: var(--color-muted);
    background: transparent;
  }

  .detail-row {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: 12px;
    margin-top: 18px;
    border-top: 1px solid var(--color-border);
    padding-top: 14px;
    color: var(--color-muted);
    font-size: 13px;
  }

  code {
    overflow-wrap: anywhere;
    color: var(--color-ink);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }

  .approvals {
    margin-top: 18px;
    border-radius: var(--radius-inset);
    padding: 14px 16px;
    background: var(--color-surface-subtle);
  }

  .execution {
    margin-top: 18px;
    border: 1px solid var(--color-accent-border);
    border-radius: var(--radius-inset);
    padding: 13px 14px;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .execution-heading,
  .execution-target,
  .execution li {
    display: flex;
    align-items: center;
  }

  .execution-heading { gap: 9px; }
  .execution-heading strong,
  .execution-heading span { display: block; }
  .execution-heading strong { font-size: 11px; }
  .execution-heading span { margin-top: 2px; font-size: var(--font-size-label); }
  .execution ol {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
    margin: 11px 0 0;
    padding: 0;
    list-style: none;
  }
  .execution li {
    min-width: 0;
    gap: 6px;
    border-radius: var(--radius-control-inner);
    padding: 6px 7px;
    color: var(--color-muted);
    background: var(--color-surface-subtle);
    font-size: var(--font-size-micro);
    font-weight: 750;
    white-space: nowrap;
  }
  .execution li.active { color: var(--color-accent-strong); background: var(--color-control); }
  .execution li.complete { color: var(--color-success); }
  .step-dot { width: 7px; height: 7px; flex: none; border-radius: 50%; background: var(--color-border-strong); }
  .execution-target {
    min-width: 0;
    gap: 6px;
    margin-top: 9px;
    border-top: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent);
    padding-top: 8px;
    font-size: var(--font-size-micro);
  }
  .execution-target code {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.execution-spin) { flex: none; animation: execution-spin 900ms linear infinite; }
  @keyframes execution-spin { to { transform: rotate(360deg); } }

  .domain-review {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    margin-top: 18px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    background: var(--color-border);
  }

  .domain-review div,
  .outcome {
    border: 0;
    border-radius: 0;
    padding: 10px;
    background: var(--color-surface-subtle);
  }

  .domain-review span,
  .domain-review strong,
  .outcome span,
  .outcome strong {
    display: block;
  }

  .domain-review span,
  .outcome span {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
    text-transform: uppercase;
  }

  .domain-review strong,
  .outcome strong {
    margin-top: 5px;
    color: var(--color-ink-strong);
    font-size: 13px;
  }

  .consolidation-review {
    margin-top: 18px;
    border-top: 1px solid var(--color-border);
    padding-top: 14px;
  }

  .continuity-review,
  .resolution-review {
    margin-top: 18px;
    border-top: 1px solid var(--color-border);
    padding-top: 14px;
  }

  .continuity-review > p {
    margin: 11px 0 0;
    color: var(--color-muted);
    font-size: 12px;
    line-height: 1.55;
  }

  .continuity-facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    background: var(--color-border);
  }

  .continuity-facts div {
    border: 0;
    border-radius: 0;
    padding: 10px;
    background: var(--color-surface-subtle);
  }

  .continuity-facts span,
  .continuity-facts strong {
    display: block;
  }

  .continuity-facts span {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
    text-transform: uppercase;
  }

  .continuity-facts strong {
    margin-top: 5px;
    color: var(--color-ink-strong);
    font-size: 13px;
  }

  .attention-note {
    border-left: 3px solid var(--color-warning);
    padding-left: 10px;
    color: var(--color-warning-strong) !important;
  }

  .resolution-items {
    display: grid;
    gap: 9px;
    margin: 12px 0 0;
    padding: 0;
    list-style: none;
  }

  .resolution-items li {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    padding: 11px;
    background: var(--color-surface-subtle);
  }

  .resolution-items li > div,
  .selected-disposition {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .resolution-items p {
    margin: 8px 0 0;
    color: var(--color-ink);
    font-size: 12px;
  }

  .resolution-items .current-summary,
  .resolution-items small {
    display: block;
    margin-top: 6px;
    color: var(--color-muted);
    font-size: 11px;
  }

  .selected-disposition {
    margin-top: 9px;
    border-top: 1px solid var(--color-border);
    padding-top: 8px;
    color: var(--color-muted);
    font-size: 11px;
  }

  .migration-rollback-review {
    margin-top: 18px;
    border-top: 1px solid var(--color-border);
    padding-top: 14px;
  }

  .rollback-facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    background: var(--color-border);
  }

  .rollback-facts div {
    border: 0;
    border-radius: 0;
    padding: 10px;
    background: var(--color-surface-subtle);
  }

  .rollback-facts span,
  .rollback-facts strong {
    display: block;
  }

  .rollback-facts span {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 750;
    text-transform: uppercase;
  }

  .rollback-facts strong {
    margin-top: 5px;
    color: var(--color-ink-strong);
    font-size: 13px;
  }

  .migration-rollback-review p {
    color: var(--color-muted);
    font-size: 12px;
  }

  .migration-rollback-review ul {
    display: grid;
    gap: 6px;
    margin: 10px 0 0;
    padding: 0;
    list-style: none;
  }

  .migration-rollback-review li {
    display: grid;
    grid-template-columns: 100px minmax(0, 1fr) auto;
    align-items: center;
    gap: 8px;
    font-size: 11px;
  }

  .consolidation-review h3 {
    margin-top: 14px;
  }

  .conflicts {
    list-style: none;
    padding-left: 0;
  }

  .conflicts li {
    display: grid;
    gap: 3px;
    border-left: 3px solid var(--color-warning);
    padding-left: 10px;
  }

  .conflicts strong {
    color: var(--color-ink);
  }

  @media (max-width: 540px) {
    .domain-review { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .continuity-facts { grid-template-columns: 1fr; }
    .execution ol { grid-template-columns: 1fr; }
  }

  @media (max-width: 420px) {
    :global(.content) {
      width: calc(100vw - 24px);
      max-height: calc(100vh - 24px);
      border-radius: var(--radius-dialog);
      padding: 16px;
    }
    .dialog-heading { grid-template-columns: minmax(0, 1fr) auto; }
    .dialog-heading .icon { display: none; }
    .detail-row { grid-template-columns: 1fr; gap: 5px; }
    .rollback-facts { grid-template-columns: 1fr; }
    .migration-rollback-review li {
      grid-template-columns: 1fr;
      align-items: start;
    }
    .footer { align-items: stretch; flex-direction: column-reverse; }
    .footer :global(button) { width: 100%; }
  }

  h3 {
    margin: 0;
    font-size: 13px;
  }

  ul {
    margin: 9px 0 0;
    padding-left: 20px;
    color: var(--color-muted);
    font-size: 13px;
    line-height: 1.7;
  }

  .blocked {
    margin: 18px 0 0;
    border: 1px solid var(--color-warning-border);
    border-radius: var(--radius-inset);
    padding: 12px;
    color: var(--color-warning-strong);
    background: var(--color-warning-soft);
    font-size: 13px;
    line-height: 1.5;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 22px;
  }
</style>
