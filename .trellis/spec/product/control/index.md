# Product Control

Use this spec for work that changes the Qiongli 2 product spine, release scope,
or acceptance claims.

## Authority

Use the narrowest source that owns the decision:

1. the master roadmap owns product direction, ordering, milestones, and the
   current execution horizon;
2. the current task under `.trellis/tasks/` owns one bounded execution scope;
3. Program Ledger v1 owns live task state and exact accepted evidence;
4. the applicable release plan and acceptance ledger own release transitions
   and publication evidence;
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
Kernel, institutional modes, arbitrary providers, unbounded Agent topologies and
remote collaboration are post-2.0 work and cannot substitute for an open
replacement row.

## Current Execution Priority

Follow the master roadmap's current execution horizon. `PLT-401`—`PLT-403`
capacity and bounds are accepted. The remaining technical packages are ordered:

1. `PLT-404`—`PLT-406` App-owned ACP v1, All Chat State and bounded
   coordinator/worker recovery;
2. `PLT-407`—`PLT-408` measured ACP/App performance and concurrency;
3. `SEC-401`—`SEC-405` external-content security;
4. one fresh-version replacement candidate after those bounded packages.

The program ledger reports state and evidence; it does not reorder this list.

The current ACP slice reuses the existing orchestration task graph, role and
worker packets, handoff bindings, evidence references, cancellation, and project
compare-and-swap owners. Each run has one coordinator and at most two bounded
worker/reviewer sessions. Workers do not recursively delegate or form an
untracked peer message bus. ACP owns Agent transport and private sessions;
Qiongli All Chat State owns only the shared, user-visible collaboration record.
External Host mode remains supported, and direct provider APIs do not return as
the default.

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

Use this contract to keep daily development, pull-request integration,
cross-platform builds, and release qualification independent. Moving to a
higher lane is explicit; success in one lane never authorizes the next.

### 2. Signatures

- Daily / **Focused**: the smallest package-native lint, type-check, test, or
  negative command that can falsify the change.
- PR / **Slice**: affected checks plus the required exact-head GitHub contexts.
- Build: a named local target command or explicit Native CI run; record source,
  target, and whether code ran or only compiled.
- Release / **Acceptance**: an explicit versioned release task and
  `workflow_dispatch` from `2.x` using the existing release owners.

### 3. Contracts

- **Focused** is the task default. A final task check remains Focused unless the
  task explicitly produces a PR-ready Slice.
- Security, authorization, schema compatibility, path ownership, and data-loss
  risks receive their focused negative checks immediately; lane separation
  never delays these checks.
- A ready source-affecting PR runs the required three-platform native matrix.
  Pure non-runtime documentation/process/evidence changes preserve the required
  context names with lightweight report steps. Mixed, unknown, runtime,
  workflow/action, fixture, and empty diffs fail safe to the full matrix.
- Draft PRs do not expand the matrix. Merge pushes do not duplicate it. Explicit
  `workflow_dispatch` always runs the full source, Lite, package, and candidate
  checks.
- A target build proves only that target. `cargo xwin test --no-run` is Windows
  compilation feedback, never a Windows runtime pass.
- **Acceptance** adds target packages, packaged product, live Hosts where
  claimed, migration/rollback, supply-chain trust, authorization, and public
  verification only for an explicit candidate.

### 4. Validation & Error Matrix

- changed trust/data/schema boundary without a focused negative check -> check
  incomplete;
- task completion automatically treated as PR, Build, or Release evidence ->
  invalid lane transition;
- Windows-target compilation reported as Windows runtime validation -> invalid
  build claim;
- pure docs/process change starts the native matrix -> classifier regression;
- mixed/runtime change skips the native matrix -> fail-closed regression;
- release authorization from green Slice evidence -> invalid release claim;
- automatic PR or merge-push activity starts packaging or promotion -> workflow
  policy failure.

### 5. Good / Base / Bad Cases

- Good: a local change runs one focused test; its ready source PR later runs one
  exact-head matrix; release work waits for a separate release task.
- Base: a pure process/docs PR runs policy tests and lightweight required native
  contexts without installing the toolchain.
- Bad: every task runs three targets, packages, live Hosts, and promotion.
- Bad: green PR checks or a cross-compiled artifact are reported as release or
  native-runtime evidence.

### 6. Tests Required

- Boundary tests cover non-runtime-only, mixed, runtime, frozen, nested-fixture,
  unknown, and empty diffs.
- Policy tests preserve the four required context identities and explicit manual
  dispatch for packaging, candidate, and promotion work.
- Roadmap tests keep task IDs, ledger state, and generated evidence views
  deterministic without making them the current planning owner.

### 7. Wrong vs Correct

Wrong: finish a task, automatically run a release checklist, and treat CI as
authorization.

Correct: record the current lane, run only its owner, and enter a higher lane
only through an explicit PR, build request, or release task.

## Scenario: Opt-in platform capacity receipts

### 1. Scope / Trigger

Use this contract only when collecting PLT-401--PLT-403 capacity evidence on
Linux, macOS, or Windows. It is a manual Build-lane check, not a daily or PR
gate.

### 2. Signatures

- Set `QIONGLI_CAPACITY_OUTPUT_DIR`, `QIONGLI_CAPACITY_SOURCE_COMMIT`, and
  `QIONGLI_CAPACITY_RUN_ID` from the clean committed source being measured.
- Run `cargo test --manifest-path packages/qiongli-native/Cargo.toml
  --workspace --lib --release --locked platform_capacity_baseline -- --ignored
  --test-threads=1`.

### 3. Contracts

- Produce `qiongli-project-capacity.json` and
  `qiongli-desktop-capacity.json` with the same source, run, target, and schema
  identity.
- Each workload records one warm-up followed by 20 samples and reports
  nearest-rank p50/p95 observations; receipts define no pass/fail budget.
- Workloads use deterministic bounded profiles and actual project, graph,
  portfolio, portable archive, startup, snapshot, and IPC owners.
- The heavy tests remain ignored and run in Native CI only through an explicit
  three-target `workflow_dispatch`.

### 4. Validation & Error Matrix

- missing or malformed source/run/output variables -> fail before measurement;
- unsupported target or missing process-memory source -> explicit test failure;
- wrong profile, sample count, target label, or receipt identity -> contract
  failure;
- ordinary PR activity starts the capacity job -> workflow policy failure.

### 5. Good / Base / Bad Cases

- Good: one exact clean source produces both receipts on all three native
  targets through one explicit dispatch.
- Base: local release-mode execution produces observation-only receipts for one
  target.
- Bad: cross-compilation is reported as native runtime evidence, or a receipt
  from another source is reused.

### 6. Tests Required

- Focused project and Desktop receipt-contract tests.
- Release workspace compile with the ignored baseline selected.
- Branch-policy tests proving manual-only execution.
- Before accepting PLT-401--PLT-403, one exact-source Linux/macOS/Windows run
  with both artifacts present.

### 7. Wrong vs Correct

Wrong: add capacity thresholds to every PR or infer runtime results from a
cross-build.

Correct: collect comparable observations manually on each native target, then
use the receipts to decide later budgets.

### Evidence closeout boundary

- A closeout records `product_source`, exact CI/promotion run IDs, candidate-set
  digest, package digests, and `publication_allowed`; it never substitutes the
  closeout commit's own SHA for the built product source.
- A non-runtime documentation/process/evidence commit does not require another
  package run when it changes no product or package input. Any product/package
  input change does.
- If protected publication requires the current branch head after an
  evidence-only commit has landed, do not authorize the older internal
  candidate. Freeze and qualify a new product candidate when release resumes.

## Scenario: Provenance-bound three-target candidate

### 1. Scope / Trigger

Use this contract when Community Alpha rebuilds macOS, Windows, and Linux
artifacts after exact-source Native CI. It keeps qualification, building, and
publication authorization as separate evidence identities.

### 2. Signatures

- Workflow inputs: `source_commit` (40 lower-hex), `native_ci_run_id` (positive
  decimal), and `request_publication_authorization` (boolean, default `false`).
- Candidate `build_run_url`:
  `https://github.com/jxpeng98/qiongli/actions/runs/<run>/attempts/<attempt>`.
- Legacy run-only URLs remain readable, but new candidate builds record the
  exact attempt.

### 3. Contracts

- `native_ci_run_id` must name a completed successful Native CI run for the
  exact current remote `2.x` source. It is qualification evidence, not the
  builder invocation.
- `build_run_url` comes from `GITHUB_RUN_ID` plus `GITHUB_RUN_ATTEMPT` in the
  promotion workflow that creates the artifacts.
- All three target receipts and the aggregate candidate use the same source,
  version, attempt URL, ordered target set, file sizes, and SHA-256 identities.
- `request_publication_authorization=false` completes the non-publishing
  candidate with the protected Environment job skipped. Only an explicit true
  value may enter that job; neither path publishes or receives a private key.

### 4. Validation & Error Matrix

- source is not current remote `2.x`, or Native CI source/status/conclusion
  differs -> exact-head preflight fails;
- run or attempt is empty, zero, non-decimal, oversized, or has extra path
  segments -> `community-alpha-promotion-invalid`;
- target source, attempt URL, version, platform, asset, evidence, or digest
  differs -> target/candidate aggregation fails closed;
- default candidate enters the protected Environment -> branch-policy failure;
- successful aggregation alone -> `publication_allowed=false`, never release
  authorization.

### 5. Good / Base / Bad Cases

- Good: one exact promotion attempt freshly builds all targets, aggregates five
  digest-bound assets, skips authorization, and completes green.
- Base: an older canonical receipt with a run-only URL remains parseable for
  historical evidence but is not emitted by the current workflow.
- Bad: record the qualifying Native CI URL as the builder, or require protected
  approval before a non-publishing candidate can complete.

### 6. Tests Required

- Focused policy: assert the default-false input, exact attempt URL, separate
  Native CI validation, safe dispatch, and authorization-job gate.
- Rust: accept current attempt and legacy run-only URLs; reject zero, malformed,
  and path-extended attempt identities; retain candidate digest/target tests.
- Acceptance: explicit exact-head Native CI plus one downstream three-target
  run, followed by byte verification of the downloaded candidate inventory.

### 7. Wrong vs Correct

Wrong: `build_run_url=.../actions/runs/$NATIVE_CI_RUN_ID`; that run qualified
the source but did not create the promoted bytes.

Correct: `build_run_url=.../actions/runs/$GITHUB_RUN_ID/attempts/$GITHUB_RUN_ATTEMPT`
inside the promotion run, while the Native CI identity is validated separately.

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

## Scenario: REL-913 installation lifecycle acceptance

### 1. Scope and Trigger

- Trigger: an exact REL-910 candidate needs clean-install, upgrade, repair,
  rollback, and uninstall evidence before legacy recovery paths can retire.
- Scope: reuse the native candidate installer, managed payload transaction,
  Host integration, reconciliation, and macOS update-helper owners. Do not add
  another installer, public command, or lifecycle schema.

### 2. Evidence Identity

- Candidate lifecycle receipts are target-native for Linux, macOS, and Windows
  and bind the exact workflow source SHA and native artifact identity.
- The macOS update receipt binds the current packaged archive plus an explicitly
  labelled, ad-hoc-signed N-1 metadata fixture derived from that archive.
- A derived predecessor fixture proves replacement mechanics only; it is not a
  previously published binary, production signature, notarization, or update
  selection receipt.

### 3. Contracts

- Preview and rejected approval mutate no candidate-owned state. Successful
  apply verifies healthy, and uninstall removes only receipt-owned payload and
  Host integration state.
- Repair is allowed for absent owned payload and refuses present byte drift.
- Successful N-1 to N replacement commits N as last-known-good; failed health
  restores the N-1 application and never advances its generation.
- User-project bytes, unrelated global v2 bytes, and unmanaged Host/home bytes
  retain the same SHA-256 through install, failure compensation, verify,
  replacement, rollback, and removal.
- One target's receipt never supplies another target's install claim.

### 4. Validation and Error Matrix

- preview, rejected approval, or failed compensation changes an owned path or
  canary -> candidate acceptance fails;
- repair accepts present drift or does not restore an absent payload -> focused
  payload transaction test fails;
- successful update keeps N-1, failed health keeps N, or last-known-good differs
  from the active bundle -> macOS journey fails;
- any project, global-state, or unmanaged-Host digest changes -> lifecycle
  acceptance fails;
- a local or historical receipt is reused for a changed source/target -> claim
  remains open.

### 5. Good, Base, Bad

- Good: all three target-native candidate jobs preserve the three canary classes,
  while macOS replaces N-1 with N and restores N-1 on failed health.
- Base: focused Rust tests prove install/remove, repair/drift, reconciliation,
  health commit, and rollback without running unrelated local suites.
- Bad: infer Windows or Linux behavior from macOS, delete a broad `.qiongli`
  root during uninstall, or call the derived predecessor a published package.

### 6. Tests Required

- Focused: Rust format; `qiongli-platform` and `qiongli` tests filtered by
  `rel_913`; shell syntax; and the candidate-matrix branch-policy test.
- Slice: exact-head Native CI must pass its ordinary Linux, macOS, and Windows
  foundation jobs before merge.
- Acceptance: explicitly dispatch Native CI on merged `2.x`, inspect all three
  candidate receipts and the packaged macOS update receipt, then record their
  exact source/run identities. Acceptance never authorizes publication.

### 7. Wrong vs Correct

Wrong: build a second lifecycle harness or recursively remove user roots because
they share a `.qiongli` name.

Correct: drive existing owners with exact candidate bytes, preserve unrelated
canaries byte-for-byte, and keep ephemeral-fixture limits visible in the receipt.

## Pre-Development Checklist

- Read the current Trellis task, generated current program index, and the Alpha
  acceptance ledger when release claims are affected.
- Name the broken user outcome and its shared owner.
- Confirm the work is selected from the master roadmap's current horizon.
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
