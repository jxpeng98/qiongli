<script lang="ts">
  import type { AppSnapshot } from '@qiongli/app-api';
  import { Filter } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { NativeSelect } from '$lib/components/ui/native-select';

  import type { TimelineMode, TimelineSelection } from '.';

  let {
    projects,
    selection,
    disabled,
    onApply
  }: {
    projects: AppSnapshot['researchLibrary']['projects'];
    selection: TimelineSelection;
    disabled: boolean;
    onApply: (selection: TimelineSelection) => void;
  } = $props();

  let mode = $state<TimelineMode>('portfolio-activity');
  let projectId = $state('');
  let projectRequired = $derived(mode === 'project-activity');
  let projectDisabled = $derived(mode === 'portfolio-activity');
  let canApply = $derived(
    !disabled && (!projectRequired || projectId.length > 0)
  );

  $effect(() => {
    mode = selection.mode;
    projectId = selection.projectId ?? '';
  });

  function changeMode(): void {
    if (mode === 'portfolio-activity') {
      projectId = '';
    } else if (mode === 'project-activity' && !projectId) {
      projectId = projects[0]?.projectId ?? '';
    }
  }

  function apply(event: SubmitEvent): void {
    event.preventDefault();
    if (!canApply) return;
    onApply({
      mode,
      projectId: projectDisabled ? null : projectId || null
    });
  }
</script>

<Card.Root class="controls" aria-labelledby="timeline-controls-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('timeline.controlsEyebrow')}</p>
      <h2 id="timeline-controls-title">{i18n.t('timeline.controlsTitle')}</h2>
    </div>
    <span>{i18n.t('timeline.nativeOrder')}</span>
  </header>

  <form onsubmit={apply}>
    <label>
      <span>{i18n.t('timeline.mode')}</span>
      <NativeSelect class="timeline-select" bind:value={mode} onchange={changeMode} disabled={disabled}>
        <option value="portfolio-activity">{i18n.t('timeline.mode.portfolio-activity')}</option>
        <option value="project-activity">{i18n.t('timeline.mode.project-activity')}</option>
        <option value="revision-history">{i18n.t('timeline.mode.revision-history')}</option>
        <option value="merge-resolution-history">
          {i18n.t('timeline.mode.merge-resolution-history')}
        </option>
      </NativeSelect>
    </label>

    <label>
      <span>{i18n.t('timeline.projectScope')}</span>
      <NativeSelect
        class="timeline-select"
        bind:value={projectId}
        disabled={disabled || projectDisabled}
        required={projectRequired}
        aria-describedby="timeline-project-scope-help"
      >
        {#if !projectRequired}
          <option value="">{i18n.t('timeline.allProjects')}</option>
        {/if}
        {#each projects as project (project.projectId)}
          <option value={project.projectId}>{project.displayName}</option>
        {/each}
      </NativeSelect>
    </label>

    <p id="timeline-project-scope-help" class="help">
      {projectDisabled
        ? i18n.t('timeline.portfolioScopeHelp')
        : projectRequired
          ? i18n.t('timeline.projectScopeRequired')
          : i18n.t('timeline.projectScopeOptional')}
    </p>

    <Button type="submit" disabled={!canApply}>
      <Filter size={16} aria-hidden="true" />
      {i18n.t('timeline.apply')}
    </Button>
  </form>
</Card.Root>

<style>
  :global(.controls) { min-width: 0; padding: 16px; }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  header > span {
    max-width: 100%;
    overflow: hidden;
    border-radius: 999px;
    padding: 4px 8px;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
    font-size: 10px;
    font-weight: 800;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  form {
    display: grid;
    grid-template-columns: minmax(180px, 0.85fr) minmax(220px, 1.15fr) auto;
    align-items: end;
    gap: 12px;
    margin-top: 14px;
  }
  label { display: grid; min-width: 0; gap: 5px; }
  label > span { color: var(--color-muted); font-size: 11px; font-weight: 700; }
  :global(.timeline-select) { width: 100%; }
  .help {
    grid-column: 1 / 3;
    margin: -3px 0 0;
    color: var(--color-muted);
    font-size: 10px;
    line-height: 1.45;
  }
  form :global([data-slot='button']) { grid-column: 3; grid-row: 1; }
  @media (max-width: 760px) {
    form { grid-template-columns: 1fr; }
    .help, form :global([data-slot='button']) { grid-column: 1; grid-row: auto; }
  }
  @media (max-width: 520px) {
    header { align-items: flex-start; flex-direction: column; }
    form :global([data-slot='button']) { width: 100%; }
  }
</style>
