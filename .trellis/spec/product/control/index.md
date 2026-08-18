# Product Control

Use this spec for work that changes the Qiongli 2 product spine, release scope,
or acceptance claims.

## Authority

Use the narrowest source that owns the decision:

1. the current task under `.trellis/tasks/` owns executable scope and next work;
2. the Alpha acceptance ledger owns accepted evidence and release authorization;
3. the Program Ledger v1 owns live state and exact evidence identity for the
   master roadmap's task IDs;
4. the master roadmap owns task identity, description, milestone order and
   long-term priorities;
5. accepted ADRs own architecture boundaries;
6. older plans and receipts are historical evidence only.

Do not copy the roadmap backlog into Trellis. Keep at most one implementation
task in progress and create the next task only when the current one is closed.

## Current Release Outcome

Alpha 3 must first produce one internally usable, self-contained product spine:

`App -> native CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero`

The packaged product must not need a user-installed Python or Node runtime.
Public distribution is a later state: target-native, live-Host, update/rollback,
trust, authorization, and public-observation evidence remain owned by A6-A9.

M0 external/manual release evidence remains open, but it is no longer a blanket
development freeze after the internal first-usable spine closes. Working-head
implementation of `EVAL-401`—`EVAL-407` does not imply that those changes are
integrated into `origin/2.x`, nor does it imply M0 exit, Alpha 3 qualification,
or publication authority. M2 and later work remains deferred until the M1
entry/exit gates are satisfied.
Do not add Graph v2, a research kernel, more providers, more agents, or remote
collaboration to an Alpha 3 or M1 false-green task.

## Current Execution Priority

Keep the immediate Trellis lane in this order:

1. close the packaged App's existing CLI/Plugin effectiveness path;
2. replace metadata-only Plugin-quality scores with executable findings and
   repair only the bounded canonical Skill gaps they expose;
3. resume the remaining M1 evaluation, governance, security and platform work.

The first slice reuses the existing CLI lifecycle and packaged-product control.
One approved integration preview may authorize only a fixed, target-matched
official Codex or Claude CLI plan recorded in a new superseding ADR. The App
must then discard prior observations and report Ready only from fresh positive
Plugin identity/version, managed/cache bundle identity and Full MCP evidence.
Claude also exposes the expected Skill component; Codex bundle identity does
not prove live Skill invocation. Never add a generic shell/command surface,
write Host caches directly, or bypass Host trust and administrator policy.

The second slice reuses Evaluation Truth V1. Fixture-declared numbers,
structural keywords and generated Plugin mirrors are not quality authority.
Model-dependent ablation remains optional observed evidence, not deterministic
CI. Keep only one implementation task active: activation precedes Plugin
quality, and Plugin quality precedes the wider M1 queue.

## Scenario: App-mediated Host Plugin activation

### 1. Scope / Trigger

Use this contract whenever App or managed CLI integration confirmation can
install or repair the bundled Codex/Claude Plugin. It prevents an approval from
turning into a generic command runner or stale state from becoming Ready.

### 2. Signatures

- App: `preview-install-selected({codex, claudeCode})` or
  `preview-reconcile-integrations({codex, claudeCode})`, followed by
  `confirm-operation({token: [0-9a-f]{32}})`.
- Managed CLI: `qiongli app plan integrations-{install|reconcile} --target
  <codex|claude|all>`, followed by `qiongli app apply --plan <absolute-plan.json>
  --expected-plan-digest <sha256> --approve-filesystem-write
  --approve-client-config-change --approve-host-trust`.
- Native owner: `prepare_host_plugin_plans` -> digest-bound confirmation ->
  `execute_host_plugin_plan_set` -> fresh Host probes.

### 3. Contracts

- Target order is `codex`, then `claude-code`; execution stops on first failure.
- Executables are canonical paths from supported-client discovery. Arguments
  are only native constants: Codex `plugin add/remove`; Claude local
  `marketplace add` plus user-scope `plugin install/uninstall`.
- The digest binds target, install/repair/verify mode, executable, fixed argv,
  product/client versions, scope, resolved home/config roots, prior observation,
  and packaged-product plan. `HOME` is required; discovered `CODEX_HOME` and
  `CLAUDE_CONFIG_DIR` are forwarded when present.
- Launch uses no shell or stdin, a cleared deterministic environment, 30-second
  mutation and 5-second probe limits, and 512 KiB stdout/stderr bounds.
- Ready requires a fresh exact-version Plugin/cache receipt for the selected
  canonical or receipt-owned local workflow variant and a Full MCP probe;
  Claude additionally requires exactly one `qiongli-workflow` Skill component.

### 4. Validation & Error Matrix

- missing home/client/executable -> `host-plugin-*-unavailable`;
- changed digest, target state, or product ->
  `managed-operation-precondition-changed` / `host-plugin-plan-changed`;
- spawn/timeout/wait/read/overflow/UTF-8/non-zero -> stable `host-command-*`;
- malformed or contradictory Host inventory -> target JSON/details error;
- exact identity, version, source, scope, cache, Skill, or MCP mismatch ->
  non-Ready observation and explicit verify/repair reason.

### 5. Good/Base/Bad Cases

- Good: one approval runs the fixed official CLI plan, clears old evidence, and
  reports Ready only after all fresh probes pass.
- Base: already-current state runs no mutation command but still verifies fresh
  evidence.
- Bad: command exit zero with stale/malformed/missing evidence remains non-Ready;
  a failed first target never launches the second.

### 6. Tests Required

- Unit: assert exact argv/order/digest inputs, no-shell bounded failure classes,
  strict inventory parsers, and partial-batch stop behavior.
- Isolated client: assert Codex Plugin/cache/MCP and Claude
  Plugin/cache/Skill/MCP observations under temporary homes only.
- Product: run App API/Desktop checks plus one packaged vertical acceptance for
  frozen product inputs; never mutate the developer's normal Host profile.

### 7. Wrong vs Correct

Wrong: execute rendered UI command text, write Host caches directly, or infer
Ready from a successful install command.

Correct: recompute the fixed native plan at confirmation, reject any digest or
state change, execute the resolved official CLI, then derive Ready only from
fresh positive evidence.

## Evidence Ladder

Run only the smallest evidence set that advances the change:

1. one focused local check while editing;
2. one exact-head CI run after the change set is frozen;
3. one packaged vertical acceptance run when package inputs changed.

Manual UI, real-profile Host, update, and publication checks are release-claim
gates, not substitutes for the development checks above. If a public claim is
not accepted, remove or narrow that claim rather than recording a false pass.

### Evidence closeout boundary

- A closeout records `product_source`, exact CI/promotion run IDs, candidate-set
  digest, package digests, and `publication_allowed`; it never substitutes the
  closeout commit's own SHA for the built product source.
- An evidence-only status commit does not require another package run when it
  changes no product or package input. Any product/package input change does.
- If protected publication requires the current branch head after an
  evidence-only commit has landed, do not authorize the older internal
  candidate. Freeze and qualify a new product candidate when release resumes.

## Pre-Development Checklist

- Read the current Trellis task, generated current program index, and the Alpha
  acceptance ledger when release claims are affected.
- Name the broken user outcome and its shared owner.
- Confirm the work is part of the Alpha 3 product spine.
- Identify one focused check before editing.

## Quality Check

- App, CLI, Plugin/Skills, MCP, and Zotero claims match the shipped contracts.
- Ready follows a fresh supported observation; copied, registered, cached, or
  previously observed state alone is insufficient.
- Every accepted result is bound to the same source and package identity.
- No historical receipt is presented as evidence for a changed candidate.
- Roadmap task state comes from Program Ledger v1; a checkbox or merged PR alone
  never establishes `accepted`.
- No extra umbrella test, duplicate backlog, or speculative abstraction was added.

Executable contracts:

- [Evaluation Truth V1](eval-truth-v1.md) — shared case schema, counters, and
  fail-closed success predicate.
- [Program Ledger v1](program-ledger-v1.md) — exact roadmap inventory, six live
  states, evidence gate, deterministic index, and CI freshness check.
- [Governance truth records](governance-truth.md) — frozen ARC-201 baseline,
  complete current ADR registry, and classification-only 1.x parity status.

Reference files:

- `docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md`
- `docs/superpowers/roadmaps/qiongli-current-program-index.md`
- `docs/superpowers/acceptance/2026-08-01-qiongli-alpha3-readiness.md`
- `docs/architecture/decisions/README.md`
