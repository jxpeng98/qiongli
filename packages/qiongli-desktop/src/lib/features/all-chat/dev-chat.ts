// Browser development replay of real native IPC fixtures; never included in production.
import { chatRequestSchema, chatSnapshotSchema, researchRequestSchema, type AppTransport, type ChatSnapshot } from '@qiongli/app-api';
import fixtures from '../../../../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-control-v1.json';

export function chatFixtureTransport(): AppTransport {
  const recordings = fixtures.map((v) => chatSnapshotSchema.parse(v));
  let view: ChatSnapshot | null = null;
  let revision = 0;
  const full = recordings[2]!;
  return { async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    if (command === 'qiongli_all_chat_research') {
      if (researchRequestSchema.parse(args?.request).type === 'read') return null as T;
      throw new Error('Research context requires the native development App');
    }
    const request = chatRequestSchema.parse(args?.request);
    if (request.type === 'start') {
      view = { ...structuredClone(recordings[0]!), projectId: request.projectId, expectedProjectRevision: request.expectedProjectRevision, status: 'idle', prompts: [], nextTurn: 1, updates: [] };
    } else if (request.type !== 'read') {
      if (!view || view.runId !== request.runId) throw new Error('Stale run');
      if (request.type === 'prompt') {
        if (view.status !== 'idle' || view.nextTurn !== request.expectedTurn || view.nextTurn > 2) throw new Error('Replay limited to two turns');
        view.prompts.push(request.prompt); view.nextTurn++;
        view.updates = structuredClone(recordings[view.prompts.length - 1]!.updates);
        view.status = 'active';
      } else if (request.type === 'control') {
        if (view.status !== 'active') throw new Error('Stale control');
        const turn = view.prompts.length;
        if (request.control.type === 'cancel') { view.status = 'interrupted'; view.error = 'all-chat-session-interrupted';
          view.updates.push({ schemaVersion: 1, connectionId: request.control.binding.connectionId, sequence: view.updates.length + 1, runId: view.runId, role: 'primary', kind: { type: 'turn', binding: request.control.binding, status: 'cancelled' } }); }
        else {
          view.updates = structuredClone(full.updates.filter(({ kind }) => {
            const b = kind.type === 'permission_pending' ? kind.request.binding : 'binding' in kind ? kind.binding : null;
            return !b || b.turnId <= turn;
          }));
          // Record the selected replay choice so the UI cannot imply a denied action was allowed.
          for (const { kind } of view.updates) {
            if (kind.type === 'permission_resolved' && kind.binding.turnId === turn) kind.choice = request.control.choice;
            if (kind.type === 'text' && kind.binding.turnId === turn) kind.content = 'Offline browser replay. No source was read and no file was changed.';
          }
          view.status = 'idle';
        }
      } else { view.status = 'closed'; }
    }
    if (view) view.revision = ++revision;
    return structuredClone(view) as T;
  } };
}
