# Qiongli 2 Accelerated Rust Migration Roadmap

Status: active execution; R1, R2, and R3A-R3M implementation are complete;
Lite Alpha.1 publication remains blocked by recorded external gates

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
`qiongli-content`, `qiongli-config`, `qiongli-runtime`, `qiongli-platform`,
`qiongli-ui`, and the isolated `qiongli-windows-security` FFI boundary.
FND-202E is complete through portability head `870d85b8`, FND-202F is complete
at `76ee339f`, CFG-201A ends at `588e564d`, and CFG-201B ends at implementation
checkpoint `90190612` on
the same rolling branch. R1 native command composition ends at implementation
checkpoint `f2a6fbe6`. The first shared Lite runtime extraction is complete at
`d7f2d64f`. Work continues through provider/domain behavior rather than
returning to another legacy inventory or Python parity phase.

R3A reaches its implementation checkpoint at `60c2ddc5`. It defines the
signed Lite launch-grant and declarative install-plan trust boundary and adds
truthful read-only `qiongli install status`; it does not yet install, mutate,
discover, register, activate, package, or release anything.

R3B reaches its accepted implementation checkpoint at `b3a6ea6b`. It adds the
first verified-plan and exact-approval-gated managed resource transaction
lifecycle, canonical receipts/state, a root-scoped recovery journal, and safe
apply/verify/repair/remove/rollback behavior. Exact implementation-and-local-
receipt head `6fdbcde5` passed Native CI and Cloudflare; the source binary still
exposes no executable install command.

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

Implementation status: in progress. R2A, R2B, R2C, and R2D are complete. R2A
closed on July 14, 2026:

- design checkpoint `1e732155` freezes the first extraction boundary and
  explicit nonclaims;
- implementation checkpoint `d7f2d64f` adds `qiongli-runtime` with typed,
  path/input-free errors and an optional verified-embedded-content adapter;
- the strict parser accepts at most 1 MiB and requires the exact Contract v2
  Lite schema version, 12 public definitions in frozen order, and 11 canonical
  typed identities. `qiongli_open_config_wizard` resolves to the canonical
  configure-provider identity;
- newline and Content-Length stdio framing now have one shared implementation
  with 8 MiB message and 64 KiB header bounds plus typed UTF-8, incomplete,
  input, output, and serialization failures;
- the canonical app proves that the registry loads from the already verified
  `marketplace-lite` embedded profile without a loose-file fallback;
- the old Rust Lite package now uses thin definitions/protocol adapters and the
  shared typed handler resolver. Its duplicate name table and local
  framing/contract parsers were removed;
- unknown Lite JSON-RPC method and tool names now return static errors without
  echoing attacker-controlled names;
- the local gate passed the native boundary, format, locked workspace check,
  strict Clippy, all 104 native Rust tests, Windows MSVC cross-target check and
  strict Clippy, plus 15 focused old Lite protocol/server tests;
- Native CI run `29324291281` passed the exact implementation head
  `d7f2d64f5028bc909f4055834e8644077501752e`: boundary in 7s, focused Lite
  compatibility in 33s, Linux in 36s, macOS in 41s, and real Windows in 1m17s;
- no Python or Node suite ran or became required. R2A does not expose an MCP
  command in the canonical executable and does not migrate provider, evidence,
  Zotero, route-preview, or task-plan behavior.

R2A deliberately left provider behavior in the compatibility package. R2B
below closes that next dependency-contiguous boundary without opening native
MCP availability.

R2B is complete on July 14, 2026:

- design checkpoint `a7a505eb` freezes the native-config authority, redacted
  access/status model, request and HTTP bounds, cooperative cancellation,
  compatibility extraction, and explicit nonclaims;
- implementation checkpoint `2eaadfb1` adds canonical identities and aliases
  for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv plus
  non-serializable zeroizing access and redacted readiness;
- native global settings adapt through the injected `SecretStore` only. Missing
  secrets and unavailable secure storage remain distinct, and no plaintext or
  process-environment fallback was added to the native path;
- canonical search requests cap UTF-8 query bytes at 4,096, per-provider
  results at 200, and total results at 1,000 before runtime construction;
- the shared client keeps fixed production endpoints, disables redirects and
  implicit proxy discovery, uses 3-second connect and 15-second request
  timeouts, and reads at most 4 MiB. Non-Windows uses bundled Rustls roots and
  Windows uses operating-system SChannel without a user-installed runtime;
- the five provider clients, response normalization, concurrent fan-out,
  canonical output order, partial-failure diagnostics, DOI/title-year
  deduplication, final limiting, and deterministic search planning now have one
  Rust owner in `qiongli-runtime`;
- cancellation is typed and checked before/after request boundaries and
  between PubMed's calls. Blocking HTTP already in flight is not claimed to be
  immediately interruptible and remains bounded by the request timeout;
- the old Lite provider/search/search-plan modules are re-exports or thin
  constructors. Legacy file/environment config resolves only at that
  compatibility edge before conversion to shared access;
- the local gate passed format, boundary, locked native check, strict native
  and Lite Clippy, all 116 native tests, 57 focused Lite compatibility tests,
  and Windows MSVC cross-target check/Clippy;
- Native CI run `29326303112` passed exact implementation head
  `2eaadfb137bb26a71ab578f0c2bcdf6dc140a869`: boundary in 5s, focused Lite in
  35s, Linux in 1m02s, macOS in 1m28s, and real Windows in 1m53s; and
- no live provider, Python, or Node suite ran. R2B does not expose provider or
  MCP commands in the canonical executable and does not implement production
  secure storage, evidence, Zotero, orchestration, UI, installation, or
  packaging.

R2C is complete on July 14, 2026:

- design checkpoint `68558ed0` freezes the side-effect-free evidence snapshot,
  read-only loopback probe, in-memory import-file export, resource bounds,
  compatibility edge, and explicit nonclaims;
- implementation checkpoint `2513c52f` adds typed evidence input with
  canonical/alias ambiguity rejection, 4,096-byte queries, 1,000 results,
  2 MiB serialized input, 32 container levels, and 100,000 JSON values;
- evidence snapshots remove credential-bearing keys recursively, retain benign
  evidence fields such as `token_budget` and `public_key`, omit compatibility
  paths and wall-clock time, and expose only static validation errors;
- the shared Zotero exporter validates at most 1,000 normalized records,
  unique exact format selection, 2 MiB input, bounded fields and provider
  metadata, and a 2 MiB combined output before returning deterministic
  CSL-JSON, RIS, BibTeX, and import-report contents in memory;
- RIS folds control and Unicode line-separator characters, while BibTeX escapes
  syntax-bearing input so record text cannot inject fields or entries;
- the shared Zotero probe accepts only bounded HTTP(S) loopback roots, strips
  path/query/fragment input, calls fixed connector/companion paths, disables
  redirects and implicit proxy discovery, bounds production requests to five
  seconds and companion bodies to 32 KiB, and never returns URL/body/library
  content;
- the old Lite evidence handler now delegates to the shared snapshot builder;
  its Zotero exporter is a re-export, and its companion module retains only
  the historical environment-to-client adapter beside shared re-exports;
- the local gate passed boundary, native and Lite format, locked native check,
  strict native and Lite Clippy, all 131 native tests, all 67 focused Lite
  compatibility tests, and Windows MSVC cross-target check/Clippy;
- Native CI run `29327768079` passed exact implementation head
  `2513c52f9a39cb97a6d41f282dcbdff920eac979`: boundary in 4s, focused Lite in
  32s, Linux in 1m08s, macOS in 1m39s, and real Windows in 1m53s; and
- no live Zotero/provider, Python, or Node suite ran. R2C does not write files,
  mutate/search a Zotero library, install the companion, expose a native Zotero
  settings surface, or add MCP mode to the canonical executable.

R2D is complete on July 14, 2026:

- design checkpoint `bb6d2b08` freezes the bounded pure-preview contract,
  typed dispatch projection, compatibility command meaning, privacy boundary,
  verification, and explicit nonclaims;
- implementation checkpoint `5509d2c1` gives all 11 canonical Lite identities
  one exhaustive config, literature, Zotero, or orchestration handler target;
- the configure-provider compatibility alias still resolves before dispatch,
  and handler selection performs no string matching after typed resolution;
- shared route input accepts a required nonblank request of at most 4,096
  bytes plus the exact optional Contract v2 platform enum; shared task input
  accepts required task ID and paper type values of at most 256 bytes plus a
  topic of at most 4,096 bytes, preserving current task-field trimming;
- route/task outputs deterministically retain the accepted Marketplace Lite
  preview flags, keep agent/shell/project-write permissions false, and retain
  the 1.x Full-runtime command only as compatibility guidance rather than
  native 2.x capability discovery;
- runtime validation rejects non-object, unknown, missing, mistyped, blank,
  unsupported-platform, and oversized input with static canary-free errors;
- the old Lite preview module is a shared-runtime re-export and its server now
  dispatches every tool through the shared typed domain projection before
  wrapping results in JSON-RPC;
- the local gate passed boundary, native and Lite format, locked native check,
  strict native and Lite Clippy, all 138 native tests, all 69 focused Lite
  compatibility tests, and Windows MSVC cross-target check/Clippy;
- Native CI run `29328767532` passed exact implementation head
  `5509d2c146e65265e1f47cc9b4badb3f258325c9`: boundary in 4s, focused Lite in
  33s, Linux in 52s, macOS in 1m22s, and real Windows in 1m59s; and
- no Python/Node suite, live provider/Zotero service, filesystem write,
  process, shell, environment lookup, or agent backend was added or required.
  The canonical executable still exposes no MCP mode.

R2E and the R2 native Lite vertical are complete on July 14, 2026:

- design checkpoint `51887f4c` freezes the closed Lite/stdio command, shared
  protocol and handler ownership, unavailable-safe credential boundary,
  copied-binary gate, release nonclaims, and R3 handoff;
- implementation checkpoint `fce20f46` exposes
  `qiongli mcp serve --profile lite --transport stdio` from the canonical
  executable, with `marketplace-lite` as the only profile alias and no Full or
  alternate transport escalation;
- one shared runtime server owns bounded line and `Content-Length` framing,
  JSON-RPC/MCP lifecycle methods and notifications, the embedded 12-name Lite
  registry, typed domain dispatch, and bounded output/error redaction;
- native config and literature status use the shared provider-access model and
  the opaque `<managed-native-config>` identifier; secret references resolve
  only through `SecretStore`, with no environment or 1.x plaintext fallback;
- valid save-provider and wizard calls are validated and return fixed
  unavailable tool errors without writing config, starting a listener,
  launching a browser, or echoing the supplied value;
- shared search-plan and literature-search parsers now serve both Rust
  entrypoints; the old Lite server removed 408 lines and added 20, a 388-line
  net reduction in duplicate compatibility-server code;
- the copied canonical binary passed initialize, exact 12-name tools/list,
  bounded line-framed domain calls, notification suppression, secret/path
  redaction, and EOF with an empty `PATH`; direct shared-server tests passed
  `Content-Length` framing;
- the local gate passed boundary, native and Lite format, locked native check,
  strict native and Lite Clippy, all 148 native tests, all 69 focused Lite
  compatibility tests, and Windows MSVC cross-target check/Clippy;
- Native CI run `29330582098` passed exact implementation head
  `fce20f469d6cc691dcd9ea74d822a8dcf75bdb38`: boundary in 5s, focused Lite in
  38s, Linux in 1m08s, macOS in 1m08s, and real Windows in 2m00s; and
- no Python/Node suite, live provider/Zotero service, config mutation,
  loopback listener, process, browser, UI, installer, packaging, launch grant,
  Marketplace activation, Full MCP, or agent backend was added or required.

R2 is therefore complete for the development vertical. This does not make the
binary an installable alpha or a release plugin. Production secure-store
mutation, signed artifact-bound launch grants, target packaging, host
installation/repair, and clean-machine installed-product proof remain R3.

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

R3A is complete on July 14, 2026:

- design checkpoint `60c50526` freezes exact Lite artifact identity, injected
  Ed25519 trust, deterministic typed plans, bounded parsing, and nonclaims;
- implementation checkpoint `60c2ddc5` adds `qiongli-platform`, strict signed
  grant verification, private verified tokens, target-matched invertible plan
  contracts, semantic digests, and closed reason-coded failures;
- the canonical source binary exposes read-only `qiongli install status` and
  truthfully reports launch grant, preview, and apply as unavailable;
- local gates passed the native boundary, format, locked check, strict Clippy,
  all 161 native Rust tests, and Windows MSVC cross-target check/Clippy;
- Native CI run `29332864357` passed exact implementation-and-local-receipt
  head `a971ae1e`: boundary in 8s, Lite compatibility in 37s, Linux in 1m08s,
  macOS in 1m05s, and real Windows in 2m19s; Cloudflare Pages also passed; and
- no production signing key, signed release artifact, filesystem executor,
  receipt, adapter, real client activation, Marketplace/Desktop install,
  package, or release exists yet.

R3B is complete at design checkpoint `714315cd` and implementation checkpoint
`b3a6ea6b`:

- one exact Marketplace Lite resource plan can materialize below an explicitly
  approved owner-only `QiongliManagedData` root only after exact grant, plan,
  payload, root, and approval validation;
- canonical active and lifecycle receipts support fresh apply, exact replay,
  read-only verify, absent-target repair, remove, and rollback;
- a root-scoped journal serializes distinct install IDs, and UID/DACL, root and
  target identity, no-replace rename, quarantine, and post-rename ambiguity
  handling fail closed without deleting uncertain data;
- local gates passed the native boundary, format, locked workspace check,
  strict Clippy, all 177 native Rust tests, and Windows MSVC cross-target
  workspace check/strict Clippy;
- Native CI run `29335762713` passed exact implementation-and-local-receipt
  head `6fdbcde5`: boundary in 4s, focused Lite in 37s, Linux in 1m15s, macOS
  in 1m24s, and real Windows in 2m22s; Cloudflare Pages also passed; and
- multi-operation plans, managed replacement/upgrade, plugin/MCP registration,
  host actions, nested destinations, client paths, production grants, packages,
  and release claims remain rejected or unavailable.

R3C is complete at design checkpoint `8c76f5f2` and implementation checkpoint
`f0455ad8`. It implements the first `INT-201` Codex local adapter vertical with
documented current-user discovery, read-only preview, exact client-config
approval, and receipt-backed registration/removal. It must not write Codex
plugin caches or claim
Desktop/Marketplace activation; real client activation evidence remains a
separate exit gate. Claude Code and Claude Desktop remain `INT-202`/`INT-203`
successor work.

The R3C boundary is frozen in
`docs/superpowers/specs/2026-07-14-qiongli-r3c-codex-local-adapter-design.md`.
It uses the documented personal marketplace at
`~/.agents/plugins/marketplace.json`, a receipt-verified fixed source below
`~/.qiongli/plugins/codex`, and an exact approval-gated merge transaction.
Codex-owned cache and enablement state remain out of scope; registration means
only that the desktop client can discover the local source for a later user
install.

R3C acceptance evidence:

- the canonical embedded pack contains a receipt-covered skills-only Codex
  manifest and has 419 entries;
- `qiongli-platform` implements redacted discovery, deterministic preview,
  exact approval, canonical registration/lifecycle receipts, root-scoped
  journaling, and apply/verify/repair/remove/rollback;
- marketplace merges preserve unrelated fields and entries and reject
  unreceipted conflicts, drift, links, unsafe permissions, oversized input,
  partial approval, and ambiguous recovery;
- `qiongli install codex status` is read-only and the source build continues to
  report production grant, preview, and apply as unavailable; and
- the native boundary, strict Clippy, all 185 native Rust tests, all 69 focused
  Lite compatibility tests, and the Windows MSVC all-target workspace check
  pass locally; and
- exact-head Native CI run `29338915660` passed `f0455ad8`: boundary in 5s,
  focused Lite in 41s, Linux in 1m15s, macOS in 2m13s, and real Windows in
  2m19s. Cloudflare Pages also passed.

R3D is complete at design checkpoint `e2e5c814`, implementation checkpoint
`468f458e`, and accepted portability head `103fe11d`. It composes the target
native binary and native Lite MCP declaration into a complete Codex plugin
layout, binds package identity to the signed artifact and embedded content
receipts, and proves a real clean-client install/enable/runtime path. Client
cache and enablement writes remain client-owned; public Marketplace and cloud
availability are not claimed.

The R3D boundary is frozen in
`docs/superpowers/specs/2026-07-14-qiongli-r3d-codex-native-plugin-design.md`.
The generated target-specific package uses one native Qiongli executable for
CLI and Lite MCP, a canonical `skills/qiongli-workflow` projection, a strict
bundle receipt, and client-owned activation. Real Codex evidence must use an
isolated home and must not modify the developer's normal cache or enablement
state.

R3D is complete. The platform now
builds and verifies a target-specific, receipt-covered package containing the
canonical skill projection, one native Qiongli executable, and a root MCP
declaration. R3C discovery accepts only that complete package and binds its
registration state to the package receipt and content root.

Local R3D acceptance proves:

- deterministic composition, private staging, locking, no-replace promotion,
  complete-tree verification, and rejection of target, lock, binary, mode,
  hard-link, receipt, extra-file, and content drift;
- the packaged binary serves all 12 Lite MCP tools with an empty `PATH`;
- Plugin Creator validates the generated package; and
- Codex CLI `0.144.1` installs, lists, enables, caches, and launches the package
  in an isolated client home without modifying normal Codex state.

The local gate passes strict host and Windows MSVC Clippy, all 187 normal native
Rust tests, and all 69 focused Lite compatibility tests. The external Codex
test passes explicitly and remains ignored in the default workspace gate.
Production grants, public Marketplace publication, cloud/web execution,
package upgrade/rollback, release artifacts, and Claude/UI/Full-runtime work
remain unavailable.

Remote R3D acceptance passed at
`103fe11dc5ada2f4e34a6c79a367f9e27a82ad5e`: Native CI run `29342795330`
passed the boundary in 6s, focused Lite in 36s, Linux in 1m33s, macOS in
2m17s, and Windows in 2m44s; Cloudflare Pages also passed. The Windows gate
includes the complete bundle and secure owner-only operations beyond the
legacy 260-character path limit.

R3E / `INT-202A` is complete at design checkpoint `461c4839` and accepted
implementation head `337cce74`. It adds a target-native, receipt-covered Claude
Code plugin package, exact direct skills-directory discovery and removal, and a
Qiongli-owned local marketplace lifecycle over the existing signed artifact,
embedded content, install-plan, approval, and transaction boundaries.

Local R3E acceptance proves:

- the verified embedded pack contains 420 entries including the canonical
  Claude manifest, while Codex and Claude composers each exclude the other
  host's manifest from the generated skill payload;
- direct discovery accepts only an exact verified package and direct removal
  preserves unmanaged or drifted paths;
- the local marketplace adapter supports preview, apply, verify, absent-entry
  repair, remove, and rollback without writing Claude-owned settings, registry,
  cache, or enablement state;
- all 199 normal native Rust tests, all 69 focused Lite compatibility tests,
  strict host Clippy, and Windows MSVC workspace check/Clippy pass; and
- Claude Code `2.1.206` strictly validates and discovers `qiongli@skills-dir`,
  adds and installs the isolated local marketplace package, verifies the copied
  cache, serves all 12 Lite MCP tools with an empty `PATH`, and removes the
  isolated plugin and marketplace.

Remote R3E acceptance passed at
`337cce741ee2837165331a9a3cd5a53d2e7bf245`: Native CI run `29345585219`
passed the boundary in 5s, focused Lite in 38s, Linux in 2m14s, Windows in
2m30s, and macOS in 2m42s; Cloudflare Pages also passed.

R3E does not add production grants or mutating install commands and does not
claim Claude Desktop, cloud/web, public marketplace, managed upgrade, release,
UI, Full MCP, agent, or orchestrator completion. `INT-203` and remote service
support remain separate gates.

R3F / `UI-201` plus the read-only portion of `UI-202` is complete at design
checkpoint `59f968c6`, implementation checkpoint `66398b2c`, and accepted
implementation-and-CI head `4d706033`. The canonical executable now exposes
`qiongli ui` with Overview, Skills, MCP, Providers, Integrations, and
Diagnostics views over a bounded redacted snapshot.

Local R3F acceptance proves:

- `qiongli-ui` depends only on eframe, egui, and zeroize and has no concrete
  config, content, platform, runtime, filesystem, process, or network service;
- all eight headless AccessKit tests pass across six views, keyboard
  activation, labelled transient input, preview confirm/cancel, recovery,
  680/1080 widths, and 100/150/200 percent scale;
- the real adapter reads verified embedded content, redacted provider/config
  state, the 12-tool Lite contract, and Codex/Claude Code discovery without
  creating host or Qiongli state;
- invalid initial snapshots are quarantined without rendering their dynamic
  text, while production apply and install confirmation remain unavailable;
- all 210 normal native Rust tests pass with two external-client tests ignored,
  and strict host and Windows MSVC Clippy pass; and
- the test profile strips debug information so the unified GUI/CLI/MCP test
  binary stays inside the unchanged 128 MiB plugin-composer input bound.

The initial implementation-head Native CI run `29349701112` found the Linux
debug test binary above that bound. The build-only fix at `4d706033` retained
the production limit. Exact-head Native CI run `29351331008` then passed the
boundary in 6s, focused Lite in 39s, macOS in 4m45s, Linux in 4m51s, and
Windows in 6m16s; Cloudflare Pages also passed.

R3F does not claim packaged-window startup, manual target screen-reader
acceptance, config or secret writes, UI-launched MCP, plugin mutation, signing,
updater, clean-machine release, Claude Desktop, cloud/web, Full MCP, agents, or
orchestrator completion.

R3G / `PKG-201A` is complete at design checkpoint `0b895a88`, implementation
checkpoint `df493e13`, and accepted implementation-and-CI head `7fddca15`. The
platform now assembles one current-target `lite` plus `portable-archive`
staging tree around the canonical binary and binds its complete target,
version, channel, profile, package, binary, and verified resource-pack identity
in canonical metadata with explicit `assembled-unpublished` status.

Local R3G acceptance proves deterministic composition, private no-replace
staging, bounded source validation, complete-tree and external resource-pack
verification, fixed path-redacted failures, and tamper/conflict rejection. The
committed artifact-local binary runs `--version`, verified content listing,
MCP initialization, the exact 12-tool Lite surface, and one bounded read-only
tool call from outside the checkout with an empty runtime `PATH`. All 215
normal native tests pass, two external-client tests remain explicitly ignored,
all 69 focused Lite compatibility tests pass, and strict host and Windows MSVC
checks and Clippy pass.

Initial Native CI run `29353736596` passed every job except the new Windows
R3G tests, whose fixture used a default-DACL parent. `7fddca15` switched that
test fixture to the existing owner-only Windows helper without weakening the
production validator. Exact-head Native CI run `29354332680` then passed the
boundary in 7s, focused Lite in 32s, Linux in 5m22s, macOS in 6m34s, and
Windows in 6m59s; Cloudflare Pages also passed.

R3G does not claim a final compressed archive, signing, notarization, public
release, updater, service-backed install mutation, packaged-window startup,
cross-target packaging, or clean-machine desktop acceptance.

R3H / `PKG-201B` is complete at design checkpoint `dad77e66` and accepted
implementation head `f1e50074`. The platform now composes the verified R3G
staging tree into one canonical, deterministic, store-only ZIP and accepts only
its exact four-entry profile. Composition and verification bind the archive to
the complete artifact identity, external verified resource pack, canonical R3G
manifest, binary digest, logical modes, CRC values, offsets, and fixed ZIP
metadata.

Local R3H acceptance proves byte-for-byte deterministic output, bounded strict
parsing, structural and payload tamper rejection, owner-private create-new
staging, target locking, persistence, no-replace promotion, fixed-path safe
extraction through the R3G commit path, and cleanup without adopting existing
caller data. The extracted artifact-local executable runs `--version`, content
inspection, MCP initialization, the exact 12-tool Lite contract, and one
bounded read-only call from outside the checkout with an empty runtime `PATH`.
All 220 normal native tests pass, the two external-client tests remain
explicitly ignored, all 50 platform tests pass, all 69 focused Lite tests pass,
and strict host and Windows MSVC checks and Clippy pass.

Exact-head Native CI run `29357292961` passed the boundary in 7s, focused Lite
in 37s, Windows in 6m33s, Linux in 7m12s, and macOS in 9m36s; Cloudflare Pages
also passed. R3H remains unsigned and unpublished and does not claim signing,
notarization, checksums, SBOM, provenance, launch grants, an installer, updater,
public distribution, packaged-window startup, cross-target output, or
clean-machine acceptance.

R3I / the remaining `PLT-202` native-payload vertical is complete at design
checkpoint `33e331c9` and accepted implementation head `25335d43`. The platform
now binds one verified current-target R3H archive to an R3A signed launch grant,
deterministic single-operation install plan, explicit filesystem approval, and
caller-approved private `QiongliManagedData` root. Archive-derived install
identity prevents different archives from sharing state, while the installed
directory remains the canonical artifact ID.

The shared service implements apply, identical replay, read-only verify,
absent-target repair, verified quarantine, remove, rollback, terminal
idempotence, restoration before the durable state point, and retained recovery
evidence for ambiguous outcomes. Strict bounded canonical state contains no
absolute path. Foreign destinations, linked state, payload drift, archive/pack
mismatch, and conflicting journals fail closed without adoption or overwrite.
The installed executable runs `--version`, content inspection, MCP initialize,
the exact 12-tool Lite contract, and one bounded read-only call outside the
source and archive trees with an empty runtime `PATH`.

Local R3I acceptance passes all 224 normal native tests, keeps the two
external-client tests explicitly ignored, passes all 54 platform tests and all
69 focused Lite compatibility tests, and passes strict host and Windows MSVC
checks and Clippy. Exact-head Native CI run `29361710636` passed the boundary
in 4s, focused Lite in 35s, Linux in 7m33s, Windows in 7m59s, and macOS in
8m12s; Cloudflare Pages also passed.

R3I remains test-signed and does not claim a production signing key, signed
release envelope, automatic managed-root discovery, user-facing install
intent, client activation, updater, public Marketplace distribution,
notarization, SBOM, provenance, cross-target output, or clean-machine release
acceptance.

R3J / `PKG-202A` is complete at design checkpoint `3554ba69` and accepted
implementation head `cc33360b`. The platform now verifies a strict bounded
canonical Ed25519-signed release envelope through a release-key role separate
from launch-grant authority. It binds the complete artifact identity, channel,
generation and validity interval plus archive, R3G manifest, resource pack,
artifact content root, executable, and attached launch grant before returning
one private verified release token.

R3I preview, apply, and repair now require that token. The plan action,
semantic digest, and canonical receipt bind its signed-payload digest; the
executor compares the exact signed grant and re-verifies the token-retained
archive immediately before extraction. Offline receipt-backed verify, remove,
rollback, recovery, and caller-data preservation remain intact.

Local R3J acceptance passes all 227 normal native Rust tests with two external
real-client tests explicitly ignored, all 57 platform tests, all 69 focused
Lite compatibility tests on the complete rerun, strict host and Windows MSVC
checks and Clippy, and the frozen boundary. Exact-head Native CI run
`29365515446` passed `cc33360b`: boundary in 6s, focused Lite in 37s, Windows in
6m51s, Linux in 9m16s, and macOS in 9m40s. Cloudflare Pages also passed.

R3J remains test-signed and does not claim production release-key provisioning,
an executable CLI/UI install intent, automatic managed-root discovery, client
activation, public publication, updater behavior, OS code signing/notarization,
checksums, SBOM, provenance, cross-target artifacts, or clean-machine Alpha.1
acceptance.

R3K is complete at design checkpoint `e6619560` and accepted implementation
head `d90d4846`. The canonical product can now embed one strict byte-canonical
public release-authority policy at build time, with separate release-envelope
and launch-grant roles, bounded key sets, generation floors, release-key
windows, product channel validation, and no runtime trust override. Ordinary
source builds embed no authority and cannot preview or apply a payload.

The current-target native CLI now composes R3J and R3I into explicit
preview/apply/verify/remove operations. Preview is non-mutating; apply requires
the exact semantic plan digest and explicit filesystem-write approval; verify
and remove remain canonical-receipt-backed. All output is versioned and path
redacted, and the managed root must already be absolute, owner-private, and
approved.

Local R3K acceptance passes 232 native Rust tests with the two real
external-client tests ignored, the focused 60 platform + 8 app-library + 16 CLI
tests, all 69 focused Lite tests, strict host and Windows checks and Clippy, the
frozen boundary, and an isolated authority-injected empty-environment status
proof. Exact-head Native CI run `29369405002` passed `d90d4846`: boundary in
5s, focused Lite in 38s, macOS in 7m55s, Linux in 8m16s, and Windows in 8m26s.
Cloudflare Pages also passed.

R3K does not select production key values, handle private signing material,
create or discover managed roots, download releases, activate clients, mutate
desktop state, publish artifacts, provide an updater, or claim clean-machine
Alpha.1 readiness.

R3L is complete at design checkpoint `789af6ba` and implementation head
`a4aa9172`. The accepted Codex and Claude Code registration adapters now share
one handle-bound coordinator that re-verifies the target-specific signed
PluginBundle grant, complete plan, exact three approvals, and outstanding host
action before delegating apply/replay, verify, repair, remove, or rollback.
Immediate post-mutation verification and accepted adapter rollback remain in
force, and previews cannot cross independently discovered handles.

The desktop boundary can receive one prepared trusted session per local target,
show the exact plan digest and approval labels, retain the verified plan behind
an OS-random 128-bit token, and apply only after exact confirmation. Source
builds receive no session and continue to report `apply: false`. The canonical
binary now provides a bounded `qiongli ui --startup-check` that validates the
embedded content, desktop service, snapshot, app state, and linked window
entrypoint without opening a window or starting a subprocess. A copied
current-target artifact passes that check outside the checkout with an empty
runtime `PATH`.

Local R3L acceptance passes 236 native Rust tests with the two real external-
client tests explicitly ignored, the three-test coordinator suite in under one
second, all 69 focused Lite tests, strict host and Windows MSVC checks and
Clippy, both formatting gates, and the frozen boundary. Exact implementation-
head Native CI run `29373107891` passed `a4aa9172`: boundary in 6s, focused
Lite in 36s, Linux in 8m17s, Windows in 8m39s, and macOS in 11m13s.
Cloudflare Pages also passed.

R3L does not assemble release inputs, select production keys, create or
discover managed roots, invoke client CLIs, mutate client-owned caches or
enablement, support desktop/cloud Marketplace bypass, display a clean-machine
window, publish artifacts, provide an updater, or publish Alpha.1.

The bounded R3M implementation and technical release-gate tail are complete:

1. R3M now assembles the accepted portable payload and target-specific plugin
   sources into an exact three-file signed current-target candidate. An
   isolated outside-checkout harness passes the empty-`PATH`
   CLI/skills/UI-preflight/Lite-MCP/Codex/Claude lifecycle, rejection,
   compensation, and unrelated-state checks with ephemeral in-memory test
   keys. Native CI reproduces that non-publishing evidence on Linux.
2. Exact implementation-head Native CI run `29401898602` and Cloudflare Pages
   passed `8d5a4233`. The non-publishing CI evidence binds clean PR merge
   candidate `c98817cf` and records exact Linux x86-64 candidate digests.
3. The R3M readiness receipt records production authority/signatures, final
   accepted `2.x` source binding, maintainer production-target selection,
   real-client evidence, displayed-window/accessibility evidence, independent
   release-note review, and explicit tag/release authorization as unresolved
   external publication gates. PR #63 remains Draft and no Alpha.1 tag or
   release exists.

Full MCP, agents, ToolHost, orchestration, and updater work remain R4 and target
the next Alpha rather than blocking the bounded Lite Alpha.1 publication.

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
4. CFG-201A, CFG-201B, and the R1 native-command slice are complete;
5. R2A shared Lite contract/framing extraction is complete at `d7f2d64f`;
6. R2B shared provider config/status and bounded search extraction is complete
   at `2eaadfb1`;
7. R2C shared evidence export and supported Zotero behavior is complete at
   `2513c52f`;
8. R2D shared route/task-plan preview and typed dispatch is complete at
   `5509d2c1`;
9. R2E canonical native Lite MCP stdio vertical slice is complete at
   `fce20f46`, including copied-binary and exact-head CI proof;
10. R3A install-plan/platform boundary and signed Lite launch-grant contracts
    are complete at `60c2ddc5`, with exact implementation-head Native CI run
    `29332864357` and Cloudflare Pages green;
11. R3B managed resource transactions are complete at `b3a6ea6b`, with exact
    implementation-and-local-receipt Native CI run `29335762713` and
    Cloudflare green;
12. R3C Codex local registration is accepted at `e1166010`, with exact-head
    Native CI run `29339197110` and Cloudflare Pages green;
13. R3D native Codex plugin composition and isolated real-client activation are
    accepted at `103fe11d`, with exact-head Native CI run `29342795330` and
    Cloudflare Pages green;
14. R3E Claude Code skills-directory and local marketplace integration is
    accepted at `337cce74`, with exact-head Native CI run `29345585219` and
    Cloudflare Pages green;
15. R3F minimal native desktop manager is accepted at `4d706033`, with
    exact-head Native CI run `29351331008` and Cloudflare Pages green;
16. R3G current-target native artifact assembly is accepted at `7fddca15`,
    with exact-head Native CI run `29354332680` and Cloudflare Pages green;
17. R3H deterministic current-target portable archive finalization is accepted
    at `f1e50074`, with exact-head Native CI run `29357292961` and Cloudflare
    Pages green;
18. R3I verified current-target payload installation through shared platform
    services is accepted at `25335d43`, with exact-head Native CI run
    `29361710636` and Cloudflare Pages green;
19. R3J signed current-target release envelope and trusted-public-key policy is
    accepted at `cc33360b`, with exact-head Native CI run `29365515446` and
    Cloudflare Pages green;
20. R3K build-time release-authority injection and explicit current-target CLI
    install intent are accepted at `d90d4846`, with exact-head Native CI run
    `29369405002` and Cloudflare Pages green;
21. R3L activation coordination, desktop typed intent, and copied-artifact
    startup preflight are accepted at `a4aa9172`, with exact-head Native CI run
    `29373107891` and Cloudflare Pages green;
22. R3M implementation is accepted at `8d5a4233`, with exact-head Native CI
    run `29401898602`, Cloudflare Pages, macOS aarch64 local acceptance, and
    Linux x86-64 CI acceptance green. The technical receipt is
    `tooling/release/acceptance/v2.0.0-alpha.1-r3m-readiness.md`; publication is
    blocked by its external-gate ledger and has not been run.
23. R3N Batches 1-3 close the observed settings, Skills destination, MCP
    self-test, source discovery, and native application-entry gaps. Batch 4 is
    accepted as automated engineering evidence at `d988070c`: exact-head run
    `29419027524` passed all nine jobs, including Type 2 Linux AppImage
    finalization and actual packaged-launcher startup on macOS, Windows, and
    Linux with an empty `PATH`. Its outputs remain non-publishing.
24. R3N Batch 5 has local partial evidence: actual Codex CLI `0.144.4` and
    Claude Code `2.1.209` passed isolated local-plugin/cache/MCP lifecycle
    checks, and the exact CI macOS package displayed the `Qiongli 2` window
    with labelled Accessibility navigation and Skills controls. The Codex
    external test now verifies client remove, absence, and receipt-owned
    catalog cleanup. This is not three-platform clean-machine or human
    accessibility acceptance; final candidate regeneration, production
    signing/notarization, Windows/Linux interactive evidence, and explicit
    publication authorization remain open gates.

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
