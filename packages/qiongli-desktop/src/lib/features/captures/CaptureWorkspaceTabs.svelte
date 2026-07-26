<script lang="ts">
  import { AlertTriangle, Inbox, Radar, Send } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import {
    captureWorkspaceModes,
    type CaptureWorkspaceMode
  } from '.';

  let {
    mode,
    counts,
    onChange
  }: {
    mode: CaptureWorkspaceMode;
    counts: Record<CaptureWorkspaceMode, number>;
    onChange: (mode: CaptureWorkspaceMode) => void;
  } = $props();

  const icons = {
    inbox: Inbox,
    outbox: Send,
    conflicts: AlertTriangle,
    coverage: Radar
  };

  let buttons = $state<HTMLButtonElement[]>([]);

  function moveFocus(event: KeyboardEvent, index: number): void {
    const key = event.key;
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(key)) return;
    event.preventDefault();
    const nextIndex = key === 'Home'
      ? 0
      : key === 'End'
        ? captureWorkspaceModes.length - 1
        : (index + (key === 'ArrowRight' ? 1 : -1) + captureWorkspaceModes.length)
          % captureWorkspaceModes.length;
    const next = captureWorkspaceModes[nextIndex];
    if (!next) return;
    onChange(next);
    buttons[nextIndex]?.focus();
  }
</script>

<div class="tabs" role="tablist" aria-label={i18n.t('captures.workspaceModes')}>
  {#each captureWorkspaceModes as item, index}
    {@const Icon = icons[item]}
    <button
      bind:this={buttons[index]}
      id={`capture-tab-${item}`}
      type="button"
      role="tab"
      aria-selected={mode === item}
      aria-controls={`capture-panel-${item}`}
      tabindex={mode === item ? 0 : -1}
      onclick={() => onChange(item)}
      onkeydown={(event) => moveFocus(event, index)}
    >
      <Icon size={16} aria-hidden="true" />
      <span>{i18n.t(`captures.mode.${item}`)}</span>
      <strong aria-label={i18n.t('captures.itemCount', { count: counts[item] })}>
        {counts[item]}
      </strong>
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: grid;
    grid-template-columns: repeat(4, minmax(130px, 1fr));
    gap: 7px;
    margin-bottom: 10px;
    overflow-x: auto;
    scrollbar-width: thin;
  }

  button {
    display: grid;
    min-height: 48px;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--color-border);
    border-radius: 11px;
    padding: 9px 11px;
    color: var(--color-muted);
    background: white;
    font: inherit;
    font-size: 12px;
    font-weight: 720;
    text-align: left;
    cursor: pointer;
  }

  button:hover {
    border-color: var(--color-border-strong);
    color: var(--color-ink);
    background: var(--color-surface-subtle);
  }

  button[aria-selected='true'] {
    border-color: #7dd3fc;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
    box-shadow: inset 0 -2px 0 var(--color-accent);
  }

  button:focus-visible {
    outline: 3px solid rgb(3 105 161 / 0.3);
    outline-offset: 2px;
  }

  button span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  button strong {
    min-width: 24px;
    border-radius: 999px;
    padding: 2px 6px;
    color: var(--color-ink);
    background: rgb(255 255 255 / 0.78);
    font-size: 10px;
    text-align: center;
  }

  @media (max-width: 700px) {
    .tabs {
      grid-template-columns: repeat(4, minmax(150px, 1fr));
      padding: 2px 2px 7px;
    }
  }
</style>
