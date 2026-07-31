<script lang="ts">
  import { AlertTriangle, Inbox, Radar, Send } from '@lucide/svelte';
  import type { Snippet } from 'svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { TabsContent, TabsList, TabsRoot, TabsTrigger } from '$lib/components/app';
  import {
    captureWorkspaceModes,
    type CaptureWorkspaceMode
  } from '.';

  let {
    mode,
    counts,
    onChange,
    panel
  }: {
    mode: CaptureWorkspaceMode;
    counts: Record<CaptureWorkspaceMode, number>;
    onChange: (mode: CaptureWorkspaceMode) => void;
    panel?: Snippet<[CaptureWorkspaceMode]>;
  } = $props();

  const icons = {
    inbox: Inbox,
    outbox: Send,
    conflicts: AlertTriangle,
    coverage: Radar
  };

  function changeMode(value: string): void {
    if (captureWorkspaceModes.includes(value as CaptureWorkspaceMode)) {
      onChange(value as CaptureWorkspaceMode);
    }
  }
</script>

<TabsRoot value={mode} onValueChange={changeMode}>
  <TabsList class="capture-tabs" aria-label={i18n.t('captures.workspaceModes')}>
    {#each captureWorkspaceModes as item}
      {@const Icon = icons[item]}
      <TabsTrigger
        id={`capture-tab-${item}`}
        value={item}
      >
        <Icon size={16} aria-hidden="true" />
        <span class="tab-label">{i18n.t(`captures.mode.${item}`)}</span>
        <strong
          class="tab-count"
          aria-label={i18n.t('captures.itemCount', { count: counts[item] })}
        >{counts[item]}</strong>
      </TabsTrigger>
    {/each}
  </TabsList>
  {#each captureWorkspaceModes as item}
    <TabsContent id={`capture-panel-${item}`} value={item} class="capture-tab-panel">
      {#if panel && mode === item}{@render panel(item)}{/if}
    </TabsContent>
  {/each}
</TabsRoot>

<style>
  :global(.capture-tabs) {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(130px, 100%), 1fr));
    gap: 7px;
    margin-bottom: 10px;
  }

  .tab-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab-count {
    min-width: 24px;
    border-radius: 999px;
    padding: 2px 6px;
    color: var(--color-ink);
    background: var(--color-surface-muted);
    font-size: 10px;
    text-align: center;
    white-space: nowrap;
  }

  @media (max-width: 700px) {
    :global(.capture-tabs) {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      padding: 2px;
    }
  }

  @media (max-width: 380px) {
    :global(.capture-tabs) { grid-template-columns: 1fr; }
  }
</style>
