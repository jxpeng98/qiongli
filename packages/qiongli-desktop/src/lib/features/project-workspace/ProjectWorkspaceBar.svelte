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
  import { StatusBadge } from '$lib/shared/ui';

  import {
    isProjectWorkspaceRoute,
    projectWorkspaceNavigation
  } from '.';

  const app = useAppState();
  const workspace = useProjectWorkspace();
  let projectNavigation = $state<HTMLElement | null>(null);

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

  $effect(() => {
    page.url.pathname;
    if (!visible || !projectNavigation || typeof window === 'undefined') return;
    const frame = window.requestAnimationFrame(() => {
      projectNavigation
        ?.querySelector<HTMLElement>('[aria-current="page"]')
        ?.scrollIntoView({ block: 'nearest', inline: 'center' });
    });
    return () => window.cancelAnimationFrame(frame);
  });

  function selectProject(event: Event): void {
    const projectId = (event.currentTarget as HTMLSelectElement).value;
    if (projectId) void workspace.selectProject(projectId);
  }
</script>

{#if visible && selectedProject}
  <section class="surface glass-material project-context" aria-label={i18n.t('projectWorkspace.context')}>
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
      bind:this={projectNavigation}
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
    grid-template-columns: minmax(180px, 0.8fr) minmax(210px, 0.9fr) minmax(420px, 2fr) auto;
    align-items: center;
    gap: 10px 14px;
    margin: -10px 0 22px;
    border-width: 1px;
    border-radius: 8px;
    padding: 8px 10px;
    background: var(--glass-surface);
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
    border-radius: 5px;
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
    border: 1px solid var(--color-border);
    border-radius: 5px;
    padding: 5px 8px;
    color: var(--color-ink);
    background: var(--glass-control);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.64);
    font: inherit;
    font-size: 11px;
  }
  .project-navigation {
    display: flex;
    min-width: 0;
    overflow-x: auto;
    gap: 4px;
    padding: 2px;
    overscroll-behavior-inline: contain;
    scrollbar-width: thin;
  }
  .project-navigation a {
    display: inline-flex;
    min-height: 36px;
    flex: 0 0 auto;
    align-items: center;
    gap: 6px;
    border: 1px solid transparent;
    border-radius: 5px;
    padding: 6px 8px;
    color: var(--color-muted);
    font-size: 11px;
    font-weight: 560;
    text-decoration: none;
    white-space: nowrap;
  }
  .project-navigation a:hover {
    border-color: var(--color-border);
    color: var(--color-ink);
    background: var(--color-surface-subtle);
  }
  .project-navigation a[aria-current='page'] {
    border-color: var(--color-border);
    color: var(--color-accent-strong);
    background: rgb(226 236 232 / 0.72);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.58);
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
  @media (max-width: 1120px) {
    .project-context {
      grid-template-columns: minmax(180px, 1fr) minmax(210px, 1fr) auto;
    }
    .project-navigation {
      grid-column: 1 / -1;
      grid-row: 2;
    }
  }
  @media (max-width: 650px) {
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
    }
    .project-evidence {
      grid-column: 2;
      grid-row: 1;
    }
    .project-evidence > span { display: none; }
  }
</style>
