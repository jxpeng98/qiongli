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

  import { useAppState, useProjectWorkspace } from '$lib/context';
  import { projectStatus } from '$lib/features/research-library';
  import { i18n } from '$lib/i18n.svelte';
  import { StatusBadge, surfaceClass } from '$lib/shared/ui';

  import {
    isProjectWorkspaceRoute,
    projectWorkspaceNavigation
  } from '.';

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
  <section class={surfaceClass('glass', 'project-context')} aria-label={i18n.t('projectWorkspace.context')}>
    <div class="project-identity">
      <span class="project-mark" aria-hidden="true"><BookOpenText size={18} /></span>
      <div>
        <p>{i18n.t('projectWorkspace.active')}</p>
        <strong title={selectedProject.displayName}>{selectedProject.displayName}</strong>
      </div>
    </div>

    <label class="project-select">
      <span>{i18n.t('projectWorkspace.select')}</span>
      <select
        value={selectedProject.projectId}
        disabled={app.loading || projects.length < 2}
        onchange={selectProject}
      >
        {#each projects as project (project.projectId)}
          <option value={project.projectId}>
            {project.displayName} · r{project.semanticRevision}
          </option>
        {/each}
      </select>
    </label>

    <nav
      class="project-navigation"
      aria-label={i18n.t('projectWorkspace.navigation')}
    >
      {#each projectWorkspaceNavigation as item (item.id)}
        {@const Icon = icons[item.id]}
        <a
          href={workspace.href(item.href, selectedProject.projectId)}
          aria-current={page.url.pathname === item.href ? 'page' : undefined}
        >
          <Icon size={15} strokeWidth={1.9} aria-hidden="true" />
          {i18n.t(item.labelKey)}
        </a>
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
    top: 10px;
    z-index: 24;
    display: grid;
    width: 100%;
    max-width: 100%;
    grid-template-columns: minmax(160px, 0.8fr) minmax(220px, 1.2fr) auto;
    align-items: center;
    gap: 10px 14px;
    margin: -10px 0 22px;
    border-width: 1px;
    border-radius: var(--radius-glass);
    padding: 10px 12px;
    box-shadow: var(--shadow-glass);
  }
  .project-identity {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 9px;
  }
  .project-mark {
    display: grid;
    width: 28px;
    height: 28px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 9px;
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
    overflow: hidden;
    color: var(--color-ink-strong);
    font-size: 12px;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .project-select {
    display: grid;
    min-width: 0;
    gap: 3px;
  }
  .project-select select {
    width: 100%;
    min-height: 36px;
    min-width: 0;
    border: 1px solid var(--glass-border);
    border-radius: 9px;
    padding: 5px 8px;
    color: var(--color-ink);
    background: var(--glass-control-background);
    box-shadow:
      0 0 0 0.5px var(--glass-outline),
      inset 0 1px 0 var(--glass-highlight-soft);
    font: inherit;
    font-size: 11px;
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
  .project-navigation a {
    display: inline-flex;
    min-width: 0;
    min-height: 36px;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid transparent;
    border-radius: 10px;
    padding: 6px 8px;
    color: var(--color-muted);
    font-size: 11px;
    font-weight: 560;
    text-decoration: none;
    text-align: center;
    white-space: normal;
  }
  .project-navigation a:hover {
    border-color: var(--glass-border);
    color: var(--color-ink);
    background: var(--glass-control-background-hover);
  }
  .project-navigation a[aria-current='page'] {
    border-color: var(--glass-border);
    color: var(--color-accent-strong);
    background: var(--glass-control-background-hover);
    box-shadow:
      0 0 0 0.5px var(--glass-outline),
      inset 0 1px 0 var(--glass-highlight-soft),
      inset 0 -1px 0 var(--glass-shade);
    font-weight: 650;
  }
  .project-evidence {
    display: flex;
    min-width: 0;
    align-items: center;
    justify-content: flex-end;
    gap: 7px;
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
      padding-inline: 10px;
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
</style>
