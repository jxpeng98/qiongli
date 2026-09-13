import { describe, it, expect } from 'vitest';
import { allChatCommittedSnapshotSchema, chatSnapshotSchema, chatRequestSchema, QiongliChatClient, pendingChatPermission } from '../src/all-chat';
import fixtures from '../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-control-v1.json';
import completed from '../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-app-v1.completed.json';
import cancelled from '../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-app-v1.cancelled.json';

describe('Rust All Chat contracts', () => {
  it('strictly decodes native IPC and committed reducer goldens without merging their statuses', async () => {
    for (const fixture of fixtures) expect(chatSnapshotSchema.parse(fixture)).toEqual(fixture);
    for (const fixture of [completed, cancelled]) allChatCommittedSnapshotSchema.parse(fixture);
    const pending = chatSnapshotSchema.parse(fixtures[0]);
    expect(pendingChatPermission(pending)?.options.map((o) => o.kind)).toEqual(['allow_once', 'reject_once']);
    expect(pendingChatPermission(chatSnapshotSchema.parse(fixtures[2]))).toBeNull();
    let invocation: unknown;
    const client = new QiongliChatClient({ async invoke<T>(command: string, args?: Record<string, unknown>) { invocation = { command, args }; return fixtures[0] as T; } });
    await client.execute({ type: 'read', projectId: pending.projectId });
    expect(invocation).toEqual({ command: 'qiongli_all_chat', args: { request: { type: 'read', projectId: pending.projectId } } });
  });
  it('rejects future schemas, extra fields, broken scope/order, unsafe integers and oversized input', () => {
    const view = chatSnapshotSchema.parse(fixtures[0]);
    for (const bad of [{ ...view, schemaVersion: 2 }, { ...view, extra: true }, { ...view, revision: Number.MAX_SAFE_INTEGER + 1 }, { ...view, nextTurn: 50 }, { ...view, updates: [...view.updates].reverse() }, { ...view, updates: view.updates.map((u) => ({ ...u, runId: `run_${'f'.repeat(32)}` })) }]) expect(chatSnapshotSchema.safeParse(bad).success).toBe(false);
    expect(chatRequestSchema.safeParse({ type: 'prompt', runId: view.runId, expectedTurn: 1, prompt: { text: '字'.repeat(30000), context: '', sourceRefs: [] } }).success).toBe(false);
    expect(chatRequestSchema.safeParse({ type: 'read', projectId: view.projectId, ignored: true }).success).toBe(false);
  });
});
