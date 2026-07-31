<script lang="ts">
  import type { ProjectArtifactView } from '@qiongli/app-api';
  import { FileText, X } from '@lucide/svelte';
  import { onMount, tick } from 'svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';

  let {
    artifact,
    onClose,
    returnFocusTarget = null
  }: {
    artifact: ProjectArtifactView;
    onClose: () => void;
    returnFocusTarget?: HTMLElement | null;
  } = $props();

  let closeButton = $state<HTMLButtonElement | null>(null);
  let previousFocus: HTMLElement | null = null;
  let lines = $derived(artifact.content.split('\n'));

  onMount(() => {
    previousFocus = returnFocusTarget
      ?? (document.activeElement instanceof HTMLElement ? document.activeElement : null);
    closeButton?.focus();
  });

  function sourceLine(index: number): number {
    return artifact.startLine + index;
  }

  async function closeViewer(): Promise<void> {
    onClose();
    await tick();
    if (previousFocus?.isConnected) previousFocus.focus();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Escape') return;
    event.preventDefault();
    void closeViewer();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<section class="artifact-viewer" aria-labelledby="artifact-viewer-title">
  <header>
    <div class="title">
      <FileText size={17} aria-hidden="true" />
      <div>
        <p>{i18n.t('artifactViewer.title')}</p>
        <h4 id="artifact-viewer-title">{artifact.artifactPath}</h4>
      </div>
    </div>
    <Button
      class="icon-button"
      variant="ghost"
      size="icon-sm"
      aria-label={i18n.t('artifactViewer.close')}
      title={i18n.t('artifactViewer.close')}
      bind:ref={closeButton}
      onclick={() => void closeViewer()}
    >
      <X size={16} aria-hidden="true" />
    </Button>
  </header>

  <div class="provenance">
    <span>{i18n.label(artifact.format)}</span>
    <span>{i18n.t('artifactViewer.lines', { start: artifact.startLine, end: artifact.endLine })}</span>
    <span>{i18n.t('artifactViewer.bytes', {
      shown: artifact.contentSizeBytes,
      total: artifact.sourceSizeBytes
    })}</span>
  </div>

  {#if artifact.sourceAnchor}
    <p class:missing={!artifact.anchorMatched} class="anchor-status">
      {artifact.anchorMatched && artifact.anchorLine
        ? i18n.t('artifactViewer.anchorFound', { line: artifact.anchorLine })
        : i18n.t('artifactViewer.anchorMissing')}
    </p>
  {/if}

  <div
    class="content"
    role="textbox"
    aria-label={i18n.t('artifactViewer.content')}
    aria-multiline="true"
    aria-readonly="true"
    tabindex="0"
  >
    {#if artifact.truncatedBefore}
      <p class="truncation">{i18n.t('artifactViewer.truncatedBefore')}</p>
    {/if}
    <ol start={artifact.startLine}>
      {#each lines as line, index (`${sourceLine(index)}:${index}`)}
        <li class:anchor={artifact.anchorLine === sourceLine(index)}>
          <code>{line.length > 0 ? line : '\u00a0'}</code>
        </li>
      {/each}
    </ol>
    {#if artifact.truncatedAfter}
      <p class="truncation">{i18n.t('artifactViewer.truncatedAfter')}</p>
    {/if}
  </div>

  <footer>
    <code>sha256:{artifact.contentDigest.slice(0, 12)}…</code>
    <span>rev {artifact.projectRevision}</span>
  </footer>
</section>

<style>
  .artifact-viewer {
    min-width: 0;
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    border-radius: var(--radius-card);
    background: var(--color-surface);
  }
  header, .title, .provenance, footer { display: flex; align-items: center; }
  header { justify-content: space-between; gap: 12px; padding: 11px 12px; border-bottom: 1px solid var(--color-border); }
  .title { min-width: 0; gap: 9px; color: var(--color-accent-strong); }
  .title div { min-width: 0; }
  .title p { margin: 0 0 2px; color: var(--color-muted); font-size: var(--font-size-label); font-weight: 760; text-transform: uppercase; }
  h4 { overflow: hidden; margin: 0; color: var(--color-text); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
  :global(.icon-button) { flex: 0 0 auto; color: var(--color-muted); }
  .provenance { flex-wrap: wrap; gap: 6px; padding: 9px 12px 0; }
  .provenance span { border-radius: 999px; padding: 2px 7px; color: var(--color-muted); background: var(--color-surface-muted); font-size: 10px; font-weight: 760; white-space: nowrap; }
  .anchor-status { margin: 8px 12px 0; color: var(--color-accent-strong); font-size: 10px; font-weight: 680; line-height: 1.45; }
  .anchor-status.missing { color: var(--color-warning-strong); }
  .content { max-height: min(52vh, 520px); margin-top: 9px; overflow-x: hidden; overflow-y: auto; border-block: 1px solid var(--color-border); background: var(--color-code-background, var(--color-surface-muted)); outline: none; }
  .content:focus-visible { box-shadow: inset 0 0 0 2px var(--color-focus); }
  ol { min-width: 0; margin: 0; padding: 10px 12px 10px 48px; }
  li { min-height: 18px; padding: 0 12px 0 7px; color: var(--color-muted); font-size: 10px; line-height: 1.7; overflow-wrap: anywhere; white-space: pre-wrap; }
  li::marker { color: color-mix(in srgb, var(--color-muted) 65%, transparent); font-variant-numeric: tabular-nums; }
  li.anchor { border-radius: 4px; color: var(--color-text); background: color-mix(in srgb, var(--color-accent) 14%, transparent); }
  code { font-family: var(--font-family-mono, ui-monospace, SFMono-Regular, Menlo, monospace); }
  .truncation { margin: 0; padding: 7px 12px; color: var(--color-muted); background: var(--color-surface); font-size: 10px; font-style: italic; }
  footer { justify-content: space-between; gap: 10px; padding: 8px 12px; color: var(--color-muted); font-size: 10px; }
  footer code { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  @media (max-width: 520px) {
    .content { max-height: 46vh; }
    .provenance { flex-wrap: wrap; }
  }
</style>
