# Qiongli 2.4.0 — Plugin reliability and research task quality

- Status: implementation authorized by the maintainer on October 5; the first
  bounded increment covers offline observation reliability and core guidance.
  Release qualification and publication remain separate decisions.
- Proposed version: `2.4.0`, reflecting the additive Antigravity adapter and
  improvements to existing research workflows. No version files change here.
- Local baseline: `186d18751a272c8fffbc0c871b061068093221e6` on `2.x`.
- Direction: [master roadmap](../roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md).
- Task state and acceptance: [existing ledger](../roadmaps/qiongli-program-ledger-v1.json).
- Prior observations: [October 5 execution record](2026-09-15-v2-0-1-transition-release-execution.md#october-5--installed-plugin-quality-baseline).

## Release outcome

A user can install the Plugin, explain supplied paper evidence, draft a bounded
source-supported paragraph, and explicitly save/recover reviewed work in a fresh
session. Each advertised result has evidence for the actual installed candidate.
Codex remains the primary research Host; Antigravity receives a separately
qualified local CLI adapter. Existing models and project-write authority remain
with their current owners.

The proposed release includes the already implemented AGY adapter (`ff62b06a`)
and baseline evaluator (`bcec1b0f`). These are integration inputs, not new work
to repeat. The principal new work closes observed quality and verification gaps.

## Starting evidence

| Observation | Consequence for this plan |
| --- | --- |
| Three installed-Codex cases: structural 2/3, complete reviewed passes 0/3 | Establish actual guidance use, fix the short paragraph, and keep fixture changes outside observation windows. This does not mean all three answers were academically wrong. |
| AGY install/update, Skill entry reads, 35-tool discovery and a native return observed | Preserve those scoped results; capture the full returned payload and finish the Host observation normally before claiming a clean session pass. |
| AGY validation required several driver corrections; the final Host exited -9 after returning a result | Exercise the driver offline, distinguish tool outcome from driver/process outcome, and retain failed attempts. |
| Literal Skill section coverage is 70/82 | Review the relevant flags in context. The count is not a semantic quality score or a requirement to add boilerplate to every file. |

Detailed hashes, candidates, approval scope and failures remain in the prior
execution record. This draft does not relabel that evidence or monitor the
previous release.

## Ordered delivery slices

### 1. Make observations repeatable — required

Extend the existing research-journey capture/review owner and Evaluation Truth V1.
Keep native installation, Host trust and project preview/approval/CAS with their
existing owners; do not introduce a second evaluator or a custom approval API.

- Freeze candidate, content, prompts, input files and fixture state before a run.
  Keep per-run manifests immutable; do not overwrite an old candidate's manifest.
- Prefer supported structured Host events/results. Capture complete MCP call
  arguments, returned payload and error state, with matching call identities.
  A model's account of a result or a terminal's collapsed object is insufficient.
- Record normal completion, denial, timeout and forced termination separately.
  Preserve configuration/project snapshots and cleanup results even on failure.
- Limit terminal interaction to the necessary normal trust/approval workflow.
  Test recorded menus, wrapping and completion/cleanup cases offline before a
  live run; never grant broader permissions to satisfy a test.

Exit evidence: the retained failure fixtures exercise the corrected driver, and
one bounded AGY status call has a complete result, normal or explicitly controlled
graceful shutdown, and verified cleanup. Unavailable capture capabilities remain
explicit gaps. Existing offline work can proceed while live evidence is pending.

### 2. Improve the three core Plugin tasks — required

Trace each observed defect to its owner before editing. Initial review scope is
`content/workflow/SKILL.md`, `content/workflow/workflows/paper-read.md`,
`content/workflow/workflows/academic-write.md`, their referenced B/F contracts,
and canonical extraction/retrieval Skills under `content/skills/B_literature/`.
Regenerate distribution payloads through the existing materializer after changes.

| Task | Required behavior |
| --- | --- |
| Explain supplied paper material | Ground central claims and numbers in visible sources; distinguish supplied excerpts from full-paper access, reported findings from interpretation, and allocation units from analysis units. |
| Write a bounded paragraph | Deliver the requested text, length and format; define the counting convention before review, preserve claim strength and citations, and avoid unsolicited project creation or saving. |
| Resume reviewed saved work | Recover current revision and receipt-backed document bindings from the project ID; read through available native tools, report changed/missing sources and preserve old history. |

Investigate missing guidance evidence before assuming that Skills were not
loaded: supported Host injection and explicit file reading can be different
mechanisms. Bind whichever mechanism was actually observed to the relevant
content version. An unsupported `resources/list` call does not justify a new
resource API merely to turn the test green.

Exit evidence: new captures of all three fixed baseline cases pass their required
structural and named answer-review checks, including guidance use. Required
unreviewed checks are not passes. Old failures stay in the original denominator;
these cases establish a bounded regression result, not broad academic acceptance.

### 3. Qualify persistence and limited generalization — required

Use one isolated public/synthetic project for an explicit sequence: reviewed
reading note → exact write preview and approval → verified persistence → new
session → project-ID recovery → source-supported paragraph. Reuse existing
storage and revision/CAS services. Test stale revision, source drift, missing
files and denied approval; retain existing text and history in every refusal.
This persistence journey is separate from direct read-only paragraph requests.

Freeze at least three small held-out cases before execution: limited/abstract-only
access, conflicting numerical or unit evidence, and changed-source continuation.
Require all declared checks, retain the fixed denominator and review complete
answers against supplied sources. Cases used to tune guidance become regression
cases; retain fresh held-out evidence before claiming generalization. This small
sample cannot establish expert or cross-discipline validity.

Exit evidence: the save/restart journey and held-out required checks pass under
their named scope. AGY receives a bounded read/recovery compatibility observation
after its status-call capture works; it does not imply simultaneous multi-Host
writes or repeat the complete academic matrix on every Host.

### 4. Deliver the version and preserve upgrades — required

Qualify a frozen candidate with the existing release owner. Include AGY's native
Plugin layout, source-retention requirement and trust/fresh-session instructions
in release notes and the Host support matrix. Distinguish source export,
registration, MCP discovery and an actual successful call in user-facing status.

Check the 2.3.0 → candidate upgrade and retained Codex/Claude/DeepSeek behavior;
preserve user models, workflow variants, research data and receipt-backed refusal
of unmanaged or changed installations. Diagnose unsupported Host/platform scope
explicitly instead of inferring it from a CLI build.

First extend `tooling/scripts/native_registry_upgrade_check.py` with explicit
predecessor selection: its current pip/npm checks select historical 1.17.0/2.1.1
versions and cannot yet establish a 2.3.0 upgrade. Preserve the old defaults and
negative cases, bind the selected predecessor and candidate artifacts, and
report an unavailable predecessor as an unqualified check rather than silently
substituting another version.

Use the existing four-target CLI distribution and archive/npm/wheel/Cargo gates
for the final source. Target-native CLI packaging and live AGY qualification are
separate claims. Remote qualification, push and publication use their existing
authorization boundaries; this plan launches none of them.

## Cost, scope and sequencing

One `gpt-6.1-sol / low` subagent performs bounded independent verification, as
requested. Existing Host models remain configured; a verifier model is not a
directive to replace AGY's model. No Astra extra-high verification is planned.
Use offline tests before live calls, declare each live batch and its limits, and
reuse evidence while its inputs remain unchanged. The earlier three-call Codex
login-reuse grant is exhausted and does not authorize new credential reuse.

Report actual input/cached/output tokens, tool calls, failures and elapsed time
when available. Compare identical tasks and model settings, keeping quality as a
condition of any efficiency claim. Do not promise a token reduction percentage
or compare different Host/model runs as a controlled benchmark.

Implement slices 1 and 2 first; content diagnosis can proceed alongside driver
repairs. Their evidence enables slice 3. Prepare slice 4 documentation early and
qualify its final artifacts only after the required behavior is stable. Keep
routine independent development moving while an external evidence gate waits.

Defer additional Hosts, Desktop expansion, a new Graph or Agent orchestration
layer, concurrent multi-Host writing, a broad 82-Skill rewrite, and new provider
integrations. These would need a separately selected outcome.

Execution continues under existing CLI-405 (journey/quality), CLI-409 (AGY) and
CLI-410 (release) ownership and existing evaluation contracts. This draft adds
no task IDs, changes no task state/dependency/accepted evidence, sets no release
date, and does not authorize publication. The subsequent maintainer instruction
authorizes local implementation through the existing development loop.

## Planning-only validation

The requested `gpt-6.1-sol / low` review passes the seven program-roadmap tests,
generated-index consistency and whitespace checks. The ledger is byte-identical
to the baseline: all 249 task definitions and 46 accepted records are unchanged.
No native build, live Host observation or release check is run for this draft.

## October 5 — first authorized implementation increment

Local implementation starts from `2813426f` on a feature branch. This increment
implements the offline portion of slices 1 and 2; neither slice's live exit gate
is declared complete. No task state, dependency or accepted record is promoted.

`e167f4bd` extends the existing installed-Plugin adapter:

- `freeze` and `seal` create new input/final manifests exclusively, checking frozen
  source/prompt/installation/catalog bytes and refusing drift or overwrite.
  These local snapshots are not a lock on live fixtures or authenticated dates.
- Actual completed calls retain IDs, event indexes and argument/result/error
  hashes. Empty/malformed payloads and conflicting error flags cannot satisfy
  required reads. The installation can bind its exact MCP server name; this
  does not implement an AGY-to-Codex event converter.
- Tool results remain separate from process/turn completion. Timeout, denial,
  cancellation, forced exit and incomplete turns retain usable call observations
  while the case remains unsuccessful. A truncated final JSONL line cannot pass.
- New catalog requirements bind installed guidance snapshots to complete command
  output or independently retained Host context, plus a separate usage review.
  Model prose alone cannot establish loading. Context provenance still requires
  inspection; no unsupported Host export capability is assumed.
- The source paragraph gets an executable whole-answer 250–350 character rule,
  with non-whitespace Unicode counting and registered anchor-citation exclusions.
  Semantic review cannot override a failed length check. Historical catalogs,
  observations and reports retain their original scope and bytes.

`f825c60f` updates only canonical workflow guidance: use available complete Host
guidance or supported installed-file reads; check the final requested prose's
length; distinguish sample, allocation, measurement and analysis units; retain
source discrepancies and excerpt-only limits. Existing project-ID/binding/read,
preview/approval/CAS and configured-model behavior remains with its owners.
These instructions address observed risks but do not prove new model behavior.

The requested `gpt-6.1-sol / low` verifier reports:

| Check | Observed result |
| --- | --- |
| Focused regression modules | 77/77 pass: Plugin baseline 20, journey observations 11, journey cases 3, routing probe 10, V1 evaluation 14, research-standard validator 19. |
| Original short-paragraph reproduction | Original answer SHA-256 `87fc5ef8048b681b30d18a06916e6efae5909f9aa1503be9c9f7f14b74c84b3f` counts 227 under the new declared rule and fails 250–350. Original capture/catalog/reports are unchanged. |
| Capability contract | `python3 scripts/validate_capability_contract.py` passes. |
| Content materialization | Existing `build_materialize_source` and `fail_if_symlinks` pass; all four changed canonical files match generated portable bytes. |
| Review | No remaining blocking finding; staged whitespace checks pass. |

The materialized package is retained at
`/tmp/qiongli-quality-portable-content-p7oe7_6f/source/qiongli-workflow`;
its local verification record SHA-256 is
`54eada462ed4561adcb7f1ef2ab1f3b8f59662a861baad551d09d7bb7114a4dc`.
An earlier full `--target plugin` materializer invocation selected the retained
Lite build and was stopped/reaped with exit 130. Its partial output remains at
`/tmp/qiongli-plugin-quality-offline-20261005-verifier-01`; it is not a passing
Plugin package check. The content-only owner subsequently supplies the required
bounded evidence without a Rust build. The first regression run also exposed a
test expectation error (a shared invalid guidance hash affects all three cases,
not one); correcting that expectation did not weaken the production check.

At integration, all seven program-roadmap tests and generated-index consistency
pass. All 249 task IDs/states/dependencies and 46 complete accepted records remain
unchanged; only CLI-405 gains this progress record and the current plan link.

No new model or Host session, credential reuse, user installation/configuration
change, native candidate build or publication is performed. Existing live 2/3
structural and 0/3 complete baseline results remain historical failures/gaps.
The next increment is the retained AGY driver's offline permission/wrapping and
shutdown fixtures plus full transport-result capture. Then freeze the changed
installed candidate and public fixtures for newly scoped live observations,
including the three Codex tasks with the declared counting convention. Their
evidence enables slice 3's save/restart and held-out cases; release/upgrade
qualification remains later work.

## October 5 — bounded AGY capture and cleanup increment

The maintainer requests the next step after `b60518bf`. Implementation
`1ae2bbdd`, followed by scoring-identity fix `0cac3641`, adds
`evals/research_journey/antigravity_observation.py` under the existing
journey owner. It prepares one new capture directory for an exact read-only status
request, binds installed identities, command, selected Qiongli environment and
fixture/configuration snapshots, and spools full raw stdout/stderr to disk. It
neither replaces the user's model nor writes trust/permission settings, links
credentials, resumes a conversation or retries a denied call.

The adapter consumes AGY's own stream events and uses the existing V1 runner for
behavior checks. Matched invocation steps, native envelope bytes and Host/process
completion are distinct observations. Full status evidence requires the existing
status schema plus matching MCP content/structuredContent; short tool summaries,
bare status objects and model descriptions cannot fill missing output. A Host's
`SUCCESS` status or process exit zero cannot erase a denied action.

The old PTY driver lowercased invocation paths, depended on a recent terminal
buffer after the final answer and hashed a reset buffer across restarts. A new
current-screen checker preserves JSON path case and exact arguments while
recognizing the single-use permission choice; it never sends approval keys.
The print driver uses one process and hashes its entire saved stream, with
SIGTERM-before-wait, bounded SIGKILL fallback and child reaping on timeout or
exceptions. Remaining members of its owned POSIX process group are reported;
other-session descendants remain outside that guarantee. The temporary old
drivers and captures are unchanged, and no repaired live interactive run is claimed.

The requested `gpt-6.1-sol / low` verifier passes **56 unique focused checks**:
22 new AGY observation cases, 20 existing Plugin baseline cases and 14 V1 cases.
The final nested-JSON and scoring-identity fixes rerun only the AGY module;
unchanged baseline and V1 results are reused. Coverage includes full synthetic envelopes, denied calls with
Host success, summaries/missing output, duplicate/foreign/truncated events,
wrapped/case-sensitive arguments, actual fake-child normal/timeout/kill/exception
cleanup, signal-exit races, missing/drifted snapshots, exact command/environment
bindings and refusal to overwrite an existing capture.

Initial validation/review exposed missing-snapshot booleans, `false == 0` exit
comparison, inconsistent selected environment, nested duplicate JSON keys and
missing installation-identity validation when scoring a resealed capture;
all now refuse success with regression coverage. A mock's one-shot exception
caused a test-only `StopIteration` and was corrected to model the repeatable
process-disappearance race. Final code review has no blocking finding and staged
whitespace checks pass. All seven roadmap tests pass; the ledger retains 249
tasks and 46 accepted evidence records without state or dependency changes.

The existing real headless record at
`/home/hermes/qiongli-codex-continuity-yqk33pu3/agy-session-1jd6tkog/events.jsonl`
is replayed read-only, SHA-256
`31214c12927530d5746ed5976f18558640565570d1b09daffc1e44c1106f9b46`.
It reports Host `SUCCESS` and `num_turns: 1`, but its response is empty, one action
is denied, and the matching MCP step has no output. The adapter retains those
facts and refuses a pass. Earlier file-read steps also mean this historical
session is not the new exactly-one-tool status scope. No historical observation
or failed attempt is promoted into a live success.

Installed `agy --help` and the [official headless documentation](https://antigravity.google/docs/cli/headless/)
confirm the structured-output interface, not that the local Host exposes the
required full MCP envelope. This turn runs no new model/Host session, permission
or login change, native build, installation or publication. The live slice-1
exit gate remains open: a normally authorized actual status invocation must
provide complete output and clean shutdown on the named installed candidate.
If only a summary is available, retain that capability gap without broadening
permission or substituting another result. Independent preparation of the fresh
installed-Codex three-case baseline can proceed; its earlier three-call login
reuse grant remains exhausted. Save/restart, held-out and upgrade qualification
remain later increments. CLI-409 owns this progress; task states, dependencies
and accepted evidence remain unchanged.

## October 5 — fresh installed-Codex candidate and frozen inputs

The next bounded increment prepares the corrected three-case baseline without
starting another model observation. Commit `8618fb44` refreshes only the embedded
content lock through the existing native generator. The package still reports
2.3.0: this is a development candidate, not a version bump or release.

| Identity | Exact observation |
| --- | --- |
| Native source | `8618fb44ee43ccc2b9b1380039fb45f0745d8ee2` |
| Content source | `7bbcb171914a9515e14372e7d69d76b9a2255d80` |
| CLI SHA-256 | `cbd45851208c0ccb2afe3d9157f2dea139a347c8b18992041415fea6a6cd88cc` |
| Embedded pack SHA-256 | `caecff2f211cf7174dfb16ed064dd2329ab6589ef55325e24174ee910e9a126c` |
| Installed Plugin receipt SHA-256 | `f8c64fbe38315f442b369f0cc5e10875b7959d1afcbbefe4401808ec209bbc04` |
| Frozen input manifest SHA-256 | `01b799b2e56b7e585847a6c4228bcd3b18ff0f89ba8fc75fa3f3771ed07060b5` |

The requested `gpt-6.1-sol / low` verifier regenerates the 446-entry pack and
passes the closest lock round-trip/source-drift test. The CLI-only build uses
Rust 1.97.0, `--profile test --locked --offline`, and the exact native source
stamp; it finishes in 210 seconds. Native snapshot confirms that stamp. No
Desktop, retained Lite package, target-platform distribution or release gate is
run. Build logs and the build receipt remain under
`/tmp/qiongli-codex-quality-next-9rubrmze`.

The existing preview/apply owner exports a new Plugin, and Codex 0.160.0's
official manager installs/enables `qiongli@quality-next` under the isolated root
`/home/hermes/qiongli-codex-quality-next-9rubrmze`. Its export, marketplace copy
and installed cache match all 468 receipt-listed file bytes and modes; the
binary and receipt match too. Lite 15 / Full 35 discovery is directly checked
without a model. The three modified workflow/reference files match canonical
bytes. The installed root Skill matches the complete native projection:
canonical Skill plus the existing Codex adapter and shared Host guidance.

Public fixtures are completed before freezing or any model call. New project
`prj_0d75b7cffdc8f871ab0f5afe56d6a624` reaches revision 4 through the existing
capture intake, exact consolidation preview, model review, dual approval and
CAS owners. Its note explicitly labels the reused historical answer; neither
that answer nor its old Skill-read failure is a new model result. The original
public source packet is unchanged. Only the separately labelled synthetic
packet receives the deliberate changed-source marker. Two current documents
read back byte-for-byte; changed-source and stale-revision reads refuse, with
project/config bytes preserved throughout those negative checks.

The new catalog, installation identity, source registry/packet, complete
installed guidance, three prompts, prepared-state/run-scope records and three
reviewed local scripts form 16 frozen input files. The paragraph prompt declares
the exact whole-answer 250–350 counting rule. Known installed guidance paths are
supplied for explicit supported file reads; this does not test automatic Skill
discovery. The continuation prompt supplies the new project ID but no current
revision, document path/hash or saved body. It permits guidance-file reads while
requiring research-document recovery through the native read-only MCP tools.
All three cases remain unattempted; there is no final capture manifest, answer,
usage measurement or new academic pass.

The temporary driver reuses the tested process-group cleanup owner, keeps full
on-disk events/stderr and process/preservation receipts, refuses retries or input
and cache drift, and leaves missing guidance evidence unsuccessful. A spawn
failure retains a null actual exit code and its process receipt; the existing
scorer marks that case unavailable rather than inventing a timeout. These three
scripts pass syntax checks and independent static review; the driver has not
been exercised with a real model in this increment.

Preparation failures remain in the local logs: an initial check incorrectly
compared the projected root Skill to canonical bytes alone; a reused variable
reported a byte count instead of 468 cache entries; serializing a Path interrupted
the first source-preview binding record after native preview. The checks and
metadata are corrected without repeating installation or applied transactions;
the original outputs and partial binding are retained. The unsafe shared `/tmp`
ancestor was identified during review before installation, so durable native
outputs use a new private home directory under existing path policy.

No authentication file is read, copied or linked, no model is called, and the
real user installation/model/configuration is unchanged. The earlier three-call
login grant remains exhausted. The next live batch, if newly authorized, is
exactly these three public-material cases on `gpt-6.1-sol / low`, once each and
at most 180 seconds per case, with only the three named native read tools
approved in the isolated invocation. Temporary login linking and cleanup require
that new grant. Preserve the prepared inputs until the decision; failure or
unavailable guidance remains in the fixed denominator. The AGY full-output/clean
shutdown gate and the later save/restart, held-out and upgrade gates stay open.

At integration, the same verifier independently checks every frozen input,
current fixture/config snapshot, exact current-document readback and retained
negative result, and confirms the old 24-file capture is unchanged. Seven
program-roadmap tests, index consistency and whitespace checks pass. All 249
task IDs/states/dependencies and 46 complete accepted records remain unchanged;
only CLI-405's progress text changes. Independent diff review has no blocking
finding. These local checks do not promote installation, academic or release
acceptance.
