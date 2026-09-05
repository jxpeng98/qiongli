import { z } from 'zod';
import type { AppTransport } from './client';
import { tauriTransport } from './client';

const integer = z.number().int().min(0).max(Number.MAX_SAFE_INTEGER);
const id = z.string().regex(/^run_[0-9a-f]{32}$/);
const projectId = z.string().regex(/^prj_[0-9a-f]{32}$/);
const short = z.string().min(1).max(512);
const role = z.enum(['primary', 'reviewer', 'verifier']);
const binding = z.object({ connectionId: short, runId: id, role, sessionId: z.string().min(1).max(256), turnId: integer.min(1) }).strict();
const choice = z.discriminatedUnion('type', [z.object({ type: z.literal('cancel') }).strict(), z.object({ type: z.literal('select'), optionId: short }).strict()]);
const activity = z.enum(['pending', 'in_progress', 'completed', 'failed']);
const permission = z.object({ binding, requestId: integer.min(1), toolCallId: short, title: z.string().min(1).max(4096), options: z.array(z.object({ optionId: short, name: z.string().min(1).max(4096), kind: z.enum(['allow_once', 'reject_once', 'allow_always', 'reject_always']), enabled: z.boolean() }).strict()).min(1).max(16) }).strict();
const sessionInfo = z.object({ adapter: short.nullable(), sessionEstablished: z.boolean(), authenticationRequired: z.boolean(), authMethodIds: z.array(short).max(64), loadAdvertised: z.boolean(), resumeAdvertised: z.boolean(), modeIds: z.array(short).max(64), currentModeId: short.nullable(), modelIds: z.array(short).max(64), currentModelId: short.nullable(), loadEnabled: z.literal(false), resumeEnabled: z.literal(false), modeSelectionEnabled: z.literal(false), modelSelectionEnabled: z.literal(false) }).strict();
export const chatUpdateSchema = z.object({ schemaVersion: z.literal(1), connectionId: short, sequence: integer.min(1), runId: id, role, kind: z.discriminatedUnion('type', [
  z.object({ type: z.literal('session'), info: sessionInfo }).strict(),
  z.object({ type: z.literal('turn'), binding, status: z.enum(['running', 'completed', 'cancelled', 'timed_out', 'failed', 'interrupted']) }).strict(),
  z.object({ type: z.literal('text'), binding, content: z.string().min(1).max(65536) }).strict(),
  z.object({ type: z.literal('plan'), binding, entries: z.array(z.object({ content: z.string().min(1).max(4096), status: activity }).strict()).max(64) }).strict(),
  z.object({ type: z.literal('tool'), binding, toolCallId: short, title: z.string().min(1).max(4096).nullable(), status: activity.nullable() }).strict(),
  z.object({ type: z.literal('permission_pending'), request: permission }).strict(),
  z.object({ type: z.literal('permission_resolved'), binding, requestId: integer.min(1), choice }).strict()
]) }).strict();
const byteLength = (text: string) => new TextEncoder().encode(text).length;
const inputText = z.string().max(65536).refine((s) => byteLength(s) <= 65536 && !s.includes('\0'));
const prompt = z.object({ text: inputText.refine((s) => s.trim().length > 0), context: inputText, sourceRefs: z.array(short.refine((s) => byteLength(s) <= 512 && !/[\x00-\x1f\x7f]/.test(s))).max(16).refine((refs) => new Set(refs).size === refs.length) }).strict();
const agent = z.enum(['offline_demo', 'codex', 'claude']);
export const chatRequestSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('read'), projectId }).strict(),
  z.object({ type: z.literal('start'), projectId, expectedProjectRevision: integer.min(1), agent }).strict(),
  z.object({ type: z.literal('prompt'), runId: id, expectedTurn: integer.min(1), prompt }).strict(),
  z.object({ type: z.literal('control'), runId: id, control: z.discriminatedUnion('type', [z.object({ type: z.literal('cancel'), binding }).strict(), z.object({ type: z.literal('permission'), binding, requestId: integer.min(1), choice }).strict()]) }).strict(),
  z.object({ type: z.literal('close'), runId: id }).strict()
]);
export const chatSnapshotSchema = z.object({ schemaVersion: z.literal(1), projectId, expectedProjectRevision: integer.min(1), runId: id, agent, revision: integer.min(1), status: z.enum(['starting', 'idle', 'active', 'closing', 'closed', 'interrupted']), nextTurn: integer.min(1).max(65), prompts: z.array(prompt).max(64), updates: z.array(chatUpdateSchema).max(2048), error: short.nullable() }).strict().superRefine((view, ctx) => {
  if (view.nextTurn !== view.prompts.length + 1 || byteLength(JSON.stringify(view)) > 8 * 1024 * 1024) ctx.addIssue({ code: 'custom', message: 'Invalid chat bounds' });
  let currentTurn = 0;
  let terminal = false;
  let session: string | undefined;
  const connection = view.updates[0]?.connectionId;
  for (const [index, update] of view.updates.entries()) {
    const kind = update.kind;
    const b = kind.type === 'permission_pending' ? kind.request.binding : 'binding' in kind ? kind.binding : null;
    let valid = update.sequence === index + 1 && update.runId === view.runId && update.role === 'primary' && update.connectionId === connection;
    if (b) {
      valid &&= b.connectionId === connection && b.runId === view.runId && b.role === 'primary' && b.turnId < view.nextTurn;
      session ??= b.sessionId;
      valid &&= b.sessionId === session;
      if (kind.type === 'turn' && kind.status === 'running') {
        valid &&= b.turnId === currentTurn + 1 && (currentTurn === 0 || terminal);
        currentTurn = b.turnId; terminal = false;
      } else {
        valid &&= b.turnId === currentTurn && !terminal;
        if (kind.type === 'turn') terminal = true;
      }
    }
    if (!valid) ctx.addIssue({ code: 'custom', path: ['updates', index], message: 'Out-of-order or unowned update' });
  }
});
export type ChatRequest = z.infer<typeof chatRequestSchema>;
export type ChatSnapshot = z.infer<typeof chatSnapshotSchema>;
export type ChatUpdate = z.infer<typeof chatUpdateSchema>;
export type ChatPermission = z.infer<typeof permission>;

export class QiongliChatClient {
  constructor(private readonly transport: AppTransport = tauriTransport) {}
  async execute(request: ChatRequest): Promise<ChatSnapshot | null> {
    return chatSnapshotSchema.nullable().parse(await this.transport.invoke('qiongli_all_chat', { request: chatRequestSchema.parse(request) }));
  }
}

export function pendingChatPermission(view: ChatSnapshot): ChatPermission | null {
  if (view.status !== 'active') return null;
  for (const { kind } of [...view.updates].reverse()) {
    if (kind.type === 'permission_pending') return kind.request;
    if (kind.type === 'permission_resolved' || kind.type === 'turn' && kind.status !== 'running') return null;
  }
  return null;
}

// Committed reducer snapshots remain separate from the transient App session stream.
const hash = z.string().regex(/^[0-9a-f]{64}$/);
const taskId = z.string().regex(/^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/);
const content = z.string().min(1).max(65536);
export const allChatCommittedSnapshotSchema = z.object({
  schemaVersion: z.literal(1), runId: id, projectId, expectedProjectRevision: integer.min(1), generation: integer,
  status: z.enum(['planned', 'running', 'paused', 'completed', 'failed', 'cancelled']),
  participants: z.array(z.object({ role, backendId: z.string().regex(/^[a-z0-9][a-z0-9._-]{0,63}$/), sessionId: z.string().min(1).max(256).nullable() }).strict()).min(1).max(3),
  events: z.array(z.object({ sequence: integer.min(1), generation: integer.min(1), kind: z.discriminatedUnion('type', [
    z.object({ type: z.literal('run_started') }).strict(),
    z.object({ type: z.literal('user_message'), content }).strict(),
    z.object({ type: z.literal('agent_session_ready'), role, sessionId: z.string().min(1).max(256) }).strict(),
    z.object({ type: z.literal('task_delegated'), by: role, to: role, taskId, taskSha256: hash }).strict(),
    z.object({ type: z.literal('task_result'), by: role, taskId, resultSha256: hash }).strict(),
    z.object({ type: z.literal('coordinator_message'), by: role, content }).strict(),
    z.object({ type: z.literal('agent_turn_completed'), by: role, finishReason: z.enum(['stop', 'length']) }).strict(),
    z.object({ type: z.literal('agent_turn_cancelled'), by: role }).strict(),
    z.object({ type: z.literal('run_completed'), by: role }).strict(),
    z.object({ type: z.literal('run_failed') }).strict(),
    z.object({ type: z.literal('run_cancelled') }).strict()
  ]) }).strict()).max(1024)
}).strict();
export type AllChatCommittedSnapshot = z.infer<typeof allChatCommittedSnapshotSchema>;
