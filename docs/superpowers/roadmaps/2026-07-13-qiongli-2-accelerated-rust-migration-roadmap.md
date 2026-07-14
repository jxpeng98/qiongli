# Qiongli 2 Accelerated Rust Migration Roadmap

Status: active execution; R1 content, config, and native command foundation
complete, with the R2 shared Lite runtime next

Decision date: July 13, 2026

Target branch: `2.x`

Active rolling branch: `feat/2x-native-alpha1`

Design authority:
`docs/superpowers/specs/2026-07-13-qiongli-2-native-acceleration-design.md`

Detailed architecture and program catalog:
`docs/superpowers/roadmaps/2026-07-10-qiongli-2-rust-native-platform-roadmap.md`

## Executive Decision

Qiongli 1.x is frozen at the accepted `v1.19.0-beta.1` baseline. Normal Python
and Node product development is closed. Active development targets one
Rust-native Qiongli 2 product.

The migration uses exactly one active rolling Draft PR at a time. The first PR
runs from the post-PR-#62 `2.x` base through `v2.0.0-alpha.1` readiness. Task,
workstream, and crate IDs are tracking labels rather than branch or PR gates.

Legacy Python and Node full suites are diagnostic-only. Required 2.x checks are
native Rust checks and proportionate boundary tests. Broader packaging,
integration, signing, and clean-machine checks run when the corresponding
artifact or public claim exists.

## Current Native Baseline

Integrated on `2.x`:

- accepted ADR 0201-0207 architecture set;
- non-publishing native alpha release identity and dry-run;
- native workspace and one multi-mode application scaffold;
- accepted 1.x MCP, CLI, content, and orchestrator source inventory;
- Contract v2 closure for 23 canonical tools and 24 public names;
- FND-202A resource-pack manifest and profile contract;
- FND-202B bounded canonical source collector;
- FND-202C deterministic pack writer and digests;
- FND-202D bounded verifier/loader through PR #62, merge commit
  `ebd2d7bef651fcbd22a7310aa50f9945604fa9eb`.

The current physical native workspace contains `apps/qiongli`,
`qiongli-content`, `qiongli-config`, and the isolated
`qiongli-windows-security` FFI boundary. FND-202E is complete through
portability head `870d85b8`, FND-202F is complete at `76ee339f`, CFG-201A ends
at `588e564d`, and CFG-201B ends at implementation checkpoint `90190612` on
the same rolling branch. R1 native command composition ends at implementation
checkpoint `f2a6fbe6`. Work continues through the shared Lite runtime rather
than returning to another legacy inventory or Python parity phase.

## Operating Rules

### Branch and PR

- keep one active native branch: `feat/2x-native-alpha1`;
- keep one Draft PR from that branch into `2.x`;
- do not create FND, CFG, MCP, UI, installer, or packaging child PRs;
- use cohesive Conventional Commits as review and rollback checkpoints;
- push the same branch after each green dependency-contiguous batch;
- turn the PR Ready only when the complete alpha.1 exit gate passes;
- create the next rolling branch only after the current one merges.

### Development validation

Run for every native batch:

- `cargo fmt --all -- --check`;
- `cargo check --workspace --all-targets`;
- `cargo clippy --workspace --all-targets --all-features --locked`;
- focused Rust tests for changed behavior;
- `cargo test --workspace --all-targets --all-features --locked` before push.

Run when the boundary is touched:

- state, path, permission, redaction, process, network, and secret tests;
- installer transaction and rollback tests;
- provider timeout and cancellation tests;
- MCP framing and profile-exposure tests;
- resource-pack reproducibility and drift tests.

Run only for milestone/release claims:

- target-specific artifacts;
- signing, checksums, SBOM, and provenance;
- clean-machine zero-runtime audits;
- real Codex and Claude activation;
- install, upgrade, repair, remove, and rollback matrices.

The full Python and Node suites are removed from required 2.x CI. They may be
run manually for a named compatibility investigation but do not block native
development or release readiness.

### Long-flow sessions

A development session continues across multiple task IDs and commits while the
next work is dependency-contiguous and safely verifiable. Completing one small
task is not a reason to stop. A session pauses only for new authority, a real
architecture choice, a concrete security/data-loss blocker, or an external
platform limitation.

## Rust Workspace Framework

The alpha implementation uses these physical boundaries:

```text
packages/qiongli-native/
  apps/qiongli/
  crates/qiongli-content/
  crates/qiongli-config/
  crates/qiongli-runtime/
  crates/qiongli-execution/
  crates/qiongli-platform/
  crates/qiongli-ui/
  crates/qiongli-testkit/
```

| Boundary | Owns | Must not own |
|---|---|---|
| Content | Embedded resources, profiles, integrity, approved materialization | UI, host registration, model calls |
| Config | Versioned state, migration, atomic writes, secret references | UI callbacks, provider transport |
| Runtime | Contracts, providers, domain services, MCP | Client-specific installation |
| Execution | Agents, ToolHost, policy, orchestrator | UI or host cache mutation |
| Platform | Install plans, integrations, doctor, updater, rollback | Duplicate domain/provider logic |
| UI | Views, view models, typed intents, accessibility | Direct filesystem, process, network, or secret access |
| Testkit | Native fixtures, fakes, target harnesses | Production behavior |
| App | Mode parsing and dependency composition | Duplicated service implementation |

The old fourteen-crate diagram remains a possible later decomposition, not an
alpha scaffolding requirement. A module splits into another crate only when an
independent security, reuse, compilation, or packaging boundary is demonstrated.

## Critical Path

```text
R0 native control plane
  -> R1 content/config foundation
  -> R2 shared Lite runtime
  -> R3 installable CLI/UI alpha.1
  -> R4 Full runtime alpha.2
  -> R5 native cutover beta.1
  -> stable hardening
```

R0-R3 live in the first rolling Draft PR. R4 and R5 use successor rolling PRs,
but only one may be active at a time.

## R0 — Native Control Plane

Purpose: remove process overhead before adding more product surface.

Implementation status on July 13, 2026:

- complete at checkpoint `4d81a933` after actionlint, 18 focused policy tests,
  and the local native format/check/Clippy/workspace-test gate passed;
- GitHub Actions run `29286377360` passed the change boundary and Linux, macOS,
  and Windows Rust jobs on that checkpoint;
- live ruleset `18800504` requires exactly those four contexts while retaining
  pull-request, deletion, non-fast-forward, review-thread, and no-bypass rules;
- Draft PR #63 remains the only active 2.x migration PR, and R1 continues in
  that same branch.

Deliverables:

- approved acceleration design and this authoritative roadmap;
- one rolling branch and Draft PR;
- native-only required 2.x CI;
- legacy compatibility workflow moved out of branch protection;
- Draft PR capability/commit/next-batch/nonclaim ledger.

Exit gate:

- all required checks operate without the full Python or Node suites;
- exact-head native Linux, macOS, and Windows jobs pass;
- no second active 2.x migration PR exists.

## R1 — Content And Config Foundation

Purpose: make the native binary self-contained and safe to configure.

Implementation status on July 14, 2026:

- FND-202E is complete through implementation checkpoint `bd11896e` and
  portability follow-ups `e3c3f93e` and `870d85b8`;
- the service accepts only an already verified in-memory pack plus a target
  capability created by the private temporary factory or an explicitly named
  trusted CLI/UI/installer approval boundary;
- temporary targets live in atomically created private containers, while
  explicit Unix targets reject group- or world-writable ancestor chains;
- managed receipts, bounded canonical-source/profile validation, private
  staging, logical modes, link/reparse rejection, lock identity, managed-tree
  drift checks, sibling promotion, rollback, and distinct post-commit cleanup
  errors are covered by focused tests;
- the final local gate passed format, workspace check, Clippy with warnings
  denied, and 48 Rust tests;
- GitHub Actions run `29291560721` passed the native boundary plus Linux,
  macOS, and Windows Rust jobs at exact code head
  `870d85b8f0ac5f57311292d06b7278441eb9d3f7`;
- two review passes closed the shared-temp, staging-permission, lock-identity,
  and post-commit ambiguity findings, with no remaining Critical or Important
  blocker inside the declared FND-202E threat model;
- public CLI/UI/MCP wiring, Windows ACL and hard-link hardening, and adversarial
  same-user handle-relative filesystem operations remain explicit successor
  work rather than FND-202E claims;
- FND-202F now builds the canonical 418-entry content tree during the Rust
  application build, verifies it against committed content-root and whole-pack
  SHA-256 identities, and embeds only the verified bytes and expected digest;
- the lock binds academic content version `v1.19.0-beta.1` and the complete
  Contract v2 source tree at canonical content commit
  `ff2c4f35cd1ee5df78a04ff90a0325273917eed8`, content root
  `7a56401cb8208ab53483631e72cbcdc5b37c20a32e46eebfc9e8e2b219352f69`,
  and pack digest
  `38b9b1342a6a699b64d416a14ae25736750c166c2f502e6402c0c4418a6dc00d`;
- `EmbeddedContent` supplies profile list/read/materialize services, while the
  application validates the pack at startup; a copied-binary test runs outside
  the checkout with an empty `PATH`;
- the FND-202F local gate passed format, workspace check, Clippy with warnings
  denied, and 52 Rust tests. Review caught and closed an initial source-commit
  provenance mismatch, then reported no remaining Critical or Important
  blocker;
- GitHub Actions run `29292694823` passed the native boundary plus Linux,
  macOS, and Windows Rust jobs at exact implementation head
  `76ee339f2c21ac1139ff422392778f3fb0857598` in 7s, 29s, 33s, and 1m3s.
- CFG-201A spans implementation commits `60886b33`, `ed3e3384`, `ebf36bd4`,
  and `bb3537e4`, plus portability fixes `31beab11` and `588e564d`: it adds a
  versioned global root, exact typed settings, opaque secret references,
  strict bounded JSON, redacted status, revision conflicts, and
  recovery-aware persistence;
- the local CFG-201A gate passed the native change boundary, format, locked
  workspace check, Clippy with warnings denied, and 84 Rust tests using the
  required native-only commands; no Python or Node suite was run or required;
- the Unix adapter implements owner-only directories/files, a bounded lock,
  synchronized staging and recovery files, atomic activation, rollback, and
  explicit post-commit cleanup state. The local macOS execution covered the
  permission, conflict, concurrency, lock-timeout, and fault-injection matrix;
- CFG-201B design commit `974e539b`, isolated adapter commit `1c3df663`, and
  shared-transaction implementation commit `90190612` add supported Windows
  persistence without weakening `qiongli-config`'s unsafe-code prohibition;
- the isolated adapter creates and verifies protected current-user-only DACLs,
  rejects reparse points and hard-linked managed files, exposes handle identity,
  and performs write-through same-volume replacement;
- Windows now runs the same bounded lock, optimistic revision, synchronized
  staging/recovery, activation, verification, cleanup, and rollback state
  machine as Unix. Windows uses `MoveFileExW` write-through semantics instead
  of claiming an undocumented directory-fsync guarantee;
- GitHub Actions run `29318765759` passed the native boundary plus Linux,
  macOS, and Windows Rust jobs at exact implementation head
  `588e564d73b1afa367c7d7ee4d539c55f893e368` in 7s, 32s, 46s, and 57s;
- the CFG-201B local gate passed the native boundary, format, locked workspace
  check, Clippy with warnings denied, and all 84 Rust tests. A local
  `x86_64-pc-windows-msvc` workspace check and Clippy pass also type-checked
  every Windows-only production and test target before push;
- GitHub Actions run `29321393463` passed the native boundary plus Linux,
  macOS, and real Windows Rust jobs at exact implementation head
  `90190612d2ba66b93ae98c536d688ba86ab78b34` in 6s, 31s, 49s, and 1m29s;
- the Windows runner executed first/replacement writes, DACL checks, reparse
  and hard-link rejection, stale/concurrent writers, lock timeout, redaction,
  pre/post-activation failure injection, cleanup, and rollback-failure tests;
- credentials remain opaque `SecretRef` values. `UnavailableSecretStore` is
  the only secret-store implementation, so this checkpoint does not claim a
  keychain, credential vault, or plaintext fallback;
- R1 command design commit `45a1cc87` and implementation checkpoint
  `f2a6fbe6` compose the accepted content and config services without adding a
  second service implementation;
- the canonical binary now exposes exact help/version, content list and
  explicitly approved materialization, redacted config show and revision-safe
  default-profile set, combined status, and foundation doctor commands;
- every successful data command emits schema-version-1 JSON. Usage failures
  return 2, operation failures return 1, and doctor returns 1 only for a
  blocking config state;
- public errors use static or allowlisted reason codes. CLI tests prove that
  materialization/config paths, environment values, provider email, malformed
  document bytes, and private arguments do not enter stdout or stderr;
- config set preserves every provider field and delegates optimistic writes to
  the accepted cross-platform store. It accepts no secret or provider value;
- the copied binary lists and materializes the embedded pack from outside the
  checkout with an empty `PATH`, while failed target and stale config writes
  preserve prior bytes;
- the local R1 gate passed the native boundary, format, locked workspace check,
  Clippy with warnings denied, and all 92 Rust tests. The Windows MSVC
  cross-target workspace check and Clippy also passed;
- GitHub Actions run `29323018180` passed the boundary, Linux, macOS, and real
  Windows jobs on exact implementation head
  `f2a6fbe608c234219526c8947f0cb50470ac1482` in 4s, 37s, 40s, and 1m19s;
- no Python or Node suite was run or required. R1 does not implement provider
  configuration, credentials, MCP, project state, 1.x migration, agents,
  orchestration, desktop UI, host installation, or release packaging.

Deliverables:

1. FND-202E atomic materializer — complete through `870d85b8`:
   - temporary or explicitly approved output root;
   - managed receipt;
   - traversal and symlink rejection;
   - atomic commit and rollback;
   - no arbitrary output path from untrusted MCP input.
2. FND-202F embedding and drift closure — complete at `76ee339f`:
   - reproducible build-time pack;
   - embedded expected digest;
   - profile list/read/materialize API;
   - source-drift guard owned by native tooling.
3. `qiongli-config` cross-platform vertical slice — complete at `90190612`:
   - `QIONGLI_CONFIG_HOME/v2/` resolution;
   - typed public settings and profile selection;
   - provider settings and secret references;
   - atomic owner-only Unix and Windows writes and redacted diagnostics;
   - an isolated safe Win32 security boundary with protected DACL, identity,
     reparse, hard-link, and write-through replacement checks.
4. Native commands — complete at `f2a6fbe6`:
   - version/help;
   - content list/materialize;
   - config show/set;
   - status/doctor foundation.

Exit gate result: passed on implementation head `f2a6fbe6` in Native CI run
`29323018180`.

- the application starts with an empty `PATH`;
- embedded content is usable without a source checkout;
- failed materialization or config writes leave prior state intact;
- the current native workspace passes the Tier 1 build/test matrix.

## R2 — Shared Lite Runtime

Purpose: move the already working Rust Lite value into the canonical native
workspace instead of reimplementing it.

Implementation status: not started. The next dependency-contiguous batch will
freeze the `qiongli-runtime` contract/error boundary and map reusable Lite
provider, evidence, Zotero, framing, and dispatch code before moving behavior.
It will not claim native MCP availability until binary-level protocol tests
pass.

Deliverables:

- `qiongli-runtime` contract and error foundation;
- provider config/status and bounded search;
- evidence export and supported Zotero operations;
- stdio JSON-RPC/MCP framing;
- Contract v2 Lite profile dispatch;
- native MCP mode in the canonical `qiongli` executable;
- old `qiongli-lite-mcp` reduced toward a thin compatibility wrapper.

Exit gate:

- advertised Lite tools answer initialize, tools/list, and bounded safe calls;
- no Python or Node process appears in the production process tree;
- provider unavailability, timeout, cancellation, and credentials remain typed
  and redacted;
- old and new Rust entrypoints do not contain divergent provider logic.

## R3 — Installable Native Alpha.1

Purpose: turn the native services into a usable local product.

Deliverables:

1. `qiongli-platform`:
   - typed target discovery;
   - `InstallPlan` preview/apply/verify/repair/remove/rollback;
   - managed ownership markers;
   - supported Codex local adapter;
   - supported Claude local adapter;
   - doctor and integration status.
2. `qiongli-ui`:
   - egui/eframe shell;
   - Skills, MCP, Providers, Integrations, and Diagnostics views;
   - typed service intents and events;
   - no business logic or direct side effects in UI callbacks.
3. `apps/qiongli`:
   - CLI, desktop UI, MCP, and internal worker modes over shared services;
   - current-target build and artifact identity;
   - truthful alpha capability output.
4. Alpha acceptance:
   - clean-machine startup;
   - embedded skill inspection/materialization;
   - provider configuration and Lite MCP launch;
   - one Codex and one Claude install/diagnose/remove journey;
   - zero user language-runtime dependency.

Exit gate:

- all advertised alpha.1 journeys pass on their exact target artifact;
- rollback removes only Qiongli-managed state;
- release notes identify unsupported Full/agent/orchestrator behavior;
- the rolling Draft PR is green and may become Ready.

## R4 — Full Native Runtime And Alpha.2

Purpose: complete the Full service and execution layer.

Deliverables:

- project, subject, guidance, experience, lifecycle, and journal-fit services;
- Full read/config/write MCP families;
- `AgentBackend` and at least one direct API adapter;
- native ToolHost with project/path/tool/approval/limit/redaction/audit policy;
- task DAG, solo/duo/triad, worker, synthesis, reviewer/verifier, artifact, and
  quality-gate orchestration;
- CLI, UI, and Full MCP access to the same execution services;
- updater foundation.

Exit gate:

- at least one direct backend completes a bounded workflow with no external
  agent CLI;
- Full production paths invoke no Python or Node;
- unavailable backends and cancelled runs produce structured recovery state;
- `v2.0.0-alpha.2` claims only verified backends and surfaces.

## R5 — Native Cutover And Beta.1

Purpose: remove production legacy dependencies and qualify the native product.

Deliverables:

- copy-on-migrate 1.x state import and rollback;
- complete macOS arm64, Windows x86_64, and Linux x86_64 artifacts;
- signing/notarization where required;
- checksums, SBOM, provenance, and target identity;
- signed update metadata and atomic rollback;
- clean-machine install/upgrade/repair/remove acceptance;
- Python and Node production invocation removal;
- final legacy disposition and support communication.

Exit gate:

- every advertised product path runs without Rust, Python, or Node;
- migration is idempotent and failed migration restores prior usable state;
- every advertised target has native startup and installer receipts;
- no open P0/P1 security, data-loss, migration, installer, or release defect;
- `v2.0.0-beta.1` promotion evidence is complete.

## Stable Promotion

Stable `v2.0.0` follows observed beta hardening rather than another migration
inventory program. It requires reproducible signed artifacts, tested recovery,
accessibility and performance acceptance, no unresolved P0/P1 defects, and
clear 1.x end-of-support communication.

## Draft PR Ledger

The active PR records:

| Section | Required content |
|---|---|
| Current capabilities | What the exact head can run now |
| Checkpoints | Integrated commits and current-head Rust evidence |
| Next batch | The next dependency-contiguous work, not every future task |
| Nonclaims | Capabilities, targets, integrations, and release properties not yet supported |

The PR body is updated after every pushed checkpoint. Test evidence from a
superseded head is not reported as current-head evidence.

## Immediate Sequence

1. acceleration design, authoritative roadmap, and Draft PR #63: complete;
2. R0 native required CI and live ruleset narrowing: complete;
3. FND-202E and FND-202F are complete in the same rolling PR;
4. CFG-201A and CFG-201B are complete; continue with the R1 native-command
   slice in the same rolling Draft PR;
5. continue into R2 without creating another branch or PR;
6. continue into R3 and prepare alpha.1 only after the complete vertical gate.

## Program Done

The Rust migration is complete when:

- CLI, desktop UI, Lite/Full MCP, providers, skills, agents, orchestrator,
  installer, updater, and supported local integrations use shared Rust services;
- production artifacts contain no Python/Node runtime launch path;
- Tier 1 artifacts install and run on clean machines without language runtimes;
- state migration and rollback are safe and documented;
- supported Codex and Claude surfaces have real activation evidence;
- beta gates pass and the native product can progress through hardening to
  stable without reopening 1.x feature development.
