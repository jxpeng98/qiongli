<script lang="ts">
  import type {
    ContinuityOperationProgress,
    PortfolioDoctor,
    PortfolioMaintenanceResult,
    StatusCode
  } from '@qiongli/app-api';
  import { AlertTriangle, CheckCircle2, LoaderCircle, ShieldCheck, XCircle } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';

  let {
    doctor,
    doctorState,
    progress,
    result,
    busy,
    onCancel
  }: {
    doctor: PortfolioDoctor | null;
    doctorState: 'idle' | 'loading' | 'ready' | 'failed';
    progress: ContinuityOperationProgress | null;
    result: PortfolioMaintenanceResult | null;
    busy: boolean;
    onCancel: (operationId: string) => void;
  } = $props();

  let progressPercent = $derived(
    progress ? Math.round(progress.completedUnits / progress.totalUnits * 100) : 0
  );
  let terminalTone: StatusCode = $derived(
    progress?.phase === 'cancelled'
      ? 'missing'
      : progress?.phase === 'recovery-required' || progress?.phase === 'failed'
        ? 'recovery-required'
        : 'busy'
  );
  let progressAnnouncement = $derived.by(() => {
    if (result) {
      return i18n.t('portfolio.resultAnnouncement', {
        operation: i18n.label(result.operation)
      });
    }
    if (!progress) return '';
    const detail = i18n.reason(progress.reasonCode);
    if (progress.phase !== 'running') {
      return i18n.t('portfolio.operationAnnouncement', {
        operation: i18n.label(progress.operation),
        detail
      });
    }
    const boundedPercent = Math.min(100, Math.floor(progressPercent / 25) * 25);
    return i18n.t('portfolio.progressAnnouncement', {
      operation: i18n.label(progress.operation),
      percent: boundedPercent,
      detail
    });
  });
</script>

{#if doctorState !== 'idle' || progress || result}
  <Card.Root class="maintenance" aria-labelledby="portfolio-maintenance-title">
    <header>
      <div>
        <p class="eyebrow">{i18n.t('portfolio.maintenanceEyebrow')}</p>
        <h2 id="portfolio-maintenance-title">{i18n.t('portfolio.maintenanceTitle')}</h2>
      </div>
    </header>

    {#if doctorState === 'loading'}
      <div class="message" role="status" aria-live="polite" aria-atomic="true">
        <LoaderCircle class="spin" size={18} aria-hidden="true" />
        <span>{i18n.t('portfolio.doctorLoading')}</span>
      </div>
    {:else if doctorState === 'failed'}
      <div class="message danger" role="alert">
        <AlertTriangle size={18} aria-hidden="true" />
        <span>{i18n.t('portfolio.doctorFailed')}</span>
      </div>
    {:else if doctor}
      <article class="doctor" aria-label={i18n.t('portfolio.doctorResult')}>
        <div>
          <ShieldCheck size={18} aria-hidden="true" />
          <div>
            <strong>{i18n.t('portfolio.doctorTitle')}</strong>
            <span>{i18n.t('portfolio.doctorDetail', {
              count: doctor.contributionCount,
              revision: doctor.libraryRevision
            })}</span>
          </div>
        </div>
        <StatusBadge
          status={doctor.status === 'equivalent'
            ? 'ready'
            : doctor.status === 'divergent' ? 'conflict' : 'missing'}
          label={i18n.t(`portfolio.doctor.${doctor.status}`)}
        />
      </article>
    {/if}

    {#if progressAnnouncement}
      <p class="sr-only operation-announcement" role="status" aria-live="polite" aria-atomic="true">
        {progressAnnouncement}
      </p>
    {/if}

    {#if progress}
      <article class="operation">
        <div class="operation-heading">
          <div>
            {#if progress.phase === 'queued' || progress.phase === 'running'}
              <LoaderCircle class="spin" size={18} aria-hidden="true" />
            {:else if progress.phase === 'cancelled'}
              <XCircle size={18} aria-hidden="true" />
            {:else}
              <AlertTriangle size={18} aria-hidden="true" />
            {/if}
            <div>
              <strong>{i18n.label(progress.operation)}</strong>
              <span>{i18n.reason(progress.reasonCode)}</span>
            </div>
          </div>
          <StatusBadge status={terminalTone} label={i18n.label(progress.phase)} />
        </div>
        <Progress
          max={progress.totalUnits}
          value={progress.completedUnits}
          aria-label={i18n.t('portfolio.operationProgress')}
          aria-valuetext={`${progressPercent}% · ${i18n.reason(progress.reasonCode)}`}
        />
        <div class="progress-detail">
          <span>{progress.completedUnits}/{progress.totalUnits}</span>
          <strong>{progressPercent}%</strong>
        </div>
        {#if progress.cancellable}
          <Button
            variant="outline"
            disabled={busy}
            onclick={() => onCancel(progress.operationId)}
          >
            {i18n.t('portfolio.cancelMaintenance')}
          </Button>
        {/if}
      </article>
    {/if}

    {#if result}
      <article class="result" aria-label={i18n.t('portfolio.maintenanceResult')}>
        <CheckCircle2 size={19} aria-hidden="true" />
        <div>
          <strong>{i18n.t('portfolio.completedOperation', {
            operation: i18n.label(result.operation)
          })}</strong>
          <span>{i18n.t('portfolio.resultCounts', {
            rebuilt: result.rebuiltProjectCount,
            reused: result.reusedProjectCount,
            removedProjects: result.removedProjectCount,
            removedContributions: result.removedContributionCount
          })}</span>
          <span>{i18n.t('portfolio.canonicalRetained')}</span>
        </div>
      </article>
    {/if}
  </Card.Root>
{/if}

<style>
  :global(.maintenance) { min-width: 0; padding: var(--ui-panel-padding); }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .message, .doctor, .operation, .result {
    margin-top: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    padding: 12px;
    background: var(--color-surface-subtle);
  }
  .message, .doctor, .doctor > div, .operation-heading, .operation-heading > div, .result {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .message { color: var(--color-muted); font-size: 12px; }
  .message.danger { color: var(--color-danger); background: var(--color-danger-soft); }
  .doctor, .operation-heading { justify-content: space-between; }
  .doctor strong, .doctor span, .operation strong, .operation span, .result strong, .result span {
    display: block;
  }
  .doctor strong, .operation strong, .result strong { color: var(--color-ink-strong); font-size: 12px; }
  .doctor span, .operation span, .result span {
    margin-top: 3px;
    color: var(--color-muted);
    font-size: 11px;
    line-height: 1.45;
  }
  .operation-heading > div { min-width: 0; }
  .operation :global([data-slot='progress']) { margin-top: 12px; }
  .progress-detail { display: flex; justify-content: space-between; margin-top: 5px; }
  .progress-detail span, .progress-detail strong { font-size: 10px; }
  .operation > :global([data-slot='button']) { margin-top: 10px; }
  .result { align-items: flex-start; color: var(--color-success); background: var(--color-success-soft); }
  @media (max-width: 520px) {
    .doctor, .operation-heading { align-items: flex-start; flex-direction: column; }
  }
</style>
