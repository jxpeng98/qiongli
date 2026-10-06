# Bounded research evidence journey

## Writing depth and expression checks

`writing-quality/tasks.md` and `sources.md` provide three synthetic requests for
an RQ-linked literature review, faithful English expression editing and cited
background with explanatory notes. Keep `review-criteria.md` out of the drafting
prompt; preserve full answers before review and retain failures. Freeze source,
task and actual guidance bytes before a trial. Use the existing source/span review
principles below, without representing a materialized-guidance/subagent exercise
as an installed-Plugin trace or awarding V1 passes without its required evidence.
These tasks do not replace or rescore the fixed installed-Plugin baseline.

## Installed-Plugin baseline

`plugin-baseline.json` freezes three journeys: public-paper explanation, a paragraph
from supplied evidence, and saved-document recovery in a fresh session. The
offline `plugin_baseline.py` adapter consumes independently captured installed-Host
observations. It does not run `observe.py capture` or pass the test resource reader
off as an installed Plugin. Installation, actual model execution and reviewed
project saves stay with their existing native/official Host owners.

Before preparing a fresh Linux Codex batch, check sandbox prerequisites without
credentials or a model call:

```sh
python3 evals/research_journey/codex_preflight.py /absolute/new-preflight \
  --guidance-file /absolute/installed/skills/qiongli-workflow/SKILL.md \
  --guidance-file /absolute/installed/skills/qiongli-workflow/workflows/paper-read.md
```

This creates a separate empty Host home and an explicit read-only permission
profile with network disabled. The resolved Codex executable is also readable
so the Host can re-execute it; its parent directory is not granted. It runs only
version, sandbox `true`, and exact
public-file reads, stopping on the first failure and retaining raw output and
process cleanup. It never links credentials, invokes a model or changes the
installed Plugin. The command is tested against Codex 0.160.0's Linux interface;
other versions/platforms need their own supported prerequisite evidence.
`preflight.json` passing proves only those sandbox/file operations. The shared
spool owner's `events.jsonl` here contains command stdout, not model JSON events.
Before any live batch, separately inspect effective Plugin/server/tool policy
through the supported Host configuration interface and confirm exactly the
authorized read tools. Missing config readback or failed sandbox access blocks
model invocation; do not substitute broader sandbox modes. Keep preflight and
effective-policy evidence with the new frozen inputs. A preflight file read
cannot fill a model observation's missing guidance-use evidence.

Prepare a NEW external capture directory. Copy `plugin-baseline.json` to
`catalog.json` before model calls; do not supply its review checks to the model.
Keep the selection of all three case IDs, even after a failure. Capture layout:

- `installation.json`: `source_commit` (native source), `cli_sha256`,
  `content_pack_sha256`, `plugin_receipt_sha256`, `codex_version`, `model`,
  `reasoning_effort`; extra source/cache/inventory evidence may be retained.
  The embedded pack's content-source commit is distinct from the native build.
  New catalogs also require `guidance_files`, mapping each required package-relative
  guidance path to the SHA-256 of its installed UTF-8 bytes. Snapshot those bytes
  in the capture before freezing. Optional `mcp_server` binds the actual installed
  server name; historical captures default to `qiongli`. A server name alone does
  not adapt another Host's event format to this Codex JSONL reader.
- `sources.csv`: the existing `observe.SOURCE_FIELDS`; each `artifact_path` names
  a captured UTF-8 source file (for example `inputs/source.md`) containing its
  `source_location` anchor. These are reviewed excerpts, not asserted full papers.
- Each case directory: `prompt.txt`, original `events.jsonl`, `answer.md`,
  `capture.json` with `exit_code`, `elapsed_seconds`, `timeout_seconds`,
  `prompt_sha256`, `events_sha256`, `answer_sha256`, and `preservation.json` with
  nonempty `before`/`after` maps of project/config relative paths to SHA-256.
  Only read-only model turns belong in these snapshots; reviewed fixture saves
  occur before the turn through preview/approval/CAS. Preserve timeout output;
  leave unavailable answer files absent rather than inventing a result.
  Optional `termination_reason` is `timeout`, `permission-denied` or `cancelled`,
  recorded by the driver, not inferred from answer prose. Otherwise a negative
  process code is `terminated`, a positive code is `nonzero-exit`, and exit zero
  still needs a complete successful turn. A tool result before forced termination
  remains observed; it cannot establish a completed case. Preserve raw stderr and
  cleanup observations as additional manifest-bound files, including on failure.
- Each case's `guidance.json` maps every catalog `guidance_paths` entry to
  `artifact_path` (the snapshotted full body), `mechanism`, and one evidence key.
  For `command_output`, use `call_id` of a matched successful command whose
  `aggregated_output` contains the full body. For `host_injection`, use
  `context_artifact` pointing to a separately retained actual Host context export
  containing the full body. Do not fabricate an export if the Host provides none.
  A source snapshot or final answer is not an injection record. A reviewer must
  still verify origin and actual use; local hashes do not authenticate a Host.
- `manifest.json`: `kind: qiongli-plugin-baseline/v1`, the ordered three `cases`
  from the catalog, and `files` mapping every retained input/observation path to
  its SHA-256. Seal after capture; no absolute paths, escaping paths or symlinks.
  Raw traces remain local and must not be published as repository fixtures.

```sh
# Prepare catalog, installation, source registry/packets, guidance snapshots and
# all three prompts; finish fixture saves before this step and before any call.
python3 evals/research_journey/plugin_baseline.py freeze /absolute/new-capture
# Run only the separately authorized observations, recording failures and cleanup.
python3 evals/research_journey/plugin_baseline.py seal /absolute/new-capture
python3 evals/research_journey/plugin_baseline.py prepare /absolute/new-capture \
  --review /absolute/new-review.json
# Review every non-whitespace answer span against sources and actual calls.
python3 evals/research_journey/plugin_baseline.py score /absolute/new-capture \
  --review /absolute/completed-review.json --report /absolute/new-report
```

`freeze` exclusively creates `frozen-inputs.json`; `seal` exclusively creates the
final manifest after checking all frozen bytes. Neither overwrites an earlier
run. Input drift refuses sealing: retain that failed directory and start a new
observation, rather than rebinding changed inputs. The freeze is local provenance,
not a lock on the live project or an authenticated timestamp. Continue comparing
the actual before/after project and configuration snapshots. Old frozen v1
catalogs/manifests remain readable with their original declared requirements;
rescoring them does not add these new criteria or establish a new observation.

Review uses the same `observe.project` span/link contract described below. Assign
the catalog's claim IDs during review; a paragraph need not expose evaluator IDs.
All substantive answer spans need review, including extra claims; headings/status
text can be `context`, never a way to hide a faulty research claim. `guidance_use`
requires inspection of actual Host activity; an answer naming a Skill or a shell
command naming a file alone does not prove the expected bytes were loaded.
Preserve failed spans and give the reviewer their actual attribution.

Structural receipts use the existing V1 runner for capture/review binding,
requested-claim coverage, required successful native recovery calls and unchanged
project/config snapshots. New catalogs additionally require installed guidance
evidence and the source paragraph's frozen length rule: 250–350 non-whitespace
Unicode code points, including punctuation/digits. Exclude only bracketed
`sources.csv` anchors (ASCII square/round or Chinese round brackets; multiple
anchors may use commas/semicolons). Count the complete delivered answer, without
reviewer-selected cropping; extra headings/notes remain task-scope defects for
whole-answer review. An unknown citation is not silently removed. The new prompt
must declare this counting convention before the observation.

Semantic judgments remain separate. Each successful trace requires
matched call starts/completions and a final answer after calls. Failed tool calls
remain visible, even if the task recovers. Empty MCP envelopes, empty content or
conflicting error flags cannot satisfy required native reads. Every completed call
retains its start/completion event indexes (zero-based) and argument/result/error
digests. Digests use sorted-key compact UTF-8 JSON; the raw event file remains the
full-payload authority, not the model's result summary. Payload-shape checking
does not establish tool-specific correctness or recover data omitted by a Host.
Failed process/turn captures retain completed/pending call observations, timing,
available usage and preservation results in the fixed denominator. A malformed
last JSONL line is recorded as a truncated tail, never a successful final turn.
Unavailable usage stays null; reported
input/cached/output counts are retained without guessing a cost or speedup.

Hash bindings are provenance, not authenticated installation, approval or
scientific truth. Independently check the installed binary/cache and snapshot
capture. A baseline on one public packet does not qualify held-out papers,
complete B2, upgrades or other Hosts; those remain subsequent bounded increments.

## Bounded Antigravity status observation

`antigravity_observation.py` extends this capture owner for one installed AGY
`qiongli_config_status` call. It consumes AGY's own `init` / `step_update` /
`result` stream; it does not manufacture Codex events or academic answer scores.
The format follows the [official headless interface](https://antigravity.google/docs/cli/headless/)
and the retained local 1.2.17 event shapes. Supported format is separate from
evidence that a particular Host actually exposes a complete result.

Prepare an isolated public workspace/configuration and verify the existing
installed Plugin through its native owner first. Supply `installation.json` with
`agy_version`, native `source_commit`, `cli_sha256`, `content_pack_sha256`,
`plugin_receipt_sha256` and the exact `mcp_server` name. These are installation
observations, not values to invent from expected tool names. The captured AGY
executable hash independently binds the Host binary. Inspect the registered MCP
command/environment: the runner selects `QIONGLI_CONFIG_HOME` and
`QIONGLI_PROJECT_ROOT` for the child, but an explicit Plugin environment can
override inheritance. Do not claim isolation until registration agrees.

After the specific live observation is authorized:

```sh
python3 evals/research_journey/antigravity_observation.py capture /absolute/new-capture \
  --workspace /absolute/public-workspace --config-root /absolute/isolated-qiongli-config \
  --identity-file /absolute/installation.json --agy /absolute/agy \
  --host-settings /absolute/actual-host-settings.json --timeout-seconds 180
python3 evals/research_journey/antigravity_observation.py score /absolute/new-capture \
  --report /absolute/new-report
```

The fixed prompt requests only the exact status call and stops on denial. The
runner uses print mode, sandbox, plan mode and low effort, with no model override,
permission override, conversation resume, automatic retry or trust/configuration
write. The existing AGY process owns authentication; this adapter does not copy,
link or read login files. Select only authorized public fixture roots. Optional
`--host-settings` records a digest, never its contents. Missing or linked snapshot
roots refuse; concurrent changes remain a failed preservation check and are never
reverted over the user's new bytes.

Each new capture exclusively retains prompt, installation identity, command,
selected Qiongli environment, initial snapshots, raw stdout/stderr, final snapshots
and process receipt. Output files sit outside the snapshot roots. The model/run
budget is at most 180 seconds; termination has a separate short grace period.
Exceptions and timeouts send SIGTERM before waiting, then SIGKILL only if needed,
and reap the child. Remaining members of the owned POSIX process group are
reported; descendants that start another session are outside that group guarantee.
Forced termination or unresolved group cleanup cannot be a normal-exit pass.
The full on-disk stream is hashed after cleanup, with no buffer reset across a
session restart: this driver runs one process and a new directory per attempt.
It replaces the earlier temporary PTY capture path for structured observations;
the old scripts and historical hashes remain unchanged evidence.

`once_permission` is an offline/current-screen checker for the retained interactive
menu. It checks exact tool and case-sensitive JSON arguments, tolerates terminal
wrapping, and requires the single-call choice rather than conversation/persistent
choices. It never sends keys or authorizes anything. Use a freshly rendered
expanded screen, not a concatenated old terminal history. Print mode provides no
interactive approval channel: do not send unsupported control messages or widen
permissions to make a test pass. A soft-denied call can coexist with Host
`status: SUCCESS` and process exit zero; the scorer still rejects it.

The status observation reuses Evaluation Truth V1. It requires matched invocation
steps and exactly one expected tool, a complete Host stream/response, no denial,
normal process/group completion and explicit unchanged snapshot maps. Complete
native result evidence requires the full MCP envelope: `structuredContent` passes
the existing status schema and equals the parsed `content` text, with no error.
A bare status object, collapsed object, short tool summary or model JSON response
is insufficient. Duplicate JSON keys, source/command/env drift and missing
snapshots fail closed. Raw output remains in the bound event file; summaries use
event indexes and value hashes. Capturing files is not a passing observation;
only the separate V1 score states that result, and no academic/Skill acceptance
follows from it. Synthetic fixtures prove parser/cleanup behavior, not that the
installed AGY currently exports the required full envelope.

## Existing evidence packets

The additive public-paper regression packet preserves actual Q1/Q2/Q3 answers
and the external model review. Run
`.venv/bin/python evals/research_journey/public-papers/check_integrity.py --observations --self-test`
to replay five frozen, coordinator-reviewed spans through `observe.project`:
units, denominators, abstract/body disagreement, expanded source access and
review attribution. The original unit conversion and human-attribution failures
remain failures. Byte, span, anchor, attribution and failed-to-pass mutations are
rejected. This checks recorded judgments and bindings; it does not grade new
answers, certify whole answers or establish installed-Host acceptance.

The same command also checks `public-papers/discipline-transition/`: two bounded
education/computing exercises with preserved tasks, source notes, predecessor
bindings and actual main-agent answers. It replays two selected manuscript spans,
annotated by the same agent after source inspection and guidance refinement.
These are self-review observations, not blind forward tests or a measured gain.
The computing packet is a short Host-read paraphrase with unknown raw byte hashes;
its local digest must not be presented as the paper's digest. Four added negative
mutations reject changed notes/answers, forged anchors and independent-review
misattribution. The original five judgments remain three passes and two failures.

These two Evaluation Truth V1 cases share one explicitly synthetic abstract and
source registry. They check declared evidence links and requested claim coverage,
not academic truth or real Host execution. No private research or model call is
needed. Use the existing single-case runner with the shared fixture:

```sh
.venv/bin/python evals/runner/run_eval.py evals/research_journey/cases/reading-to-manuscript.yaml \
  evals/research_journey/fixtures/reading-to-manuscript --json-receipt /tmp/reading-journey.json
.venv/bin/python evals/runner/run_eval.py evals/research_journey/cases/source-to-paragraph.yaml \
  evals/research_journey/fixtures/reading-to-manuscript --json-receipt /tmp/paragraph-journey.json
.venv/bin/python -m unittest tests.test_research_journey_evals -v
```

The reading task requires reading observations. The paragraph task consumes the
source directly and does not inspect or require a new reading artifact. Scope is
declared before evaluation; deleting a required step from a failed result does
not turn it into a completed narrower task. Reordering rows, reusing evidence,
adding annotations and safely paraphrasing prose are allowed. Both requests need
the association C1 and causal limitation C2. Unused C3 can remain `needs_evidence`.

The fixed inputs are `source.md`, `sources.csv`, `references.bib` and
`required_claims.csv`; exact-byte digests bind them. Changing a source invalidates
this checkpoint, not the scientific claim itself. Review the affected claim and
bind fresh evidence before claiming verification again; never silently update a
digest to obtain a pass. Outputs are not compared to an exact prose answer.

`reading.csv` contains tabular observations of a note, summary and matrix;
`ledger.csv` retains the canonical evidence-ledger columns plus evidence scope;
`manuscript.csv` puts a passage alongside its declared claim/source mapping.
These CSVs are **test observations**, not replacements for canonical Markdown
artifacts or evidence that arbitrary Markdown has been parsed. The optional
adapter below binds model observations to actual answer and source bytes.

The case enforces source identity, locator and artifact binding, abstract-only
scope, supported status for used claims, nonempty narrative fields, and inclusion
of requested claim IDs. Cross-artifact checks compare whole tuples so swapping
sources between claims cannot hide behind matching individual column values.
In a multi-column `subset`, unused incomplete ledger rows may remain; they cannot
support a complete active claim. Paths are data in these tuples; only the pinned
source registry can establish their binding to a checked source file.

Semantic review remains necessary: a syntactically valid locator need not support
the claim, a nonempty passage may misstate the study, and a model may omit an
unmarked claim from its observations. A passing receipt proves only the declared
structural checks. It does not establish full B2 completion, causal validity,
submission readiness, installed Skill activation or live approval/CAS behavior.
The status-only full-cycle harness remains historical coverage, not this gate.

## Answer-bound observations

`observe.py` reuses the routing probe's isolated Codex capture and snapshot-only
`read_resource` MCP, then projects reviewed answer spans into the existing V1
cases. It has no new dependency, production parser or research-project writes.
Both cases are fixed before capture. Only the four input files above, the
request and entry are supplied; passing fixture CSVs and assertions are withheld.
The current configured model and reasoning effort are retained. As with the
routing probe, a custom provider or profile is unsupported, not silently changed.

Choose fresh local paths; capture invokes the model twice (up to 180 seconds per
turn). Prepare and score are offline and never call a model:

```sh
.venv/bin/python evals/research_journey/observe.py capture /tmp/journey-capture
.venv/bin/python evals/research_journey/observe.py prepare /tmp/journey-capture \
  --review /tmp/journey-review.json
# Review the actual answer/source, then edit journey-review.json as described below.
.venv/bin/python evals/research_journey/observe.py score /tmp/journey-capture \
  --review /tmp/journey-review.json --report /tmp/journey-report
.venv/bin/python -m unittest tests.test_research_journey_observations tests.test_skill_routing_probe -v
```

The capture retains exact final answers, raw events, partial timeout output,
source/guidance/case/producer snapshots, prompt/event hashes, elapsed time and
configured model/Host version. A failed trace stops new captures; failed and
unattempted cases still count in the fixed denominator of two. These are isolated
supplied-entry observations, not installed-Plugin activation or project access.
Guidance reads are counted only when matching successful MCP starts/results were
actually observed. Source access is explicitly `supplied-in-prompt`.

`prepare` starts with **unreviewed**, unmapped whole-answer spans. Set `reviewer`
to `{ "kind": "human" | "model", "id": "named reviewer" }`; a model's self-review
must not be labeled independent or human. Keep the captured answer/event hashes.
For each case, replace `segments` with ordered spans covering every non-whitespace
character. `start`/`end` are zero-based Python Unicode character offsets, end
exclusive; `quote` must exactly equal that answer substring. Each segment has:

- `role`: `reading`, `manuscript`, `summary`, `context` (headings, formatting or task-status
  text that makes no research claim), or
  `unmapped` (unresolved content). Do not hide substantive claims as context.
- `links`: zero or more objects containing exactly `claim_id`, `source_location`,
  `status` and `claim_type`. Known source locations come from `sources.csv`;
  source identity, path and abstract-only scope are derived from that registry,
  never supplied or widened by the reviewer. Duplicate links are rejected.
  Status is `supported`, `needs_evidence` or `unsupported`; claim type is
  `finding`, `limitation` or `speculation`. Only supported findings/limitations
  can pass the original manuscript case. Unused C3 need not produce a row.
- `verdict`: `pass`, `fail`, `unreviewed`, or `not-evidence` (context only), plus
  a `reason`. A completed judgment needs a nonempty reason; passing content
  needs a source link. Preserve defective prose and mark it, never repair it in
  `quote` or substitute a fixture. Mapped reading quotes populate the existing
  note/summary observation cells; this does not assert full B2 completion.
  Multiple spans sharing a source (reading) or full claim/source tuple
  (manuscript/ledger) join with newlines in one record. Annotation granularity
  must not multiply source records; every constituent quote remains separately
  bound in the review. Different claim/source tuples are not merged.

Complete all five `checks`, each with `status` (`pass`, `fail`, `unreviewed`),
nonempty `reason` and affected zero-based segment indices in `segments`:

| Check | Review against the supplied synthetic abstract |
|---|---|
| `association` | C1 remains a positive association; any sample/design/r values agree |
| `causality` | C2 is retained; association is not promoted to a causal effect |
| `statistics` | No invented CI, p-value, adjusted analysis or significance claim |
| `access` | No claimed full-text, external or project access beyond supplied abstract |
| `added_claims` | No unsupported additional finding or use of pending C3 as evidence |

Failure checks must reference at least one failed span. Full span coverage is
mechanical; whether a span is correctly classified and supported is still a
reviewer assertion, not automated entailment. Unassigned/incomplete review cannot
pass. Honest labeling and reviewer identity are not authenticated by local hashes.

Scoring revalidates frozen inputs, answer/event hashes, actual resource results,
span coverage and review binding, creates CSVs only from exact answer quotes,
and invokes the canonical V1 runner once per case. The original 16/13 assertions
plus one binding digest give 17/14 assertions. `summary.json` reports
`structural_passed` and `reviewed_passed` separately: a causal overclaim with valid
tuples can be structural-pass/semantic-fail. Missing or invalid bindings cannot
pass. Regrading uses a new report with a new review digest and leaves capture and
prior reports untouched. A report is local diagnostic evidence, not acceptance.

Keep raw events/stderr and complete capture snapshots in local test output, not
the repository. Portable evidence should include only necessary synthetic final
answers/source excerpts, review reasons, hashes and redacted metadata. Do not
publish private research, user configuration, credentials or raw reasoning.
Two first observations diagnose this configuration; they do not measure an
accuracy rate, model superiority, longer C→F continuity or cross-Host readiness.

## C→F continuity checkpoint pair

Use `capture NEW_DIRECTORY --continuity` for **one fixed journey with two
ordered checkpoints**, not two independent research studies. The default
two-case lane and existing v1 captures remain unchanged. Prepare and score infer
the frozen lane; do not pass `--continuity` to them or change a capture's selection.
The continuity pair has an explicit **360-second budget per checkpoint** for
long-form summaries and guidance reads; the original short-case/routing lanes
remain at 180 seconds. New capture receipts record `timeout_seconds`. Increasing
this budget does not upgrade an earlier timeout: keep the earlier attempt and
its denominator, and label any new capture as a different-budget observation,
not a same-budget quality improvement. No model setting is changed.

1. `c-stage-summary` receives R1 plus the canonical-shaped synthetic
   `context/research_state.md`, `decision_log.md`, `stage_handoff.md` and the
   latest `STG-B-001` summary. It returns a chat-only `STG-C-001` design-boundary
   summary, not a claim that collection, ethics or analysis has been completed.
2. `f-stage-continuation` receives that **actual captured C answer**, with its
   content hash, as `context/stage_summaries/STG-C-001.md`. A new isolated turn
   also receives R2 source/state/handoff: the synthetic abstract corrects
   n=120/r=.32 to n=118/r=.23 without changing C1/C2, DEC-001/DEC-002, the
   citekey, source anchors or abstract-only interpretation limits. The requested
   output is a bounded F paragraph and `STG-F-001` correction/handoff summary.

R2 is snapshotted before capture but withheld from the C request. The F prompt
hash binds its exact predecessor text; validating F also validates C's original
events, final answer and guidance results. A missing/replaced C capture cannot
qualify F, even if replacement C hashes are locally recomputed. This is explicit
context transfer, not a resumed installed session or an authenticated transcript.
No C review is silently promoted to approval of the changed R2 source.

The new `qiongli-research-continuity/v1` manifest fixes both checkpoints and all
context/revision snapshots. Both remain in the denominator after failures or a
timeout. Each checkpoint has 19 V1 assertions: the original direct-source checks,
four context-file digests, one literal identity/section check and the answer
binding. The shared `manuscript.csv` slot is only a claim-passage projection:
`summary` spans from C populate it without asking C to draft a manuscript or
inventing a saved project artifact. F uses `manuscript` or `summary` as appropriate.

Review every substantive span against its **current** source and context, using
the original five checks plus these four required checks:

| Check | Required review |
|---|---|
| `stable_ids` | Keep claim/decision IDs, citekey and locators attached to the same meanings; do not renumber to hide changes |
| `source_revision` | C uses only R1; F explicitly reconciles old and corrected R2 values, never treating the prior summary as current evidence |
| `stage_limits` | No invented collection, ethics approval, analysis execution, full lifecycle completion, project writes or cleanup |
| `summary_history` | Preserve predecessor/history, unresolved C3, decision rationale and revisit triggers; summaries do not replace originals |

Decision/status/history text can be `context` only when it makes no research
claim; its correctness still belongs to these explicit semantic checks. Mark an
incorrect unmapped statement `unmapped`/`fail` with an affected check and reason,
not `not-evidence` merely to hide it. Passing identity-token checks alone cannot
prove decisions' meaning, complete source coverage, valid relative links or
corrected scientific values. A stale numeric claim with valid IDs can still
structurally pass and must remain a semantic failure. Whole-journey success
requires both checkpoints to pass structure and named review; it is not a
full-study or installed-Host acceptance gate.

## Public-paper trial packets

`public-papers/` adds three bounded, real-paper tasks: a primary-school quantitative
experiment, a university qualitative interview study, and an audit of a published
health-professions systematic review. These are **supplied-evidence appraisal and
writing tasks**, not fresh literature searches, complete systematic reviews or
reanalyses of underlying data. `manifest.json` binds the public XML identity,
license, raw SHA-256, exact normalized paragraph selectors/hashes, task files and
source packets. Full downloaded XML stays outside the repository; attributed
CC BY text excerpts are redistributed with flattening/omission disclosed. Figures,
tables, transcripts and supplements are not included. The quantitative article's
XML names CC BY but does not specify its version; no version is invented.

Freeze `manifest.json`, `criteria.md` and all task/source bytes before observations.
Supply only a case's `task.md` and `source.md`, plus selected frozen guidance, to the
runner; withhold criteria and the manifest (which exposes continuation anchors).
The quantitative continuation supplies `continuation-task.md`, both source files
and the **actual prior answer with its SHA-256**. It adds real previously withheld
paragraphs from the same article, not a fabricated correction. Keep Q1/Q2 in
order, preserve history and never replace a failed predecessor with fixture prose.
Other cases have one checkpoint. Record the chosen denominator before capture;
three paper journeys/four checkpoints is the full set. Do not widen a one-case
observation into suite, installed-Plugin, independent-review or cross-Host success.

The separate reviewer criteria require exact answer spans, source-bound reasons,
method/denominator limits, stable IDs and honest missing-evidence states. Review
cannot be replaced by keyword presence or a corpus digest. Reuse existing V1
assertions when projecting actual answer-bound observations; this packet adds no
scoring framework and is not a new `observe.py capture` lane.

Run the stdlib-only integrity check (the optional raw directory must contain
`quant.xml`, `qual.xml`, `review.xml` downloaded from manifest URLs):

```sh
python3 evals/research_journey/public-papers/check_integrity.py --self-test
python3 evals/research_journey/public-papers/check_integrity.py --raw-dir /path/to/raw-xml
```

The check verifies task/source/rubric hashes and excerpt identity/anchor binding;
when raw XML is supplied it also verifies the raw bytes, article DOI, license and
exact selected paragraph text. Its negative cases reject a changed count, swapped
paper and forged anchor even if the excerpt file hash is recomputed for the last
case. These are local integrity checks, not model quality or scientific truth
checks. Hashes bind a frozen snapshot; they do not authenticate its preparer.

## Manuscript writing and reverse lookup

`writing-quality/` retains the bounded prose/LR/note tasks. The separate
[`manuscript-trace/`](manuscript-trace/README.md) packet exercises results/discussion
writing and claim detection from an actual faulty body and its notes, including
assertions absent from the map. It uses invented sources, existing claim records
and actual native packet readback in an isolated fixture. Freeze original answers
before reading its separate criteria; preserve source-support, prose-quality and
tool-observation results separately. It adds no automated semantic scorer or
installed-Host qualification, and same-agent drafting/review remains self-review.
