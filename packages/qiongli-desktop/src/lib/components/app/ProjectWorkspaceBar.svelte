<script lang="ts">
  import {
    BookOpenText,
    CalendarClock,
    Files,
    GitBranch,
    Inbox,
    LayoutDashboard,
    Network
  } from '@lucide/svelte';
  import { page } from '$app/state';

  import { Button } from '$lib/components/ui/button';
  import { NativeSelect } from '$lib/components/ui/native-select';
  import { useAppState, useProjectWorkspace } from '$lib/context';
  import { projectStatus } from '$lib/features/research-library';
  import { i18n } from '$lib/i18n.svelte';
  import StatusBadge from './StatusBadge.svelte';

  import {
    isProjectWorkspaceRoute,
    projectWorkspaceNavigation
  } from '$lib/features/project-workspace';

  const app = useAppState();
  const workspace = useProjectWorkspace();

  const icons = {
    overview: LayoutDashboard,
    artifacts: Files,
    captures: Inbox,
    'academic-graph': Network,
    timeline: CalendarClock,
    'run-in-client': GitBranch
  } as const;

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === workspace.projectId) ?? null
  );
  let visible = $derived(
    isProjectWorkspaceRoute(page.url.pathname) && selectedProject !== null
  );

  function selectProject(event: Event): void {
    const projectId = (event.currentTarget as HTMLSelectElement).value;
    if (projectId) void workspace.selectProject(projectId);
  }
</script>

{#if visible && selectedProject}
  <section class="project-context" aria-label={i18n.t('projectWorkspace.context')}>
    <div class="project-identity">
      <span class="project-mark" aria-hidden="true"><BookOpenText size={18} /></span>
      <div>
        <p>{i18n.t('projectWorkspace.active')}</p>
        <strong title={selectedProject.displayName}>{selectedProject.displayName}</strong>
      </div>
    </div>

    <label class="project-select">
      <span>{i18n.t('projectWorkspace.select')}</span>
      <NativeSelect
        class="project-native-select"
        size="sm"
        value={selectedProject.projectId}
        disabled={app.loading || projects.length < 2}
        onchange={selectProject}
      >
        {#each projects as project (project.projectId)}
          <option value={project.projectId}>
            {project.displayName} · r{project.semanticRevision}
          </option>
        {/each}
      </NativeSelect>
    </label>

    <nav
      class="project-navigation"
      aria-label={i18n.t('projectWorkspace.navigation')}
    >
      {#each projectWorkspaceNavigation as item (item.id)}
        {@const Icon = icons[item.id]}
        <Button
          href={workspace.href(item.href, selectedProject.projectId)}
          variant="ghost"
          size="sm"
          class="project-nav-link"
          aria-current={page.url.pathname === item.href ? 'page' : undefined}
        >
          <Icon size={15} strokeWidth={1.9} aria-hidden="true" />
          {i18n.t(item.labelKey)}
        </Button>
      {/each}
    </nav>

    <div class="project-evidence">
      <span>{i18n.t('projectWorkspace.revision', { revision: selectedProject.semanticRevision })}</span>
      <StatusBadge
        status={projectStatus(selectedProject)}
        label={i18n.label(selectedProject.health)}
      />
    </div>
  </section>
{/if}

<style>
  .project-context {
    position: sticky;
    top: 8px;
    z-index: var(--z-sticky-context);
    display: grid;
    width: 100%;
    max-width: 100%;
    grid-template-columns: minmax(160px, 0.8fr) minmax(220px, 1.2fr) auto;
    align-items: center;
    gap: 6px 10px;
    margin: -6px 0 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 8px 10px;
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }
  .project-identity {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 7px;
  }
  .project-identity > div { min-width: 0; }
  .project-mark {
    display: grid;
    width: 26px;
    height: 26px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 50%;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }
  .project-identity p,
  .project-identity strong {
    display: block;
    margin: 0;
  }
  .project-identity p,
  .project-select > span {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 600;
  }
  .project-identity strong {
    display: -webkit-box;
    overflow: hidden;
    color: var(--color-ink-strong);
    font-size: 12px;
    font-weight: 650;
    line-height: 1.35;
    overflow-wrap: anywhere;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }
  .project-select {
    display: grid;
    min-width: 0;
    gap: 3px;
  }
  :global(.project-native-select) {
    width: 100%;
    min-width: 0;
  }
  .project-navigation {
    display: grid;
    min-width: 0;
    grid-column: 1 / -1;
    grid-row: 2;
    grid-template-columns: repeat(auto-fit, minmax(min(112px, 100%), 1fr));
    gap: 4px;
    padding: 2px;
  }
  :global(.project-nav-link) {
    min-width: 0;
    min-height: 28px;
    color: var(--color-muted);
    font-size: 11px;
    text-align: center;
    white-space: normal;
  }
  :global(.project-nav-link:hover) {
    color: var(--color-ink);
    background: var(--color-surface-subtle);
  }
  :global(.project-nav-link[aria-current='page']) {
    border-color: var(--color-ink-strong);
    color: var(--color-on-accent);
    background: var(--color-ink-strong);
    box-shadow: none;
    font-weight: 600;
  }
  .project-evidence {
    display: flex;
    min-width: 0;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
  }
  .project-evidence > span {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 600;
    white-space: nowrap;
  }
  @media (max-width: 760px) {
    .project-context {
      position: relative;
      top: auto;
      grid-template-columns: minmax(0, 1fr) auto;
      padding-inline: 8px;
    }
    .project-select {
      grid-column: 1 / -1;
      grid-row: 2;
    }
    .project-navigation {
      grid-column: 1 / -1;
      grid-row: 3;
      grid-template-columns: repeat(auto-fit, minmax(min(104px, 100%), 1fr));
    }
    .project-evidence {
      grid-column: 2;
      grid-row: 1;
    }
    .project-evidence > span { display: none; }
  }
  @media (max-width: 440px) {
    .project-context { grid-template-columns: 1fr; }
    .project-identity,
    .project-select,
    .project-navigation,
    .project-evidence { grid-column: 1; }
    .project-evidence { grid-row: 2; justify-content: flex-start; }
    .project-select { grid-row: 3; }
    .project-navigation { grid-row: 4; }
  }
</style>
