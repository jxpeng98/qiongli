import { z } from 'zod';
import { tauriTransport, type AppTransport } from './client';
import { appEventSchema, type AppEvent } from './schema';

const integer = z.number().int().min(1).max(Number.MAX_SAFE_INTEGER);
const digest = z.string().regex(/^[0-9a-f]{64}$/);
const sourceId = z.string().regex(/^src_[0-9a-f]{64}$/);
const runId = z.string().regex(/^run_[0-9a-f]{32}$/);
const bytes = (value: string) => new TextEncoder().encode(value).length;
const text = (max: number) => z.string().min(1).max(max).refine((value) =>
  bytes(value) <= max && /[^\p{White_Space}]/u.test(value) && !/[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f\ud800-\udfff]/u.test(value));
const lines = (value: string) => {
  const result = value.split(/\r?\n/);
  if (value.endsWith('\n')) result.pop();
  return result;
};

// The native registered-artifact reader owns path authorization and source digests.
export const researchSourceSelectionSchema = z.object({
  artifactPath: z.string().min(1).max(256).refine((value) => bytes(value) <= 256),
  startLine: integer,
  endLine: integer
}).strict().refine((value) => value.startLine <= value.endLine);
export const researchContextSourceSchema = z.object({
  sourceId,
  selection: researchSourceSelectionSchema,
  contentDigest: digest,
  content: text(16 * 1024),
  truncatedBefore: z.boolean(),
  truncatedAfter: z.boolean()
}).strict().refine((source) => lines(source.content).length === source.selection.endLine - source.selection.startLine + 1,
  { message: 'Source content does not cover the selected line range' });
export const researchContextManifestSchema = z.object({
  schemaVersion: z.literal(2),
  projectId: z.string().regex(/^prj_[0-9a-f]{32}$/),
  projectRevision: integer,
  sources: z.tuple([researchContextSourceSchema, researchContextSourceSchema]),
  methodPath: z.literal('skills/B_literature/paper-extractor.md'),
  methodDigest: digest,
  allowedTools: z.tuple([z.literal('fs/read_text_file')]),
  readPaths: z.tuple([z.string(), z.string(), z.string()])
}).strict().superRefine((manifest, ctx) => {
  const [a, b] = manifest.sources;
  const expectedPaths = [`/qiongli-context/${a.sourceId}.txt`, `/qiongli-context/${b.sourceId}.txt`,
    `/qiongli-context/method-${manifest.methodDigest}.md`];
  if (manifest.readPaths.some((path, index) => path !== expectedPaths[index])) {
    ctx.addIssue({ code: 'custom', path: ['readPaths'], message: 'Read paths must match the selected excerpts and method' });
  }
  if (a.sourceId === b.sourceId || a.selection.artifactPath === b.selection.artifactPath &&
      a.selection.startLine <= b.selection.endLine && b.selection.startLine <= a.selection.endLine) {
    ctx.addIssue({ code: 'custom', path: ['sources'], message: 'Selected sources must be distinct and nonoverlapping' });
  }
});
export const researchCitationSchema = z.object({
  sourceId,
  startLine: integer,
  endLine: integer,
  quote: text(1000)
}).strict().refine((value) => value.startLine <= value.endLine);
export const researchFindingSchema = z.object({
  text: text(750),
  citations: z.array(researchCitationSchema).min(1).max(2)
}).strict();
export const researchComparisonDraftSchema = z.object({
  methods: z.tuple([researchFindingSchema, researchFindingSchema]),
  conclusions: z.tuple([researchFindingSchema, researchFindingSchema]),
  comparison: researchFindingSchema,
  limitations: researchFindingSchema
}).strict();
export const researchCandidateSchema = z.object({
  runId,
  turnId: integer,
  manifestDigest: digest,
  draft: researchComparisonDraftSchema
}).strict();

export const researchContractSchema = z.object({
  manifest: researchContextManifestSchema,
  candidate: researchCandidateSchema
}).strict().superRefine(({ manifest, candidate }, ctx) => {
  const draft = candidate.draft;
  const findings = [...draft.methods, ...draft.conclusions, draft.comparison, draft.limitations];
  for (const finding of findings) {
    for (const citation of finding.citations) {
      const source = manifest.sources.find((item) => item.sourceId === citation.sourceId);
      if (!source || citation.startLine < source.selection.startLine || citation.endLine > source.selection.endLine ||
          !lines(source.content).slice(citation.startLine - source.selection.startLine,
            citation.endLine - source.selection.startLine + 1).join('\n').includes(citation.quote)) {
        ctx.addIssue({ code: 'custom', path: ['candidate', 'draft'], message: 'Citation does not match loaded source lines' });
      }
    }
  }
  for (const group of [draft.methods, draft.conclusions]) {
    for (const [index, finding] of group.entries()) {
      if (finding.citations.some((citation) => citation.sourceId !== manifest.sources[index]?.sourceId)) {
        ctx.addIssue({ code: 'custom', path: ['candidate', 'draft'], message: 'Indexed finding cites a different source' });
      }
    }
  }
  if (manifest.sources.some((source) => !draft.comparison.citations.some((citation) => citation.sourceId === source.sourceId))) {
    ctx.addIssue({ code: 'custom', path: ['candidate', 'draft', 'comparison'], message: 'Comparison must cite both sources' });
  }
});

const bindingSchema = z.object({ runId, turnId: integer, manifestDigest: digest }).strict();
export type ResearchContextManifest = z.infer<typeof researchContextManifestSchema>;
export type ResearchSourceSelection = z.infer<typeof researchSourceSelectionSchema>;
export type ResearchCandidate = z.infer<typeof researchCandidateSchema>;

// Expected bindings must come from native state; this consumer never computes authority hashes.
export function parseResearchCandidate(value: unknown, manifest: ResearchContextManifest,
  expected: Pick<ResearchCandidate, 'runId' | 'turnId' | 'manifestDigest'>): ResearchCandidate {
  const binding = bindingSchema.parse(expected);
  const boundCandidate = researchCandidateSchema.refine((candidate) => candidate.runId === binding.runId &&
    candidate.turnId === binding.turnId && candidate.manifestDigest === binding.manifestDigest,
  { message: 'Candidate does not belong to the expected native context and turn' }).parse(value);
  return researchContractSchema.parse({ manifest, candidate: boundCandidate }).candidate;
}

export const researchRequestSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('start'), projectId: z.string().regex(/^prj_[0-9a-f]{32}$/), expectedProjectRevision: integer,
    contextAccess: z.literal('selected_excerpts'),
    selections: z.tuple([researchSourceSelectionSchema, researchSourceSelectionSchema]) }).strict(),
  z.object({ type: z.literal('read'), runId }).strict(),
  z.object({ type: z.literal('dismiss'), runId, turnId: integer }).strict()
]);
export const researchSnapshotSchema = z.object({
  runId, manifestDigest: digest, manifest: researchContextManifestSchema,
  candidate: researchCandidateSchema.nullable(), error: text(512).nullable()
}).strict().superRefine((snapshot, ctx) => {
  if (!snapshot.candidate) return;
  if (snapshot.candidate.runId !== snapshot.runId || snapshot.candidate.manifestDigest !== snapshot.manifestDigest ||
      !researchContractSchema.safeParse({ manifest: snapshot.manifest, candidate: snapshot.candidate }).success) {
    ctx.addIssue({ code: 'custom', path: ['candidate'], message: 'Candidate does not belong to this research snapshot' });
  }
});
export type ResearchRequest = z.infer<typeof researchRequestSchema>;
export type ResearchSnapshot = z.infer<typeof researchSnapshotSchema>;

export class QiongliResearchClient {
  constructor(private readonly transport: AppTransport = tauriTransport) {}
  async previewCapture(candidate: ResearchCandidate): Promise<AppEvent> {
    return appEventSchema.parse(await this.transport.invoke('qiongli_research_capture', {
      candidate: researchCandidateSchema.parse(candidate)
    }));
  }
  async execute(request: ResearchRequest): Promise<ResearchSnapshot | null> {
    const parsed = researchRequestSchema.parse(request);
    const snapshot = researchSnapshotSchema.nullable().parse(await this.transport.invoke('qiongli_all_chat_research', { request: parsed }));
    if (snapshot && (parsed.type === 'start'
      ? snapshot.manifest.projectId !== parsed.projectId || snapshot.manifest.projectRevision !== parsed.expectedProjectRevision ||
        snapshot.manifest.sources.some(({ selection }, index) => {
          const requested = parsed.selections[index]!;
          return selection.artifactPath !== requested.artifactPath || selection.startLine !== requested.startLine || selection.endLine !== requested.endLine;
        })
      : snapshot.runId !== parsed.runId)) throw new Error('Research response does not match the requested scope');
    return snapshot;
  }
}
