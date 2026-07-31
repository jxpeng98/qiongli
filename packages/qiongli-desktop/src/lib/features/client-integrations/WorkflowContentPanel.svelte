<script lang="ts">
  import { Boxes, CheckCircle2, FileText, FolderOpen, RefreshCw, SearchCheck, Shield, ShieldOff, Trash2, Wrench } from '@lucide/svelte';

  import type { AppIntent, AppSnapshot, ManagedSkillsTargetId } from '@qiongli/app-api';
  import type { AppState } from '$lib/app-state.svelte';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';
  import { ActionGroup, StatusBadge } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { NativeSelect } from '$lib/components/ui/native-select';

  let { appState }: { appState?: AppState } = $props();
  const contextApp = useAppState();
  let app = $derived(appState ?? contextApp);
  type DestinationScope = 'qiongli-managed' | 'registered-project' | 'custom-folder';
  let selectedProfile = $state<'skill-only' | 'marketplace-lite' | 'full'>('marketplace-lite');
  let selectedScope = $state<DestinationScope>('qiongli-managed');
  let selectedProjectId = $state<string | null>(null);
  type ManagedDestination = AppSnapshot['content']['managedSkills']['destinations'][number];

  let profileLabels = $derived({
    'skill-only': i18n.t('content.profile.skills'),
    'marketplace-lite': i18n.t('content.profile.lite'),
    full: i18n.t('content.profile.full')
  } as const);
  let scopeLabels = $derived({
    'qiongli-managed': i18n.t('content.preset.managed'),
    'registered-project': i18n.t('content.preset.project'),
    'custom-folder': i18n.t('content.preset.custom')
  } as const);
  let projectDestinations = $derived(
    app.snapshot?.content.managedSkills.destinations.filter(
      (destination) => destination.preset === 'current-project' && destination.projectId !== null
    ) ?? []
  );
  let registeredProjects = $derived(
    app.snapshot?.researchLibrary.projects.filter((project) =>
      projectDestinations.some((destination) => destination.projectId === project.projectId)
    ) ?? []
  );
  let visibleManagedDestinations = $derived(
    app.snapshot?.content.managedSkills.destinations.filter((destination) =>
      destination.preset !== 'current-project'
      || destination.state !== 'missing'
      || (
        selectedScope === 'registered-project'
        && destination.projectId === selectedProjectId
      )
    ) ?? []
  );
  let scopeOptions = $derived.by((): DestinationScope[] => {
    const options: DestinationScope[] = ['qiongli-managed'];
    if (projectDestinations.length > 0) options.push('registered-project');
    options.push('custom-folder');
    return options;
  });
  let destinationLabels = $derived({
    'qiongli-managed': scopeLabels['qiongli-managed'],
    'current-project': scopeLabels['registered-project'],
    'custom-folder': i18n.t('content.preset.custom')
  } as const);
  let profileDescriptions = $derived({
    'skill-only': i18n.t('content.profileDescription.skills'),
    'marketplace-lite': i18n.t('content.profileDescription.lite'),
    full: i18n.t('content.profileDescription.full')
  } as const);
  let selectedDestination = $derived.by((): ManagedDestination | null => {
    if (!app.snapshot) return null;
    if (selectedScope === 'custom-folder') {
      return app.snapshot.content.managedSkills.destinations.find(
        (destination) => destination.targetId === app.selectedCustomSkillsTargetId
      ) ?? null;
    }
    if (selectedScope === 'registered-project') {
      return projectDestinations.find(
        (destination) => destination.projectId === selectedProjectId
      ) ?? null;
    }
    return app.snapshot.content.managedSkills.destinations.find(
      (destination) => destination.preset === 'qiongli-managed'
    ) ?? null;
  });
  let selectedProject = $derived(
    registeredProjects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let selectedStatus = $derived(selectedDestination?.status ?? 'missing');
  let selectedInstalled = $derived(
    selectedDestination !== null
      && ['current', 'update-available', 'drifted'].includes(selectedDestination.state)
  );
  let selectedProfileLocked = $derived(
    selectedInstalled && selectedDestination?.profile !== null
  );
  let destinationReady = $derived(
    selectedScope === 'registered-project'
      ? selectedProjectId !== null
      : selectedScope !== 'custom-folder' || app.selectedCustomSkillsTargetId !== null
  );
  let projectInstallReady = $derived(
    selectedScope !== 'registered-project'
      || (selectedProject?.lifecycle === 'active' && selectedProject.health === 'ready')
  );
  let canInstallSelected = $derived(
    app.snapshot?.capabilities.skillsMaterialize === true
      && (
        selectedScope === 'custom-folder'
          ? destinationReady
            && (selectedDestination === null || selectedDestination.state === 'missing')
          : destinationReady
            && projectInstallReady
            && selectedDestination?.state === 'missing'
      )
  );

  $effect(() => {
    if (!scopeOptions.includes(selectedScope)) selectedScope = 'qiongli-managed';
  });

  $effect(() => {
    if (registeredProjects.some((project) => project.projectId === selectedProjectId)) return;
    selectedProjectId = registeredProjects[0]?.projectId ?? null;
  });

  $effect(() => {
    const installedProfile = selectedDestination?.profile;
    if (installedProfile) selectedProfile = installedProfile;
  });

  function previewMaterialization(): Promise<unknown> {
    if (selectedScope === 'registered-project') {
      if (!selectedProjectId) return Promise.resolve(null);
      return app.execute({
        action: 'preview-project-skills-materialization',
        profile: selectedProfile,
        projectId: selectedProjectId
      });
    }
    return app.execute({
      action: 'preview-skills-preset-materialization',
      profile: selectedProfile,
      preset: selectedScope
    });
  }

  function verifyPreset(): Promise<unknown> {
    if (!selectedDestination) return Promise.resolve(null);
    return verifyTarget(selectedDestination.targetId);
  }

  function removePreset(): Promise<unknown> {
    if (!selectedDestination) return Promise.resolve(null);
    return removeTarget(selectedDestination.targetId);
  }

  function selectCustomDestination(): Promise<unknown> {
    return app.execute({ action: 'select-skills-destination' });
  }

  function verifyTarget(targetId: ManagedSkillsTargetId): Promise<unknown> {
    return app.execute({ action: 'verify-managed-skills-target', targetId });
  }

  function updateTarget(targetId: ManagedSkillsTargetId): Promise<unknown> {
    return app.execute({ action: 'preview-update-managed-skills-target', targetId });
  }

  function removeTarget(targetId: ManagedSkillsTargetId): Promise<unknown> {
    const intent: AppIntent = {
      action: 'preview-remove-managed-skills-target',
      targetId
    };
    return app.execute(intent);
  }

  function detachTarget(targetId: ManagedSkillsTargetId): Promise<unknown> {
    return app.execute({
      action: 'preview-detach-managed-skills-target',
      targetId
    });
  }

  function destinationActionName(destination: ManagedDestination): string {
    if (destination.projectId) {
      return registeredProjects.find(
        (project) => project.projectId === destination.projectId
      )?.displayName ?? i18n.t('content.preset.project');
    }
    const label = destinationLabels[destination.preset];
    return destination.preset === 'custom-folder'
      ? `${label} …${destination.targetId.slice(-8)}`
      : label;
  }
</script>

{#if app.snapshot}
  <Card.Root
    id="workflow-content"
    class="managed-content"
    role="region"
    aria-labelledby="workflow-content-title"
    aria-busy={app.loading}
  >
    <header>
      <span class="content-icon"><Boxes size={19} aria-hidden="true" /></span>
      <div>
        <p class="eyebrow">{i18n.t('content.advanced')}</p>
        <h2 id="workflow-content-title">{i18n.t('content.advancedTitle')}</h2>
        <p class="content-description">{i18n.t('content.advancedDescription')}</p>
      </div>
      <StatusBadge status={app.snapshot.content.status} />
    </header>

    <div class="content-summary">
      <span><strong>{app.snapshot.content.contentVersion}</strong>{i18n.t('common.version')}</span>
      <span><strong>{i18n.t('content.entries', { count: app.snapshot.content.entryCount })}</strong></span>
      <span><strong>{app.snapshot.mcp.publicToolCount}</strong>{i18n.t('content.tools')}</span>
    </div>

    <div class="content-controls">
      <label>
        <span class="label-line">
          <span>{i18n.t('content.destination')}</span>
          <StatusBadge status={selectedStatus} />
        </span>
        <NativeSelect bind:value={selectedScope} disabled={app.loading}>
          {#each scopeOptions as value}<option {value}>{scopeLabels[value]}</option>{/each}
        </NativeSelect>
      </label>
      {#if selectedScope === 'registered-project'}
        <label>
          {i18n.t('content.project')}
          <NativeSelect bind:value={selectedProjectId} disabled={app.loading || registeredProjects.length === 0}>
            {#each registeredProjects as project (project.projectId)}
              <option value={project.projectId}>{project.displayName}</option>
            {/each}
          </NativeSelect>
        </label>
      {/if}
      <label>
        {i18n.t('content.profile')}
        <NativeSelect bind:value={selectedProfile} disabled={app.loading || selectedProfileLocked}>
          {#each Object.entries(profileLabels) as [value, label]}<option {value}>{label}</option>{/each}
        </NativeSelect>
      </label>
      <ActionGroup class="content-actions">
        {#if selectedScope === 'custom-folder'}
          <Button variant="outline" disabled={app.loading} onclick={selectCustomDestination}>
            <FolderOpen size={14} aria-hidden="true" />
            {i18n.t(app.selectedCustomSkillsTargetId ? 'content.customSelected' : 'content.chooseCustom')}
          </Button>
        {/if}
        <Button disabled={app.loading || !canInstallSelected} onclick={previewMaterialization}>{i18n.t('content.previewInstall')}</Button>
        <Button variant="outline" disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize || selectedDestination?.state !== 'update-available'} onclick={() => selectedDestination && updateTarget(selectedDestination.targetId)}>{i18n.t('content.previewUpdate')}</Button>
        <Button variant="outline" disabled={app.loading || !selectedInstalled} onclick={verifyPreset}>{i18n.t('content.verify')}</Button>
        <Button variant="destructive" disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize || !selectedInstalled || selectedDestination?.state === 'drifted'} onclick={removePreset}>{i18n.t('content.previewRemove')}</Button>
        {#if selectedDestination?.state === 'drifted'}
          <Button variant="outline" disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize} onclick={() => detachTarget(selectedDestination.targetId)}>
            <ShieldOff size={14} aria-hidden="true" />
            {i18n.t('content.previewDetach')}
          </Button>
        {/if}
      </ActionGroup>
    </div>
    {#if selectedDestination?.state === 'drifted'}
      <p class="drift-guidance">{i18n.t('content.driftGuidance')}</p>
    {:else if selectedDestination?.state === 'unmanaged'}
      <p class="drift-guidance">{i18n.t('content.unmanagedGuidance')}</p>
    {:else if selectedScope === 'registered-project' && selectedDestination?.state === 'missing' && !projectInstallReady}
      <p class="drift-guidance">{i18n.t('content.projectInstallBlocked')}</p>
    {/if}

    <details class="profile-details">
      <summary><Wrench size={14} aria-hidden="true" />{i18n.t('content.chooseBoundary')}</summary>
      <div>
        {#each app.snapshot.content.profiles as profile}
          <article class:selected={selectedProfile === profile.id}>
            <span class="profile-icon">
              {#if profile.id === 'skill-only'}<FileText size={15} aria-hidden="true" />
              {:else if profile.id === 'marketplace-lite'}<Boxes size={15} aria-hidden="true" />
              {:else}<Shield size={15} aria-hidden="true" />{/if}
            </span>
            <span>
              <strong>{profileLabels[profile.id]}</strong>
              <small>{profileDescriptions[profile.id]}</small>
            </span>
            {#if selectedProfile === profile.id}<CheckCircle2 size={15} aria-label={i18n.t('common.selected')} />{/if}
          </article>
        {/each}
      </div>
    </details>

    <details class="managed-details">
      <summary><Shield size={14} aria-hidden="true" />{i18n.t('content.managedDestinations')}</summary>
      <div>
        {#each visibleManagedDestinations as destination (destination.targetId)}
          <article class="managed-destination">
            <span>
              <strong>{destinationActionName(destination)}</strong>
              <small>{destination.symbolicPath}</small>
              {#if destination.preset === 'custom-folder'}
                <code>…{destination.targetId.slice(-8)}</code>
              {/if}
            </span>
            <span class="destination-version">
              {destination.profile ? profileLabels[destination.profile] : i18n.t('content.notInstalled')}
              {#if destination.productVersion} · {destination.productVersion}{/if}
            </span>
            <StatusBadge status={destination.status} />
            {#if destination.state !== 'missing' && destination.state !== 'unmanaged'}
              <span class="destination-actions">
                <Button
                  class="icon-action"
                  variant="ghost"
                  size="icon"
                  disabled={app.loading}
                  aria-label={i18n.t('content.targetVerify', { destination: destinationActionName(destination) })}
                  onclick={() => verifyTarget(destination.targetId)}
                ><SearchCheck size={14} aria-hidden="true" /></Button>
                <Button
                  class="icon-action"
                  variant="ghost"
                  size="icon"
                  disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize || destination.state !== 'update-available'}
                  aria-label={i18n.t('content.targetUpdate', { destination: destinationActionName(destination) })}
                  onclick={() => updateTarget(destination.targetId)}
                ><RefreshCw size={14} aria-hidden="true" /></Button>
                {#if destination.state === 'drifted'}
                  <Button
                    class="icon-action"
                    variant="ghost"
                    size="icon"
                    disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize}
                    aria-label={i18n.t('content.targetDetach', { destination: destinationActionName(destination) })}
                    onclick={() => detachTarget(destination.targetId)}
                  ><ShieldOff size={14} aria-hidden="true" /></Button>
                {:else}
                  <Button
                    class="icon-action danger"
                    variant="ghost"
                    size="icon"
                    disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize}
                    aria-label={i18n.t('content.targetRemove', { destination: destinationActionName(destination) })}
                    onclick={() => removeTarget(destination.targetId)}
                  ><Trash2 size={14} aria-hidden="true" /></Button>
                {/if}
              </span>
            {/if}
          </article>
        {/each}
      </div>
    </details>
  </Card.Root>
{/if}

<style>
  :global(.managed-content) { overflow: hidden; margin-top: 14px; }
  header {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
  }
  .content-icon, .profile-icon {
    display: grid;
    flex: none;
    place-items: center;
    border-radius: var(--radius-control);
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }
  .content-icon { width: 34px; height: 34px; }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 14px; }
  .content-description {
    max-width: 680px;
    margin: 3px 0 0;
    color: var(--color-muted);
    font-size: var(--font-size-micro);
    line-height: 1.4;
  }
  .content-summary {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    border-block: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }
  .content-summary span {
    display: flex;
    min-width: 0;
    align-items: baseline;
    gap: 6px;
    border-right: 1px solid var(--color-border);
    padding: 8px 11px;
    color: var(--color-muted);
    font-size: var(--font-size-micro);
  }
  .content-summary span:last-child { border-right: 0; }
  .content-summary strong { color: var(--color-ink-strong); font-size: 11px; }
  .content-controls {
    display: grid;
    grid-template-columns: repeat(3, minmax(150px, 1fr));
    align-items: end;
    gap: 9px;
    padding: 11px 14px;
  }
  label { color: var(--color-muted); font-size: var(--font-size-micro); font-weight: 750; text-transform: uppercase; }
  .label-line {
    display: flex;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  label :global([data-slot='native-select-wrapper']) { width: 100%; margin-top: 4px; text-transform: none; }
  :global(.content-actions) {
    grid-column: 1 / -1;
    justify-content: flex-start;
  }
  :global(.content-actions) :global([data-slot='button']) { min-height: 44px; font-size: var(--font-size-label); white-space: nowrap; }
  .drift-guidance {
    margin: 0;
    border-top: 1px solid var(--color-border);
    padding: 9px 14px;
    color: var(--color-warning-strong);
    background: var(--color-warning-soft);
    font-size: var(--font-size-micro);
    line-height: 1.45;
  }
  .profile-details, .managed-details { border-top: 1px solid var(--color-border); padding: 0 14px; }
  .profile-details summary, .managed-details summary {
    display: flex;
    min-height: 44px;
    align-items: center;
    gap: 6px;
    width: fit-content;
    color: var(--color-accent-strong);
    cursor: pointer;
    font-size: var(--font-size-label);
    font-weight: 750;
  }
  .profile-details > div { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 7px; padding-bottom: 11px; }
  .managed-details > div { display: grid; gap: 6px; padding-bottom: 11px; }
  article {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: start;
    gap: 7px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    padding: 8px;
  }
  article.selected { border-color: var(--color-accent); }
  .profile-icon { width: 27px; height: 27px; }
  article strong, article small { display: block; }
  article strong { color: var(--color-ink-strong); font-size: var(--font-size-label); }
  article small { margin-top: 2px; color: var(--color-muted); font-size: var(--font-size-micro); line-height: 1.35; }
  .managed-destination {
    grid-template-columns: minmax(0, 1fr) auto auto auto;
    align-items: center;
  }
  .managed-destination small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .destination-version {
    color: var(--color-muted);
    font-size: var(--font-size-micro);
    white-space: nowrap;
  }
  .managed-destination code {
    display: block;
    margin-top: 3px;
    color: var(--color-muted);
    font-size: var(--font-size-micro);
  }
  .destination-actions { display: flex; align-items: center; gap: 4px; }
  :global(.icon-action) {
    width: 34px;
    height: 34px;
    color: var(--color-accent-strong);
  }
  :global(.icon-action.danger) { color: var(--color-danger); }
  @media (max-width: 850px) {
    .content-controls { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
  @media (max-width: 620px) {
    .content-controls, .profile-details > div { grid-template-columns: 1fr; }
    :global(.content-actions) { grid-column: auto; }
    :global(.content-actions) :global([data-slot='button']) { flex: 1 1 120px; }
    .managed-destination { grid-template-columns: minmax(0, 1fr) auto; }
    .destination-version { grid-column: 1; }
    .destination-actions { grid-column: 1 / -1; }
  }
  @media (max-width: 420px) {
    .content-summary { grid-template-columns: 1fr; }
    .content-summary span { border-right: 0; border-bottom: 1px solid var(--color-border); }
    .content-summary span:last-child { border-bottom: 0; }
  }
</style>
