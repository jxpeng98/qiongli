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

Qiongli 2 must first produce one dependable 1.19 replacement spine:

`App -> native CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero`

The packaged product must not need a user-installed Python or Node runtime.
Graph v1 replacement acceptance additionally requires one representative
migrated project with source-bound scholarly semantics, usable
query/visualization, deterministic rebuild and truthful empty/sparse diagnostics.

Historical M0 external/manual evidence remains valid only for its exact source
and scope; it cannot qualify a changed 2.0 candidate. Graph v2, a Typed Research
Kernel, institutional modes, more providers/agents and remote collaboration are
post-2.0 work and cannot substitute for an open replacement row.

## Current Execution Priority

Keep the immediate Trellis lane in this order:

1. publish the 1.19-to-2.0 replacement matrix and three verification tiers;
2. prove the native CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero vertical;
3. stabilize App on those native owners;
4. accept Graph v1 on one representative migrated project;
5. freeze and qualify one exact 2.0 candidate, then begin the existing 90-day
   post-Stable 1.x maintenance countdown.

The first slice reuses the existing CLI lifecycle and packaged-product control.
One approved integration preview may authorize only a fixed, target-matched
official Codex or Claude CLI plan recorded in a new superseding ADR. The App
must then discard prior observations and report Ready only from fresh positive
Plugin identity/version, managed/cache bundle identity and Full MCP evidence.
Claude also exposes the expected Skill component; Codex bundle identity does
not prove live Skill invocation. Never add a generic shell/command surface,
write Host caches directly, or bypass Host trust and administrator policy.

The Graph slice reuses the existing Graph v1 projection and canonical artifact
extractors. `project`/`artifact` nodes and `contains` edges are structural
inventory and never establish semantic continuity. Repair readiness and the
canonical Skill output contract; do not activate Graph v2, a research kernel,
another graph store, or automatic prose-to-fact inference. Fixture-declared
numbers, structural keywords and generated Plugin mirrors are not migrated-user
quality authority. Model-dependent ablation remains optional observed evidence,
not deterministic CI. Keep only one implementation task active and close the
shared product vertical before App polish.

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

## Scenario: Three-tier verification

### 1. Scope / Trigger

Use this contract for every implementation, business-slice integration and
release-candidate check. It prevents routine development from spending time and
agent context on unrelated package/release evidence.

### 2. Signatures

- Focused: the smallest package-native lint, type-check, unit, integration or
  negative command named by the task.
- Slice: affected commands plus the exact-head `Native CI` contexts `Native 2.x
  change boundary` and `Rust native foundation (Linux|macOS|Windows)`.
- Acceptance: an explicit `workflow_dispatch` of `Native CI` on `2.x`, followed
  by the existing exact promotion workflow when all candidate jobs pass.

### 3. Contracts

- **Focused** runs in every implementation loop and falsifies only the changed
  behavior. Security, authorization, schema compatibility, path ownership and
  data-loss risks receive focused negative checks immediately.
- **Slice** runs after one complete user-visible business slice or small-version
  checkpoint is frozen. It covers every affected package/cross-contract check and
  the required exact-head three-platform native source matrix.
- **Acceptance** runs only for an explicit cutover or release candidate. It adds
  workspace/source, target packages, packaged product, current live Hosts,
  migration/rollback, trust/supply-chain and claimed manual journeys.
- A final Trellis check is full **task scope** at Slice tier, not unrelated
  repository or release work.

### 4. Validation & Error Matrix

- changed trust/data/schema boundary without a focused negative check -> check
  incomplete;
- complete business slice without affected package/cross-contract coverage ->
  Slice incomplete;
- ordinary push/PR starts package assembly, packaged acceptance or promotion ->
  workflow-policy failure;
- release authorization from green Slice evidence -> invalid release claim;
- higher-tier failure without a focused reproduction -> return to Focused.

### 5. Good / Base / Bad Cases

- Good: a business slice uses focused loops, one compact Slice, then waits for an
  explicit candidate before package/Host/migration evidence.
- Base: a docs-only policy task runs its focused policy tests and exact-head
  source CI, but no product package.
- Bad: every edit runs the full workspace and three target packages, or a green
  PR is reported as release acceptance.

### 6. Tests Required

- Policy tests assert the four required context identities remain unchanged.
- Policy tests assert portable frontend checks run once on Linux while all three
  native Rust jobs remain.
- Policy tests assert package assembly, packaged-product acceptance, Lite
  candidate acceptance and promotion require explicit `workflow_dispatch` on
  `2.x`.
- Roadmap tests assert the deterministic task inventory and generated index.

### 7. Wrong vs Correct

Wrong: end every task with unrelated full-workspace, package, live-Host and
promotion runs, then copy successful logs into the task.

Correct: report command, tier, result and concise counts; on failure show the
first actionable error and smallest focused reproduction, then rerun only the
invalidated higher-tier job. If a public claim is not accepted, remove or narrow
it rather than recording a false pass.

### Evidence closeout boundary

- A closeout records `product_source`, exact CI/promotion run IDs, candidate-set
  digest, package digests, and `publication_allowed`; it never substitutes the
  closeout commit's own SHA for the built product source.
- An evidence-only status commit does not require another package run when it
  changes no product or package input. Any product/package input change does.
- If protected publication requires the current branch head after an
  evidence-only commit has landed, do not authorize the older internal
  candidate. Freeze and qualify a new product candidate when release resumes.

## Scenario: REL-905 data lifecycle policy

### 1. Scope and Trigger

- Trigger: a user needs to back up, export, uninstall, delete, or understand the
  1.x support boundary before changing Qiongli-owned state.
- Scope: bilingual documentation and its source-bound policy check only. Do not
  add a backup service, purge command, public schema, or migration workflow.

### 2. Authority and Check

- User authority: `docs/guide/data-lifecycle.md` and its Chinese counterpart.
- Maintenance authority: `docs/maintainer/release-branch-policy.md`.
- Focused check: `python -m unittest tests.test_data_lifecycle_policy -v`.

### 3. Contracts

- Users own full project roots, including `<project>/.qiongli/v2`, and the
  resolved global v2 root.
- A complete recovery checkpoint includes both roots from stopped writers;
  secure credentials are backed up separately.
- Portable export is a privacy-filtered exchange format, not a complete backup,
  and excludes private state, credentials, conversations, and build/cache data.
- Uninstall and removal affect receipt- or Host-owned integration state; data
  retention and deliberate deletion remain separate choices.
- The 1.x support window ends 90 days after actual Qiongli 2 Stable publication.
  Alpha, Beta, policy publication, and ordinary merges do not start the clock.

### 4. Claim Matrix

| Claim | Accepted source | Invalid substitute |
| --- | --- | --- |
| Recoverable backup | stopped project plus global v2 roots and separate credential recovery | portable export alone |
| Product uninstall | exact receipt- or Host-owned integration removal | broad recursive data deletion |
| 1.x end date | 90 days after actual Stable publication | Alpha, Beta, policy, or merge date |

### 5. Good, Base, Bad

- Good: the user can identify every owner, back up both local roots, distinguish
  export from recovery, and separate uninstall from deletion.
- Base: the bilingual policy is discoverable and one dependency-free test binds
  its claims to the existing maintenance authority.
- Bad: documentation promises automated purge, calls portable export a full
  backup, or invents a calendar end date before Stable publication.

### 6. Tests Required

- Run the focused policy test, docs build, roadmap check, task validation, and
  exact-head source CI.
- Do not build packages or run Host/promotion acceptance for this docs-only Slice.

### 7. Wrong vs Correct

Wrong: introduce a speculative lifecycle subsystem to describe behavior already
owned by project storage, receipts, Agent Hosts, credential stores, and providers.

Correct: publish one bilingual policy over those existing owners and keep one
small source-bound test that fails when discoverability or the support boundary
drifts.

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
- [Public schema authority](public-schema-authority.md) — Rust-owned changed
  contracts, truthful migration baselines, and closed compatibility classes.
- [Authorization policy and receipt v1](authorization-policy-v1.md) — closed
  roles/actions, non-transitive authority, and redacted evidence receipts.

Reference files:

- `docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md`
- `docs/superpowers/roadmaps/qiongli-current-program-index.md`
- `docs/superpowers/acceptance/2026-08-01-qiongli-alpha3-readiness.md`
- `docs/architecture/decisions/README.md`
