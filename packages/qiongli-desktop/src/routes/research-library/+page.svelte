<script lang="ts">
  import type { ArticleProjectSummary } from '@qiongli/app-api';
  import {
    AlertTriangle,
    Archive,
    ArrowUpRight,
    ArrowRightLeft,
    BookOpenText,
    CheckCircle2,
    CircleGauge,
    Ellipsis,
    FileQuestion,
    FolderOpen,
    FolderPlus,
    Package,
    PackageOpen,
    Plus,
    RefreshCw,
    RotateCcw,
    Search,
    Stethoscope,
    Network
  } from '@lucide/svelte';

  import { useAppState, useProjectWorkspace } from '$lib/context';
  import {
    filterProjects,
    projectStatus,
    type ProjectLifecycleFilter,
    type ProjectSort
  } from '$lib/features/research-library';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  const projectWorkspace = useProjectWorkspace();

  let query = $state('');
  let lifecycle = $state<ProjectLifecycleFilter>('all');
  let sort = $state<ProjectSort>('academically-updated');
  let showCreate = $state(false);
  let showMigration = $state(false);
  let projectActionsOpen = $state(false);
  let projectActionsMenu = $state<HTMLDetailsElement | null>(null);
  let createName = $state('');
  let createKind = $state<'article' | 'review' | 'dissertation-article' | 'manuscript'>('article');
  let createStage = $state<'idea' | 'framing' | 'literature' | 'design' | 'analysis' | 'writing' | 'review' | 'submission'>('idea');

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let visibleProjects = $derived(filterProjects(projects, query, lifecycle, sort));
  let activeCount = $derived(projects.filter((project) => project.lifecycle === 'active').length);
  let attentionCount = $derived(projects.filter((project) => project.health !== 'ready').length);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === projectWorkspace.projectId) ?? null
  );
  let createNameValid = $derived(
    createName.length > 0 &&
    createName.length <= 160 &&
    createName.trim() === createName &&
    !/[\u0000-\u001f\u007f]/.test(createName)
  );

  function dismissProjectActions(event: PointerEvent): void {
    if (
      projectActionsOpen
      && event.target instanceof Node
      && !projectActionsMenu?.contains(event.target)
    ) {
      projectActionsOpen = false;
    }
  }

  function handleProjectActionsKey(event: KeyboardEvent): void {
    if (!projectActionsOpen || event.key !== 'Escape') return;
    event.preventDefault();
    projectActionsOpen = false;
    projectActionsMenu?.querySelector<HTMLElement>('summary')?.focus();
  }

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

  async function migrateProject(): Promise<void> {
    if (!createNameValid) return;
    const selection = await app.execute({
      action: 'select-project-migration-locations',
      suggestedName: directoryName(createName, 'migrated-qiongli-project')
    });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-migration',
      directoryToken: selection.token,
      displayName: createName,
      projectKind: createKind,
      stage: createStage
    });
  }

  async function recoverProjectMigration(): Promise<void> {
    const selection = await app.execute({
      action: 'select-project-migration-recovery-locations'
    });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-migration-recovery',
      directoryToken: selection.token
    });
  }

  async function rollbackProjectMigration(): Promise<void> {
    const selection = await app.execute({
      action: 'select-project-migration-rollback-locations'
    });
    if (selection?.type !== 'project-directory-selected') return;
    await app.execute({
      action: 'preview-project-migration-rollback',
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
    return i18n.date(project.academicallyUpdatedAtUnix);
  }

  function sentence(value: string): string {
    return i18n.label(value);
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

<svelte:window onpointerdown={dismissProjectActions} onkeydown={handleProjectActionsKey} />

<svelte:head>
  <title>{i18n.t('library.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('library.eyebrow')}
  title={i18n.t('library.title')}
  description={i18n.t('library.description')}
>
  {#snippet actions()}
    <button
      class="button-primary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={() => {
        showCreate = !showCreate;
        showMigration = false;
      }}
    >
      <Plus size={16} aria-hidden="true" />
      {i18n.t('library.newProject')}
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={registerProject}
    >
      <FolderPlus size={16} aria-hidden="true" />
      {i18n.t('library.register')}
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectLibrary}
      onclick={refreshLibrary}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </button>
    <details
      class="header-actions-menu"
      bind:this={projectActionsMenu}
      bind:open={projectActionsOpen}
    >
      <summary class="button-secondary">
        <Ellipsis size={16} aria-hidden="true" />
        {i18n.t('library.moreActions')}
      </summary>
      <div class="surface">
        <button
          type="button"
          disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
          onclick={() => {
            projectActionsOpen = false;
            void importProject();
          }}
        >
          <PackageOpen size={16} aria-hidden="true" />
          {i18n.t('library.import')}
        </button>
        <button
          type="button"
          disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
          onclick={() => {
            projectActionsOpen = false;
            showMigration = !showMigration;
            showCreate = false;
          }}
        >
          <ArrowRightLeft size={16} aria-hidden="true" />
          {i18n.t('library.migrate')}
        </button>
      </div>
    </details>
  {/snippet}
</PageHeader>

{#if showCreate}
  <section class="surface create-panel" aria-label={i18n.t('library.createAria')}>
    <div>
      <p class="eyebrow">{i18n.t('library.projectEyebrow')}</p>
      <h2>{i18n.t('library.createTitle')}</h2>
      <p>{i18n.t('library.createHelp')}</p>
    </div>
    <label class="create-name">
      <span>{i18n.t('library.projectName')}</span>
      <input
        bind:value={createName}
        maxlength="160"
        placeholder={i18n.t('library.createNamePlaceholder')}
      />
    </label>
    <label>
      <span>{i18n.t('library.type')}</span>
      <select bind:value={createKind}>
        <option value="article">{i18n.label('article')}</option>
        <option value="review">{i18n.label('review')}</option>
        <option value="dissertation-article">{i18n.label('dissertation-article')}</option>
        <option value="manuscript">{i18n.label('manuscript')}</option>
      </select>
    </label>
    <label>
      <span>{i18n.t('library.stage')}</span>
      <select bind:value={createStage}>
        <option value="idea">{i18n.label('idea')}</option>
        <option value="framing">{i18n.label('framing')}</option>
        <option value="literature">{i18n.label('literature')}</option>
        <option value="design">{i18n.label('design')}</option>
        <option value="analysis">{i18n.label('analysis')}</option>
        <option value="writing">{i18n.label('writing')}</option>
        <option value="review">{i18n.label('review')}</option>
        <option value="submission">{i18n.label('submission')}</option>
      </select>
    </label>
    <div class="create-actions">
      <button class="button-quiet" type="button" onclick={() => showCreate = false}>{i18n.t('common.cancel')}</button>
      <button class="button-primary" type="button" disabled={app.loading || !createNameValid} onclick={createProject}>
        <FolderPlus size={16} aria-hidden="true" />{i18n.t('library.choosePreview')}
      </button>
    </div>
  </section>
{/if}

{#if showMigration}
  <section class="surface create-panel migration-panel" aria-label={i18n.t('library.migrateTitle')}>
    <div>
      <p class="eyebrow">{i18n.t('library.migrationEyebrow')}</p>
      <h2>{i18n.t('library.migrateTitle')}</h2>
      <p>{i18n.t('library.migrateHelp')}</p>
      <p>{i18n.t('library.rollbackHelp')}</p>
    </div>
    <label class="create-name">
      <span>{i18n.t('library.projectName')}</span>
      <input bind:value={createName} maxlength="160" placeholder={i18n.t('library.migrateNamePlaceholder')} />
    </label>
    <label>
      <span>{i18n.t('library.type')}</span>
      <select bind:value={createKind}>
        <option value="article">{i18n.label('article')}</option>
        <option value="review">{i18n.label('review')}</option>
        <option value="dissertation-article">{i18n.label('dissertation-article')}</option>
        <option value="manuscript">{i18n.label('manuscript')}</option>
      </select>
    </label>
    <label>
      <span>{i18n.t('library.stage')}</span>
      <select bind:value={createStage}>
        <option value="idea">{i18n.label('idea')}</option>
        <option value="framing">{i18n.label('framing')}</option>
        <option value="literature">{i18n.label('literature')}</option>
        <option value="design">{i18n.label('design')}</option>
        <option value="analysis">{i18n.label('analysis')}</option>
        <option value="writing">{i18n.label('writing')}</option>
        <option value="review">{i18n.label('review')}</option>
        <option value="submission">{i18n.label('submission')}</option>
      </select>
    </label>
    <div class="create-actions migration-actions">
      <button class="button-quiet" type="button" disabled={app.loading} onclick={recoverProjectMigration}>
        <RotateCcw size={16} aria-hidden="true" />{i18n.t('library.resumeMigration')}
      </button>
      <button class="button-danger" type="button" disabled={app.loading} onclick={rollbackProjectMigration}>
        <RotateCcw size={16} aria-hidden="true" />{i18n.t('library.rollbackMigration')}
      </button>
      <button class="button-quiet" type="button" onclick={() => showMigration = false}>{i18n.t('common.cancel')}</button>
      <button class="button-primary" type="button" disabled={app.loading || !createNameValid} onclick={migrateProject}>
        <ArrowRightLeft size={16} aria-hidden="true" />{i18n.t('library.chooseMigrationPreview')}
      </button>
    </div>
  </section>
{/if}

{#if !app.snapshot}
  <section
    class="surface loading"
    role="status"
    aria-busy="true"
    aria-live="polite"
    aria-atomic="true"
  >
    <div class="skeleton wide"></div>
    <div class="skeleton"></div>
    <p>{i18n.t('library.loading')}</p>
  </section>
{:else if app.snapshot.researchLibrary.health === 'inspection-blocked'}
  <section class="surface state-panel state-danger">
    <AlertTriangle size={24} aria-hidden="true" />
    <div>
      <h2>{i18n.t('library.blocked')}</h2>
      <p>{i18n.t('library.blockedDetail')}</p>
    </div>
  </section>
{:else}
  <section class="metrics" aria-label={i18n.t('library.summaryAria')}>
    <article class="surface metric">
      <span class="metric-icon"><BookOpenText size={18} aria-hidden="true" /></span>
      <div><strong>{projects.length}</strong><span>{i18n.t('library.projects')}</span></div>
    </article>
    <article class="surface metric">
      <span class="metric-icon positive"><CheckCircle2 size={18} aria-hidden="true" /></span>
      <div><strong>{activeCount}</strong><span>{i18n.t('library.active')}</span></div>
    </article>
    <article class="surface metric">
      <span class:warning={attentionCount > 0} class="metric-icon"><AlertTriangle size={18} aria-hidden="true" /></span>
      <div><strong>{attentionCount}</strong><span>{i18n.t('library.attention')}</span></div>
    </article>
    <article class="surface metric">
      <span class="metric-icon"><CircleGauge size={18} aria-hidden="true" /></span>
      <div><strong>{app.snapshot.researchLibrary.revision}</strong><span>{i18n.t('library.revision')}</span></div>
    </article>
  </section>

  {#if projects.length === 0}
    <section class="surface empty-state">
      <span><FileQuestion size={27} aria-hidden="true" /></span>
      <h2>{i18n.t('library.emptyTitle')}</h2>
      <p>{i18n.t('library.emptyDetail')}</p>
      <div class="empty-actions">
        <button
          class="button-primary"
          type="button"
          disabled={app.loading}
          onclick={() => {
            showCreate = true;
            showMigration = false;
          }}
        >
          <Plus size={16} aria-hidden="true" />{i18n.t('library.create')}
        </button>
        <button class="button-secondary" type="button" disabled={app.loading} onclick={registerProject}>
          <FolderPlus size={16} aria-hidden="true" />{i18n.t('library.chooseExisting')}
        </button>
        <button class="button-secondary" type="button" disabled={app.loading} onclick={importProject}>
          <PackageOpen size={16} aria-hidden="true" />{i18n.t('library.import')}
        </button>
        <button
          class="button-secondary"
          type="button"
          disabled={app.loading}
          onclick={() => {
            showMigration = true;
            showCreate = false;
          }}
        >
          <ArrowRightLeft size={16} aria-hidden="true" />{i18n.t('library.migrate')}
        </button>
      </div>
    </section>
  {:else}
    <section class="surface library">
      <div class="library-heading">
        <div>
          <p class="eyebrow">{i18n.t('library.index')}</p>
          <h2>{i18n.t('library.academicProjects')}</h2>
          <p>{i18n.t('library.sorted')}</p>
        </div>
        <StatusBadge
          status={app.snapshot.researchLibrary.health === 'ready' ? 'ready' : 'recovery-required'}
          label={sentence(app.snapshot.researchLibrary.health)}
        />
      </div>

      <div class="controls">
        <label class="search-control">
          <span class="sr-only">{i18n.t('library.search')}</span>
          <Search size={17} aria-hidden="true" />
          <input bind:value={query} type="search" placeholder={i18n.t('library.searchPlaceholder')} />
        </label>
        <label>
          <span>{i18n.t('library.lifecycle')}</span>
          <select bind:value={lifecycle}>
            <option value="all">{i18n.t('library.all')}</option>
            <option value="active">{i18n.t('library.active')}</option>
            <option value="archived">{i18n.t('library.archived')}</option>
            <option value="attention">{i18n.t('library.attention')}</option>
          </select>
        </label>
        <label>
          <span>{i18n.t('library.sort')}</span>
          <select bind:value={sort}>
            <option value="academically-updated">{i18n.label('academically-updated')}</option>
            <option value="name">{i18n.t('library.projectName')}</option>
            <option value="stage">{i18n.label('research-stage')}</option>
          </select>
        </label>
      </div>

      {#if visibleProjects.length === 0}
        <div class="no-results">
          <Search size={21} aria-hidden="true" />
          <p>{i18n.t('library.noResults')}</p>
        </div>
      {:else}
        <div class="project-list">
          {#each visibleProjects as project (project.projectId)}
            <article class:selected={projectWorkspace.projectId === project.projectId}>
              <button
                class="project-main"
                type="button"
                onclick={() => void projectWorkspace.selectProject(project.projectId)}
              >
                <span class="project-title">
                  <strong>{project.displayName}</strong>
                  <small>{project.rootLabel}</small>
                </span>
                <span class="project-tags">
                  <span>{sentence(project.projectKind)}</span>
                  <span>{sentence(project.stage)}</span>
                  {#if project.lifecycle === 'archived'}<span><Archive size={12} aria-hidden="true" />{i18n.t('library.archived')}</span>{/if}
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
            <p class="eyebrow">{i18n.t('library.overview')}</p>
            <h2>{selectedProject.displayName}</h2>
            <p><code>{selectedProject.projectId}</code> · {i18n.t('library.next', { action: sentence(selectedProject.nextAction) })}</p>
          </div>
          <div class="overview-actions">
            {#if selectedProject.health === 'ready' || selectedProject.health === 'revision-drift'}
              <a
                class="button-primary"
                href={projectWorkspace.href('/academic-graph', selectedProject.projectId)}
              >
                <Network size={15} aria-hidden="true" />{i18n.t('projectWorkspace.explore')}
              </a>
              <button class="button-secondary" type="button" disabled={app.loading} onclick={() => openProject(selectedProject)}>
                <FolderOpen size={15} aria-hidden="true" />{i18n.t('projectWorkspace.reveal')}
              </button>
              <button class="button-secondary" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'refresh')}>
                <RefreshCw size={15} aria-hidden="true" />{i18n.t('library.refreshRevision')}
              </button>
              <button class="button-secondary" type="button" disabled={app.loading} onclick={() => exportProject(selectedProject)}>
                <Package size={15} aria-hidden="true" />{i18n.t('library.export')}
              </button>
            {:else if selectedProject.health === 'missing-manifest'}
              <button class="button-primary" type="button" disabled={app.loading} onclick={() => repairManifest(selectedProject)}>
                <Stethoscope size={15} aria-hidden="true" />{i18n.t('library.repair')}
              </button>
            {/if}
            {#if selectedProject.health === 'ready' || selectedProject.health === 'revision-drift'}
              {#if selectedProject.lifecycle === 'active'}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'archive')}>
                  <Archive size={15} aria-hidden="true" />{i18n.t('library.archive')}
                </button>
              {:else}
                <button class="button-secondary" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'restore')}>
                  <CheckCircle2 size={15} aria-hidden="true" />{i18n.t('library.restore')}
                </button>
              {/if}
            {/if}
          </div>
        </div>

        <div class="overview-grid">
          <article>
            <span>{i18n.t('library.focal')}</span>
            <p>{selectedProject.overview.focalQuestion ?? i18n.t('library.notRecorded')}</p>
          </article>
          <article>
            <span>{i18n.t('library.thesis')}</span>
            <p>{selectedProject.overview.thesis ?? i18n.t('library.notRecorded')}</p>
          </article>
          <article>
            <span>{i18n.t('library.evidence')}</span>
            <p>{selectedProject.overview.evidencePosition ?? i18n.t('library.noEvidence')}</p>
          </article>
          <article class="evidence-card">
            <span>{i18n.t('library.coverage')}</span>
            <strong>{selectedProject.overview.claimEvidenceCoveragePercent === null ? '—' : `${selectedProject.overview.claimEvidenceCoveragePercent}%`}</strong>
            <div class="progress" aria-hidden="true"><i style:width={`${selectedProject.overview.claimEvidenceCoveragePercent ?? 0}%`}></i></div>
            <small>{i18n.t('library.risks', { count: selectedProject.overview.unresolvedRiskCount })}</small>
          </article>
        </div>

        <div class="priorities">
          <span>{i18n.t('library.priorities')}</span>
          {#if selectedProject.overview.nextPriorities.length === 0}
            <p>{i18n.t('library.noPriorities')}</p>
          {:else}
            <ol>
              {#each selectedProject.overview.nextPriorities as priority}<li>{priority}</li>{/each}
            </ol>
          {/if}
        </div>
        <div class="danger-zone">
          <div>
            <strong>{i18n.t('library.removeTitle')}</strong>
            <p>{i18n.t('library.removeDetail')}</p>
          </div>
          <button class="button-danger" type="button" disabled={app.loading} onclick={() => previewProject(selectedProject, 'unregister')}>{i18n.t('library.unregister')}</button>
        </div>
      </section>
    {/if}
  {/if}
{/if}

<style>
  .create-panel { display: grid; grid-template-columns: minmax(230px, 1.3fr) minmax(220px, 1fr) 150px 150px auto; align-items: end; gap: 9px; margin-bottom: 12px; padding: 13px; border-left: 3px solid var(--color-accent); }
  .create-panel h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .create-panel > div:first-child > p:last-child { margin: 6px 0 0; color: var(--color-muted); font-size: 11px; line-height: 1.5; }
  .create-panel label { display: grid; gap: 5px; }
  .create-panel label > span { color: var(--color-muted); font-size: 10px; font-weight: 800; letter-spacing: 0.05em; text-transform: uppercase; }
  .create-actions, .empty-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 7px; }
  .create-actions { justify-content: flex-end; }
  .migration-panel { border-left-color: var(--color-warning); }
  .migration-actions { min-width: 300px; }
  .loading { min-height: 220px; padding: 30px; }
  .loading p { color: var(--color-muted); }
  .skeleton { width: 42%; height: 18px; margin-bottom: 14px; border-radius: 6px; background: #e2e8f0; }
  .skeleton.wide { width: 68%; height: 30px; }

  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; margin-bottom: 10px; }
  .header-actions-menu { position: relative; min-width: 0; }
  .header-actions-menu > summary { width: 100%; cursor: pointer; list-style: none; }
  .header-actions-menu > summary::-webkit-details-marker { display: none; }
  .header-actions-menu > div { position: absolute; top: calc(100% + 6px); right: 0; z-index: 30; display: grid; width: min(240px, calc(100vw - 24px)); gap: 3px; padding: 6px; box-shadow: var(--shadow-overlay); }
  .header-actions-menu > div button { display: flex; min-height: 44px; align-items: center; gap: 8px; border: 0; border-radius: 7px; padding: 8px 10px; color: var(--color-ink); background: transparent; font-size: 11px; font-weight: 700; text-align: left; white-space: nowrap; }
  .header-actions-menu > div button:hover:not(:disabled) { color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .metric { display: flex; min-height: 62px; align-items: center; gap: 9px; padding: 10px; }
  .metric-icon { display: grid; width: 34px; height: 34px; flex: none; place-items: center; border-radius: 5px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .metric-icon.positive { color: var(--color-success); background: var(--color-success-soft); }
  .metric-icon.warning { color: var(--color-warning); background: var(--color-warning-soft); }
  .metric strong, .metric span { display: block; }
  .metric strong { color: var(--color-ink-strong); font-size: 21px; line-height: 1; }
  .metric div span { margin-top: 5px; color: var(--color-muted); font-size: 11px; font-weight: 560; }

  .state-panel { display: flex; align-items: flex-start; gap: 14px; padding: 22px; }
  .state-danger { border-color: #fecaca; color: var(--color-danger); background: var(--color-danger-soft); }
  .state-panel h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .state-panel p { margin: 7px 0 0; color: var(--color-muted); font-size: 13px; line-height: 1.6; }

  .empty-state { padding: 32px 20px; text-align: center; }
  .empty-state > span { display: grid; width: 46px; height: 46px; place-items: center; margin: 0 auto 16px; border-radius: 6px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .empty-state h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; }
  .empty-state p { max-width: 650px; margin: 10px auto 0; color: var(--color-muted); font-size: 13px; line-height: 1.65; }
  .empty-actions { justify-content: center; margin-top: 18px; }
  code { overflow-wrap: anywhere; }

  .library { padding: 15px; }
  .library-heading, .overview-title { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; }
  .library-heading h2, .overview-title h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; letter-spacing: -0.02em; }
  .library-heading > div > p:last-child, .overview-title > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }

  .controls { display: grid; grid-template-columns: minmax(240px, 1fr) 150px 160px; gap: 8px; margin-top: 12px; padding: 9px; border: 1px solid var(--color-border); border-radius: 6px; background: var(--color-surface-subtle); }
  .controls label:not(.search-control) { display: grid; gap: 5px; }
  .controls label > span { color: var(--color-muted); font-size: 10px; font-weight: 620; letter-spacing: 0.02em; }
  .search-control { display: flex; min-height: 44px; align-items: center; gap: 9px; align-self: end; border: 1px solid var(--color-border-strong); border-radius: 5px; padding: 0 11px; color: var(--color-muted); background: var(--color-surface); }
  input, select { width: 100%; min-height: 44px; border: 1px solid var(--color-border-strong); border-radius: 5px; padding: 8px 10px; color: var(--color-ink); background: var(--color-surface); font: inherit; font-size: 12px; }
  .search-control input { min-height: 42px; border: 0; padding: 0; }

  .project-list { margin-top: 14px; border-top: 1px solid var(--color-border); }
  .project-list article { border-bottom: 1px solid var(--color-border); }
  .project-list article.selected { margin-inline: -8px; border: 1px solid #aac5be; border-radius: 6px; background: var(--color-accent-soft); }
  .project-main { display: grid; width: 100%; min-height: 62px; grid-template-columns: minmax(190px, 1.4fr) minmax(160px, 1fr) 100px auto auto; align-items: center; gap: 11px; border: 0; padding: 8px 6px; color: inherit; background: transparent; text-align: left; cursor: pointer; }
  .project-list article.selected .project-main { padding-inline: 14px; }
  .project-main:hover { background: rgb(241 245 249 / 0.72); }
  .project-title strong, .project-title small, .revision strong, .revision small { display: block; }
  .project-title strong { color: var(--color-ink-strong); font-size: 13px; }
  .project-title small, .revision small { margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  .project-tags { display: flex; flex-wrap: wrap; gap: 5px; }
  .project-tags span { display: inline-flex; max-width: 100%; align-items: center; gap: 4px; overflow: hidden; border: 1px solid var(--color-border); border-radius: 4px; padding: 3px 7px; color: var(--color-muted); background: var(--color-surface); font-size: 10px; font-weight: 560; text-overflow: ellipsis; white-space: nowrap; }
  .revision { text-align: right; }
  .revision strong { color: var(--color-ink); font-size: 12px; }
  .no-results { display: flex; min-height: 130px; align-items: center; justify-content: center; gap: 10px; color: var(--color-muted); }

  .overview { margin-top: 10px; padding: 15px; border-left: 2px solid var(--color-accent); }
  .overview-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 7px; }
  .overview-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 11px; margin-top: 19px; }
  .overview-grid article, .priorities { border: 1px solid var(--color-border); border-radius: 6px; padding: 15px; background: var(--color-surface-subtle); }
  .overview-grid article > span, .priorities > span { color: var(--color-accent-strong); font-size: 10px; font-weight: 620; letter-spacing: 0.02em; }
  .overview-grid p, .priorities p, .priorities ol { margin: 8px 0 0; color: var(--color-ink); font-size: 12px; line-height: 1.6; }
  .evidence-card strong { display: block; margin-top: 10px; color: var(--color-ink-strong); font-size: 24px; }
  .evidence-card small { display: block; margin-top: 8px; color: var(--color-muted); font-size: 11px; }
  .progress { height: 6px; margin-top: 8px; overflow: hidden; border-radius: 999px; background: #cbd5e1; }
  .progress i { display: block; height: 100%; border-radius: inherit; background: var(--color-accent); }
  .priorities { margin-top: 11px; }
  .priorities ol { padding-left: 20px; }
  .priorities li + li { margin-top: 5px; }
  .danger-zone { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 11px; border: 1px solid #d9b5ad; border-radius: 6px; padding: 14px 15px; background: var(--color-danger-soft); }
  .danger-zone strong { color: #991b1b; font-size: 12px; }
  .danger-zone p { margin: 4px 0 0; color: var(--color-muted); font-size: 11px; }
  .sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }

  @media (max-width: 1200px) {
    .create-panel { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .create-panel > div:first-child, .create-name, .create-actions { grid-column: 1 / -1; }
    .project-main { grid-template-columns: minmax(180px, 1.2fr) minmax(140px, 1fr) auto auto; }
    .revision { display: none; }
  }

  @media (max-width: 900px) {
    .controls { grid-template-columns: 1fr 1fr; }
    .search-control { grid-column: 1 / -1; }
  }

  @media (max-width: 760px) {
    .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
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
