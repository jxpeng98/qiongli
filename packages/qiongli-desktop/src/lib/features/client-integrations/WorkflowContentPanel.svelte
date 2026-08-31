<script lang="ts">
  import { Boxes, CheckCircle2, Eye, FileText, FolderOpen, RefreshCw, RotateCcw, Save, SearchCheck, Shield, ShieldOff, Trash2, Wrench } from '@lucide/svelte';

  import type { AppIntent, AppSnapshot, ContentCustomization, ManagedSkillsTargetId } from '@qiongli/app-api';
  import type { AppState } from '$lib/app-state.svelte';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';
  import { ActionGroup, DescriptionTip, StatusBadge } from '$lib/components/app';
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
  let customization = $state<ContentCustomization | null>(null);
  let customizationKey = $state('');
  let selectedPreviewPath = $state('');
  let resourceDraft = $state('');
  let resourceDraftKey = $state('');
  let reloadAfterVariantOperation = $state(false);
  let guidanceDraft = $state('');
  let showFullSource = $state(false);
  type ManagedDestination = AppSnapshot['content']['managedSkills']['destinations'][number];

  const GUIDANCE_STARTER = `# Qiongli Local Guidance

Add only the preferences that should apply to this research project.

- Preferred language:
- Output style:
- Subject or method emphasis:
- Review strictness:
`;

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
  let currentCustomizationKey = $derived(
    `${selectedProfile}:${selectedScope === 'registered-project' ? selectedProjectId ?? '' : ''}`
  );
  let activeCustomization = $derived(
    customizationKey === currentCustomizationKey ? customization : null
  );
  let selectedPreview = $derived(
    activeCustomization?.resources.find((resource) => resource.path === selectedPreviewPath)
      ?? activeCustomization?.resources[0]
      ?? null
  );
  let previewContent = $derived(
    selectedPreview === null || showFullSource || selectedPreview.content.length <= 4_000
      ? selectedPreview?.content ?? ''
      : `${selectedPreview.content.slice(0, 4_000)}\n\n…`
  );
  let resourceBytes = $derived(new TextEncoder().encode(resourceDraft).byteLength);
  let resourceChanged = $derived(
    selectedPreview?.editable === true && resourceDraft !== selectedPreview.content
  );
  let guidanceBytes = $derived(new TextEncoder().encode(guidanceDraft).byteLength);
  let guidanceChanged = $derived(
    activeCustomization?.guidance !== null
      && guidanceDraft !== (activeCustomization?.guidance.content || GUIDANCE_STARTER)
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

  $effect(() => {
    const resource = selectedPreview;
    const key = resource
      ? `${customizationKey}:${resource.path}:${resource.currentSha256}`
      : '';
    if (key === resourceDraftKey) return;
    resourceDraftKey = key;
    resourceDraft = resource?.content ?? '';
  });

  $effect(() => {
    if (!reloadAfterVariantOperation || app.preview !== null || app.loading) return;
    reloadAfterVariantOperation = false;
    void loadCustomization();
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

  async function loadCustomization(): Promise<void> {
    const profile = selectedProfile;
    const projectId = selectedScope === 'registered-project' ? selectedProjectId : null;
    const requestKey = `${profile}:${projectId ?? ''}`;
    const event = await app.execute({
      action: 'load-content-customization',
      profile,
      projectId
    }, (candidate) => candidate.type === 'content-customization');
    if (event?.type !== 'content-customization') return;
    customization = event.customization;
    customizationKey = requestKey;
    selectedPreviewPath = event.customization.resources[0]?.path ?? '';
    guidanceDraft = event.customization.guidance?.content || GUIDANCE_STARTER;
    showFullSource = false;
  }

  function previewGuidance(): Promise<unknown> {
    const guidance = activeCustomization?.guidance;
    if (!guidance || guidanceDraft.trim().length === 0 || guidanceBytes > 32 * 1_024) {
      return Promise.resolve(null);
    }
    return app.execute({
      action: 'preview-project-guidance',
      projectId: guidance.projectId,
      expectedSha256: guidance.contentSha256,
      content: guidanceDraft
    });
  }

  async function previewWorkflowResourceReplace(): Promise<void> {
    const current = activeCustomization;
    const resource = selectedPreview;
    if (!current || !resource?.editable || !resourceChanged || resourceDraft.trim().length === 0
      || resourceBytes > 128 * 1_024) return;
    const event = await app.execute({
      action: 'preview-workflow-resource-replace',
      expectedRevision: current.revision,
      expectedVariantSha256: current.variantSha256,
      path: resource.path,
      expectedCurrentSha256: resource.currentSha256,
      content: resourceDraft
    });
    reloadAfterVariantOperation = event?.type === 'preview';
  }

  async function previewWorkflowResourceReset(): Promise<void> {
    const current = activeCustomization;
    const resource = selectedPreview;
    if (!current || !resource?.editable || !resource.overridden) return;
    const event = await app.execute({
      action: 'preview-workflow-resource-reset',
      expectedRevision: current.revision,
      expectedVariantSha256: current.variantSha256,
      path: resource.path,
      expectedCurrentSha256: resource.currentSha256
    });
    reloadAfterVariantOperation = event?.type === 'preview';
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
        <div class="content-title-row">
          <h2 id="workflow-content-title">{i18n.t('content.advancedTitle')}</h2>
          <DescriptionTip text={i18n.t('content.advancedDescription')} />
        </div>
      </div>
      <StatusBadge status={app.snapshot.content.status} />
    </header>

    <div class="content-summary">
      <span><strong>{app.snapshot.content.contentVersion}</strong>{i18n.t('common.version')}</span>
      <span><strong>{i18n.t('content.entries', { count: app.snapshot.content.entryCount })}</strong></span>
      <span><strong>{app.snapshot.mcp.publicToolCount}</strong>{i18n.t('content.tools')}</span>
    </div>

    <p class="alternative-notice">{i18n.t('content.alternativeNotice')}</p>

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
        <Button variant="outline" disabled={app.loading || !destinationReady} onclick={loadCustomization}>
          <Eye size={14} aria-hidden="true" />
          {i18n.t('content.previewCustomize')}
        </Button>
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

    {#if activeCustomization}
      <section class="customizer" aria-labelledby="content-customizer-title">
        <div class="customizer-heading">
          <div>
            <strong id="content-customizer-title">{i18n.t('content.customizerTitle')}</strong>
            <small>{i18n.t('content.customizerDescription')}</small>
          </div>
          <div class="customizer-selection">
            <span class:customized={activeCustomization.state === 'customized'}>
              {i18n.t(activeCustomization.state === 'customized'
                ? 'content.variantCustomized'
                : 'content.variantCanonical')}
            </span>
            <code>r{activeCustomization.revision}</code>
            <NativeSelect
              aria-label={i18n.t('content.previewResource')}
              value={selectedPreview?.path ?? ''}
              onchange={(event) => {
                selectedPreviewPath = event.currentTarget.value;
                showFullSource = false;
              }}
            >
              {#each activeCustomization.resources as resource}
                <option value={resource.path}>
                  {resource.overridden ? '• ' : ''}{resource.path}{resource.editable ? '' : ` · ${i18n.t('content.readOnly')}`}
                </option>
              {/each}
            </NativeSelect>
          </div>
        </div>
        {#if activeCustomization.cleanupRequired}
          <p class="customizer-note warning">{i18n.t('content.variantCleanupRequired')}</p>
        {/if}
        {#if selectedPreview}
          {#if selectedPreview.editable}
            <label class="resource-editor">
              <span>{selectedPreview.path}</span>
              <textarea
                bind:value={resourceDraft}
                rows="16"
                maxlength="131072"
                disabled={app.loading}
                spellcheck="false"
                aria-describedby="content-resource-boundary"
              ></textarea>
            </label>
            <div class="resource-actions">
              <small id="content-resource-boundary" class:invalid={resourceBytes > 128 * 1_024}>
                {i18n.t('content.resourceBoundary', { bytes: resourceBytes })}
              </small>
              <span class="resource-action-buttons">
                <Button
                  variant="outline"
                  disabled={app.loading || !selectedPreview.overridden}
                  onclick={previewWorkflowResourceReset}
                >
                  <RotateCcw size={14} aria-hidden="true" />
                  {i18n.t('content.previewResourceReset')}
                </Button>
                <Button
                  disabled={app.loading || !resourceChanged || resourceDraft.trim().length === 0 || resourceBytes > 128 * 1_024}
                  onclick={previewWorkflowResourceReplace}
                >
                  <Save size={14} aria-hidden="true" />
                  {i18n.t('content.previewResourceSave')}
                </Button>
              </span>
            </div>
          {:else}
            <pre class="source-preview"><code>{previewContent}</code></pre>
            <p class="customizer-note">{i18n.t('content.resourceReadOnly')}</p>
            {#if selectedPreview.content.length > 4_000}
              <Button variant="ghost" size="sm" onclick={() => showFullSource = !showFullSource}>
                {i18n.t(showFullSource ? 'content.previewLess' : 'content.previewFull')}
              </Button>
            {/if}
          {/if}
        {/if}

        {#if activeCustomization.guidance}
          <label class="guidance-editor">
            <span>{i18n.t('content.guidanceLabel')}</span>
            <textarea
              bind:value={guidanceDraft}
              rows="10"
              maxlength="32768"
              disabled={app.loading}
              aria-describedby="content-guidance-boundary"
            ></textarea>
          </label>
          <div class="guidance-actions">
            <small id="content-guidance-boundary" class:invalid={guidanceBytes > 32 * 1_024}>
              {i18n.t('content.guidanceBoundary', { bytes: guidanceBytes })}
            </small>
            <Button
              disabled={app.loading || !guidanceChanged || guidanceDraft.trim().length === 0 || guidanceBytes > 32 * 1_024}
              onclick={previewGuidance}
            >
              <Save size={14} aria-hidden="true" />
              {i18n.t('content.previewGuidanceSave')}
            </Button>
          </div>
        {:else}
          <p class="customizer-note">{i18n.t('content.guidanceProjectOnly')}</p>
        {/if}
      </section>
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
  :global(.managed-content) { overflow: hidden; margin-top: 10px; }
  header {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
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
  .content-title-row { display: flex; min-width: 0; align-items: center; gap: var(--space-1); }
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
  .content-summary strong { color: var(--color-ink-strong); font-size: var(--font-size-label); }
  .alternative-notice { margin: 0; border-bottom: 1px solid var(--color-border); padding: 8px 11px; color: var(--color-warning-strong); background: var(--color-warning-soft); font-size: var(--font-size-label); line-height: 1.4; }
  .content-controls {
    display: grid;
    grid-template-columns: repeat(3, minmax(150px, 1fr));
    align-items: end;
    gap: 9px;
    padding: 7px 10px;
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
  :global(.content-actions) :global([data-slot='button']) { min-height: 44px; font-size: var(--font-size-label); }
  .drift-guidance {
    margin: 0;
    border-top: 1px solid var(--color-border);
    padding: 7px 10px;
    color: var(--color-warning-strong);
    background: var(--color-warning-soft);
    font-size: var(--font-size-micro);
    line-height: 1.45;
  }
  .customizer { border-top: 1px solid var(--color-border); padding: 10px; }
  .customizer-heading, .guidance-actions, .resource-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .customizer-heading > div { min-width: 0; }
  .customizer-heading strong, .customizer-heading small { display: block; }
  .customizer-heading strong { color: var(--color-ink-strong); font-size: var(--font-size-label); }
  .customizer-heading small { margin-top: 3px; color: var(--color-muted); font-size: var(--font-size-micro); }
  .customizer-selection { display: grid; width: min(440px, 58%); grid-template-columns: auto auto minmax(180px, 1fr); align-items: center; gap: 6px; }
  .customizer-selection > span { border-radius: var(--radius-pill); padding: 4px 8px; color: var(--color-success); background: var(--color-success-soft); font-size: var(--font-size-micro); font-weight: 750; white-space: nowrap; }
  .customizer-selection > span.customized { color: var(--color-info); background: var(--color-info-soft); }
  .customizer-selection > code { color: var(--color-muted); font-size: var(--font-size-micro); }
  .customizer-selection :global([data-slot='native-select-wrapper']) { width: 100%; }
  .source-preview {
    margin: 9px 0 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-inset);
    padding: 10px;
    color: var(--color-ink);
    background: var(--color-code-background);
    font-size: var(--font-size-label);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .guidance-editor, .resource-editor { display: grid; gap: 5px; margin-top: 12px; text-transform: none; }
  .guidance-editor textarea, .resource-editor textarea {
    width: 100%;
    min-height: 180px;
    resize: vertical;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-control);
    padding: 10px;
    color: var(--color-ink);
    background: var(--color-control);
    font: 12px/1.5 var(--font-family-mono);
  }
  .guidance-editor textarea:focus-visible, .resource-editor textarea:focus-visible { border-color: var(--color-ring); outline: 3px solid color-mix(in srgb, var(--color-ring) 24%, transparent); outline-offset: 1px; }
  .guidance-actions, .resource-actions { margin-top: 7px; }
  .guidance-actions small, .resource-actions small, .customizer-note { color: var(--color-muted); font-size: var(--font-size-micro); }
  .guidance-actions small.invalid, .resource-actions small.invalid { color: var(--color-danger); }
  .resource-action-buttons { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 7px; }
  .customizer-note { margin: 10px 0 0; }
  .customizer-note.warning { color: var(--color-warning-strong); }
  .profile-details, .managed-details { border-top: 1px solid var(--color-border); padding: 0 10px; }
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
    .customizer-heading, .guidance-actions, .resource-actions { align-items: stretch; flex-direction: column; }
    .customizer-selection { width: 100%; grid-template-columns: auto auto minmax(0, 1fr); }
    .resource-action-buttons { justify-content: flex-start; }
  }
  @media (max-width: 420px) {
    .content-summary { grid-template-columns: 1fr; }
    .content-summary span { border-right: 0; border-bottom: 1px solid var(--color-border); }
    .content-summary span:last-child { border-bottom: 0; }
  }
</style>
