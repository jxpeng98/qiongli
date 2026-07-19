<script lang="ts">
  import type { ArticleProjectSummary } from '@qiongli/app-api';
  import {
    AlertTriangle,
    Archive,
    ArrowUpRight,
    BookOpenText,
    CheckCircle2,
    CircleGauge,
    FileQuestion,
    FolderOpen,
    FolderPlus,
    Package,
    PackageOpen,
    Plus,
    RefreshCw,
    Search,
    Stethoscope
  } from '@lucide/svelte';

  import { useAppState } from '$lib/context';
  import {
    filterProjects,
    projectStatus,
    type ProjectLifecycleFilter,
    type ProjectSort
  } from '$lib/features/research-library';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';

  const app = useAppState();

  let query = $state('');
  let lifecycle = $state<ProjectLifecycleFilter>('all');
  let sort = $state<ProjectSort>('academically-updated');
  let selectedProjectId = $state<string | null>(null);
  let showCreate = $state(false);
  let createName = $state('');
  let createKind = $state<'article' | 'review' | 'dissertation-article' | 'manuscript'>('article');
  let createStage = $state<'idea' | 'framing' | 'literature' | 'design' | 'analysis' | 'writing' | 'review' | 'submission'>('idea');

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let visibleProjects = $derived(filterProjects(projects, query, lifecycle, sort));
  let activeCount = $derived(projects.filter((project) => project.lifecycle === 'active').length);
  let attentionCount = $derived(projects.filter((project) => project.health !== 'ready').length);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let createNameValid = $derived(
    createName.length > 0 &&
    createName.length <= 160 &&
    createName.trim() === createName &&
    !/[\u0000-\u001f\u007f]/.test(createName)
  );

  async function refreshLibrary(): Promise<void> {
    await app.execute({ action: 'refresh-research-library' });
  }

  async function registerProject(): Promise<void> {
    const selection = await app.execute({ action: 'select-project-directory' });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-register',
      directoryToken: selection.token
    });
  }

  async function createProject(): Promise<void> {
    if (!createNameValid) return;
    const selection = await app.execute({
      action: 'select-project-create-destination',
      suggestedName: directoryName(createName, 'article-project')
    });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-create',
      directoryToken: selection.token,
      displayName: createName,
      projectKind: createKind,
      stage: createStage
    });
  }

  async function importProject(): Promise<void> {
    const selection = await app.execute({
      action: 'select-project-import-locations',
      suggestedName: 'imported-qiongli-project'
    });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-import',
      directoryToken: selection.token
    });
  }

  async function openProject(project: ArticleProjectSummary): Promise<void> {
    await app.execute({ action: 'open-project', projectId: project.projectId });
  }

  async function exportProject(project: ArticleProjectSummary): Promise<void> {
    const selection = await app.execute({
      action: 'select-project-export-destination',
      projectId: project.projectId
    });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-export',
      directoryToken: selection.token
    });
  }

  async function repairManifest(project: ArticleProjectSummary): Promise<void> {
    await app.execute({
      action: 'preview-project-repair-manifest',
      projectId: project.projectId
    });
  }

  async function previewProject(
    project: ArticleProjectSummary,
    operation: 'archive' | 'restore' | 'refresh' | 'unregister'
  ): Promise<void> {
    switch (operation) {
      case 'archive':
        await app.execute({ action: 'preview-project-archive', projectId: project.projectId });
        break;
      case 'restore':
        await app.execute({ action: 'preview-project-restore', projectId: project.projectId });
        break;
      case 'refresh':
        await app.execute({ action: 'preview-project-refresh', projectId: project.projectId });
        break;
      case 'unregister':
        await app.execute({ action: 'preview-project-unregister', projectId: project.projectId });
        break;
    }
  }

  function projectDate(project: ArticleProjectSummary): string {
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: 'medium'
    }).format(new Date(project.academicallyUpdatedAtUnix * 1_000));
  }

  function sentence(value: string): string {
    return value.replaceAll('-', ' ').replace(/^./, (letter) => letter.toUpperCase());
  }

  function directoryName(value: string, fallback: string): string {
    const name = value
      .normalize('NFKD')
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
      .slice(0, 80);
    return name || fallback;
  }
</script>

<PageHeader
  eyebrow="Article continuity"
  title="Research Library"
  description="Track the academic state of every article in one private local index. Qiongli stores project identity and revisions here while the research artifacts remain portable inside each project."
>
  {#snippet actions()}
    <button
      class="button-primary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={() => showCreate = !showCreate}
    >
      <Plus size={16} aria-hidden="true" />
      New project
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={registerProject}
    >
      <FolderPlus size={16} aria-hidden="true" />
      Register project
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={importProject}
    >
      <PackageOpen size={16} aria-hidden="true" />
      Import portable
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectLibrary}
      onclick={refreshLibrary}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      Refresh library
    </button>
  {/snippet}
</PageHeader>

{#if showCreate}
  <section class="surface create-panel" aria-label="Create article project">
    <div>
      <p class="eyebrow">Native project creation</p>
      <h2>Create a portable article project</h2>
      <p>Choose the academic identity here; the native picker chooses the new directory only after the form is valid.</p>
    </div>
    <label class="create-name">
      <span>Project name</span>
      <input bind:value={createName} maxlength="160" placeholder="e.g. Trustworthy research agents" />
    </label>
    <label>
      <span>Type</span>
      <select bind:value={createKind}>
        <option value="article">Article</option>
        <option value="review">Review</option>
        <option value="dissertation-article">Dissertation article</option>
        <option value="manuscript">Manuscript</option>
      </select>
    </label>
    <label>
      <span>Starting stage</span>
      <select bind:value={createStage}>
        <option value="idea">Idea</option>
        <option value="framing">Framing</option>
        <option value="literature">Literature</option>
        <option value="design">Design</option>
        <option value="analysis">Analysis</option>
        <option value="writing">Writing</option>
        <option value="review">Review</option>
        <option value="submission">Submission</option>
      </select>
    </label>
    <div class="create-actions">
      <button class="button-quiet" type="button" onclick={() => showCreate = false}>Cancel</button>
      <button class="button-primary" type="button" disabled={app.loading || !createNameValid} onclick={createProject}>
        <FolderPlus size={16} aria-hidden="true" />Choose location & preview
      </button>
    </div>
  </section>
{/if}

{#if !app.snapshot}
  <section class="surface loading" aria-busy="true">
    <div class="skeleton wide"></div>
    <div class="skeleton"></div>
    <p>Loading the project index from the native service…</p>
  </section>
{:else if app.snapshot.researchLibrary.health === 'inspection-blocked'}
  <section class="surface state-panel state-danger">
    <AlertTriangle size={24} aria-hidden="true" />
    <div>
      <h2>Research Library cannot be inspected</h2>
      <p>The native service could not safely inspect the private project index. No paths or partial data were exposed.</p>
    </div>
  </section>
{:else}
  <section class="metrics" aria-label="Research library summary">
    <article class="surface metric">
      <span class="metric-icon"><BookOpenText size={18} aria-hidden="true" /></span>
      <div><strong>{projects.length}</strong><span>Projects</span></div>
    </article>
    <article class="surface metric">
      <span class="metric-icon positive"><CheckCircle2 size={18} aria-hidden="true" /></span>
      <div><strong>{activeCount}</strong><span>Active</span></div>
    </article>
    <article class="surface metric">
      <span class:warning={attentionCount > 0} class="metric-icon"><AlertTriangle size={18} aria-hidden="true" /></span>
      <div><strong>{attentionCount}</strong><span>Need attention</span></div>
    </article>
    <article class="surface metric">
      <span class="metric-icon"><CircleGauge size={18} aria-hidden="true" /></span>
      <div><strong>{app.snapshot.researchLibrary.revision}</strong><span>Library revision</span></div>
    </article>
  </section>

  {#if projects.length === 0}
    <section class="surface empty-state">
      <span><FileQuestion size={27} aria-hidden="true" /></span>
      <h2>No article projects registered yet</h2>
      <p>Create a new portable project, register an existing <code>RESEARCH/&lt;topic&gt;</code> directory, or import a verified Qiongli package from another machine.</p>
      <div class="empty-actions">
        <button class="button-primary" type="button" disabled={app.loading} onclick={() => showCreate = true}>
          <Plus size={16} aria-hidden="true" />Create project
        </button>
        <button class="button-secondary" type="button" disabled={app.loading} onclick={registerProject}>
          <FolderPlus size={16} aria-hidden="true" />Choose existing
        </button>
        <button class="button-secondary" type="button" disabled={app.loading} onclick={importProject}>
          <PackageOpen size={16} aria-hidden="true" />Import portable
        </button>
      </div>
    </section>
  {:else}
    <section class="surface library">
      <div class="library-heading">
        <div>
          <p class="eyebrow">Project index</p>
          <h2>Academic projects</h2>
          <p>Sorted by the time research content changed, not by incidental file-system activity.</p>
        </div>
        <StatusBadge
          status={app.snapshot.researchLibrary.health === 'ready' ? 'ready' : 'recovery-required'}
          label={sentence(app.snapshot.researchLibrary.health)}
        />
      </div>

      <div class="controls">
        <label class="search-control">
          <span class="sr-only">Search projects</span>
          <Search size={17} aria-hidden="true" />
          <input bind:value={query} type="search" placeholder="Search title, question, or thesis" />
        </label>
        <label>
          <span>Lifecycle</span>
          <select bind:value={lifecycle}>
            <option value="all">All projects</option>
            <option value="active">Active</option>
            <option value="archived">Archived</option>
            <option value="attention">Needs attention</option>
          </select>
        </label>
        <label>
          <span>Sort</span>
          <select bind:value={sort}>
            <option value="academically-updated">Academic update</option>
            <option value="name">Project name</option>
            <option value="stage">Research stage</option>
          </select>
        </label>
      </div>

      {#if visibleProjects.length === 0}
        <div class="no-results">
          <Search size={21} aria-hidden="true" />
          <p>No projects match the current filters.</p>
        </div>
      {:else}
        <div class="project-list">
          {#each visibleProjects as project (project.projectId)}
            <article class:selected={selectedProjectId === project.projectId}>
              <button class="project-main" type="button" onclick={() => selectedProjectId = project.projectId}>
                <span class="project-title">
                  <strong>{project.displayName}</strong>
                  <small>{project.rootLabel}</small>
                </span>
                <span class="project-tags">
                  <span>{sentence(project.projectKind)}</span>
                  <span>{sentence(project.stage)}</span>
                  {#if project.lifecycle === 'archived'}<span><Archive size={12} aria-hidden="true" />Archived</span>{/if}
                </span>
                <span class="revision"><strong>r{project.semanticRevision}</strong><small>{projectDate(project)}</small></span>
                <StatusBadge status={projectStatus(project)} label={sentence(project.health)} />
                <ArrowUpRight size={17} aria-hidden="true" />
              </button>
            </article>
          {/each}
        </div>
      {/if}
    </section>

    {#if selectedProject}
      <section class="surface overview" aria-live="polite">
        <div class="overview-title">
          <div>
            <p class="eyebrow">Project overview</p>
            <h2>{selectedProject.displayName}</h2>
            <p><code>{selectedProject.projectId}</code> · {sentence(selectedProject.nextAction)} next</p>
          </div>
          <div class="overview-actions">
            {#if selectedProject.health === 'ready' || selectedProject.health === 'revision-drift'}
              <button class="button-primary" type="button" disabled={app.loading} onclick={() => openProject(selectedProject)}>
                <FolderOpen size={15} aria-hidden="true" />Open project
              </button>
              <button class="button-secondary" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'refresh')}>
                <RefreshCw size={15} aria-hidden="true" />Refresh revision
              </button>
              <button class="button-secondary" type="button" disabled={app.loading} onclick={() => exportProject(selectedProject)}>
                <Package size={15} aria-hidden="true" />Export portable
              </button>
            {:else if selectedProject.health === 'missing-manifest'}
              <button class="button-primary" type="button" disabled={app.loading} onclick={() => repairManifest(selectedProject)}>
                <Stethoscope size={15} aria-hidden="true" />Doctor: repair manifest
              </button>
            {/if}
            {#if selectedProject.health === 'ready' || selectedProject.health === 'revision-drift'}
              {#if selectedProject.lifecycle === 'active'}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'archive')}>
                  <Archive size={15} aria-hidden="true" />Archive
                </button>
              {:else}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'restore')}>
                  <CheckCircle2 size={15} aria-hidden="true" />Restore
                </button>
              {/if}
            {/if}
            <button class="button-quiet" type="button" onclick={() => selectedProjectId = null}>Close</button>
          </div>
        </div>

        <div class="overview-grid">
          <article>
            <span>Focal question</span>
            <p>{selectedProject.overview.focalQuestion ?? 'Not recorded in the canonical research state yet.'}</p>
          </article>
          <article>
            <span>Working thesis</span>
            <p>{selectedProject.overview.thesis ?? 'Not recorded in the canonical research state yet.'}</p>
          </article>
          <article>
            <span>Evidence position</span>
            <p>{selectedProject.overview.evidencePosition ?? 'No evidence position has been summarized yet.'}</p>
          </article>
          <article class="evidence-card">
            <span>Claim–evidence coverage</span>
            <strong>{selectedProject.overview.claimEvidenceCoveragePercent === null ? '—' : `${selectedProject.overview.claimEvidenceCoveragePercent}%`}</strong>
            <div class="progress" aria-hidden="true"><i style:width={`${selectedProject.overview.claimEvidenceCoveragePercent ?? 0}%`}></i></div>
            <small>{selectedProject.overview.unresolvedRiskCount} unresolved risks</small>
          </article>
        </div>

        <div class="priorities">
          <span>Next priorities</span>
          {#if selectedProject.overview.nextPriorities.length === 0}
            <p>No next priorities recorded.</p>
          {:else}
            <ol>
              {#each selectedProject.overview.nextPriorities as priority}<li>{priority}</li>{/each}
            </ol>
          {/if}
        </div>
        <div class="danger-zone">
          <div>
            <strong>Remove from this Research Library</strong>
            <p>The portable manifest and every academic artifact remain in the project directory.</p>
          </div>
          <button class="button-danger" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'unregister')}>Unregister</button>
        </div>
      </section>
    {/if}
  {/if}
{/if}

<style>
  .create-panel { display: grid; grid-template-columns: minmax(230px, 1.3fr) minmax(220px, 1fr) 170px 170px auto; align-items: end; gap: 12px; margin-bottom: 18px; padding: 18px; border-top: 3px solid var(--color-accent); }
  .create-panel h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .create-panel > div:first-child > p:last-child { margin: 6px 0 0; color: var(--color-muted); font-size: 11px; line-height: 1.5; }
  .create-panel label { display: grid; gap: 5px; }
  .create-panel label > span { color: var(--color-muted); font-size: 10px; font-weight: 800; letter-spacing: 0.05em; text-transform: uppercase; }
  .create-actions, .empty-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 7px; }
  .create-actions { justify-content: flex-end; }
  .loading { min-height: 220px; padding: 30px; }
  .loading p { color: var(--color-muted); }
  .skeleton { width: 42%; height: 18px; margin-bottom: 14px; border-radius: 6px; background: #e2e8f0; }
  .skeleton.wide { width: 68%; height: 30px; }

  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 18px; }
  .metric { display: flex; min-height: 86px; align-items: center; gap: 13px; padding: 16px; }
  .metric-icon { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .metric-icon.positive { color: var(--color-success); background: var(--color-success-soft); }
  .metric-icon.warning { color: var(--color-warning); background: var(--color-warning-soft); }
  .metric strong, .metric span { display: block; }
  .metric strong { color: var(--color-ink-strong); font-size: 21px; line-height: 1; }
  .metric div span { margin-top: 5px; color: var(--color-muted); font-size: 11px; font-weight: 700; }

  .state-panel { display: flex; align-items: flex-start; gap: 14px; padding: 22px; }
  .state-danger { border-color: #fecaca; color: var(--color-danger); background: var(--color-danger-soft); }
  .state-panel h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .state-panel p { margin: 7px 0 0; color: var(--color-muted); font-size: 13px; line-height: 1.6; }

  .empty-state { padding: 52px 24px; text-align: center; }
  .empty-state > span { display: grid; width: 50px; height: 50px; place-items: center; margin: 0 auto 16px; border-radius: 14px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .empty-state h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; }
  .empty-state p { max-width: 650px; margin: 10px auto 0; color: var(--color-muted); font-size: 13px; line-height: 1.65; }
  .empty-actions { justify-content: center; margin-top: 18px; }
  code { overflow-wrap: anywhere; }

  .library { padding: 22px; }
  .library-heading, .overview-title { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; }
  .library-heading h2, .overview-title h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; letter-spacing: -0.02em; }
  .library-heading > div > p:last-child, .overview-title > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }

  .controls { display: grid; grid-template-columns: minmax(240px, 1fr) 170px 180px; gap: 10px; margin-top: 19px; padding: 13px; border: 1px solid var(--color-border); border-radius: 12px; background: var(--color-surface-subtle); }
  .controls label:not(.search-control) { display: grid; gap: 5px; }
  .controls label > span { color: var(--color-muted); font-size: 10px; font-weight: 800; letter-spacing: 0.05em; text-transform: uppercase; }
  .search-control { display: flex; min-height: 42px; align-items: center; gap: 9px; align-self: end; border: 1px solid var(--color-border-strong); border-radius: 9px; padding: 0 11px; color: var(--color-muted); background: white; }
  input, select { width: 100%; min-height: 42px; border: 1px solid var(--color-border-strong); border-radius: 9px; padding: 8px 10px; color: var(--color-ink); background: white; font: inherit; font-size: 12px; }
  .search-control input { min-height: 38px; border: 0; padding: 0; }

  .project-list { margin-top: 14px; border-top: 1px solid var(--color-border); }
  .project-list article { border-bottom: 1px solid var(--color-border); }
  .project-list article.selected { margin-inline: -8px; border: 1px solid #7dd3fc; border-radius: 10px; background: var(--color-accent-soft); }
  .project-main { display: grid; width: 100%; min-height: 76px; grid-template-columns: minmax(190px, 1.4fr) minmax(160px, 1fr) 110px auto auto; align-items: center; gap: 15px; border: 0; padding: 12px 7px; color: inherit; background: transparent; text-align: left; cursor: pointer; }
  .project-list article.selected .project-main { padding-inline: 14px; }
  .project-main:hover { background: rgb(241 245 249 / 0.72); }
  .project-title strong, .project-title small, .revision strong, .revision small { display: block; }
  .project-title strong { color: var(--color-ink-strong); font-size: 13px; }
  .project-title small, .revision small { margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  .project-tags { display: flex; flex-wrap: wrap; gap: 5px; }
  .project-tags span { display: inline-flex; align-items: center; gap: 4px; border: 1px solid var(--color-border); border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: white; font-size: 10px; font-weight: 700; }
  .revision { text-align: right; }
  .revision strong { color: var(--color-ink); font-size: 12px; }
  .no-results { display: flex; min-height: 130px; align-items: center; justify-content: center; gap: 10px; color: var(--color-muted); }

  .overview { margin-top: 18px; padding: 22px; border-top: 3px solid var(--color-accent); }
  .overview-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 7px; }
  .overview-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 11px; margin-top: 19px; }
  .overview-grid article, .priorities { border: 1px solid var(--color-border); border-radius: 12px; padding: 15px; background: var(--color-surface-subtle); }
  .overview-grid article > span, .priorities > span { color: var(--color-accent-strong); font-size: 10px; font-weight: 800; letter-spacing: 0.06em; text-transform: uppercase; }
  .overview-grid p, .priorities p, .priorities ol { margin: 8px 0 0; color: var(--color-ink); font-size: 12px; line-height: 1.6; }
  .evidence-card strong { display: block; margin-top: 10px; color: var(--color-ink-strong); font-size: 24px; }
  .evidence-card small { display: block; margin-top: 8px; color: var(--color-muted); font-size: 11px; }
  .progress { height: 6px; margin-top: 8px; overflow: hidden; border-radius: 999px; background: #cbd5e1; }
  .progress i { display: block; height: 100%; border-radius: inherit; background: var(--color-accent); }
  .priorities { margin-top: 11px; }
  .priorities ol { padding-left: 20px; }
  .priorities li + li { margin-top: 5px; }
  .danger-zone { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 11px; border: 1px solid #fecaca; border-radius: 12px; padding: 14px 15px; background: var(--color-danger-soft); }
  .danger-zone strong { color: #991b1b; font-size: 12px; }
  .danger-zone p { margin: 4px 0 0; color: var(--color-muted); font-size: 11px; }
  .sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }

  @media (max-width: 1200px) {
    .create-panel { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .create-panel > div:first-child, .create-name, .create-actions { grid-column: 1 / -1; }
    .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .project-main { grid-template-columns: minmax(180px, 1.2fr) minmax(140px, 1fr) auto auto; }
    .revision { display: none; }
  }

  @media (max-width: 760px) {
    .controls { grid-template-columns: 1fr 1fr; }
    .search-control { grid-column: 1 / -1; }
    .project-main { grid-template-columns: 1fr auto; }
    .project-tags { grid-column: 1 / -1; grid-row: 2; }
    .project-main :global(.status) { grid-column: 1; grid-row: 3; justify-self: start; }
    .project-main > :last-child { grid-column: 2; grid-row: 1 / 4; }
    .overview-grid { grid-template-columns: 1fr; }
  }

  @media (max-width: 520px) {
    .create-panel { grid-template-columns: 1fr; }
    .create-panel > div:first-child, .create-name, .create-actions { grid-column: auto; }
    .create-actions, .empty-actions { align-items: stretch; flex-direction: column; }
    .metrics, .controls { grid-template-columns: 1fr; }
    .search-control { grid-column: auto; }
    .library, .overview { padding: 17px; }
    .library-heading, .overview-title, .danger-zone { align-items: flex-start; flex-direction: column; }
    .overview-actions { justify-content: flex-start; }
  }
</style>
