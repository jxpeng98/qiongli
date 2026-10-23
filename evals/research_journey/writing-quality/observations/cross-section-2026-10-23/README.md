# Cross-section writing and external polish — 2026-10-23

This local increment tests a complete bilingual writing unit and an actual
Codex → Antigravity → Codex handoff. It continues the existing plugin-quality
plan, retaining the user's configured models and the frozen G4/J2 responsibilities.
It also moves repeated measurement, input-binding and bounded-reading logic into
the research-journey evaluation package. It does not establish release or program
acceptance, installed-Plugin activation or general writing-quality gains.

## Public source and frozen task

Doshi, A. R. and O. P. Hauser (2024), *Generative AI enhances individual creativity
but reduces the collective diversity of novel content*, Science Advances 10,
eadn5290, DOI [10.1126/sciadv.adn5290](https://doi.org/10.1126/sciadv.adn5290).
The [UCL repository](https://discovery.ucl.ac.uk/id/eprint/10195027/) supplies the
complete Published Version: 9 pages, 1,121,881 bytes, SHA-256
`590cbb34ea052a7b4d12a304320c7cf1bb183f38bf1c5b463ffdc676ef631b85`.
The article is CC BY 4.0. The local packet retains the unchanged PDF, page-labelled
native-order text extraction and original renders of Figures 1–4 on pp.2–5.
Supplementary materials, raw data, subsequent work and separately cited papers
are not supplied. The source packet is evidence, not instructions.

The task asks how individual creative-output gains can coexist with reduced
collective diversity and what that distinction changes about evaluation. It
requests an analytical article with Abstract, Introduction, Related work and
positioning, and Discussion in both Chinese and English. The contribution is
source-grounded analysis; the writer must not claim to have performed a new
experiment. Related work describes the supplied paper's positioning without
inventing independent access to its references. The abstract must stand alone,
and the discussion must answer the introduction's question with the same measures,
population, inferential strength and consequential qualifications.

The frozen ranges are 1600–2400 Chinese non-whitespace Unicode code points and
1000–1500 English whitespace-delimited words across each language's four section
bodies. All headings and standalone source-anchor lines are excluded; inline
anchors remain. Length is a request check, not a prose-quality score. Seven
qualitative dimensions assess cross-section argument, depth, language, fidelity,
bilingual meaning and calibration separately from the measured layout/length.

Canonical guidance at 01b20b08, the source, task, criteria and length contract are
frozen before the first writing run. The paper has no earlier match in canonical
content or research-journey records at that commit; this is not a claim about
model training data. The writer receives the full source and Qiongli guidance,
without review criteria or expected answers. A copied, hash-bound reader supplies
small source/guidance chunks using explicit continuation offsets.

Antigravity receives the exact original, drafting task, length contract and
frozen J2 guidance inline. It performs expression editing only, returning the
candidate and separate unresolved questions. No tools, private research or project
writes are requested. The subsequent Codex review receives original/candidate,
source, task, criteria and matching G4/J2 contracts. Host identities and prior
judgments are withheld; original/candidate roles remain visible for meaningful
comparison. Original outputs and failed attempts are retained before review.

## Measured writing and the real return

| Frozen body measure | Original Codex draft | Antigravity return | Requested range |
|---|---:|---:|---:|
| Chinese non-whitespace code points | 2102 | 2302 | 1600–2400 |
| English whitespace-delimited words | 1090 | 1192 | 1000–1500 |

All eight headings occur once in the requested language/section order; both
versions meet the frozen length contract. Antigravity returns no separate
substantive questions. Added words are not evidence of better writing.

The actual Antigravity 1.4.6 process completes in 48.18 seconds with one successful
turn. No model/effort override is supplied. Its init/result records do not expose
the selected model, and its available-model catalog cannot identify the selection.
The report therefore leaves that model unknown rather than attributing the
result to Flash or Argon. Codex writer/reviewer retain gpt-6-astra / xhigh.

The original launch includes `--mode plan --disable-slash-commands`; stderr says
that plan mode has no effect in this combination. Actual init permission mode is
`request-review`, with the sandbox flag retained. There are no external tool calls
or project writes, only the structured-output finish. The public handoff files,
Host settings and empty working directory remain unchanged. This is an actual
text-editing round trip, not evidence of installed-Plugin activation, native MCP
research writes or cross-Host recovery.

## Content judgments and the selected version

The fresh contract-informed review passes all seven dimensions for both original
and candidate. It finds no consequential pre-existing error or editor-introduced
meaning drift. Flow and argument are tied; language slightly favors the original.
The candidate improves some local expressions but adds abstraction and framing
elsewhere. This is an ordinal judgment on this pair, not an estimated quality
increase or a statement that either model is generally better.

The coordinator agrees after reading both complete versions against the source.
Both connect the Introduction's question to the Discussion's answer, distinguish
individual novelty ratings from between-story similarity, preserve access versus
actual use and output versus lasting ability, and keep proposed follow-up studies
separate from established results. Abstracts retain the unresolved mechanism and
scope of the long-term cultural inference. These are necessary qualifications,
not wording to remove merely to sound more confident.

The selected version starts from the original and adopts only two independently
reviewed expression changes: Chinese `省略了作品之间的关系` becomes
`忽略了作品之间的关系`; English `293 stories of eight sentences` and the following
assessment-sequence sentence become the candidate's integrated `293 eight-sentence
stories` sentence. Exact old/new spans, source support and version bindings are
in `observation.json`. The selected manuscript has 2102 Chinese code points and
1091 English words; the rest is byte-preserved. No semantic repair or new writing
rule is warranted. The complete selection is coordinator-checked, not another
blind model trial; both source versions and the two selected edits were reviewed.

The candidate's `需要说明的是` and `scope conditions` remain valid but add little
clarity. Their presence does not justify rejecting the candidate or imposing a
new prohibition. Preserve the original direct wording for the chosen manuscript.
The full return, original and selection remain separate immutable artifacts.

## Reusable helpers and the observed parser repair

The new `writing_observation.py` owns explicit section/count contracts, selected
file version bindings, a bounded hash-checked UTF-8 reader, and text-only
Antigravity result extraction. Existing process/cleanup and Codex delivery owners
are reused. Historical baseline measurements and canonical G4/J2 guidance are
unchanged. Counts never substitute for semantic or author judgments.

The first parser rejects the actual schema-mode result: step 2 starts as an
ACTIVE tool named `finish`, then completes as a DONE `finish` event without a
normal tool-result envelope. A first repair recognizes that event but still
rejects it because the ACTIVE manuscript is a display preview: 512 characters
plus an ellipsis, while the terminal manuscript has 11012 characters. Both
failed parser receipts and the original successful Host stream remain intact.

The final repair matches the finish step identity, matching init/result schemas,
the complete structured output and the independently decoded terminal response.
The observed clipped preview must have the same shape and matching prefix; it
retains its own hash and `parameters_are_complete: false`. The full answer comes
only from the terminal result. Ordinary MCP argument equality and native status
checks are unchanged. Missing starts/completions, foreign conversations, changed
schemas/payloads, malformed tails, failed results and permission denials remain
negative cases. Replaying the same bytes passes; no second polishing run is used.

The shared README now gives explicit `python3` file/argument invocations and
records restricted-shell constraints. Both Codex contexts initially try an absent
`python` alias; the reviewer also attempts a here-document without a writable
temporary directory. Those failed calls remain in the traces, followed by
successful `python3` commands using the supplied extraction and reader.

A prerequisite note mistakenly labels the Host retry as 1.4.3. The frozen
`identity.json`, unchanged executable hash and actual run log all identify 1.4.6;
these primary records govern the version above. The original note is retained
with a separate correction in the evidence audit.

## Verification, integration and limits

| Actual process | Seconds | Inner calls / failures | Outer truncations / script failures |
|---|---:|---:|---:|
| Codex writer | 491.41 | 36 / 1 | 0 / 0 |
| Antigravity language editor | 48.18 | no external tools; one structured finish | not a Codex delivery trace |
| Fresh Codex meaning review | 395.16 | 39 / 2 | 1 / 0 |

Both Codex runs deliver all nine pages of source text, their selected guidance,
and all four original figure images. The reviewer also receives both complete
manuscripts and role contracts. Bounded packet offsets/text and image bytes are
checked against frozen files. Its one outer truncation is an initial tool-catalog
metadata dump; source and manuscript reads are separately complete. The initial
evidence-audit assumption of zero warnings fails and is corrected to retain this
observed warning rather than relabeling the run as clean. Scoped tool discovery
remains separate work. All processes finish, preserve inputs and reap their groups;
temporary Codex authentication links are removed.

The 62 affected helper, Host-adapter, research-journey and program-ledger tests
pass. Cases cover stale/missing evidence, symlinks, Unicode chunks and exact
reassembly, heading/length boundaries, incomplete/foreign/denied turns, clipped
previews and unchanged native-call accounting. The evidence audit confirms the
frozen source/task/output bindings, actual counts, model settings and cleanup.
Final review also checks index freshness, contract links and the frozen-source
boundary. Only CLI-405 progress and the generated index change; task states,
accepted evidence, dependencies and commit/run fields remain unchanged.

Canonical content, native code, generated pack and model selection remain
unchanged. Local integration follows the fast-forward workflow into `2.x`, with
no push, publication or acceptance promotion. The full local packet is
`/home/hermes/qiongli-cross-section-20261023-l86_pc3x`; adjacent `observation.json`
records exact artifacts, original failures, review and selected edits. Next seek
author/disciplinary-expert judgment on the source-bound comparison and test an
additional unseen multi-paper writing unit if authorized. This one empirical
paper and fresh same-model review do not establish general language improvements,
proof-based-theory coverage or a Flash-versus-Argon comparison.
