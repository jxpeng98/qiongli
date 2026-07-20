<script lang="ts">
  import { Boxes, CheckCircle2, FileText, PackageCheck, Shield, Wrench } from '@lucide/svelte';

  import type { AppIntent } from '@qiongli/app-api';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';
  import { i18n } from '$lib/i18n.svelte';

  const app = useAppState();
  let selectedProfile = $state<'skill-only' | 'marketplace-lite' | 'full'>('marketplace-lite');
  let selectedPreset = $state<'qiongli-managed' | 'detected-codex' | 'detected-claude-code' | 'current-project'>('qiongli-managed');

  let profileLabels = $derived({
    'skill-only': i18n.t('content.profile.skills'),
    'marketplace-lite': i18n.t('content.profile.lite'),
    full: i18n.t('content.profile.full')
  } as const);

  let presetLabels = $derived({
    'qiongli-managed': i18n.t('content.preset.managed'),
    'detected-codex': i18n.t('content.preset.codex'),
    'detected-claude-code': i18n.t('content.preset.claude'),
    'current-project': i18n.t('content.preset.project')
  } as const);

  let profileDescriptions = $derived({
    'skill-only': i18n.t('content.profileDescription.skills'),
    'marketplace-lite': i18n.t('content.profileDescription.lite'),
    full: i18n.t('content.profileDescription.full')
  } as const);

  function previewMaterialization(): Promise<unknown> {
    return app.execute({
      action: 'preview-skills-preset-materialization',
      profile: selectedProfile,
      preset: selectedPreset
    });
  }

  function verifyPreset(): Promise<unknown> {
    return app.execute({ action: 'verify-skills-preset', preset: selectedPreset });
  }

  function removePreset(): Promise<unknown> {
    const intent: AppIntent = { action: 'preview-skills-preset-removal', preset: selectedPreset };
    return app.execute(intent);
  }
</script>

<PageHeader
  eyebrow={i18n.t('content.eyebrow')}
  title={i18n.t('content.title')}
  description={i18n.t('content.description')}
/>

{#if !app.snapshot}
  <section class="surface empty" aria-busy="true">{i18n.t('content.loading')}</section>
{:else}
  <section class="inventory surface">
    <div class="pack-icon"><PackageCheck size={21} aria-hidden="true" /></div>
    <div class="pack-copy">
      <div class="title-line"><h2>{app.snapshot.content.packId}</h2><StatusBadge status={app.snapshot.content.status} /></div>
      <p>{i18n.t('common.version')} {app.snapshot.content.contentVersion} · {i18n.t('content.entries', { count: app.snapshot.content.entryCount })}</p>
    </div>
    <div class="facts" aria-label="Workflow inventory">
      <span><strong>{app.snapshot.content.profiles.length}</strong>{i18n.t('content.profiles')}</span>
      <span><strong>{app.snapshot.mcp.publicToolCount}</strong>{i18n.t('content.tools')}</span>
      <span><strong>1</strong>{i18n.t('content.pack')}</span>
    </div>
  </section>

  <section class="profiles-section">
    <div class="section-title">
      <div><p class="eyebrow">{i18n.t('content.profileEyebrow')}</p><h2>{i18n.t('content.chooseBoundary')}</h2></div>
      <p>{i18n.t('content.samePack')}</p>
    </div>

    <div class="profile-grid" role="radiogroup" aria-label={i18n.t('content.profile')}>
      {#each app.snapshot.content.profiles as profile}
        <label class="profile surface" class:selected={selectedProfile === profile.id}>
          <input type="radio" name="profile" value={profile.id} bind:group={selectedProfile} />
          <span class="profile-icon">
            {#if profile.id === 'skill-only'}<FileText size={18} aria-hidden="true" />
            {:else if profile.id === 'marketplace-lite'}<Boxes size={18} aria-hidden="true" />
            {:else}<Shield size={18} aria-hidden="true" />{/if}
          </span>
          <span class="profile-copy"><strong>{profileLabels[profile.id]}</strong><small>{profileDescriptions[profile.id]}</small></span>
          <span class="profile-meta">{i18n.t('content.resourceKinds', { count: profile.includedResourceKinds })}</span>
          {#if selectedProfile === profile.id}<CheckCircle2 class="check" size={17} aria-label={i18n.t('common.selected')} />{/if}
        </label>
      {/each}
    </div>
  </section>

  <section class="advanced surface">
    <div class="advanced-heading">
      <span class="advanced-icon"><Wrench size={18} aria-hidden="true" /></span>
      <div>
        <p class="eyebrow">{i18n.t('content.advanced')}</p>
        <h2>{i18n.t('content.advancedTitle')}</h2>
        <p>{i18n.t('content.advancedDescription')}</p>
      </div>
    </div>
    <div class="form-row">
      <label>{i18n.t('content.destination')}<select bind:value={selectedPreset}>{#each Object.entries(presetLabels) as [value, label]}<option {value}>{label}</option>{/each}</select></label>
      <label>{i18n.t('content.profile')}<select bind:value={selectedProfile}>{#each Object.entries(profileLabels) as [value, label]}<option {value}>{label}</option>{/each}</select></label>
    </div>
    <div class="actions">
      <button class="button-primary" type="button" disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize} onclick={previewMaterialization}>{i18n.t('content.previewInstall')}</button>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={verifyPreset}>{i18n.t('content.verify')}</button>
      <button class="button-danger" type="button" disabled={app.loading} onclick={removePreset}>{i18n.t('content.previewRemove')}</button>
    </div>
  </section>
{/if}

<style>
  .empty { padding: 20px; color: var(--color-muted); }
  .inventory { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 12px; padding: 14px 16px; border-left: 3px solid var(--color-accent); }
  .pack-icon, .advanced-icon, .profile-icon { display: grid; flex: none; place-items: center; border-radius: 9px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .pack-icon { width: 38px; height: 38px; }
  .title-line { display: flex; align-items: center; gap: 9px; }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 16px; }
  .pack-copy p { margin: 4px 0 0; color: var(--color-muted); font-size: 11px; }
  .facts { display: flex; align-items: stretch; }
  .facts span { display: grid; min-width: 82px; place-items: center; border-left: 1px solid var(--color-border); padding: 2px 12px; color: var(--color-muted); font-size: 9px; font-weight: 700; text-align: center; }
  .facts strong { color: var(--color-ink-strong); font-size: 17px; }
  .profiles-section { margin-top: 16px; }
  .section-title { display: flex; align-items: end; justify-content: space-between; gap: 20px; margin-bottom: 9px; }
  .section-title > p { margin: 0; color: var(--color-muted); font-size: 11px; }
  .profile-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 9px; }
  .profile { position: relative; display: grid; min-height: 92px; grid-template-columns: auto minmax(0, 1fr); align-items: start; gap: 9px; padding: 12px; cursor: pointer; }
  .profile:hover, .profile.selected { border-color: var(--color-accent); }
  .profile.selected { box-shadow: 0 0 0 2px rgb(3 105 161 / .12); }
  .profile input { position: absolute; width: 1px; height: 1px; opacity: 0; }
  .profile-icon { width: 32px; height: 32px; }
  .profile-copy { min-width: 0; }
  .profile-copy strong, .profile-copy small { display: block; }
  .profile-copy strong { color: var(--color-ink-strong); font-size: 13px; }
  .profile-copy small { margin-top: 3px; color: var(--color-muted); font-size: 10px; line-height: 1.35; }
  .profile-meta { grid-column: 2; color: var(--color-accent-strong); font-size: 9px; font-weight: 750; }
  :global(.check) { position: absolute; top: 9px; right: 9px; color: var(--color-accent); }
  .advanced { display: grid; grid-template-columns: minmax(250px, 1.4fr) minmax(310px, 1fr) auto; align-items: end; gap: 14px; margin-top: 16px; padding: 14px 16px; }
  .advanced-heading { display: grid; grid-template-columns: auto 1fr; gap: 10px; }
  .advanced-icon { width: 34px; height: 34px; }
  .advanced h2 { font-size: 14px; }
  .advanced-heading div > p:last-child { margin: 4px 0 0; color: var(--color-muted); font-size: 10px; line-height: 1.4; }
  .form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  label { color: var(--color-ink); font-size: 10px; font-weight: 750; }
  select { display: block; width: 100%; height: 36px; margin-top: 4px; border: 1px solid var(--color-border-strong); border-radius: 8px; padding: 0 8px; color: var(--color-ink); background: white; font: inherit; font-size: 11px; }
  .actions { display: flex; max-width: 170px; flex-direction: column; gap: 6px; }
  .actions button { min-height: 34px; font-size: 11px; }
  @media (max-width: 1080px) { .advanced { grid-template-columns: 1fr 1fr; } .actions { grid-column: 1 / -1; max-width: none; flex-direction: row; } }
  @media (max-width: 760px) { .inventory { grid-template-columns: auto 1fr; } .facts { grid-column: 1 / -1; border-top: 1px solid var(--color-border); padding-top: 9px; } .facts span { flex: 1; } .profile-grid, .advanced { grid-template-columns: 1fr; } .section-title { align-items: flex-start; flex-direction: column; gap: 4px; } .actions { grid-column: auto; } }
</style>
