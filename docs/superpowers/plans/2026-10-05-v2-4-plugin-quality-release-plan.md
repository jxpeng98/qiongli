# Qiongli 2.4.0 — Plugin reliability and research task quality

- Status: planning draft requested by the maintainer; implementation selection
  and release authorization are separate decisions.
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
date, and does not authorize implementation or publication by itself.

## Planning-only validation

The requested `gpt-6.1-sol / low` review passes the seven program-roadmap tests,
generated-index consistency and whitespace checks. The ledger is byte-identical
to the baseline: all 249 task definitions and 46 accepted records are unchanged.
No native build, live Host observation or release check is run for this draft.
