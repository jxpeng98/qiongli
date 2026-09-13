<script lang="ts">
  import { onMount } from 'svelte';
  import type { AppTransport, ChatRequest, ChatSnapshot, ChatUpdate, ChatPermission, ResearchRequest, ResearchSnapshot, ResearchSourceSelection, ResearchCandidate } from '@qiongli/app-api';
  import { Button } from '$lib/components/ui/button';
  import { chatText as t } from './copy';

  let { projectId, projectRevision, sources = [], onLoadSources, onReviewCandidate, transport }: {
    projectId: string; projectRevision: number; sources?: string[];
    onLoadSources: () => void; onReviewCandidate?: (candidate: ResearchCandidate) => Promise<unknown>; transport?: AppTransport;
  } = $props();
  let view = $state<ChatSnapshot | null>(null);
  let loaded = $state(false);
  let busy = $state(false);
  let error = $state('');
  let text = $state('');
  let context = $state('');
  let useContext = $state(false);
  let sourceRefs = $state<string[]>([]);
  let permission = $state<ChatPermission | null>(null);
  let research = $state<ResearchSnapshot | null>(null);
  let researchError = $state('');
  let editedCandidate = $state<ResearchCandidate | null>(null);
  let editingKey = '';
  let approvedSelection = $state('');
  let selections = $state<[ResearchSourceSelection, ResearchSourceSelection]>([
    { artifactPath: '', startLine: 1, endLine: 1 }, { artifactPath: '', startLine: 1, endLine: 1 }
  ]);
  const selectionKey = $derived(JSON.stringify([projectId, projectRevision, selections]));
  let execute: ((request: ChatRequest) => Promise<ChatSnapshot | null>) | undefined;
  let executeResearch: ((request: ResearchRequest) => Promise<ResearchSnapshot | null>) | undefined;
  let findPermission: ((view: ChatSnapshot) => ChatPermission | null) | undefined;
  let mounted = false;
  let reading = $state(false);
  let epoch = 0;
  let pollTimer: ReturnType<typeof setTimeout>;
  const terminal = $derived(view?.status === 'closed' || view?.status === 'interrupted');
  const activeBinding = $derived(view?.updates.findLast((u) => u.kind.type === 'turn' && u.kind.status === 'running')?.kind);
  $effect(() => {
    const candidate = research?.candidate;
    const key = candidate ? `${candidate.runId}:${candidate.turnId}:${candidate.manifestDigest}` : '';
    if (key !== editingKey) {
      editingKey = key;
      editedCandidate = candidate ? structuredClone($state.snapshot(candidate)) : null;
    }
  });

  onMount(() => {
    mounted = true;
    void (async () => {
      const { QiongliChatClient, QiongliResearchClient, pendingChatPermission } = await import('@qiongli/app-api');
      let selectedTransport = transport;
      if (!selectedTransport && import.meta.env.DEV && new URLSearchParams(window.location.search).get('fixture') === 'source-read-only') {
        selectedTransport = (await import('./dev-chat')).chatFixtureTransport();
      }
      findPermission = pendingChatPermission;
      const client = new QiongliChatClient(selectedTransport);
      const researchClient = new QiongliResearchClient(selectedTransport);
      execute = (request) => client.execute(request);
      executeResearch = (request) => researchClient.execute(request);
      await read();
    })().catch(() => { if (mounted) error = 'chat.unavailable'; });
    return () => { mounted = false; epoch++; clearTimeout(pollTimer); };
  });

  function accept(next: ChatSnapshot | null) {
    if (next && next.projectId !== projectId) throw new Error('Wrong project');
    if (view && next?.runId === view.runId && next.revision < view.revision) return;
    if (view?.runId !== next?.runId) { research = null; researchError = ''; }
    view = next;
    permission = next ? findPermission?.(next) ?? null : null;
    loaded = true;
  }
  async function read() {
    if (!mounted || !execute || reading) return;
    clearTimeout(pollTimer);
    if (busy) { pollTimer = setTimeout(() => void read(), 400); return; }
    reading = true;
    const observed = epoch;
    try {
      const next = await execute({ type: 'read', projectId });
      if (mounted && observed === epoch) { accept(next); await readResearch(next, observed); }
    } catch (cause) { if (mounted && observed === epoch) error = cause === 'all-chat-development-only' ? 'chat.developmentOnly' : historyError(cause) ? 'chat.historyUnavailable' : 'chat.unavailable'; }
    finally {
      reading = false;
      if (mounted) pollTimer = setTimeout(() => void read(), 400);
    }
  }
  async function readResearch(chat: ChatSnapshot | null, observed: number) {
    if (!executeResearch || !chat) return;
    try {
      const next = await executeResearch({ type: 'read', runId: chat.runId });
      if (next && (next.manifest.projectId !== projectId || next.candidate && next.candidate.turnId >= chat.nextTurn)) throw new Error('Wrong research scope');
      if (mounted && observed === epoch) { research = next; researchError = ''; }
    } catch {
      if (mounted && observed === epoch) { research = null; researchError = 'research.unavailable'; }
    }
  }
  function historyError(cause: unknown) { return typeof cause === 'string' && cause.startsWith('all-chat-history-'); }
  async function act(request: ChatRequest) {
    if (!execute || busy) return false;
    busy = true; error = ''; epoch++;
    try {
      const next = await execute(request);
      if (mounted) { accept(next); await readResearch(next, epoch); }
      return true;
    } catch (cause) { if (mounted) error = historyError(cause) ? 'chat.historyUnavailable' : 'chat.actionFailed'; return false; }
    finally { busy = false; }
  }
  async function send() {
    if (!view || !text.trim()) return;
    if (await act({ type: 'prompt', runId: view.runId, expectedTurn: view.nextTurn, prompt: { text, context: research ? '' : useContext ? context : '', sourceRefs: research ? [] : sourceRefs } })) text = '';
  }
  async function researchAction(request: ResearchRequest) {
    if (!executeResearch || !execute || busy) return;
    busy = true; error = ''; researchError = ''; epoch++;
    try {
      const next = await executeResearch(request);
      if (request.type === 'start') approvedSelection = '';
      const chat = await execute({ type: 'read', projectId });
      if (next && (next.manifest.projectId !== projectId || chat?.runId !== next.runId)) throw new Error('Wrong research session');
      if (mounted) { accept(chat); research = next; }
    } catch { if (mounted) researchError = 'research.actionFailed'; }
    finally { busy = false; }
  }
  async function reviewCandidate() {
    if (!onReviewCandidate || !editedCandidate || busy) return;
    busy = true; researchError = '';
    try { await onReviewCandidate(structuredClone($state.snapshot(editedCandidate))); }
    catch { if (mounted) researchError = 'research.actionFailed'; }
    finally { busy = false; }
  }
  function updatesFor(turn: number): ChatUpdate['kind'][] {
    return view?.updates.flatMap(({ kind }) => {
      const binding = kind.type === 'permission_pending' ? kind.request.binding : 'binding' in kind ? kind.binding : null;
      return binding?.turnId === turn ? [kind] : [];
    }) ?? [];
  }
  function decide(optionId: string | null) {
    if (!view || !permission) return;
    void act({ type: 'control', runId: view.runId, control: { type: 'permission', binding: permission.binding, requestId: permission.requestId, choice: optionId ? { type: 'select', optionId } : { type: 'cancel' } } });
  }
</script>

<section class="rounded-xl border border-border bg-card p-4 text-foreground md:p-6" aria-labelledby="all-chat-title" data-all-chat>
  <header class="mb-5 flex flex-wrap items-start justify-between gap-3">
    <div><h2 class="mb-2 text-xl font-semibold" id="all-chat-title">{t('chat.title')}</h2><p>{t('chat.description')}</p></div>
    <span class="text-sm font-semibold" role="status">{permission ? t('chat.permission') : t(`chat.status.${view?.status ?? 'ready'}`)}</span>
  </header>
  <div class="flex flex-wrap items-center gap-3">
    <label for="chat-agent">{t('chat.agent')}</label>
    <select class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm md:w-auto" id="chat-agent" disabled={!!view && !terminal}>
      <option value="offline_demo">{t('chat.demo')}</option>
      <option disabled>Codex — {t('chat.later')}</option>
      <option disabled>Claude — {t('chat.later')}</option>
    </select>
    {#if !view || terminal}
      <Button disabled={busy || !loaded} onclick={() => act({ type: 'start', projectId, expectedProjectRevision: projectRevision, agent: 'offline_demo' })}>{t(view ? 'chat.restart' : 'chat.start')}</Button>
    {:else}
      <Button variant="outline" disabled={busy || view.status === 'closing'} onclick={() => act({ type: 'close', runId: view!.runId })}>{t('chat.close')}</Button>
    {/if}
  </div>
  <p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{t('chat.memory')}</p>
  {#if researchError}<div class="my-3" role="alert"><p>{t(researchError)}</p><Button variant="outline" disabled={busy || reading} onclick={() => void read()}>{t('chat.retry')}</Button></div>{/if}
  {#if terminal && !research}<p class="mt-2 text-sm text-muted-foreground">{t('research.notRestored')}</p>{/if}
  {#if research}
    <section class="my-5 space-y-3 border-y border-border py-4" aria-label={t('research.loaded')}>
      <h3 class="font-semibold">{t('research.loaded')}</h3>
      <p class="text-sm text-muted-foreground">{t('research.demoNote')}</p>
      <p class="text-sm">{t('research.readAccess')}</p>
      <p class="break-words text-sm">{t('research.method')}: {research.manifest.methodPath}</p>
      {#each research.manifest.sources as source, index}
        <details open>
          <summary class="cursor-pointer py-2 text-sm font-medium">{t('research.excerpt', { number: index + 1 })}: {source.selection.artifactPath} · L{source.selection.startLine}–L{source.selection.endLine}</summary>
          <p class="text-sm text-muted-foreground">{source.truncatedBefore ? t('research.beforeOmitted') : t('research.fromBeginning')} · {source.truncatedAfter ? t('research.afterOmitted') : t('research.toEnd')}</p>
          <pre class="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-words text-sm">{source.content}</pre>
        </details>
      {/each}
      {#if research.error}<p role="alert">{t('research.candidateFailed')}</p>{/if}
      {#if research.candidate && editedCandidate}
        {@const candidate = editedCandidate}
        <section aria-label={t('research.candidate')} class="space-y-4 border-t border-border pt-4">
          <h3 class="font-semibold">{t('research.candidate')} · {t('chat.turn', { number: candidate.turnId })}</h3>
          <p class="text-sm text-muted-foreground">{t(onReviewCandidate ? 'research.captureReviewNote' : 'research.unsaved')}</p>
          {#each [{ label: 'research.methods', findings: candidate.draft.methods }, { label: 'research.conclusions', findings: candidate.draft.conclusions }, { label: 'research.comparison', findings: [candidate.draft.comparison] }, { label: 'research.limitations', findings: [candidate.draft.limitations] }] as group}
            <div><h4 class="font-medium">{t(group.label)}</h4>
              {#each group.findings as finding, index}
                <label class="mt-2 block text-sm" for={`candidate-${group.label}-${index}`}>{t('research.editFinding', { group: t(group.label), number: index + 1 })}</label>
                <textarea id={`candidate-${group.label}-${index}`} class="mt-1 min-h-20 w-full resize-y rounded-lg border border-input bg-background px-3 py-2 text-sm disabled:opacity-50" rows="3" maxlength={750} disabled={busy} bind:value={finding.text}></textarea>
                {#each finding.citations as citation}
                  <blockquote class="my-2 border-l-2 border-border pl-3 text-sm">
                    <p class="whitespace-pre-wrap break-words">{citation.quote}</p>
                    <p class="mt-1 break-words text-muted-foreground">{research.manifest.sources.find((source) => source.sourceId === citation.sourceId)?.selection.artifactPath} · L{citation.startLine}–L{citation.endLine}</p>
                  </blockquote>
                {/each}
              {/each}
            </div>
          {/each}
          <div class="flex flex-wrap gap-3">
            {#if onReviewCandidate}<Button disabled={busy || view?.status !== 'idle' || !!research.error} onclick={reviewCandidate}>{t('research.reviewCapture')}</Button>{/if}
            <Button variant="outline" disabled={busy} onclick={() => researchAction({ type: 'dismiss', runId: candidate.runId, turnId: candidate.turnId })}>{t('research.dismiss')}</Button>
          </div>
        </section>
      {/if}
    </section>
  {/if}
  {#if error}
    <div class="my-3" role="alert"><p>{t(error)}</p><Button variant="outline" disabled={busy || reading} onclick={() => { error = ''; void read(); }}>{t('chat.retry')}</Button></div>
  {/if}
  {#if view?.error}<p class="my-3" role="status">{t(historyError(view.error) ? 'chat.historyUnavailable' : 'chat.interrupted')}</p>{/if}
  {#if !view?.prompts.length}
    <p class="py-8 text-muted-foreground">{t('chat.empty')}</p>
  {:else}
    <ol class="my-7 list-none space-y-6 p-0" aria-label={t('chat.timeline')}>
      {#each view.prompts as prompt, index}
        {@const updates = updatesFor(index + 1)}
        {@const lastPlan = updates.findLast((k) => k.type === 'plan')}
        {@const turnStatus = updates.findLast((k) => k.type === 'turn')}
        <li>
          <div class="mb-2 flex gap-3 text-sm font-semibold">{t('chat.you')} <span>{t('chat.turn', { number: index + 1 })}</span></div>
          <p class="max-w-prose whitespace-pre-wrap break-words text-sm leading-relaxed">{prompt.text}</p>
          {#if prompt.context || prompt.sourceRefs.length}
            <details><summary class="cursor-pointer py-2 text-sm font-medium">{t('chat.contextUsed')}</summary><p class="max-w-prose whitespace-pre-wrap break-words text-sm leading-relaxed">{prompt.context}</p>{#each prompt.sourceRefs as source}<p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{source} · {t('chat.notRead')}</p>{/each}</details>
          {/if}
          <div class="mt-5">
            <div class="mb-2 flex gap-3 text-sm font-semibold">{t('chat.demo')} <span>{t('chat.primary')}</span></div>
            {#if lastPlan?.type === 'plan'}<ul class="my-2 list-disc pl-5 text-sm leading-relaxed">{#each lastPlan.entries as entry}<li>{entry.content} · {t(`chat.activity.${entry.status}`)}</li>{/each}</ul>{/if}
            {#each updates.filter((k) => k.type === 'tool') as tool}{#if tool.type === 'tool'}<p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{tool.title ?? tool.toolCallId} · {tool.status ? t(`chat.activity.${tool.status}`) : t('chat.activity.pending')}</p>{/if}{/each}
            <p class="max-w-prose whitespace-pre-wrap break-words text-sm leading-relaxed">{updates.filter((k) => k.type === 'text').map((k) => k.type === 'text' ? k.content : '').join('')}</p>
            {#if turnStatus?.type === 'turn'}<p class="mt-2 text-sm text-muted-foreground">{permission?.binding.turnId === index + 1 ? t('chat.permission') : t(`chat.turnStatus.${terminal && turnStatus.status === 'running' ? 'interrupted' : turnStatus.status}`)}</p>{/if}
          </div>
        </li>
      {/each}
    </ol>
  {/if}
  {#if permission}
    <section class="mb-5 border-y border-border py-5" aria-labelledby="chat-permission-title">
      <h3 class="text-base font-semibold" id="chat-permission-title">{t('chat.permission')}</h3><p>{permission.title}</p>
      <p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{t('chat.permissionNote')}</p>
      <div class="mt-3 flex flex-wrap items-center gap-3">{#each permission.options as option}<Button variant={option.kind === 'allow_once' ? 'default' : 'outline'} disabled={busy || !option.enabled} onclick={() => decide(option.optionId)}>{t(`chat.choice.${option.kind}`)}</Button>{/each}<Button variant="ghost" disabled={busy} onclick={() => decide(null)}>{t('chat.dismissPermission')}</Button></div>
    </section>
  {/if}
  <form onsubmit={(event) => { event.preventDefault(); void send(); }}>
    <details class="my-5 border-t border-border">
      <summary class="cursor-pointer py-2 text-sm font-medium">{t('chat.context')}</summary>
      <label class="flex items-center gap-2 py-2 text-sm"><input type="checkbox" disabled={!!research} bind:checked={useContext} />{t('chat.manualContext')}</label>
      {#if useContext}<label for="chat-context">{t('chat.contextText')}</label><textarea class="mt-2 min-h-20 w-full resize-y rounded-lg border border-input bg-background px-3 py-2 text-sm disabled:opacity-50" id="chat-context" disabled={!!research} bind:value={context} maxlength={65536} rows="3"></textarea>{/if}
      <div class="mt-3 flex flex-wrap items-center justify-between gap-3"><span>{t('chat.sources')}</span><Button type="button" variant="ghost" onclick={onLoadSources}>{t('chat.loadSources')}</Button></div>
      <p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{t('chat.sourcesNote')}</p>
      {#each sources as source}<label class="flex items-center gap-2 py-2 text-sm"><input type="checkbox" disabled={!!research} value={source} bind:group={sourceRefs} />{source}</label>{:else}<p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{t('chat.noSources')}</p>{/each}
      <fieldset class="mt-4 space-y-3 border-t border-border pt-3" disabled={busy || !!view && !terminal}>
        <legend class="pt-3 font-medium">{t('research.select')}</legend>
        <p class="text-sm text-muted-foreground">{t('research.selectionNote')}</p>
        {#each selections as selection, index}
          <div class="flex flex-wrap items-end gap-3">
            <label class="min-w-0 flex-1 text-sm" for={`research-source-${index}`}>{t('research.excerpt', { number: index + 1 })}
              <select id={`research-source-${index}`} class="mt-1 w-full rounded-lg border border-input bg-background px-3 py-2" bind:value={selection.artifactPath}>
                <option value="">{t('research.chooseSource')}</option>{#each sources as source}<option value={source}>{source}</option>{/each}
              </select>
            </label>
            <label class="text-sm" for={`research-start-${index}`}>{t('research.startLine', { number: index + 1 })}<input id={`research-start-${index}`} class="mt-1 block w-24 rounded-lg border border-input bg-background px-2 py-2" type="number" min="1" step="1" bind:value={selection.startLine} /></label>
            <label class="text-sm" for={`research-end-${index}`}>{t('research.endLine', { number: index + 1 })}<input id={`research-end-${index}`} class="mt-1 block w-24 rounded-lg border border-input bg-background px-2 py-2" type="number" min="1" step="1" bind:value={selection.endLine} /></label>
          </div>
        {/each}
        {#if !view || terminal}
          <label class="flex items-start gap-2 py-2 text-sm"><input type="checkbox" class="mt-1" checked={approvedSelection === selectionKey} onchange={(event) => approvedSelection = event.currentTarget.checked ? selectionKey : ''} />{t('research.authorizeRead')}</label>
          <Button type="button" disabled={busy || !loaded || approvedSelection !== selectionKey || !selections.every((selection) => sources.includes(selection.artifactPath) && Number.isSafeInteger(selection.startLine) && selection.startLine > 0 && Number.isSafeInteger(selection.endLine) && selection.endLine >= selection.startLine)} onclick={() => researchAction({ type: 'start', projectId, expectedProjectRevision: projectRevision, contextAccess: 'selected_excerpts', selections })}>{t('research.start')}</Button>{/if}
      </fieldset>
    </details>
    <label for="chat-prompt">{t('chat.message')}</label>
    <textarea class="mt-2 min-h-20 w-full resize-y rounded-lg border border-input bg-background px-3 py-2 text-sm disabled:opacity-50" id="chat-prompt" bind:value={text} maxlength={65536} rows="3" disabled={!view || view.status !== 'idle' || busy} placeholder={t('chat.placeholder')}></textarea>
    <div class="mt-3 flex flex-wrap items-center justify-end gap-3"><p class="mt-2 text-sm leading-relaxed text-muted-foreground break-words">{t('chat.sendHint')}</p>
      {#if view?.status === 'active' && activeBinding?.type === 'turn'}<Button type="button" variant="outline" disabled={busy} onclick={() => act({ type: 'control', runId: view!.runId, control: { type: 'cancel', binding: activeBinding.binding } })}>{t('chat.cancel')}</Button>{/if}
      <Button type="submit" disabled={busy || !text.trim() || view?.status !== 'idle'}>{t('chat.send')}</Button>
    </div>
  </form>
</section>

<style>
  summary:focus-visible { outline: 2px solid var(--ring); outline-offset: 2px; }
  textarea::placeholder { color: var(--muted-foreground); }
  [aria-label] > li + li { border-top: 1px solid var(--border); padding-top: 24px; }
</style>
