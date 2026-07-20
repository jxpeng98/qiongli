<script lang="ts">
  import { ShieldCheck, X } from '@lucide/svelte';
  import { Dialog } from 'bits-ui';

  import type {
    CaptureConsolidationPreview,
    CaptureIntakePreview,
    OperationPreview
  } from '@qiongli/app-api';

  let {
    preview,
    intake = null,
    consolidation = null,
    busy,
    onConfirm,
    onCancel
  }: {
    preview: OperationPreview;
    intake?: CaptureIntakePreview | null;
    consolidation?: CaptureConsolidationPreview | null;
    busy: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<Dialog.Root open onOpenChange={(open) => !open && onCancel()}>
  <Dialog.Portal>
    <Dialog.Overlay class="overlay" />
    <Dialog.Content class="content">
      <div class="dialog-heading">
        <div class="icon"><ShieldCheck size={22} aria-hidden="true" /></div>
        <div>
          <Dialog.Title class="title">{preview.title}</Dialog.Title>
          <Dialog.Description class="description">{preview.summary}</Dialog.Description>
        </div>
        <button class="close" type="button" aria-label="Cancel operation" onclick={onCancel}>
          <X size={18} aria-hidden="true" />
        </button>
      </div>

      {#if preview.displayTarget}
        <div class="detail-row"><span>Destination</span><code>{preview.displayTarget}</code></div>
      {/if}
      {#if preview.planDigestSha256}
        <div class="detail-row"><span>Plan digest</span><code>{preview.planDigestSha256.slice(0, 16)}…</code></div>
      {/if}

      {#if intake}
        <section class="domain-review" aria-label="Capture intake review">
          <div><span>Disposition</span><strong>{intake.disposition}</strong></div>
          <div><span>Changes</span><strong>{intake.changeCount}</strong></div>
          <div><span>Decisions</span><strong>{intake.decisionCount}</strong></div>
          <div><span>Evidence</span><strong>{intake.evidenceCount}</strong></div>
        </section>
      {/if}

      {#if consolidation}
        <section class="consolidation-review" aria-label="Academic consolidation review">
          <div class="outcome"><span>Review outcome</span><strong>{consolidation.outcome}</strong></div>
          {#if consolidation.artifactDeltas.length > 0}
            <h3>Canonical artifact deltas</h3>
            <ul>
              {#each consolidation.artifactDeltas as delta}
                <li><code>{delta.relativePath}</code> · {delta.effect} · {delta.previousBytes} → {delta.nextBytes} bytes</li>
              {/each}
            </ul>
          {/if}
          {#if consolidation.conflicts.length > 0}
            <h3>Conflicts requiring resolution</h3>
            <ul class="conflicts">
              {#each consolidation.conflicts as conflict}
                <li><strong>{conflict.kind}</strong><span>{conflict.resolution}</span></li>
              {/each}
            </ul>
          {/if}
        </section>
      {/if}

      {#if preview.approvalsRequired.length}
        <section class="approvals">
          <h3>Changes requiring approval</h3>
          <ul>
            {#each preview.approvalsRequired as approval}
              <li>{approval}</li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if preview.blockedReason}
        <p class="blocked" role="alert">This build cannot apply the plan: <code>{preview.blockedReason}</code></p>
      {/if}

      <div class="footer">
        <button class="button-secondary" type="button" disabled={busy} onclick={onCancel}>Cancel</button>
        <button class="button-primary" type="button" disabled={busy || !preview.canConfirm} onclick={onConfirm}>
          {busy ? 'Applying…' : 'Confirm changes'}
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  :global(.overlay) {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: rgb(2 6 23 / 0.48);
    backdrop-filter: blur(3px);
  }

  :global(.content) {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: 51;
    width: min(580px, calc(100vw - 48px));
    max-height: calc(100vh - 48px);
    overflow: auto;
    transform: translate(-50%, -50%);
    border: 1px solid var(--color-border-strong);
    border-radius: 18px;
    padding: 22px;
    color: var(--color-ink);
    background: white;
    box-shadow: 0 28px 80px rgb(2 6 23 / 0.28);
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
    border-radius: 12px;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  :global(.title) {
    color: var(--color-ink-strong);
    font-size: 19px;
    font-weight: 780;
    line-height: 1.3;
  }

  :global(.description) {
    margin-top: 6px;
    color: var(--color-muted);
    font-size: 14px;
    line-height: 1.55;
  }

  .close {
    display: inline-flex;
    border: 0;
    border-radius: 8px;
    padding: 6px;
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
    border-radius: 12px;
    padding: 14px 16px;
    background: var(--color-surface-subtle);
  }

  .domain-review {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
    margin-top: 18px;
  }

  .domain-review div,
  .outcome {
    border: 1px solid var(--color-border);
    border-radius: 10px;
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
    border: 1px solid #fde68a;
    border-radius: 10px;
    padding: 12px;
    color: #854d0e;
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
