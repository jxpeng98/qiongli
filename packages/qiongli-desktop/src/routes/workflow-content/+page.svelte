<script lang="ts">
  import { Boxes, CheckCircle2, FileText, PackageCheck, Shield, Wrench } from '@lucide/svelte';

  import type { AppIntent } from '@qiongli/app-api';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';
  import { useAppState } from '$lib/context';

  const app = useAppState();
  let selectedProfile = $state<'skill-only' | 'marketplace-lite' | 'full'>('marketplace-lite');
  let selectedPreset = $state<'qiongli-managed' | 'detected-codex' | 'detected-claude-code' | 'current-project'>('qiongli-managed');

  const profileLabels = {
    'skill-only': 'Skills',
    'marketplace-lite': 'Plugin Lite',
    full: 'Full workflow'
  } as const;

  const presetLabels = {
    'qiongli-managed': 'Qiongli managed library',
    'detected-codex': 'Detected Codex plugin',
    'detected-claude-code': 'Detected Claude Code plugin',
    'current-project': 'Current project'
  } as const;

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
  eyebrow="Workflow content"
  title="One academic workflow, packaged for each client"
  description="Qiongli stores canonical workflow content once. Skills remain an advanced standalone destination; client integrations install a plugin-shaped bundle containing Skills, Lite MCP, registration, and receipts."
/>

{#if !app.snapshot}
  <section class="surface empty" aria-busy="true">Loading embedded workflow inventory…</section>
{:else}
  <section class="inventory surface">
    <div class="inventory-main">
      <div class="pack-icon"><PackageCheck size={25} aria-hidden="true" /></div>
      <div>
        <div class="title-line">
          <h2>{app.snapshot.content.packId}</h2>
          <StatusBadge status={app.snapshot.content.status} />
        </div>
        <p>Version {app.snapshot.content.contentVersion} · {app.snapshot.content.entryCount} verified entries</p>
      </div>
    </div>
    <div class="inventory-facts">
      <div><strong>{app.snapshot.content.profiles.length}</strong><span>profiles</span></div>
      <div><strong>{app.snapshot.mcp.publicToolCount}</strong><span>Lite MCP tools</span></div>
      <div><strong>1</strong><span>canonical pack</span></div>
    </div>
  </section>

  <section class="section-block">
    <div class="section-title">
      <div>
        <p class="eyebrow">Profiles</p>
        <h2>Choose the content boundary</h2>
      </div>
      <p>Profiles are views over the same verified embedded pack.</p>
    </div>

    <div class="profile-grid" role="radiogroup" aria-label="Workflow profile">
      {#each app.snapshot.content.profiles as profile}
        <label class="profile surface" class:selected={selectedProfile === profile.id}>
          <input type="radio" name="profile" value={profile.id} bind:group={selectedProfile} />
          <div class="profile-icon">
            {#if profile.id === 'skill-only'}<FileText size={21} aria-hidden="true" />
            {:else if profile.id === 'marketplace-lite'}<Boxes size={21} aria-hidden="true" />
            {:else}<Shield size={21} aria-hidden="true" />{/if}
          </div>
          <div class="profile-heading">
            <h3>{profileLabels[profile.id]}</h3>
            {#if selectedProfile === profile.id}<CheckCircle2 size={18} aria-label="Selected" />{/if}
          </div>
          <p>{profile.description}</p>
          <span>{profile.includedResourceKinds} resource kinds</span>
        </label>
      {/each}
    </div>
  </section>

  <section class="advanced surface">
    <div class="advanced-heading">
      <div class="advanced-icon"><Wrench size={20} aria-hidden="true" /></div>
      <div>
        <p class="eyebrow">Advanced standalone content</p>
        <h2>Materialize workflow content without a client plugin</h2>
        <p>Use this only for a shared Qiongli library or project-local workflow. For Codex and Claude Code, Client Integrations is the recommended path.</p>
      </div>
    </div>

    <div class="form-row">
      <label>
        Destination
        <select bind:value={selectedPreset}>
          {#each Object.entries(presetLabels) as [value, label]}
            <option {value}>{label}</option>
          {/each}
        </select>
      </label>
      <label>
        Profile
        <select bind:value={selectedProfile}>
          {#each Object.entries(profileLabels) as [value, label]}
            <option {value}>{label}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="actions">
      <button class="button-primary" type="button" disabled={app.loading || !app.snapshot.capabilities.skillsMaterialize} onclick={previewMaterialization}>Preview installation</button>
      <button class="button-secondary" type="button" disabled={app.loading} onclick={verifyPreset}>Verify content</button>
      <button class="button-danger" type="button" disabled={app.loading} onclick={removePreset}>Preview removal</button>
    </div>
  </section>
{/if}

<style>
  .empty { padding: 28px; color: var(--color-muted); }
  .inventory { display: grid; gap: 18px; padding: 22px 24px; border-top: 3px solid var(--color-accent); }
  .inventory-main { display: flex; align-items: center; gap: 14px; }
  .pack-icon, .advanced-icon { display: grid; flex: none; width: 44px; height: 44px; place-items: center; border-radius: 12px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .title-line { display: flex; align-items: center; gap: 12px; }
  h2, h3 { margin: 0; color: var(--color-ink-strong); }
  .inventory h2 { font-size: 20px; }
  .inventory p { margin: 6px 0 0; color: var(--color-muted); font-size: 12px; }
  .inventory-facts { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-top: 1px solid var(--color-border); padding-top: 16px; }
  .inventory-facts div { border-left: 1px solid var(--color-border); padding: 3px 18px; text-align: center; }
  .inventory-facts div:first-child { border-left: 0; }
  .inventory-facts strong, .inventory-facts span { display: block; }
  .inventory-facts strong { font-size: 20px; }
  .inventory-facts span { margin-top: 3px; color: var(--color-muted); font-size: 10px; font-weight: 650; }
  .section-block { margin-top: 30px; }
  .section-title { display: flex; align-items: end; justify-content: space-between; gap: 24px; margin-bottom: 14px; }
  .section-title h2, .advanced h2 { font-size: 19px; }
  .section-title > p { max-width: 420px; margin: 0; color: var(--color-muted); font-size: 12px; }
  .profile-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; }
  .profile { position: relative; min-height: 200px; padding: 18px; cursor: pointer; }
  .profile:hover, .profile.selected { border-color: var(--color-accent); }
  .profile.selected { box-shadow: 0 0 0 2px rgb(3 105 161 / 0.13), var(--shadow-card); }
  .profile input { position: absolute; width: 1px; height: 1px; opacity: 0; }
  .profile-icon { display: grid; width: 38px; height: 38px; place-items: center; margin-bottom: 18px; border-radius: 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .profile-heading { display: flex; align-items: center; justify-content: space-between; gap: 10px; color: var(--color-accent); }
  .profile h3 { font-size: 15px; }
  .profile p { min-height: 52px; margin: 9px 0 16px; color: var(--color-muted); font-size: 12px; line-height: 1.5; }
  .profile > span { color: var(--color-muted); font-size: 11px; font-weight: 700; }
  .advanced { margin-top: 28px; padding: 22px 24px; }
  .advanced-heading { display: grid; grid-template-columns: auto 1fr; gap: 14px; }
  .advanced-heading > div > p:last-child { max-width: 760px; margin: 7px 0 0; color: var(--color-muted); font-size: 12px; line-height: 1.55; }
  .form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 20px; border-top: 1px solid var(--color-border); padding-top: 18px; }
  label { color: var(--color-ink); font-size: 12px; font-weight: 720; }
  select { display: block; width: 100%; height: 42px; margin-top: 7px; border: 1px solid var(--color-border-strong); border-radius: 9px; padding: 0 11px; color: var(--color-ink); background: white; font: inherit; }
  .actions { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 18px; }

  @media (max-width: 700px) {
    .section-title { align-items: flex-start; flex-direction: column; }
    .profile-grid, .form-row { grid-template-columns: 1fr; }
    .profile { min-height: 0; }
    .profile p { min-height: 0; }
    .inventory, .advanced { padding: 18px; }
  }

  @media (max-width: 440px) {
    .inventory-main, .title-line { align-items: flex-start; flex-direction: column; }
    .inventory-facts div { padding-inline: 6px; }
    .advanced-heading { grid-template-columns: 1fr; }
  }
</style>
