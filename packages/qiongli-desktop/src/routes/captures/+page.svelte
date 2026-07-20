<script lang="ts">
  import type { CaptureInboxEntry } from '@qiongli/app-api';
  import {
    AlertTriangle,
    ArrowRight,
    CheckCircle2,
    FileInput,
    Inbox,
    RefreshCw,
    ScanSearch
  } from '@lucide/svelte';

  import { useAppState } from '$lib/context';
  import { canReviewCapture, captureStatus } from '$lib/features/captures';
  import { PageHeader, StatusBadge } from '$lib/shared/ui';

  const app = useAppState();

  let selectedProjectId = $state<string | null>(null);
  let requestedProjectId = $state<string | null>(null);
  let selectedCaptureId = $state<string | null>(null);

  let projects = $derived(app.snapshot?.researchLibrary.projects ?? []);
  let selectedProject = $derived(
    projects.find((project) => project.projectId === selectedProjectId) ?? null
  );
  let inbox = $derived(
    app.captureInbox?.projectId === selectedProjectId ? app.captureInbox : null
  );

  $effect(() => {
    if (projects.length === 0) {
      selectedProjectId = null;
      requestedProjectId = null;
      return;
    }
    if (!selectedProjectId || !projects.some((project) => project.projectId === selectedProjectId)) {
      selectedProjectId = projects[0].projectId;
      requestedProjectId = null;
    }
    if (selectedProjectId && requestedProjectId !== selectedProjectId && !app.loading) {
      requestedProjectId = selectedProjectId;
      void app.execute({ action: 'load-capture-inbox', projectId: selectedProjectId });
    }
  });

  function chooseProject(event: Event): void {
    selectedProjectId = (event.currentTarget as HTMLSelectElement).value || null;
    selectedCaptureId = null;
    requestedProjectId = null;
  }

  async function refreshInbox(): Promise<void> {
    if (!selectedProjectId) return;
    requestedProjectId = selectedProjectId;
    await app.execute({ action: 'load-capture-inbox', projectId: selectedProjectId });
  }

  async function importCapture(): Promise<void> {
    if (!selectedProjectId) return;
    const selection = await app.execute({
      action: 'select-capture-file',
      projectId: selectedProjectId
    });
    if (selection?.type !== 'capture-file-selected') return;
    await app.execute({
      action: 'preview-capture-intake',
      fileToken: selection.token
    });
  }

  async function inspectCapture(entry: CaptureInboxEntry): Promise<void> {
    if (!selectedProjectId) return;
    selectedCaptureId = entry.captureId;
    await app.execute({
      action: 'read-capture',
      projectId: selectedProjectId,
      captureId: entry.captureId
    });
  }

  async function reviewCapture(entry: CaptureInboxEntry): Promise<void> {
    if (!selectedProjectId) return;
    selectedCaptureId = entry.captureId;
    await app.execute({
      action: 'preview-capture-consolidation',
      projectId: selectedProjectId,
      captureId: entry.captureId
    });
  }

  function sentence(value: string): string {
    return value.replaceAll('-', ' ').replace(/^./, (letter) => letter.toUpperCase());
  }

  function captureDate(entry: CaptureInboxEntry): string {
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short'
    }).format(new Date(entry.capturedAtUnix * 1_000));
  }
</script>

<PageHeader
  eyebrow="Cross-client research memory"
  title="Capture Inbox"
  description="Review bounded academic changes collected outside the current app. Qiongli stores normalized research captures—not chats or sessions—and consolidates only an explicitly reviewed plan."
>
  {#snippet actions()}
    <label class="project-picker">
      <span>Article project</span>
      <select value={selectedProjectId ?? ''} onchange={chooseProject} disabled={app.loading || projects.length === 0}>
        {#each projects as project}
          <option value={project.projectId}>{project.displayName}</option>
        {/each}
      </select>
    </label>
    <button
      class="button-primary"
      type="button"
      disabled={app.loading || !selectedProject || !app.snapshot?.capabilities.captureMutation}
      onclick={importCapture}
    >
      <FileInput size={16} aria-hidden="true" />Import capture
    </button>
    <button
      class="button-secondary"
      type="button"
      disabled={app.loading || !selectedProject || !app.snapshot?.capabilities.captureInbox}
      onclick={refreshInbox}
    >
      <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />Refresh
    </button>
  {/snippet}
</PageHeader>

{#if !app.snapshot}
  <section class="surface loading" aria-busy="true">
    <p>Loading the private Research Library…</p>
  </section>
{:else if projects.length === 0}
  <section class="surface empty-state">
    <Inbox size={30} aria-hidden="true" />
    <h2>Create or register an article project first</h2>
    <p>The Capture Inbox is project-bound. It never creates a session-wide store with ambiguous academic ownership.</p>
    <a class="button-primary" href="/research-library">Open Research Library</a>
  </section>
{:else if selectedProject?.health === 'inspection-blocked'}
  <section class="surface blocked-state">
    <AlertTriangle size={24} aria-hidden="true" />
    <div><h2>Capture Inbox cannot be inspected</h2><p>Project state could not be verified safely. No partial capture data or host path was exposed.</p></div>
  </section>
{:else if !inbox}
  <section class="surface loading" aria-busy="true">
    <p>Inspecting normalized captures for {selectedProject?.displayName}…</p>
  </section>
{:else}
  <section class="metrics" aria-label="Capture Inbox summary">
    <article class="surface metric"><span class="metric-icon"><Inbox size={18} aria-hidden="true" /></span><div><strong>{inbox.entries.length}</strong><span>Captures</span></div></article>
    <article class="surface metric"><span class="metric-icon attention"><ScanSearch size={18} aria-hidden="true" /></span><div><strong>{inbox.pendingReviewCount}</strong><span>Pending review</span></div></article>
    <article class="surface metric"><span class:warning={inbox.staleCount + inbox.conflictedCount > 0} class="metric-icon"><AlertTriangle size={18} aria-hidden="true" /></span><div><strong>{inbox.staleCount + inbox.conflictedCount}</strong><span>Need resolution</span></div></article>
    <article class="surface metric"><span class="metric-icon positive"><CheckCircle2 size={18} aria-hidden="true" /></span><div><strong>{inbox.appliedCount}</strong><span>Consolidated</span></div></article>
  </section>

  <section class="surface inbox-panel">
    <div class="panel-heading">
      <div>
        <p class="eyebrow">Portable review queue</p>
        <h2>{selectedProject?.displayName}</h2>
        <p>Project r{inbox.projectRevision} · {sentence(inbox.projectStage)} stage</p>
      </div>
      <StatusBadge status={inbox.entries.length === 0 ? 'ready' : 'attention'} label={inbox.entries.length === 0 ? 'Clear' : 'Review available'} />
    </div>

    {#if inbox.entries.length === 0}
      <div class="empty-inbox">
        <CheckCircle2 size={26} aria-hidden="true" />
        <h3>No research captures yet</h3>
        <p>Export a bounded Qiongli capture from a supported client or choose a portable capture JSON file.</p>
      </div>
    {:else}
      <div class="capture-list">
        {#each inbox.entries as entry (entry.captureId)}
          <article class:selected={selectedCaptureId === entry.captureId}>
            <button class="capture-main" type="button" onclick={() => inspectCapture(entry)}>
              <span class="capture-title"><strong>{entry.summary}</strong><small>{entry.task}</small></span>
              <span class="capture-meta"><span>{sentence(entry.source)}</span><span>{sentence(entry.disposition)}</span><span>r{entry.baseRevision}</span></span>
              <span class="capture-date">{captureDate(entry)}</span>
              <StatusBadge status={captureStatus(entry)} label={sentence(entry.state)} />
              <ArrowRight size={17} aria-hidden="true" />
            </button>
            <button
              class="button-secondary review-button"
              type="button"
              disabled={app.loading || !canReviewCapture(entry)}
              onclick={() => reviewCapture(entry)}
            >
              {entry.state === 'applied' ? 'Consolidated' : 'Review plan'}
            </button>
          </article>
        {/each}
      </div>
    {/if}
  </section>

  {#if app.capture && app.capture.captureId === selectedCaptureId}
    <section class="surface detail-panel" aria-live="polite">
      <div class="panel-heading">
        <div><p class="eyebrow">Normalized academic content</p><h2>Capture detail</h2><p><code>{app.capture.captureId}</code></p></div>
        <button class="button-quiet" type="button" onclick={() => selectedCaptureId = null}>Close</button>
      </div>
      <p class="capture-summary">{app.capture.summary}</p>
      <div class="detail-grid">
        <section><h3>Academic changes</h3>{#if app.capture.changes.length}<ul>{#each app.capture.changes as change}<li><strong>{sentence(change.area)}</strong><span>{change.summary}</span></li>{/each}</ul>{:else}<p>None recorded.</p>{/if}</section>
        <section><h3>Decision candidates</h3>{#if app.capture.decisions.length}<ul>{#each app.capture.decisions as decision}<li><strong>{sentence(decision.relation)}</strong><span>{decision.statement}</span><small>{decision.rationale}</small></li>{/each}</ul>{:else}<p>None recorded.</p>{/if}</section>
        <section><h3>Evidence references</h3>{#if app.capture.evidence.length}<ul>{#each app.capture.evidence as evidence}<li><code>{evidence.locator}</code><span>{evidence.relevance}</span>{#if evidence.limitation}<small>{evidence.limitation}</small>{/if}</li>{/each}</ul>{:else}<p>None recorded.</p>{/if}</section>
        <section><h3>Contradictions & next actions</h3>{#if app.capture.contradictions.length}<ul class="danger-list">{#each app.capture.contradictions as contradiction}<li><strong>{contradiction.statement}</strong><span>{contradiction.consequence}</span></li>{/each}</ul>{/if}{#if app.capture.nextActions.length}<ol>{#each app.capture.nextActions as action}<li>{action}</li>{/each}</ol>{:else if app.capture.contradictions.length === 0}<p>None recorded.</p>{/if}</section>
      </div>
    </section>
  {/if}
{/if}

<style>
  .project-picker { display: grid; gap: 4px; min-width: min(280px, 100%); }
  .project-picker span { color: var(--color-muted); font-size: 10px; font-weight: 800; letter-spacing: .05em; text-transform: uppercase; }
  select { min-height: 40px; border: 1px solid var(--color-border-strong); border-radius: 10px; padding: 7px 10px; color: var(--color-ink); background: white; font: inherit; font-size: 12px; }
  .loading, .empty-state { min-height: 210px; padding: 34px; }
  .loading { color: var(--color-muted); }
  .empty-state { display: grid; place-items: center; align-content: center; text-align: center; }
  .empty-state h2 { margin: 12px 0 0; color: var(--color-ink-strong); }
  .empty-state p { max-width: 620px; margin: 8px 0 18px; color: var(--color-muted); line-height: 1.6; }
  .blocked-state { display: flex; gap: 13px; padding: 22px; border-color: #fecaca; color: var(--color-danger); background: var(--color-danger-soft); }
  .blocked-state h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  .blocked-state p { margin: 6px 0 0; color: var(--color-muted); line-height: 1.55; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 18px; }
  .metric { display: flex; min-height: 86px; align-items: center; gap: 13px; padding: 16px; }
  .metric-icon { display: grid; width: 36px; height: 36px; flex: none; place-items: center; border-radius: 10px; color: var(--color-accent-strong); background: var(--color-accent-soft); }
  .metric-icon.attention, .metric-icon.warning { color: var(--color-warning); background: var(--color-warning-soft); }
  .metric-icon.positive { color: var(--color-success); background: var(--color-success-soft); }
  .metric strong, .metric span { display: block; }
  .metric strong { color: var(--color-ink-strong); font-size: 21px; line-height: 1; }
  .metric div span { margin-top: 5px; color: var(--color-muted); font-size: 11px; font-weight: 700; }
  .inbox-panel, .detail-panel { padding: 22px; }
  .panel-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; }
  .panel-heading h2 { margin: 0; color: var(--color-ink-strong); font-size: 20px; }
  .panel-heading > div > p:last-child { margin: 7px 0 0; color: var(--color-muted); font-size: 12px; }
  .capture-list { margin-top: 17px; border-top: 1px solid var(--color-border); }
  .capture-list article { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 10px; border-bottom: 1px solid var(--color-border); }
  .capture-list article.selected { margin-inline: -8px; border: 1px solid #7dd3fc; border-radius: 11px; padding-right: 8px; background: var(--color-accent-soft); }
  .capture-main { display: grid; min-height: 86px; grid-template-columns: minmax(220px, 1.5fr) minmax(160px, .8fr) 130px auto auto; align-items: center; gap: 14px; border: 0; padding: 12px 8px; color: inherit; background: transparent; text-align: left; cursor: pointer; }
  .capture-title strong, .capture-title small { display: block; }
  .capture-title strong { color: var(--color-ink-strong); font-size: 13px; line-height: 1.45; }
  .capture-title small { margin-top: 5px; color: var(--color-muted); font-size: 10px; }
  .capture-meta { display: flex; flex-wrap: wrap; gap: 5px; }
  .capture-meta span { border: 1px solid var(--color-border); border-radius: 999px; padding: 3px 7px; color: var(--color-muted); background: white; font-size: 10px; font-weight: 700; }
  .capture-date { color: var(--color-muted); font-size: 10px; }
  .review-button { min-height: 34px; padding: 6px 10px; font-size: 11px; }
  .empty-inbox { padding: 52px 20px; color: var(--color-muted); text-align: center; }
  .empty-inbox h3 { margin: 12px 0 0; color: var(--color-ink-strong); }
  .empty-inbox p { margin: 7px 0 0; }
  .detail-panel { margin-top: 18px; border-top: 3px solid var(--color-accent); }
  .capture-summary { margin: 18px 0 0; border-left: 3px solid var(--color-accent); padding: 3px 0 3px 13px; color: var(--color-ink); font-size: 14px; line-height: 1.65; }
  .detail-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 11px; margin-top: 17px; }
  .detail-grid section { border: 1px solid var(--color-border); border-radius: 12px; padding: 15px; background: var(--color-surface-subtle); }
  .detail-grid h3 { margin: 0; color: var(--color-ink-strong); font-size: 13px; }
  .detail-grid ul, .detail-grid ol { margin: 11px 0 0; padding-left: 20px; color: var(--color-ink); font-size: 12px; line-height: 1.55; }
  .detail-grid li + li { margin-top: 10px; }
  .detail-grid li strong, .detail-grid li span, .detail-grid li small { display: block; }
  .detail-grid li strong, .detail-grid code { color: var(--color-accent-strong); }
  .detail-grid li span { margin-top: 3px; }
  .detail-grid li small { margin-top: 3px; color: var(--color-muted); }
  .detail-grid section > p { color: var(--color-muted); font-size: 12px; }
  .danger-list { color: var(--color-danger) !important; }
  code { overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

  @media (max-width: 1180px) {
    .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .capture-main { grid-template-columns: minmax(200px, 1fr) minmax(150px, .7fr) auto auto; }
    .capture-date { display: none; }
  }
  @media (max-width: 760px) {
    .capture-list article { grid-template-columns: 1fr; padding-bottom: 10px; }
    .capture-main { grid-template-columns: 1fr auto; }
    .capture-meta { grid-column: 1 / -1; }
    .capture-main :global(.status) { justify-self: start; }
    .review-button { justify-self: start; margin-left: 8px; }
    .detail-grid { grid-template-columns: 1fr; }
  }
  @media (max-width: 520px) {
    .metrics { grid-template-columns: 1fr; }
    .inbox-panel, .detail-panel { padding: 17px; }
    .panel-heading { flex-direction: column; }
  }
</style>
