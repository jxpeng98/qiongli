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
  import { MetricCard, MetricGrid, PageHeader, StatePanel, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import { Input } from '$lib/components/ui/input';
  import { NativeSelect } from '$lib/components/ui/native-select';
  import { Progress } from '$lib/components/ui/progress';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  const projectWorkspace = useProjectWorkspace();

  let query = $state('');
  let lifecycle = $state<ProjectLifecycleFilter>('all');
  let sort = $state<ProjectSort>('academically-updated');
  let showCreate = $state(false);
  let showMigration = $state(false);
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

<svelte:head>
  <title>{i18n.t('library.title')} · {i18n.t('app.name')}</title>
</svelte:head>

<PageHeader
  eyebrow={i18n.t('library.eyebrow')}
  title={i18n.t('library.title')}
  description={i18n.t('library.description')}
>
  {#snippet actions()}
    <Button
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={() => {
        showCreate = !showCreate;
        showMigration = false;
      }}
    >
      <Plus size={16} aria-hidden="true" />
      {i18n.t('library.newProject')}
    </Button>
    <Button
      variant="outline"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
      onclick={registerProject}
    >
      <FolderPlus size={16} aria-hidden="true" />
      {i18n.t('library.register')}
    </Button>
    <Button
      variant="outline"
      type="button"
      disabled={app.loading || !app.snapshot?.capabilities.projectLibrary}
      onclick={refreshLibrary}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
      {i18n.t('common.refresh')}
    </Button>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Button variant="outline" {...props}>
            <Ellipsis size={16} aria-hidden="true" />
            {i18n.t('library.moreActions')}
          </Button>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Item
          disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
          onclick={() => void importProject()}
        >
          <PackageOpen size={16} aria-hidden="true" />
          {i18n.t('library.import')}
        </DropdownMenu.Item>
        <DropdownMenu.Item
          disabled={app.loading || !app.snapshot?.capabilities.projectMutation}
          onclick={() => {
            showMigration = true;
            showCreate = false;
          }}
        >
          <ArrowRightLeft size={16} aria-hidden="true" />
          {i18n.t('library.migrate')}
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  {/snippet}
</PageHeader>

<Dialog.Root bind:open={showCreate}>
  <Dialog.Content class="create-dialog" aria-label={i18n.t('library.createAria')}>
    <Dialog.Header>
      <p class="eyebrow">{i18n.t('library.projectEyebrow')}</p>
      <Dialog.Title>{i18n.t('library.createTitle')}</Dialog.Title>
      <Dialog.Description>{i18n.t('library.createHelp')}</Dialog.Description>
    </Dialog.Header>
    <div class="create-fields">
      <label class="create-name">
        <span>{i18n.t('library.projectName')}</span>
        <Input bind:value={createName} maxlength={160} placeholder={i18n.t('library.createNamePlaceholder')} />
      </label>
      <label>
        <span>{i18n.t('library.type')}</span>
        <NativeSelect bind:value={createKind}>
          <option value="article">{i18n.label('article')}</option>
          <option value="review">{i18n.label('review')}</option>
          <option value="dissertation-article">{i18n.label('dissertation-article')}</option>
          <option value="manuscript">{i18n.label('manuscript')}</option>
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('library.stage')}</span>
        <NativeSelect bind:value={createStage}>
          <option value="idea">{i18n.label('idea')}</option>
          <option value="framing">{i18n.label('framing')}</option>
          <option value="literature">{i18n.label('literature')}</option>
          <option value="design">{i18n.label('design')}</option>
          <option value="analysis">{i18n.label('analysis')}</option>
          <option value="writing">{i18n.label('writing')}</option>
          <option value="review">{i18n.label('review')}</option>
          <option value="submission">{i18n.label('submission')}</option>
        </NativeSelect>
      </label>
    </div>
    <Dialog.Footer>
      <Button variant="ghost" onclick={() => showCreate = false}>{i18n.t('common.cancel')}</Button>
      <Button disabled={app.loading || !createNameValid} onclick={createProject}>
        <FolderPlus size={16} aria-hidden="true" />{i18n.t('library.choosePreview')}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={showMigration}>
  <Dialog.Content class="migration-dialog sm:max-w-2xl" aria-label={i18n.t('library.migrateTitle')}>
    <Dialog.Header>
      <p class="eyebrow">{i18n.t('library.migrationEyebrow')}</p>
      <Dialog.Title>{i18n.t('library.migrateTitle')}</Dialog.Title>
      <Dialog.Description>{i18n.t('library.migrateHelp')} {i18n.t('library.rollbackHelp')}</Dialog.Description>
    </Dialog.Header>
    <div class="create-fields">
      <label class="create-name">
        <span>{i18n.t('library.projectName')}</span>
        <Input bind:value={createName} maxlength={160} placeholder={i18n.t('library.migrateNamePlaceholder')} />
      </label>
      <label>
        <span>{i18n.t('library.type')}</span>
        <NativeSelect bind:value={createKind}>
          <option value="article">{i18n.label('article')}</option>
          <option value="review">{i18n.label('review')}</option>
          <option value="dissertation-article">{i18n.label('dissertation-article')}</option>
          <option value="manuscript">{i18n.label('manuscript')}</option>
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('library.stage')}</span>
        <NativeSelect bind:value={createStage}>
          <option value="idea">{i18n.label('idea')}</option>
          <option value="framing">{i18n.label('framing')}</option>
          <option value="literature">{i18n.label('literature')}</option>
          <option value="design">{i18n.label('design')}</option>
          <option value="analysis">{i18n.label('analysis')}</option>
          <option value="writing">{i18n.label('writing')}</option>
          <option value="review">{i18n.label('review')}</option>
          <option value="submission">{i18n.label('submission')}</option>
        </NativeSelect>
      </label>
    </div>
    <Dialog.Footer class="migration-actions">
      <Button variant="ghost" disabled={app.loading} onclick={recoverProjectMigration}>
        <RotateCcw size={16} aria-hidden="true" />{i18n.t('library.resumeMigration')}
      </Button>
      <Button variant="destructive" disabled={app.loading} onclick={rollbackProjectMigration}>
        <RotateCcw size={16} aria-hidden="true" />{i18n.t('library.rollbackMigration')}
      </Button>
      <Button variant="ghost" onclick={() => showMigration = false}>{i18n.t('common.cancel')}</Button>
      <Button disabled={app.loading || !createNameValid} onclick={migrateProject}>
        <ArrowRightLeft size={16} aria-hidden="true" />{i18n.t('library.chooseMigrationPreview')}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

{#if !app.snapshot}
  <StatePanel role="status" busy live="polite" atomic>
    {#snippet children()}
      <div class="skeleton wide"></div>
      <div class="skeleton"></div>
      <p>{i18n.t('library.loading')}</p>
    {/snippet}
  </StatePanel>
{:else if app.snapshot.researchLibrary.health === 'inspection-blocked'}
  <StatePanel tone="danger" role="alert" title={i18n.t('library.blocked')} description={i18n.t('library.blockedDetail')}>
    {#snippet icon()}<AlertTriangle size={24} />{/snippet}
  </StatePanel>
{:else}
  <div class="metrics-wrap">
    <MetricGrid label={i18n.t('library.summaryAria')}>
      <MetricCard value={projects.length} label={i18n.t('library.projects')}>
        {#snippet icon()}<BookOpenText size={18} />{/snippet}
      </MetricCard>
      <MetricCard value={activeCount} label={i18n.t('library.active')} tone="success">
        {#snippet icon()}<CheckCircle2 size={18} />{/snippet}
      </MetricCard>
      <MetricCard value={attentionCount} label={i18n.t('library.attention')} tone={attentionCount > 0 ? 'warning' : 'neutral'}>
        {#snippet icon()}<AlertTriangle size={18} />{/snippet}
      </MetricCard>
      <MetricCard value={app.snapshot.researchLibrary.revision} label={i18n.t('library.revision')}>
        {#snippet icon()}<CircleGauge size={18} />{/snippet}
      </MetricCard>
    </MetricGrid>
  </div>

  {#if projects.length === 0}
    <StatePanel centered title={i18n.t('library.emptyTitle')} description={i18n.t('library.emptyDetail')}>
      {#snippet icon()}<FileQuestion size={27} />{/snippet}
      {#snippet actions()}
        <Button
          type="button"
          disabled={app.loading}
          onclick={() => {
            showCreate = true;
            showMigration = false;
          }}
        >
          <Plus size={16} aria-hidden="true" />{i18n.t('library.create')}
        </Button>
        <Button variant="outline" disabled={app.loading} onclick={registerProject}>
          <FolderPlus size={16} aria-hidden="true" />{i18n.t('library.chooseExisting')}
        </Button>
        <Button variant="outline" disabled={app.loading} onclick={importProject}>
          <PackageOpen size={16} aria-hidden="true" />{i18n.t('library.import')}
        </Button>
        <Button
          variant="outline"
          type="button"
          disabled={app.loading}
          onclick={() => {
            showMigration = true;
            showCreate = false;
          }}
        >
          <ArrowRightLeft size={16} aria-hidden="true" />{i18n.t('library.migrate')}
        </Button>
      {/snippet}
    </StatePanel>
  {:else}
    <Card.Root class="library" role="region" aria-label={i18n.t('library.academicProjects')}>
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
          <Input bind:value={query} type="search" placeholder={i18n.t('library.searchPlaceholder')} />
        </label>
        <label>
          <span>{i18n.t('library.lifecycle')}</span>
          <NativeSelect bind:value={lifecycle}>
            <option value="all">{i18n.t('library.all')}</option>
            <option value="active">{i18n.t('library.active')}</option>
            <option value="archived">{i18n.t('library.archived')}</option>
            <option value="attention">{i18n.t('library.attention')}</option>
          </NativeSelect>
        </label>
        <label>
          <span>{i18n.t('library.sort')}</span>
          <NativeSelect bind:value={sort}>
            <option value="academically-updated">{i18n.label('academically-updated')}</option>
            <option value="name">{i18n.t('library.projectName')}</option>
            <option value="stage">{i18n.label('research-stage')}</option>
          </NativeSelect>
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
              <Button
                variant="ghost"
                class="project-main"
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
              </Button>
            </article>
          {/each}
        </div>
      {/if}
    </Card.Root>

    {#if selectedProject}
      <Card.Root class="overview" role="region" aria-live="polite" aria-label={i18n.t('library.overview')}>
        <div class="overview-title">
          <div>
            <p class="eyebrow">{i18n.t('library.overview')}</p>
            <h2>{selectedProject.displayName}</h2>
            <p><code>{selectedProject.projectId}</code> · {i18n.t('library.next', { action: sentence(selectedProject.nextAction) })}</p>
          </div>
          <div class="overview-actions">
            {#if selectedProject.health === 'ready' || selectedProject.health === 'revision-drift'}
              <Button
                href={projectWorkspace.href('/academic-graph', selectedProject.projectId)}
              >
                <Network size={15} aria-hidden="true" />{i18n.t('projectWorkspace.explore')}
              </Button>
              <Button variant="outline" disabled={app.loading} onclick={() => openProject(selectedProject)}>
                <FolderOpen size={15} aria-hidden="true" />{i18n.t('projectWorkspace.reveal')}
              </Button>
              <Button variant="outline" disabled={app.loading} onclick={() => previewProject(selectedProject, 'refresh')}>
                <RefreshCw size={15} aria-hidden="true" />{i18n.t('library.refreshRevision')}
              </Button>
              <Button variant="outline" disabled={app.loading} onclick={() => exportProject(selectedProject)}>
                <Package size={15} aria-hidden="true" />{i18n.t('library.export')}
              </Button>
            {:else if selectedProject.health === 'missing-manifest'}
              <Button disabled={app.loading} onclick={() => repairManifest(selectedProject)}>
                <Stethoscope size={15} aria-hidden="true" />{i18n.t('library.repair')}
              </Button>
            {/if}
            {#if selectedProject.health === 'ready' || selectedProject.health === 'revision-drift'}
              {#if selectedProject.lifecycle === 'active'}
                <Button variant="outline" disabled={app.loading} onclick={() => previewProject(selectedProject, 'archive')}>
                  <Archive size={15} aria-hidden="true" />{i18n.t('library.archive')}
                </Button>
              {:else}
                <Button variant="outline" disabled={app.loading} onclick={() => previewProject(selectedProject, 'restore')}>
                  <CheckCircle2 size={15} aria-hidden="true" />{i18n.t('library.restore')}
                </Button>
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
            <Progress value={selectedProject.overview.claimEvidenceCoveragePercent ?? 0} aria-label={i18n.t('library.coverage')} />
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
          <Button variant="destructive" disabled={app.loading} onclick={() => previewProject(selectedProject, 'unregister')}>{i18n.t('library.unregister')}</Button>
        </div>
      </Card.Root>
    {/if}
  {/if}
{/if}

<style>
  :global(.create-dialog), :global(.migration-dialog) { max-height: min(88vh, 760px); overflow-y: auto; }
  .create-fields { display: grid; grid-template-columns: minmax(0, 1fr) minmax(140px, .55fr); gap: 12px; }
  .create-fields label { display: grid; gap: 6px; min-width: 0; }
  .create-fields label > span { color: var(--color-muted); font-size: 10px; font-weight: 800; letter-spacing: 0.05em; text-transform: uppercase; }
  .create-name { grid-column: 1 / -1; }
  :global(.migration-actions) { flex-wrap: wrap; }
  .skeleton { width: 42%; height: 18px; margin-bottom: 14px; border-radius: 6px; background: var(--color-skeleton); }
  .skeleton.wide { width: 68%; height: 30px; }

  .metrics-wrap { margin-bottom: 10px; }
  code { overflow-wrap: anywhere; }

  :global(.library) { padding: 15px; }
  .library-heading, .overview-title { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; }
  .library-heading h2, .overview-title h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; letter-spacing: -0.02em; }
  .library-heading > div > p:last-child, .overview-title > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }

  .controls { display: grid; grid-template-columns: minmax(240px, 1fr) 150px 160px; gap: 8px; margin-top: 12px; padding: 9px; border: 1px solid var(--color-border); border-radius: 6px; background: var(--color-surface-subtle); }
  .controls label:not(.search-control) { display: grid; gap: 5px; }
  .controls label > span { color: var(--color-muted); font-size: 10px; font-weight: 620; letter-spacing: 0.02em; }
  .search-control { display: flex; min-height: 44px; align-items: center; gap: 9px; align-self: end; border: 1px solid var(--color-border-strong); border-radius: 5px; padding: 0 11px; color: var(--color-muted); background: var(--color-surface); }
  .search-control :global([data-slot='input']) { min-height: 42px; border: 0; padding: 0; box-shadow: none; }

  .project-list { margin-top: 14px; border-top: 1px solid var(--color-border); }
  .project-list article { border-bottom: 1px solid var(--color-border); }
  .project-list article.selected { margin-inline: -8px; border: 1px solid var(--color-accent-border); border-radius: 6px; background: var(--color-accent-soft); }
  :global(.project-main) { display: grid; width: 100%; min-height: 62px; height: auto; grid-template-columns: minmax(190px, 1.4fr) minmax(160px, 1fr) 100px auto auto; align-items: center; gap: 11px; border: 0; padding: 8px 6px; color: inherit; background: transparent; text-align: left; white-space: normal; cursor: pointer; }
  .project-list article.selected :global(.project-main) { padding-inline: 14px; }
  :global(.project-main:hover) { background: var(--color-control-hover); }
  .project-title strong, .project-title small, .revision strong, .revision small { display: block; }
  .project-title strong { color: var(--color-ink-strong); font-size: 13px; }
  .project-title small, .revision small { margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  .project-tags { display: flex; flex-wrap: wrap; gap: 5px; }
  .project-tags span { display: inline-flex; max-width: 100%; align-items: center; gap: 4px; overflow: hidden; border: 1px solid var(--color-border); border-radius: 4px; padding: 3px 7px; color: var(--color-muted); background: var(--color-surface); font-size: 10px; font-weight: 560; text-overflow: ellipsis; white-space: nowrap; }
  .revision { text-align: right; }
  .revision strong { color: var(--color-ink); font-size: 12px; }
  .no-results { display: flex; min-height: 130px; align-items: center; justify-content: center; gap: 10px; color: var(--color-muted); }

  :global(.overview) { margin-top: 10px; padding: 15px; border-left: 2px solid var(--color-accent); }
  .overview-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 7px; }
  .overview-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 11px; margin-top: 19px; }
  .overview-grid article, .priorities { border: 1px solid var(--color-border); border-radius: 6px; padding: 15px; background: var(--color-surface-subtle); }
  .overview-grid article > span, .priorities > span { color: var(--color-accent-strong); font-size: 10px; font-weight: 620; letter-spacing: 0.02em; }
  .overview-grid p, .priorities p, .priorities ol { margin: 8px 0 0; color: var(--color-ink); font-size: 12px; line-height: 1.6; }
  .evidence-card strong { display: block; margin-top: 10px; color: var(--color-ink-strong); font-size: 24px; }
  .evidence-card small { display: block; margin-top: 8px; color: var(--color-muted); font-size: 11px; }
  .evidence-card :global([data-slot='progress']) { margin-top: 8px; }
  .priorities { margin-top: 11px; }
  .priorities ol { padding-left: 20px; }
  .priorities li + li { margin-top: 5px; }
  .danger-zone { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 11px; border: 1px solid var(--color-danger-border); border-radius: 6px; padding: 14px 15px; background: var(--color-danger-soft); }
  .danger-zone strong { color: var(--color-danger); font-size: 12px; }
  .danger-zone p { margin: 4px 0 0; color: var(--color-muted); font-size: 11px; }
  .sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }

  @media (max-width: 1200px) {
    :global(.project-main) { grid-template-columns: minmax(180px, 1.2fr) minmax(140px, 1fr) auto auto; }
    .revision { display: none; }
  }

  @media (max-width: 1040px) {
    .controls { grid-template-columns: 1fr 1fr; }
    .search-control { grid-column: 1 / -1; }
  }

  @media (max-width: 1040px) {
    :global(.project-main) { grid-template-columns: 1fr auto; }
    .project-tags { grid-column: 1 / -1; grid-row: 2; }
    :global(.project-main) :global(.status) { grid-column: 1; grid-row: 3; justify-self: start; }
    :global(.project-main) > :last-child { grid-column: 2; grid-row: 1 / 4; }
    .overview-grid { grid-template-columns: 1fr; }
  }

  @media (max-width: 520px) {
    .create-fields { grid-template-columns: 1fr; }
    .create-name { grid-column: auto; }
    .controls { grid-template-columns: 1fr; }
    .search-control { grid-column: auto; }
    :global(.library), :global(.overview) { padding: 14px; }
    .library-heading, .overview-title, .danger-zone { align-items: flex-start; flex-direction: column; }
    .overview-actions { justify-content: flex-start; }
  }
</style>
