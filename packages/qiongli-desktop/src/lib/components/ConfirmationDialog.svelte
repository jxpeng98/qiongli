<script lang="ts">
  import { ShieldCheck, X } from '@lucide/svelte';
  import { Dialog } from 'bits-ui';

  import type { OperationPreview } from '@qiongli/app-api';

  let { preview, busy, onConfirm, onCancel }: {
    preview: OperationPreview;
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
