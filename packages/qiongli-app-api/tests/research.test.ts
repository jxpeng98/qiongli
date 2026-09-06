import { describe, expect, it } from 'vitest';
import { parseResearchCandidate, researchContractSchema, researchContextManifestSchema, researchFindingSchema, QiongliResearchClient, researchSnapshotSchema, researchRequestSchema } from '../src/research';

const fixtureUrl = new URL('../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-research-v2.json', import.meta.url);
const { default: fixture } = await import(/* @vite-ignore */ fixtureUrl.href, { with: { type: 'json' } });
const controlsUrl = new URL('../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-research-control-v2.json', import.meta.url);
const { default: controls } = await import(/* @vite-ignore */ controlsUrl.href, { with: { type: 'json' } });
const golden = () => researchContractSchema.parse(fixture);

describe('Rust research context and candidate contract', () => {
  it('consumes native goldens and checks native context/turn bindings without creating digests', () => {
    const { manifest, candidate } = golden();
    const binding = { runId: candidate.runId, turnId: candidate.turnId, manifestDigest: candidate.manifestDigest };
    expect(parseResearchCandidate(candidate, manifest, binding)).toEqual(candidate);
    for (const wrong of [{ ...binding, runId: `run_${'f'.repeat(32)}` }, { ...binding, turnId: binding.turnId + 1 },
      { ...binding, manifestDigest: 'f'.repeat(64) }]) {
      expect(() => parseResearchCandidate(candidate, manifest, wrong)).toThrow();
    }
  });

  it('rejects closed-shape, scope, permission, UTF-8 bounds and citation violations', () => {
    const { manifest, candidate } = golden();
    for (const bad of [{ ...manifest, schemaVersion: 1 }, { ...manifest, extra: true },
      { ...manifest, projectRevision: Number.MAX_SAFE_INTEGER + 1 }, { ...manifest, allowedTools: ['shell'] },
      { ...manifest, allowedTools: [] }, { ...manifest, allowedTools: ['fs/read_text_file', 'fs/write_text_file'] },
      { ...manifest, readPaths: [...manifest.readPaths, '/etc/passwd'] },
      { ...manifest, readPaths: [manifest.readPaths[1], manifest.readPaths[0], manifest.readPaths[2]] },
      { ...manifest, readPaths: [manifest.readPaths[0], manifest.readPaths[1], `/qiongli-context/method-${'f'.repeat(64)}.md`] },
      { ...manifest, readPaths: [manifest.readPaths[0] + '/../other.txt', ...manifest.readPaths.slice(1)] },
      { ...manifest, sources: [manifest.sources[0], manifest.sources[0]] }, { ...manifest, methodDigest: 'A'.repeat(64) },
      { ...manifest, sources: [{ ...manifest.sources[0], content: 'one line only' }, manifest.sources[1]] },
      { ...manifest, sources: [{ ...manifest.sources[0], content: '字'.repeat(6000) }, manifest.sources[1]] }]) {
      expect(researchContextManifestSchema.safeParse(bad).success).toBe(false);
    }
    const overlap = structuredClone(manifest);
    overlap.sources[1].selection = { ...overlap.sources[0].selection };
    expect(researchContextManifestSchema.safeParse(overlap).success).toBe(false);
    for (const value of ['字'.repeat(251), '\u0085', 'bad\0text', 'bad\u007ftext', '\ud800']) {
      expect(researchFindingSchema.safeParse({ ...candidate.draft.comparison, text: value }).success).toBe(false);
    }
    const mutations: Array<(value: ReturnType<typeof golden>) => void> = [
      (value) => { value.candidate.draft.methods[0].citations[0]!.sourceId = `src_${'f'.repeat(64)}`; },
      (value) => { value.candidate.draft.methods[0].citations[0]!.startLine = Number.MAX_SAFE_INTEGER; },
      (value) => { value.candidate.draft.methods[0].citations[0]!.quote = 'not present in the selected source'; },
      (value) => { value.candidate.draft.methods.reverse(); },
      (value) => { value.candidate.draft.conclusions.reverse(); },
      (value) => { value.candidate.draft.comparison.citations = [value.candidate.draft.methods[0].citations[0]!]; },
      (value) => { value.candidate.draft.limitations.citations = []; }
    ];
    for (const mutate of mutations) {
      const value = structuredClone({ manifest, candidate });
      mutate(value);
      expect(researchContractSchema.safeParse(value).success).toBe(false);
    }
    expect(researchContractSchema.safeParse({ manifest, candidate: { ...candidate, reviewed: true } }).success).toBe(false);
  });

  it('uses the research IPC command and rejects unowned response candidates', async () => {
    for (const entry of controls as Array<{ request: unknown; response: unknown }>) {
      const client = new QiongliResearchClient({ async invoke<T>() { return entry.response as T; } });
      expect(await client.execute(researchRequestSchema.parse(entry.request))).toEqual(entry.response);
    }
    const { manifest, candidate } = golden();
    const snapshot = { runId: candidate.runId, manifestDigest: candidate.manifestDigest, manifest, candidate, error: null };
    let invocation: unknown;
    const client = new QiongliResearchClient({ async invoke<T>(command: string, args?: Record<string, unknown>) {
      invocation = { command, args }; return snapshot as T;
    } });
    const request = { type: 'start' as const, contextAccess: 'selected_excerpts' as const, projectId: manifest.projectId, expectedProjectRevision: manifest.projectRevision,
      selections: manifest.sources.map((source) => source.selection) as [typeof manifest.sources[0]['selection'], typeof manifest.sources[1]['selection']] };
    const { contextAccess: _access, ...withoutAccess } = request;
    expect(researchRequestSchema.safeParse(withoutAccess).success).toBe(false);
    expect(researchRequestSchema.safeParse({ ...request, contextAccess: 'project' }).success).toBe(false);
    expect(researchRequestSchema.safeParse({ ...request, allowedTools: ['shell'] }).success).toBe(false);
    expect(await client.execute(request)).toEqual(snapshot);
    expect(invocation).toEqual({ command: 'qiongli_all_chat_research', args: { request } });
    const otherSelection = structuredClone(request);
    otherSelection.selections[0].endLine++;
    await expect(client.execute(otherSelection)).rejects.toThrow('Research response does not match the requested scope');
    await expect(client.execute({ type: 'read', runId: `run_${'f'.repeat(32)}` })).rejects.toThrow();
    expect(researchSnapshotSchema.safeParse({ ...snapshot, candidate: { ...candidate, manifestDigest: 'f'.repeat(64) } }).success).toBe(false);
  });

  it('requests an existing Capture intake preview without confirming or declaring a saved artifact', async () => {
    const { candidate, manifest } = golden();
    const preview = { type: 'capture-intake-preview', intake: {
      schemaVersion: 1, planDigest: '0'.repeat(64), captureId: `cap_${'0'.repeat(64)}`, projectId: manifest.projectId,
      disposition: 'unresolved-candidate', effect: 'append-pending-history', source: 'manual', delivery: 'manual',
      expectedLibraryRevision: 1, expectedProjectRevision: 1, changeCount: 4, decisionCount: 0, evidenceCount: 2,
      contradictionCount: 0, nextActionCount: 0, historyEntry: 'context/captures/fixture.json', approvalsRequired: ['filesystem-write']
    }, preview: { token: '0'.repeat(32), kind: 'capture-intake', title: 'Review Capture intake', summary: 'Append pending Capture',
      displayTarget: 'context/captures/fixture.json', planDigestSha256: '0'.repeat(64), approvalsRequired: ['filesystem-write'],
      canConfirm: true, blockedReason: null } };
    const calls: unknown[] = [];
    let response: unknown = preview;
    const client = new QiongliResearchClient({ async invoke<T>(command: string, args?: Record<string, unknown>) {
      calls.push({ command, args }); return response as T;
    } });
    expect(await client.previewCapture(candidate)).toEqual(preview);
    expect(calls).toEqual([{ command: 'qiongli_research_capture', args: { candidate } }]);
    response = { ...preview, saved: true };
    await expect(client.previewCapture(candidate)).rejects.toThrow();
    const invalid = structuredClone(candidate);
    invalid.draft.methods[0].text = '字'.repeat(251);
    const before = calls.length;
    await expect(client.previewCapture(invalid)).rejects.toThrow();
    expect(calls).toHaveLength(before);
  });
});
